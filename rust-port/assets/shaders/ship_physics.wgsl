@group(0) @binding(0)
var<storage, read_write> positions: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> materials: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read_write> masks: array<vec4<u32>>;

@group(0) @binding(3)
var<storage, read_write> force_buffer: array<vec4<f32>>;

@group(0) @binding(4)
var<storage, read> settings: array<vec4<f32>>;

@group(0) @binding(5)
var<storage, read_write> water_buffer: array<vec4<f32>>;

@group(0) @binding(6)
var<storage, read_write> water_outflow_1: array<vec4<f32>>;

@group(0) @binding(7)
var<storage, read_write> water_outflow_2: array<vec4<f32>>;

@group(0) @binding(8)
var<storage, read_write> water_velocity_1: array<vec4<f32>>;

@group(0) @binding(9)
var<storage, read_write> water_velocity_2: array<vec4<f32>>;

const DIRECTIONS: array<vec2<i32>, 8> = array<vec2<i32>, 8>(
    vec2<i32>(1, 0),
    vec2<i32>(1, 1),
    vec2<i32>(0, 1),
    vec2<i32>(-1, 1),
    vec2<i32>(-1, 0),
    vec2<i32>(-1, -1),
    vec2<i32>(0, -1),
    vec2<i32>(1, -1),
);

const SPRING_LENGTHS: array<f32, 8> = array<f32, 8>(
    1.0,
    1.41421356237,
    1.0,
    1.41421356237,
    1.0,
    1.41421356237,
    1.0,
    1.41421356237,
);

const FLOW_LENGTHS: array<f32, 8> = array<f32, 8>(
    1.0,
    0.70710678118,
    1.0,
    0.70710678118,
    1.0,
    0.70710678118,
    1.0,
    0.70710678118,
);

const AIR: f32 = 1.225;
const WATER: f32 = 1025.0;
const SIXTIETH: f32 = 1.0f / 60.0f / 60.0f;
const WATER_STATE_OFFSET: u32 = 0xffffffffu;

fn dimensions() -> vec2<u32> {
    return vec2<u32>(u32(settings[0].x), u32(settings[0].y));
}

fn cell_count() -> u32 {
    let size = dimensions();
    return size.x * size.y;
}

fn neighbor_index(index: u32, direction: u32) -> u32 {
    let size = dimensions();
    let coordinate = vec2<i32>(i32(index % size.x), i32(index / size.x));
    let neighbor = coordinate + DIRECTIONS[direction];
    if neighbor.x < 0 || neighbor.y < 0
        || neighbor.x >= i32(size.x) || neighbor.y >= i32(size.y)
    {
        return WATER_STATE_OFFSET;
    }
    return u32(neighbor.y) * size.x + u32(neighbor.x);
}

fn wave_height(x: f32, time: f32) -> f32 {
    let wave_size = settings[2].zw;
    let inverse_wave = 3.141592 / wave_size.x;
    // The original driver rounds invWave*3 before multiplying by x and
    // contracts the weighted sum as fma(first, .7, second*.3). Reassociation
    // changes submerged fill depth; original GPU intermediate probes cover it.
    let a = sin(x * inverse_wave + time * 0.3);
    let b = sin(bitcast<f32>(bitcast<u32>(inverse_wave * 3.0) ^ bitcast<u32>(settings[8].z)) * x - time);
    return (fma(a, 0.7, b * 0.3) + 1.0) * 0.5 * wave_size.y;
}

fn outflow_weight(index: u32, direction: u32) -> f32 {
    if direction < 4u {
        return water_outflow_1[index][direction];
    }
    return water_outflow_2[index][direction - 4u];
}

fn stored_outflow_velocity(index: u32, direction: u32) -> f32 {
    if direction < 4u {
        return water_velocity_1[index][direction];
    }
    return water_velocity_2[index][direction - 4u];
}

fn is_water_flow_cell(index: u32) -> bool {
    let flags = masks[index].x;
    // Source set1If7 requires the dynamic bit set by FILTER_DYNAMIC before
    // FILTER_PERMEABLE can set the flow bit. Ground, hull and rope are excluded.
    return flags != 0u && (flags & 7u) == 0u;
}

fn outflow_velocity(index: u32, direction: u32) -> f32 {
    let neighbor = neighbor_index(index, direction);
    if neighbor == WATER_STATE_OFFSET {
        return 0.0;
    }
    let position = positions[index];
    let neighbor_position = positions[neighbor];
    let offset = neighbor_position.xy - position.xy;
    var normal = vec2<f32>(0.0);
    if any(offset != vec2<f32>(0.0)) {
        normal = normalize(offset);
    }
    let water = water_buffer[index + cell_count()];
    let neighbor_water = water_buffer[neighbor + cell_count()];
    let funk = 1.0 + settings[4].y * (water.x - 1.0);
    let projected_velocity = dot(normal, water.zw);
    let pressure_difference = water.x - neighbor_water.x;
    let height_difference = position.y - neighbor_position.y;
    let pressure_head = pressure_difference + height_difference;
    let gravity_y = -settings[1].x;
    let bernoulli_velocity = sign(pressure_head) * sqrt(-2.0 * gravity_y * abs(pressure_head));
    // The source GL driver contracts this multiply/add. Preserve its single
    // rounding instead of a separately rounded product near pressure reversal.
    return max(fma(funk, bernoulli_velocity, projected_velocity), 0.0);
}

@compute @workgroup_size(64, 1, 1)
fn forces(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    let current_mask = masks[index];
    var output_force = force_buffer[index].xy;
    var struts = current_mask.y;
    var water_struts = current_mask.z;
    let material = materials[index];
    if current_mask.x != 0u && (current_mask.x & 4u) == 0u {
        let dimensions_iterations_delta = settings[0];
        let gravity_rigidity_damping_strength = settings[1];
        let position = positions[index];
        let fps = settings[8].y;
        let b = 0.03 * fps * dimensions_iterations_delta.z;
        for (var direction = 0u; direction < 8u; direction += 1u) {
            let bit = 1u << direction;
            if (struts & bit) == 0u {
                continue;
            }
            let neighbor = neighbor_index(index, direction);
            if neighbor == WATER_STATE_OFFSET {
                struts &= ~bit;
                continue;
            }
            let neighbor_material = materials[neighbor];
            if neighbor_material.w == 0.0 {
                struts &= ~bit;
                continue;
            }
            let neighbor_mask = masks[neighbor];
            let neighbor_position = positions[neighbor];
            let difference = neighbor_position - position;
            let length = length(difference.xy);
            let soft = select(1.0, 0.001, ((current_mask.x | neighbor_mask.x) & 1u) != 0u);
            let spring_mass = min(material.w, neighbor_material.w);
            let stiffness = 750.0 * spring_mass * b * gravity_rigidity_damping_strength.y;
            let elastic_load = (length - SPRING_LENGTHS[direction]) * stiffness * soft;
            if any(difference.xy != vec2<f32>(0.0)) {
                output_force += normalize(difference.xy) * elastic_load
                    + b * gravity_rigidity_damping_strength.z * difference.zw;
            }

            let shared_material = min(material, neighbor_material);
            let limit = select(shared_material.z, shared_material.y, elastic_load >= 0.0);
            if abs(elastic_load) > limit * gravity_rigidity_damping_strength.w
                * dimensions_iterations_delta.z * fps
            {
                struts &= ~bit;
                water_struts &= ~bit;
            }
        }

        let wave = wave_height(position.x, settings[3].x);
        let physics_delta = settings[8].x;
        let previous_wave = wave_height(position.x, settings[3].x - physics_delta);
        let wave_velocity = (previous_wave - wave) * fps
            / (1.0 - min(0.0, position.y / settings[2].w));
        // Original GLSL mix uses a rounded WATER + (AIR - WATER) * factor.
        // Selecting AIR directly loses its cancellation rounding on the source
        // driver, which accumulates through force feedback over repeated steps.
        let air_factor = select(0.0, 1.0, position.y >= wave);
        let density = WATER + (AIR - WATER) * air_factor;
        let speed = position.zw + vec2<f32>(0.0, wave_velocity);
        if any(speed != vec2<f32>(0.0)) {
            let exposed_area = select(0.01, 0.5, water_struts != 255u);
            output_force -= normalize(speed) * 0.5 * dot(speed, speed) * density
                * exposed_area * settings[2].x;
        }
        let gravity = vec2<f32>(0.0, -gravity_rigidity_damping_strength.x);
        // Original FORCES contracts buoyancy subtraction and the final
        // reciprocal-mass multiply/add independently (128 native samples).
        output_force = fma(vec2<f32>(-density * settings[2].y), gravity, output_force);
        let inverse_mass = 1.0 / material.w;
        output_force = fma(output_force, vec2<f32>(inverse_mass), gravity);
    }

    force_buffer[index] = vec4<f32>(output_force, 0.0, 0.0);
    masks[index].y = struts;
    masks[index].z = water_struts;
    masks[index].w = 0u;
}

@compute @workgroup_size(64, 1, 1)
fn integrate(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    let current_mask = masks[index];
    if current_mask.x == 0u || (current_mask.x & 4u) != 0u {
        return;
    }

    // Source deltaT is a CPU-rounded uniform, not a shader division.
    let dt = settings[8].x;
    let velocity = fma(force_buffer[index].xy, vec2<f32>(dt), positions[index].zw);
    // Original AMD OpenGL contracts the Y square into the rounded X square.
    // Explicit order preserves measured source reflection bits on Vulkan.
    let inverse_speed = 1.0 / (fma(velocity.y, velocity.y, velocity.x * velocity.x) + 1.0);
    let reflected_velocity = vec2<f32>(inverse_speed, -inverse_speed) * velocity;
    let displacement = velocity * dt;
    // Source GLSL explicitly tests isnan(collision). A self-comparison in
    // WGSL did not reproduce that branch on the tested GPU,
    // producing NaN positions for a stationary point exactly on the floor.
    // Every finite-input division by zero reaches source's collision=1 branch
    // (NaN for 0/0, otherwise an infinity outside [0,1]); preserve that branch
    // without generating an exceptional division in the first place.
    var collision = 1.0;
    if displacement.y != 0.0 {
        let candidate = (settings[3].y - positions[index].y) / displacement.y;
        let nan = (bitcast<u32>(candidate) & 0x7fffffffu) > 0x7f800000u;
        if !nan && candidate >= 0.0 && candidate <= 1.0 { collision = candidate; }
    }
    // Preserve the source driver's difference-form interpolation even when
    // collision is one; weighted mixing returns velocity with different bits.
    // Row 8 Z is positive-zero bits: XOR is an identity precision barrier.
    // It materializes the rounded reflection before subtraction, preventing
    // Vulkan contraction of the reflection multiply into the difference.
    // The original GL driver rounds these operations separately.
    let integrated_velocity = fma(velocity - bitcast<vec2<f32>>(bitcast<vec2<u32>>(reflected_velocity) ^ vec2<u32>(bitcast<u32>(settings[8].z))),
        vec2<f32>(collision), reflected_velocity);
    var position = fma(integrated_velocity, vec2<f32>(dt), positions[index].xy);
    var next_velocity = fma(velocity - bitcast<vec2<f32>>(bitcast<vec2<u32>>(reflected_velocity) ^ vec2<u32>(bitcast<u32>(settings[8].z))),
        vec2<f32>(f32(collision == 1.0)), reflected_velocity);
    if position.y < settings[3].y {
        position.y = (position.y - settings[3].y) * 0.01 + settings[3].y;
        next_velocity.x *= 0.5;
    }
    positions[index] = vec4<f32>(position, next_velocity);
}

@compute @workgroup_size(64, 1, 1)
fn water_fill(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    let water_index = index + count;
    var output_water = water_buffer[index];
    let position = positions[index];
    let state = masks[index];
    // Source water fill runs for every live, non-ground material texel. Hull
    // and rope texels store the outside water depth here; the following
    // material-mass pass uses it to reproduce the source buoyancy response.
    // Interior water transfer itself remains restricted to flow cells.
    if state.x == 0u || (state.x & 4u) != 0u {
        water_buffer[water_index] = output_water;
        return;
    }
    let surface = wave_height(position.x, settings[3].x);
    let height = -min(position.y - surface, 0.0);
    if (state.x & 3u) != 0u {
        output_water.x = height;
    } else if (state.z & 255u) != 255u {
        let difference = height - output_water.x;
        let gravity = settings[1].x;
        // Source sets "deltaT", but this shader declares "DeltaT"; its default 1.0 remains active.
        // Preserve the original rounded multiply chain before the fixed timestep.
        // The uniform-zero bit barrier prevents driver reassociation, as in integration.
        // Preserve the source f32 sqrt result before the following multiplies.
        // Packed host bindings: settings[3].w=u_inflow, settings[4].x=u_flow.
        let speed = bitcast<f32>(bitcast<u32>(sqrt(2.0 * gravity * abs(difference)))
            ^ bitcast<u32>(settings[8].z));
        let influx_velocity = bitcast<f32>(bitcast<u32>(sign(difference) * speed * settings[3].w)
            ^ bitcast<u32>(settings[8].z));
        let scaled_velocity = bitcast<f32>(bitcast<u32>(influx_velocity * settings[4].x)
            ^ bitcast<u32>(settings[8].z));
        var new_velocity = scaled_velocity * SIXTIETH;
        new_velocity = -min(-new_velocity, output_water.x);
        // Round the clamped increment before adding it to the stored amount.
        output_water.x += bitcast<f32>(bitcast<u32>(new_velocity) ^ bitcast<u32>(settings[8].z));
    }
    water_buffer[water_index] = output_water;
}

@compute @workgroup_size(64, 1, 1)
fn water_flow(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    if !is_water_flow_cell(index) {
        water_outflow_1[index] = vec4<f32>(0.0);
        water_outflow_2[index] = vec4<f32>(0.0);
        water_velocity_1[index] = vec4<f32>(0.0);
        water_velocity_2[index] = vec4<f32>(0.0);
        return;
    }

    let position = positions[index];
    let water = water_buffer[index + count];
    var weights = vec4<f32>(0.0);
    var weights_2 = vec4<f32>(0.0);
    var velocities = vec4<f32>(0.0);
    var velocities_2 = vec4<f32>(0.0);
    var total_weight = 0.0;
    for (var direction = 0u; direction < 8u; direction += 1u) {
        let velocity = outflow_velocity(index, direction);
        let weight = velocity * FLOW_LENGTHS[direction];
        if direction < 4u {
            weights[direction] = weight;
            velocities[direction] = velocity;
        } else {
            weights_2[direction - 4u] = weight;
            velocities_2[direction - 4u] = velocity;
        }
        total_weight += weight;
    }
    var normal_factor = 0.0;
    if total_weight != 0.0 {
        normal_factor = water.x * settings[4].x * settings[3].z / total_weight;
    }
    water_outflow_1[index] = weights * normal_factor;
    water_outflow_2[index] = weights_2 * normal_factor;
    water_velocity_1[index] = velocities;
    water_velocity_2[index] = velocities_2;
}

@compute @workgroup_size(64, 1, 1)
fn water_transport(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    let input_water = water_buffer[index + count];
    if !is_water_flow_cell(index) {
        // Preserve material-cell water depth while the interior flow pass runs.
        water_buffer[index + count * 2u] = input_water;
        return;
    }

    let position = positions[index];
    let state = masks[index];
    var output_water = input_water;
    var momentum = output_water.zw * output_water.x;
    let is_permeable = (state.x & 2u) == 0u;
    for (var direction = 0u; direction < 8u; direction += 1u) {
        let neighbor = neighbor_index(index, direction);
        if neighbor == WATER_STATE_OFFSET {
            continue;
        }
        let bit = 1u << direction;
        let neighbor_mask = masks[neighbor];
        let permeable = is_permeable
            && (state.z & bit) != 0u
            && (neighbor_mask.x & 2u) == 0u;
        let normal_delta = positions[neighbor].xy - position.xy;
        var normal = vec2<f32>(0.0);
        if any(normal_delta != vec2<f32>(0.0)) {
            normal = normalize(normal_delta);
        }
        let direction_weight = outflow_weight(index, direction);
        if permeable {
            let opposite = (direction + 4u) & 7u;
            let neighbor_weight = outflow_weight(neighbor, opposite);
            let neighbor_velocity = stored_outflow_velocity(neighbor, opposite);
            output_water.x -= direction_weight;
            momentum -= input_water.zw * direction_weight;
            output_water.x += neighbor_weight;
            momentum -= normal * neighbor_velocity * neighbor_weight;
        } else {
            let velocity = stored_outflow_velocity(index, direction);
            momentum -= normal * velocity * direction_weight;
        }
    }
    if output_water.x != 0.0 {
        momentum /= output_water.x;
    } else {
        momentum = vec2<f32>(0.0);
    }
    output_water.z = momentum.x;
    output_water.w = momentum.y;
    water_buffer[index + count * 2u] = output_water;
}

@compute @workgroup_size(64, 1, 1)
fn update_mass(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count {
        return;
    }
    let state = masks[index];
    if state.x == 0u || (state.x & 4u) != 0u {
        return;
    }
    let amount = clamp(water_buffer[index + count * 2u].x, 0.0, 1.0);
    let hull = (state.x & 2u) != 0u;
    let water_weight = select(settings[4].z, 1.0, hull);
    // The original GLSL driver contracts both mix operations as a + (b-a)*t.
    // Preserve each subtraction before the fused multiply-add.
    let fluid_density = fma(WATER * water_weight - AIR, amount, AIR);
    let base_mass = materials[index].x;
    materials[index].w = fma(fluid_density - base_mass, settings[4].w, base_mass);
}

@compute @workgroup_size(64, 1, 1)
fn commit_water(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= cell_count() {
        return;
    }
    water_buffer[index] = water_buffer[index + cell_count() * 2u];
}

// ShipPhysics.posChangePass: move all points, including air and ground, without
// changing velocities. Kept separate from integration so paused moves commit.
@compute @workgroup_size(64, 1, 1)
fn move_positions(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= cell_count() { return; }
    positions[index] = vec4<f32>(positions[index].xy + settings[5].xy, positions[index].zw);
}

// Original FloodTool/DryTool sample the current position/water textures. Do not
// upload a stale CPU snapshot or discard momentum and transport state planes.
@compute @workgroup_size(64, 1, 1)
fn brush_water(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= cell_count() { return; }
    let brush = settings[6];
    if brush.w == 0.0 { return; }
    let dist = distance(positions[index].xy, brush.xy);
    if dist < brush.z {
        let amount = brush.z - dist;
        if brush.w > 0.0 { water_buffer[index].x += amount; }
        else { water_buffer[index].x = max(0.0, water_buffer[index].x - amount); }
    }
}

// Original finalPass reads a stable mask texture before repairing reciprocal
// links. A second plane supplies that snapshot without storage-buffer races.
@compute @workgroup_size(64, 1, 1)
fn snapshot_masks(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= cell_count() { return; }
    masks[index + cell_count()] = masks[index];
}

@compute @workgroup_size(64, 1, 1)
fn repair_masks(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    let count = cell_count();
    if index >= count { return; }
    var output_mask = masks[index + count];
    for (var direction = 0u; direction < 8u; direction += 1u) {
        let neighbor = neighbor_index(index, direction);
        var other_struts = 0u;
        if neighbor != WATER_STATE_OFFSET { other_struts = masks[neighbor + count].y; }
        let opposite = (direction + 4u) & 7u;
        if (other_struts & (1u << opposite)) == 0u {
            let keep = ~(1u << direction);
            output_mask.y &= keep;
            output_mask.z &= keep;
        }
    }
    masks[index] = output_mask;
}
// BreakTool's pass leaves mask X/W and all physical points intact. It removes
// links from a cut point and incoming links from its eight live-position peers.
@compute @workgroup_size(64, 1, 1)
fn break_links(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= cell_count() { return; }
    let brush = settings[7];
    if brush.w == 0.0 { return; }
    var links = masks[index].yz;
    if distance(positions[index].xy, brush.xy) < brush.z { links = vec2<u32>(0u); }
    for (var direction = 0u; direction < 8u; direction += 1u) {
        let neighbor = neighbor_index(index, direction);
        if neighbor != WATER_STATE_OFFSET {
            if distance(positions[neighbor].xy, brush.xy) < brush.z {
                links &= vec2<u32>(~(1u << direction));
            }
        }
    }
    masks[index].y = links.x;
    masks[index].z = links.y;
}

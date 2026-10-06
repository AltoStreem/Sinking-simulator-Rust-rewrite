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
    return (
        sin(x * inverse_wave + time * 0.3) * 0.7
        + sin(inverse_wave * 3.0 * x - time) * 0.3
        + 1.0
    ) * 0.5 * wave_size.y;
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
    return flags != 0u && (flags & 4u) == 0u && (flags & 3u) == 0u;
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
    return max(projected_velocity + funk * bernoulli_velocity, 0.0);
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
    let material = materials[index];
    if current_mask.x != 0u && (current_mask.x & 4u) == 0u && material.w != 0.0 {
        let dimensions_iterations_delta = settings[0];
        let gravity_rigidity_damping_strength = settings[1];
        let position = positions[index];
        let b = 0.03 * dimensions_iterations_delta.z
            / max(dimensions_iterations_delta.w, 0.00001)
            * dimensions_iterations_delta.z;
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
            let fps = dimensions_iterations_delta.z
                / max(dimensions_iterations_delta.w, 0.00001);
            if abs(elastic_load) > limit * gravity_rigidity_damping_strength.w
                * dimensions_iterations_delta.z * fps
            {
                struts &= ~bit;
            }
        }

        let wave = wave_height(position.x, settings[3].x);
        let physics_delta = dimensions_iterations_delta.w
            / max(dimensions_iterations_delta.z, 1.0);
        let previous_wave = wave_height(position.x, settings[3].x - physics_delta);
        let wave_velocity = (previous_wave - wave) * (dimensions_iterations_delta.z
            / max(dimensions_iterations_delta.w, 0.00001))
            / (1.0 - min(0.0, position.y / max(settings[2].w, 0.00001)));
        let density = select(WATER, AIR, position.y >= wave);
        let speed = position.zw + vec2<f32>(0.0, wave_velocity);
        if any(speed != vec2<f32>(0.0)) {
            let exposed_area = select(0.01, 0.5, (current_mask.z & struts) != 255u);
            output_force -= normalize(speed) * 0.5 * dot(speed, speed) * density
                * exposed_area * settings[2].x;
        }
        let gravity = vec2<f32>(0.0, -gravity_rigidity_damping_strength.x);
        output_force -= density * settings[2].y * gravity;
        output_force = output_force / material.w + gravity;
    }

    force_buffer[index] = vec4<f32>(output_force, 0.0, 0.0);
    masks[index].y = struts;
    masks[index].z &= struts;
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

    let dt = settings[0].w / max(settings[0].z, 1.0);
    let velocity = force_buffer[index].xy * dt + positions[index].zw;
    let inverse_speed = 1.0 / (dot(velocity, velocity) + 1.0);
    let reflected_velocity = vec2<f32>(inverse_speed, -inverse_speed) * velocity;
    let displacement = velocity * dt;
    var collision = (settings[3].y - positions[index].y) / displacement.y;
    if collision != collision || collision < 0.0 || collision > 1.0 {
        collision = 1.0;
    }
    let integrated_velocity = mix(reflected_velocity, velocity, collision);
    var position = integrated_velocity * dt + positions[index].xy;
    var next_velocity = mix(reflected_velocity, velocity, f32(collision == 1.0));
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
        let gravity = max(settings[1].x, 0.0);
        var new_velocity = sign(difference) * sqrt(2.0 * gravity * abs(difference))
            * settings[3].z * settings[3].w * settings[4].x / 3600.0;
        new_velocity = -min(-new_velocity, output_water.x);
        output_water.x += new_velocity;
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
    let fluid_density = mix(AIR, WATER * water_weight, amount);
    materials[index].w = mix(materials[index].x, fluid_density, settings[4].w);
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

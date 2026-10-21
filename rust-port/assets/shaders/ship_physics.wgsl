@group(0) @binding(0)
var<storage, read_write> positions: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> positions_after_force: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read_write> materials: array<vec4<f32>>;

@group(0) @binding(3)
var<storage, read_write> masks: array<vec4<u32>>;

@group(0) @binding(4)
var<storage, read_write> force_buffer: array<vec4<f32>>;

@group(0) @binding(5)
var<storage, read> settings: array<vec4<f32>>;

@group(0) @binding(6)
var<storage, read_write> water_buffer: array<vec4<f32>>;

@group(0) @binding(7)
var<storage, read_write> water_outflow_1: array<vec4<f32>>;

@group(0) @binding(8)
var<storage, read_write> water_outflow_2: array<vec4<f32>>;

@group(0) @binding(9)
var<storage, read_write> water_velocity_1: array<vec4<f32>>;

@group(0) @binding(10)
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
    let a = sin(x * inverse_wave + time * 0.3);
// The source GLSL expression is invWave * 3 * x - time. On the reference
// AMD OpenGL driver, its trailing multiply-subtract is contracted to an FMA;
// preserve the rounded invWave*3 factor, then fuse the final multiply/subtract.
let inverse_wave_three = inverse_wave * 3.0;
let second_wave_argument = fma(inverse_wave_three, x, -time);
let b = sin(second_wave_argument);
    // The source GLSL is left-to-right `a * 0.7 + b * 0.3`; its measured
    // OpenGL result matches fusing the first product with the second term.
    return (fma(a, 0.7, b * 0.3) + 1.0) * 0.5 * wave_size.y;
}

fn force_wave_height(x: f32, time: f32) -> f32 {
    let wave_size = settings[2].zw;
    let inverse_wave = 3.141592 / wave_size.x;
    // The source GLSL's `xpos * invWave + time * .3` contracts the final
    // multiply-add on the reference OpenGL driver. Keep the time product
    // rounded, then fuse it with the position product; source-seeded, whole-
    // field force replays match tolerance only with this phase contraction.
    let first_wave_argument = fma(x, inverse_wave, time * 0.3);
    let a = sin(first_wave_argument);
    let inverse_wave_three = inverse_wave * 3.0;
    // The source force program's current-time wave matches a fused multiply-
    // subtract in the reference OpenGL driver. Keep this call-site-specific:
    // the previous-time call has a separately rounded phase on that driver.
    let second_wave_argument = fma(inverse_wave_three, x, -time);
    let b = sin(second_wave_argument);
    // Preserve the source GLSL weighted-sum operand order after the force
    // pass's source-matched phase calculations above.
    return (fma(a, 0.7, b * 0.3) + 1.0) * 0.5 * wave_size.y;
}

// The source GLSL calls the same wave expression with `time` and
// `time - deltaT`. On the captured AMD OpenGL driver, the previous-time call's
// first phase matches a separately rounded position/time sum, while the
// current-time call's first phase matches the fused expression above. Keep the
// source's observable rounding at this call site; applying it to both first
// phases exceeds force tolerance at the earlier frame-1 step-2 checkpoint.
fn previous_force_wave_height(x: f32, time: f32) -> f32 {
    let wave_size = settings[2].zw;
    let inverse_wave = 3.141592 / wave_size.x;
    let first_product = bitcast<f32>(
        bitcast<u32>(x * inverse_wave) ^ bitcast<u32>(settings[8].z),
    );
    let time_product = bitcast<f32>(
        bitcast<u32>(time * 0.3) ^ bitcast<u32>(settings[8].z),
    );
    let first_wave_argument = first_product + time_product;
    let a = sin(first_wave_argument);
    let inverse_wave_three = inverse_wave * 3.0;
    let second_wave_product = bitcast<f32>(
        bitcast<u32>(inverse_wave_three * x) ^ bitcast<u32>(settings[8].z),
    );
    let second_wave_argument = second_wave_product - time;
    let b = sin(second_wave_argument);
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
    let position = positions[index];
    let water = water_buffer[index + cell_count()];
    var neighbor_position = vec4<f32>(0.0);
    var neighbor_water = vec4<f32>(0.0);
    if neighbor != WATER_STATE_OFFSET {
        neighbor_position = positions[neighbor];
        neighbor_water = water_buffer[neighbor + cell_count()];
    }

    // The source GLSL fetches out-of-range neighbors without a bounds check.
    // That read is undefined, but the original AMD OpenGL path returns zero
    // texels here; preserve the observed source behavior for edge-cell flow.
    let offset = neighbor_position.xy - position.xy;
    var normal = vec2<f32>(0.0);
    if any(offset != vec2<f32>(0.0)) {
        normal = normalize(offset);
    }
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
    // ShipPhysics' GLSL pass samples the stable input mask texture while it
    // writes a separate output texture. The Rust storage buffer keeps the
    // live result in the first plane, so read this invocation and every
    // neighbor from the snapshot plane prepared immediately before `forces`.
    let current_mask = masks[index + count];
    // The source FORCES pass is stencil-gated by FILTER_DYNAMIC. Discarded
    // air and ground fragments leave every output lane untouched. The source
    // upload pads RGB integer data to RGBA, and the force shader explicitly
    // clears the output alpha lane for fragments that reach this pass.
    if current_mask.x == 0u || (current_mask.x & 4u) != 0u {
        return;
    }
    // The Java force target attaches only out_force and out_mask. Its shader
    // declares out_pos_vel, but that output has no target in this pass. Keep
    // the unchanged position in a scratch buffer so INTEGRATE reads the same
    // pre-integration state as the source's unchanged pos-velocity texture.
    let position = positions[index];
    positions_after_force[index] = position;
    var output_force = force_buffer[index].xy;
    var struts = current_mask.y;
    var water_struts = current_mask.z;
    let material = materials[index];
    let dimensions_iterations_delta = settings[0];
    let gravity_rigidity_damping_strength = settings[1];
    let fps = settings[8].y;
    let b = 0.03 * fps * dimensions_iterations_delta.z;
    for (var direction = 0u; direction < 8u; direction += 1u) {
            let bit = 1u << direction;
            if (struts & bit) == 0u {
                continue;
            }
            let neighbor = neighbor_index(index, direction);
            // The source force shader fetches all eight neighbors without a
            // bounds check. Its observed OpenGL driver returns zero texels for
            // out-of-range fetches; preserve those operands instead of
            // deleting the spring here (which changes both damping and mask
            // state if an edge strut exists).
            var neighbor_material = vec4<f32>(0.0);
            var neighbor_mask = vec4<u32>(0u);
            var neighbor_position = vec4<f32>(0.0);
            if neighbor != WATER_STATE_OFFSET {
                neighbor_material = materials[neighbor];
                neighbor_mask = masks[neighbor + count];
                neighbor_position = positions[neighbor];
            }
            // The source GLSL retains links to zero-effective-mass neighbors;
            // their spring load is zero, but their velocity can still damp this point.
            let difference = neighbor_position - position;
            // Spell out the source GLSL length/normalize dot operation. On the
            // measured AMD OpenGL driver, reducing y*y + x*x with one rounding
            // matches the real-ship spring intermediates; WGSL length/normalize
            // builtins were one ULP off and the stiff springs amplified it.
            let spring_length_squared = fma(
                difference.y,
                difference.y,
                difference.x * difference.x,
            );
            let length = sqrt(spring_length_squared);
            let spring_normal = difference.xy * inverseSqrt(spring_length_squared);
            // Match the source GLSL mix(1, 0.001, flag) interpolation. The
            // original OpenGL driver fuses the difference form; selecting the
            // endpoint loses its rounding and causes spring-force drift.
            let soft_factor = select(
                0.0,
                1.0,
                ((current_mask.x | neighbor_mask.x) & 1u) != 0u,
            );
            let soft = fma(0.001 - 1.0, soft_factor, 1.0);
            let spring_mass = min(material.w, neighbor_material.w);
            // The source GLSL evaluates (25 * 30 * mass) * b * rigidity.
            // Vulkan reassociated this chain as (750 * b) * mass on the
            // measured driver, moving stiffness by one ULP at cell 6125. Keep
            // the source's intermediate f32 rounding before applying b.
            let stiffness_mass = bitcast<f32>(
                bitcast<u32>(750.0 * spring_mass) ^ bitcast<u32>(settings[8].z),
            );
            let stiffness_step_rate = bitcast<f32>(
                bitcast<u32>(stiffness_mass * b) ^ bitcast<u32>(settings[8].z),
            );
            let stiffness = stiffness_step_rate * gravity_rigidity_damping_strength.y;
            let elastic_load = (length - SPRING_LENGTHS[direction]) * stiffness * soft;
            if any(difference.xy != vec2<f32>(0.0)) {
                let damping_scale = b * gravity_rigidity_damping_strength.z;
                let damping_term = damping_scale * difference.zw;
                let spring_and_damping = fma(
                    spring_normal,
                    vec2<f32>(elastic_load),
                    damping_term,
                );
                output_force += spring_and_damping;
            }

            let shared_material = min(material, neighbor_material);
            let limit = select(shared_material.y, shared_material.z, elastic_load < 0.0);
            if abs(elastic_load) > limit * gravity_rigidity_damping_strength.w
                * dimensions_iterations_delta.z * fps
            {
                struts &= ~bit;
                water_struts &= ~bit;
            }
    }

    let wave = force_wave_height(position.x, settings[3].x);
    let physics_delta = settings[8].x;
    let previous_wave = previous_force_wave_height(position.x, settings[3].x - physics_delta);
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
        // Preserve the source's first rounded product, density*u_buoyancy,
        // then fuse its multiplication by gravity with `force -= ...`. A
        // direct uninstrumented AMD GL replay at captured Pacmaster operands
        // confirms the fused result (for cell 6679: 0x43d47dae vs separate
        // 0x43d47da0). Capturing an extra intermediate in a debug GLSL shader
        // can inhibit that contraction, so that intermediate is not the oracle.
        // settings[8].z is positive zero and acts as the precision barrier that
        // keeps the first product separate.
    let buoyancy_scale = bitcast<f32>(
        bitcast<u32>(density * settings[2].y) ^ bitcast<u32>(settings[8].z),
    );
    output_force = fma(vec2<f32>(-buoyancy_scale), gravity, output_force);
    let inverse_mass = 1.0 / material.w;
    output_force = fma(output_force, vec2<f32>(inverse_mass), gravity);

    // Source writes RG32F; sampling that two-channel texture yields B=0, A=1.
    // Materialize the same default swizzle in this vec4 storage buffer.
    force_buffer[index] = vec4<f32>(output_force, 0.0, 1.0);
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
    let position_input = positions_after_force[index];
    let velocity = fma(force_buffer[index].xy, vec2<f32>(dt), position_input.zw);
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
        let candidate = (settings[3].y - position_input.y) / displacement.y;
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
    var position = fma(integrated_velocity, vec2<f32>(dt), position_input.xy);
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
        // The source FLOW_WATER render pass is stencil-gated. Discarded
        // fragments leave these persistent scratch texels untouched; clearing
        // them here changes later transport reads from neighboring cells.
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
        // Source GLSL fetches each neighbor without checking texture bounds.
        // The original AMD path returns zero texels for these reads, so retain
        // its edge outflow momentum instead of dropping the out-of-range side.
        var neighbor_mask = vec4<u32>(0u);
        var neighbor_position = vec4<f32>(0.0);
        if neighbor != WATER_STATE_OFFSET {
            neighbor_mask = masks[neighbor];
            neighbor_position = positions[neighbor];
        }
        let bit = 1u << direction;
        let permeable = is_permeable
            && (state.z & bit) != 0u
            && (neighbor_mask.x & 2u) == 0u;
        let normal_delta = neighbor_position.xy - position.xy;
        var normal = vec2<f32>(0.0);
        if any(normal_delta != vec2<f32>(0.0)) {
            normal = normalize(normal_delta);
        }
        let direction_weight = outflow_weight(index, direction);
        if permeable {
            let opposite = (direction + 4u) & 7u;
            var neighbor_weight = 0.0;
            var neighbor_velocity = 0.0;
            if neighbor != WATER_STATE_OFFSET {
                neighbor_weight = outflow_weight(neighbor, opposite);
                neighbor_velocity = stored_outflow_velocity(neighbor, opposite);
            }
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

// ShipPhysics.posChangePass copies every texel and changes only its position.
// The source pass uses the GL_ALWAYS state, so air and ground points move too.
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

// Source force and final passes read a stable mask texture while writing new
// masks. A second plane supplies that snapshot without storage-buffer races.
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
    // The source adds FILTER_OCCUPIED as a setup-stencil pass, then runs
    // REPAIR_MASK with getExec() (GL_ALWAYS). Keep repairing empty texels too;
    // the shader's output is not stencil-gated by FILTER_OCCUPIED.
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
        // BreakTool's source fragment shader fetches idx + s[i] without a
        // bounds check. The original AMD OpenGL driver returns a zero texel
        // for these out-of-range texture reads, so edge links still test the
        // zero-position sentinel against the brush instead of being skipped.
        var neighbor_position = vec2<f32>(0.0);
        if neighbor != WATER_STATE_OFFSET {
            neighbor_position = positions[neighbor].xy;
        }
        if distance(neighbor_position, brush.xy) < brush.z {
            links &= vec2<u32>(~(1u << direction));
        }
    }
    masks[index].y = links.x;
    masks[index].z = links.y;
}

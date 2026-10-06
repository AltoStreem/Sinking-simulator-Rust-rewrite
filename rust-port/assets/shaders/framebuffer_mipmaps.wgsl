@group(0) @binding(0) var previous_level: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var next_level: texture_storage_2d<rgba8unorm, write>;
@compute @workgroup_size(8, 8, 1)
fn generate_mipmap(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(next_level);
    if any(id.xy >= size) { return; }
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
    textureStore(next_level, vec2<i32>(id.xy), textureSampleLevel(previous_level, linear_sampler, uv, 0.0));
}
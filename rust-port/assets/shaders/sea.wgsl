#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> waves_time_darkness: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> view: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> col: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var tex_sampler: sampler;

// Source LINEAR_MIPMAP_LINEAR and transparent GL_CLAMP_TO_BORDER sampling.
fn border_texel(point: vec2<i32>, size: vec2<i32>, level: i32) -> vec4<f32> {
    if any(point < vec2<i32>(0)) || any(point >= size) { return vec4<f32>(0.0); }
    return textureLoad(tex, point, level);
}
fn linear_level(uv: vec2<f32>, level: i32) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(tex, level));
    let position = uv * vec2<f32>(size) - vec2<f32>(0.5);
    let base = vec2<i32>(floor(position));
    let fraction = fract(position);
    return mix(mix(border_texel(base, size, level), border_texel(base + vec2<i32>(1, 0), size, level), fraction.x),
        mix(border_texel(base + vec2<i32>(0, 1), size, level), border_texel(base + vec2<i32>(1, 1), size, level), fraction.x), fraction.y);
}
fn reflection_sample(uv: vec2<f32>, bias: f32) -> vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(tex));
    let dx = dpdx(uv) * dimensions;
    let dy = dpdy(uv) * dimensions;
    let implicit_lod = log2(max(max(length(dx), length(dy)), 0.000001));
    let lod = clamp(implicit_lod + bias, 0.0, f32(textureNumLevels(tex) - 1u));
    let lower = i32(floor(lod));
    let upper = i32(ceil(lod));
    return mix(linear_level(uv, lower), linear_level(uv, upper), fract(lod));
}
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let inv_wave = 3.141592 / waves_time_darkness.x;
    let x = mesh.world_position.x;
    let time = waves_time_darkness.z;
    let waterdepth = ((sin(x * inv_wave + time * 0.3) * 0.7
        + sin(inv_wave * 3.0 * x - time) * 0.3) + 1.0) * 0.5
        * waves_time_darkness.y + view.z;
    let y = mesh.world_position.y;
    // Source screenCoord.y + 4/resolution.y equals two framebuffer pixels.
    let y2 = y + 2.0 * (view.y - view.x) / view.w;
    let reflected_v = 1.0 - (2.0 * waterdepth - y2 - view.x) / (view.y - view.x);
    let distance = abs(mesh.uv.y - reflected_v);
    let reflection = reflection_sample(vec2<f32>(mesh.uv.x, reflected_v), distance * 20.0);
    let t = clamp((distance - 0.25) / -0.25, 0.0, 1.0);
    let reflection_weight = reflection.a * t * t * (3.0 - 2.0 * t) * 0.4;
    let water_color = mix(mix(col, reflection, reflection_weight),
        vec4<f32>(0.0, 0.0, 0.0, 1.0), (waterdepth - y) / 1000.0 * waves_time_darkness.w);
    let coverage = smoothstep(y, y2, waterdepth);
    let color = water_color * coverage;
    let prev = textureSample(tex, tex_sampler, mesh.uv);
    return vec4<f32>(mix(prev.rgb, color.rgb, color.a), max(prev.a, color.a));
}
@group(0) @binding(0) var stars: texture_storage_2d<rgba8unorm, write>;
fn noise(p: vec2<f32>) -> f32 {
    return fract(415.92653 * (cos(p.x * 37.0) + cos(p.y * 57.0)));
}
fn star(p: vec2<f32>) -> f32 {
    let value = noise(p);
    if value >= 0.99 { return pow((value - 0.99) / (1.0 - 0.99), 6.0); }
    return 0.0;
}
fn field(p: vec2<f32>) -> f32 {
    let f = fract(p);
    let cell = floor(p);
    return star(cell) * (1.0 - f.x) * (1.0 - f.y)
        + star(cell + vec2<f32>(0.0, 1.0)) * (1.0 - f.x) * f.y
        + star(cell + vec2<f32>(1.0, 0.0)) * f.x * (1.0 - f.y)
        + star(cell + vec2<f32>(1.0, 1.0)) * f.x * f.y;
}
@compute @workgroup_size(8, 8, 1)
fn bake_stars(@builtin(global_invocation_id) index: vec3<u32>) {
    let size = textureDimensions(stars);
    if any(index.xy >= size) { return; }
    // Store top-left rows for Bevy sampling, preserving GL bottom-left fragment centers.
    let gl_pixel = vec2<f32>(f32(index.x) + 0.5, f32(size.y - index.y) - 0.5);
    let value = field(gl_pixel * 0.8);
    textureStore(stars, vec2<i32>(index.xy), vec4<f32>(value, value, value, 1.0));
}

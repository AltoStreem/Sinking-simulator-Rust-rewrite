#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> surface_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> deep_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> depth_params: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Sea.java: mix(col, vec4(0,0,0,1), depth / 1000 * waterDarkness).
    // This adapter still lacks source framebuffer reflection composition.
    let depth_t = (depth_params.z - mesh.world_position.y) / 1000.0 * depth_params.y;
    return mix(surface_color, vec4<f32>(0.0, 0.0, 0.0, 1.0), depth_t);
}
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> surface_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> deep_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> depth_params: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let depth = clamp(
        (depth_params.z - mesh.world_position.y) / max(depth_params.x, 1.0),
        0.0,
        1.0,
    );
    let depth_t = pow(depth, 0.72);
    let color = mix(surface_color.rgb, deep_color.rgb, depth_t) * depth_params.y;
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}

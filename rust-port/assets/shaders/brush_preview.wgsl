#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> cursor_radius: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> color: vec4<f32>;
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(mesh.world_position.xy, cursor_radius.xy);
    let radius = cursor_radius.z;
    var alpha = 0.5;
    if dist > radius { alpha -= smoothstep(radius, radius + dist / 10.0, dist) * 0.5; }
    return vec4<f32>(color.rgb, alpha);
}

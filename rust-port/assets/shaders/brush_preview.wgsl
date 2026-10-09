#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> cursor_radius: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var background: texture_2d<f32>;
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(mesh.world_position.xy, cursor_radius.xy);
    let radius = cursor_radius.z;
    var alpha = 0.5;
    if dist > radius { alpha -= smoothstep(radius, radius + dist / 10.0, dist) * 0.5; }
    alpha *= color.a;
    let previous = textureLoad(background, vec2<i32>(mesh.position.xy), 0);
    let rgb = color.rgb * alpha + previous.rgb * (1.0 - alpha);
#ifdef SRGB_OUTPUT
    return vec4<f32>(rgb, alpha * alpha + previous.a * (1.0 - alpha));
#else
    let linear = select(pow(max((rgb + 0.055) / 1.055, vec3<f32>(0.0)), vec3<f32>(2.4)),
        rgb / 12.92, rgb <= vec3<f32>(0.04045));
    return vec4<f32>(linear, alpha * alpha + previous.a * (1.0 - alpha));
#endif
}

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> water_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> wave_params: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let x = mesh.world_position.x;
    let time = wave_params.y;
    let wave = sin(x * wave_params.x + time * 0.3) * 0.7
        + sin(3.0 * x * wave_params.x - time) * 0.3;
    let depth_tint = (1.0 - mesh.uv.y) * 0.16;
    let shimmer = 0.93 + max(wave, 0.0) * 0.07;
    let color = water_color.rgb * shimmer + vec3<f32>(0.035, 0.075, 0.11) * depth_tint;
    return vec4<f32>(color, water_color.a);
}

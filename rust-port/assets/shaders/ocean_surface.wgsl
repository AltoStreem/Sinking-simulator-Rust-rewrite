#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> ocean_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> wave_params: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let x = mesh.world_position.x;
    let phase = x * wave_params.x;
    let wave = sin(phase + wave_params.y * 0.3) * 0.7
        + sin(phase * 3.0 - wave_params.y) * 0.3;
    let surface = (wave + 1.0) * 0.5 * wave_params.z;
    let depth = surface - mesh.world_position.y;
    if depth < 0.0 {
        discard;
    }

    let crest = exp(-depth * 0.55);
    let ripple = sin(phase * 2.1 - wave_params.y * 1.4);
    let shimmer = 0.94 + ripple * 0.025 + max(wave, 0.0) * 0.035;
    let color = ocean_color.rgb * shimmer + vec3<f32>(0.12, 0.23, 0.30) * crest * 0.3;
    return vec4<f32>(color, ocean_color.a);
}

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> effect_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> effect_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let depth = max(effect_params.z - mesh.world_position.y, 0.0);
    let time = effect_params.x;
    let phase = mesh.world_position.x * effect_params.y;

    let caustics_a = sin(phase * 1.9 + time * 1.4 + depth * 0.025);
    let caustics_b = sin(phase * 2.7 - time * 0.8 - depth * 0.041);
    let caustics = max(caustics_a * caustics_b, 0.0);
    let attenuation = exp(-depth * 0.0025);
    let darkness = clamp(1.0 / max(effect_params.w, 0.1), 0.25, 1.2);
    let strength = (0.18 + caustics * 0.18) * attenuation * darkness;
    let rgb = effect_color.rgb * (0.72 + caustics * 0.55);
    return vec4<f32>(rgb, clamp(strength * effect_color.a, 0.0, 0.38));
}

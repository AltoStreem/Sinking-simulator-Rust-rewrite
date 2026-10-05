#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var ship_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var ship_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> tint: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> wave_params: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world_x = mesh.world_position.x;
    let world_y = mesh.world_position.y;
    let time = wave_params.y;
    let phase = world_x * wave_params.x;
    let wave = sin(phase + time * 0.3) * 0.7 + sin(phase * 3.0 - time) * 0.3;
    let surface = wave_params.w + (wave + 1.0) * 0.5 * wave_params.z;

    if world_y > surface {
        discard;
    }

    let depth = clamp(surface - world_y, 0.0, 360.0);
    let ripple = sin(phase * 2.7 - time * 1.7 + depth * 0.11);
    let uv = mesh.uv + vec2<f32>(ripple * 0.008, ripple * 0.003);
    let ship = textureSample(ship_texture, ship_sampler, uv);
    let fade = exp(-depth * 0.004);
    let wave_shade = 0.88 + 0.12 * max(wave, 0.0);
    return vec4<f32>(ship.rgb * tint.rgb * wave_shade, ship.a * tint.a * fade);
}

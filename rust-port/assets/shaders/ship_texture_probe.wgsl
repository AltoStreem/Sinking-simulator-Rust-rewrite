#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let sample_color = textureSample(source_texture, source_sampler, mesh.uv);
    // The test uses an opaque blend mode and alpha one to expose raw sampled
    // RGB directly in the RGBA8 target, independently of ShipMaterial's blend.
    return vec4<f32>(sample_color.rgb, 1.0);
}

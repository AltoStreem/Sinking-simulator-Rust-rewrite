// SeaFrame already contains the source default framebuffer's encoded RGBA8
// values. CompositingSpace::Srgb routes this camera through a linear UNORM
// target, so copy those values without Sprite's linear_to_srgb conversion.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var sea_frame: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var sea_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sea_frame, sea_sampler, mesh.uv);
}

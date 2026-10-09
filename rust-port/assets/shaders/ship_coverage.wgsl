#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Ship's source stencil replaces coverage for every emitted triangle,
    // independent of the display texture's alpha and water/light shading.
    return vec4<f32>(1.0);
}

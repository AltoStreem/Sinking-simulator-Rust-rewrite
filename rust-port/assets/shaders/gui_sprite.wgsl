// ImGui's Frag_Color * texture(Texture, Frag_UV), using encoded GUI image views.
#import bevy_render::maths::affine3_to_square
#import bevy_render::color_operations::linear_to_srgb
#import bevy_sprite::sprite_view_bindings::view
struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) i_model_transpose_col0: vec4<f32>,
    @location(1) i_model_transpose_col1: vec4<f32>,
    @location(2) i_model_transpose_col2: vec4<f32>,
    @location(3) i_color: vec4<f32>,
    @location(4) i_uv_offset_scale: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
};
@vertex
fn vertex(input:VertexInput)->VertexOutput {
    var output:VertexOutput;
    let position=vec3<f32>(f32(input.index & 1u),f32((input.index & 2u) >> 1u),0.0);
    output.clip_position=view.clip_from_world * affine3_to_square(mat3x4<f32>(
        input.i_model_transpose_col0,input.i_model_transpose_col1,input.i_model_transpose_col2)) * vec4<f32>(position,1.0);
    output.uv=position.xy * input.i_uv_offset_scale.zw + input.i_uv_offset_scale.xy;
    output.color=vec4<f32>(linear_to_srgb(input.i_color.rgb),input.i_color.a);
    return output;
}
@group(1) @binding(0) var sprite_texture:texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler:sampler;
@fragment
fn fragment(input:VertexOutput)->@location(0) vec4<f32> {
    return input.color * textureSample(sprite_texture,sprite_sampler,input.uv);
}

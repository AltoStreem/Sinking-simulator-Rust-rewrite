// Source ImGui colors/vertex gradients and texture products are encoded RGB.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::color_operations::linear_to_srgb
struct ColorMaterial {
    color:vec4<f32>, uv_transform:mat3x3<f32>, flags:u32, alpha_cutoff:f32,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material:ColorMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var image:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var image_sampler:sampler;
@fragment
fn fragment(mesh:VertexOutput)->@location(0) vec4<f32> {
    var color=vec4<f32>(linear_to_srgb(material.color.rgb),material.color.a);
#ifdef VERTEX_COLORS
    color *= mesh.color;
#endif
    let uv=(material.uv_transform * vec3<f32>(mesh.uv,1.0)).xy;
    if (material.flags & 1u)!=0u {color *= textureSample(image,image_sampler,uv);}
    let mode=material.flags & 3221225472u;
    if mode==0u {color.a=1.0;}
    if mode==1073741824u {
        if color.a<material.alpha_cutoff {discard;}
        color.a=1.0;
    }
    return color;
}

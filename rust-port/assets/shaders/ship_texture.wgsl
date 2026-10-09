#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> sea_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var ship_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var ship_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var internal_lights: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var internal_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var external_lights: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var external_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var<storage, read> water: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var<storage, read> masks: array<vec4<u32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var<uniform> coverage_mode: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(12) var hull_coverage: texture_2d<f32>;

// The original OpenGL textures and framebuffer operated on RGB code values.
// Recover those values from Bevy's sRGB texture decoding, apply the source
// shader, and retain source RGB in the UNORM scene framebuffer. Sea converts
// the completed composition to linear RGB for the final sRGB attachment.
fn source_rgb(linear: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(linear,vec3<f32>(0.0)),vec3<f32>(1.0/2.4)) - 0.055, linear * 12.92, linear <= vec3<f32>(0.0031308));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let inside=all(mesh.uv >= vec2<f32>(0.0)) && all(mesh.uv < vec2<f32>(1.0));
    let border=select(0.0,1.0,inside);
    let exlight=textureSample(external_lights,external_sampler,mesh.uv)*border;
    let inlight=textureSample(internal_lights,internal_sampler,mesh.uv)*border;
    let sample_color=textureSample(ship_texture,ship_sampler,mesh.uv)*border;
    var color=vec4<f32>(source_rgb(sample_color.rgb),sample_color.a);
    let dimensions=vec2<u32>(params.zw);
    let pixel=min(vec2<u32>(clamp(mesh.uv,vec2<f32>(0.0),vec2<f32>(1.0))*params.zw),dimensions-vec2<u32>(1u));
    let index=pixel.y*dimensions.x+pixel.x;
    let brightness=clamp(vec3<f32>(smoothstep(-0.2,0.8,params.x))+source_rgb(exlight.rgb)*exlight.a,vec3<f32>(0.0),vec3<f32>(1.0));
    if (masks[index].x & 2u) == 0u {
        color=mix(color,vec4<f32>(sea_color.rgb,1.0),clamp(water[index].x,0.0,1.0)*0.75*params.y);
    }
    color=vec4<f32>(color.rgb*brightness+source_rgb(inlight.rgb)*inlight.a*(vec3<f32>(1.0)-brightness),color.a);
    // Source stencil ifnot1 tests geometry coverage, including transparent texels.
    if coverage_mode.x > 0.5 {
        if textureLoad(hull_coverage, vec2<i32>(mesh.position.xy), 0).r > 0.5 { discard; }
    }
    return color;
}

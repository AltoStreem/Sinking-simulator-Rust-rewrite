#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> resolution: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var sky_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var star_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var star_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let day = sky_params.x;
    let world_y = mesh.world_position.y - sky_params.w;
    let center_y = sky_params.y;
    let half_height = sky_params.z;
    var limit_y = center_y;
    if world_y > 0.0 {
        limit_y += half_height;
    } else if world_y < 0.0 {
        limit_y -= half_height;
    }

    var ytex = 0.0;
    if world_y > 0.0 {
        ytex = 0.5 - (world_y / limit_y) * 0.5;
    } else {
        ytex = 0.5 + (world_y / limit_y) * 0.5;
    }

    let baked_star = textureSample(star_map, star_sampler, mesh.uv).x;
    let stars = baked_star
        * smoothstep(0.0, 3.0, world_y)
        * (1.0 - day);
    let sky = textureSample(sky_map, sky_sampler, vec2<f32>(day, ytex)).rgb;
    let rgb = sky + vec3<f32>(stars);
    // ScreenFbo is RGBA8_UNORM: retain source RGB values through composition.
    return vec4<f32>(rgb, 1.0);
}

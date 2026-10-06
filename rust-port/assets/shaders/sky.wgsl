#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> resolution: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var sky_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky_sampler: sampler;

fn hash_noise(p: vec2<f32>) -> f32 {
    return fract(415.92653 * (cos(p.x * 37.0) + cos(p.y * 57.0)));
}

fn star_at(p: vec2<f32>) -> f32 {
    let value = hash_noise(p);
    if value >= 0.99 {
        return pow((value - 0.99) / 0.01, 6.0);
    }
    return 0.0;
}

fn star_field(sample_pos: vec2<f32>) -> f32 {
    let cell = floor(sample_pos);
    let f = fract(sample_pos);
    let a = star_at(cell);
    let b = star_at(cell + vec2<f32>(0.0, 1.0));
    let c = star_at(cell + vec2<f32>(1.0, 0.0));
    let d = star_at(cell + vec2<f32>(1.0, 1.0));
    return a * (1.0 - f.x) * (1.0 - f.y)
        + b * (1.0 - f.x) * f.y
        + c * f.x * (1.0 - f.y)
        + d * f.x * f.y;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let day = clamp(sky_params.x, 0.0, 1.0);
    let world_y = mesh.world_position.y - sky_params.w;
    let center_y = sky_params.y;
    let half_height = max(sky_params.z, 0.001);
    var limit_y = center_y;
    if world_y > 0.0 {
        limit_y += half_height;
    } else if world_y < 0.0 {
        limit_y -= half_height;
    }

    var ytex = 0.0;
    if abs(limit_y) < 0.000001 {
        ytex = 0.5;
    } else if world_y > 0.0 {
        ytex = 0.5 - (world_y / limit_y) * 0.5;
    } else {
        ytex = 0.5 + (world_y / limit_y) * 0.5;
    }
    ytex = clamp(ytex, 0.0, 1.0);

    // OpenGL bakes stars into RGBA8 at bottom-left framebuffer coordinates.
    let star_pixel = vec2<f32>(mesh.position.x, resolution.y - mesh.position.y);
    let baked_star = floor(clamp(star_field(star_pixel * 0.8),0.0,1.0)*255.0+0.5)/255.0;
    let stars = baked_star
        * smoothstep(0.0, 3.0, world_y)
        * (1.0 - day);
    let sky = textureSample(sky_map, sky_sampler, vec2<f32>(day, ytex)).rgb;
    let rgb = sky + vec3<f32>(stars);
    let linear = select(pow(max((rgb+0.055)/1.055,vec3<f32>(0.0)),vec3<f32>(2.4)),rgb/12.92,rgb <= vec3<f32>(0.04045));
    return vec4<f32>(linear, 1.0);
}

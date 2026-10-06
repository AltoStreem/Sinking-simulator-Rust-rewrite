//! Translation of Sky.java's camera-relative horizon and source star field.
use crate::{SEA_LEVEL, Simulation, WorldCamera};
use bevy::{
    camera::visibility::RenderLayers,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, TextureFormat},
    shader::ShaderRef,
    sprite_render::Material2d,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct SkyMaterial {
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(1)]
    pub resolution: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub texture: Handle<Image>,
}
impl Material2d for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky.wgsl".into()
    }
}
#[derive(Component)]
pub(crate) struct SkyMesh(pub Handle<Mesh>, pub Handle<SkyMaterial>);

pub(crate) fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let texture = match image::open("assets/config/sky.png") {
        Ok(image) => {
            let mut texture = crate::texture_2d::ship_texture(image.to_rgba8());
            // Sky uses RGBA8 in the source, with LINEAR_MIPMAP_LINEAR.
            texture.texture_descriptor.format = TextureFormat::Rgba8Unorm;
            texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
            images.add(texture)
        }
        Err(error) => {
            bevy::log::warn!("Could not load original sky map: {error}");
            assets
                .load_builder()
                .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                    settings.is_srgb = false;
                    settings.sampler = ImageSampler::linear();
                })
                .load("config/sky.png")
        }
    };
    let mut mesh = crate::fullscreen::Fullscreen::mesh();
    crate::fullscreen::Fullscreen::fit(
        &mut mesh,
        Rect::from_corners(Vec2::new(-640.0, -360.0), Vec2::new(640.0, 360.0)),
    );
    let mesh = meshes.add(mesh);
    let material = materials.add(SkyMaterial {
        params: Vec4::new(1.0, 0.0, 360.0, SEA_LEVEL),
        resolution: Vec4::ZERO,
        texture,
    });
    commands.spawn((
        SkyMesh(mesh.clone(), material.clone()),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, -10.0),
        RenderLayers::layer(0),
    ));
}

pub(crate) fn animate_sky(
    simulation: Res<Simulation>,
    windows: Query<&Window>,
    cameras: Query<(&Transform, &Projection), With<WorldCamera>>,
    mut sky_meshes: Query<(&SkyMesh, &mut Transform), Without<WorldCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera_transform, Projection::Orthographic(projection))) = cameras.single() else {
        return;
    };
    let area = projection.area;
    for (sky, mut transform) in &mut sky_meshes {
        if let Some(mut mesh) = meshes.get_mut(&sky.0) {
            crate::fullscreen::Fullscreen::fit(&mut mesh, area);
        }
        transform.translation.x = camera_transform.translation.x;
        transform.translation.y = camera_transform.translation.y;
        if let Some(mut material) = materials.get_mut(&sky.1) {
            material.params = Vec4::new(
                simulation.day,
                camera_transform.translation.y - SEA_LEVEL,
                area.height() * 0.5,
                SEA_LEVEL,
            );
            material.resolution = Vec4::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
                0.0,
                0.0,
            );
        }
    }
}

/// Reference for SkyShader's inverse-camera mapping. Source y=0 is the sea.
#[cfg(test)]
fn horizon_uv(world_y: f32, center_y: f32, half_height: f32) -> f32 {
    let limit = center_y + world_y.signum() * half_height;
    if limit == 0.0 {
        return 0.5;
    }
    if world_y > 0.0 {
        0.5 - world_y / limit * 0.5
    } else {
        0.5 + world_y / limit * 0.5
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_sky_horizon_tracks_panning_and_zoom() {
        assert_eq!(horizon_uv(0.0, 0.0, 360.0), 0.5);
        assert_eq!(horizon_uv(360.0, 0.0, 360.0), 0.0);
        assert_eq!(horizon_uv(-360.0, 0.0, 360.0), 1.0);
        assert_eq!(horizon_uv(500.0, 140.0, 360.0), 0.0);
        assert_eq!(horizon_uv(-220.0, 140.0, 360.0), 1.0);
        assert_eq!(horizon_uv(720.0, 0.0, 720.0), 0.0);
    }
    #[test]
    fn source_sky_wgsl_parses_and_validates() {
        let body = include_str!("../assets/shaders/sky.wgsl")
            .lines()
            .filter(|line| !line.starts_with("#import"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("#{MATERIAL_BIND_GROUP}", "2");
        let interface = "struct VertexOutput { @builtin(position) position:vec4<f32>, @location(0) world_position:vec4<f32>, @location(1) world_normal:vec3<f32>, @location(2) uv:vec2<f32>, };\n";
        let source = format!("{interface}{body}");
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}

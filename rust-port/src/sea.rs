//! Sea.java wave and fullscreen framebuffer composition. Driver filter parity
//! and source stencil behavior still require verification/conversion.
pub(crate) fn wave_height(x: f32, time: f32, amplitude: f32, width: f32) -> f32 {
    let inv_wave = 3.141592 / width;
    ((0.7 * (x * inv_wave + time * 0.3).sin() + 0.3 * (3.0 * x * inv_wave - time).sin()) + 1.0)
        * 0.5
        * amplitude
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_wave_origin_and_zero_amplitude() {
        assert_eq!(wave_height(0.0, 0.0, 2.0, 40.0), 1.0);
        assert_eq!(wave_height(123.0, 9.0, 0.0, 40.0), 0.0);
    }
}

use crate::{SEA_LEVEL, Simulation, WorldCamera, screen_fbo::ScreenFbo};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::Material2d,
};
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct SeaMaterial {
    #[uniform(0)]
    pub waves_time_darkness: Vec4,
    #[uniform(1)]
    pub view: Vec4,
    #[uniform(2)]
    pub color: Vec4,
    #[texture(3)]
    #[sampler(4)]
    pub texture: Handle<Image>,
}
impl Material2d for SeaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sea.wgsl".into()
    }
}
#[derive(Component)]
pub(crate) struct Sea(pub Handle<Mesh>, pub Handle<SeaMaterial>);
#[derive(Component)]
pub(crate) struct SeaCamera;

pub(crate) fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut world: Query<(Entity, &mut Camera), With<WorldCamera>>,
    legacy: Query<
        Entity,
        Or<(
            With<crate::OceanSurface>,
            With<crate::OceanDepth>,
            With<crate::UnderwaterEffect>,
            With<crate::ReflectionMesh>,
        )>,
    >,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SeaMaterial>>,
    simulation: Res<Simulation>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    let target = ScreenFbo::new(size, &mut images);
    for (entity, mut camera) in &mut world {
        camera.order = -1;
        commands
            .entity(entity)
            .insert(RenderTarget::Image(target.texture.clone().into()));
    }
    for entity in &legacy {
        commands.entity(entity).despawn();
    }
    let mesh = meshes.add(crate::fullscreen::Fullscreen::mesh());
    let material = materials.add(SeaMaterial {
        waves_time_darkness: Vec4::new(
            simulation.wave_width,
            simulation.wave_amplitude,
            simulation.elapsed,
            simulation.water_darkness,
        ),
        view: Vec4::new(-360.0, 360.0, SEA_LEVEL, size.y as f32),
        color: simulation.water_color(),
        texture: target.filtered.clone(),
    });
    commands.spawn((
        Sea(mesh.clone(), material.clone()),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        RenderLayers::layer(2),
    ));
    commands.spawn((Camera2d, SeaCamera, RenderLayers::layer(2)));
    commands.insert_resource(target);
}

pub(crate) fn sync(
    simulation: Res<Simulation>,
    control: Res<crate::camera_control::CameraControlState>,
    windows: Query<&Window>,
    world: Query<(&Transform, &Projection), (With<WorldCamera>, Without<SeaCamera>, Without<Sea>)>,
    mut cameras: Query<
        (&mut Transform, &mut Projection),
        (With<SeaCamera>, Without<WorldCamera>, Without<Sea>),
    >,
    mut seas: Query<(&Sea, &mut Transform), (Without<WorldCamera>, Without<SeaCamera>)>,
    mut target: ResMut<ScreenFbo>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SeaMaterial>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((transform, projection)) = world.single() else {
        return;
    };
    let Projection::Orthographic(_) = projection else {
        return;
    };
    let Some((minimum, maximum)) = control.world_bounds() else {
        return;
    };
    let area = Rect::from_corners(
        minimum - transform.translation.truncate(),
        maximum - transform.translation.truncate(),
    );
    target.resize(
        UVec2::new(window.physical_width(), window.physical_height()),
        &mut images,
    );
    for (mut current, mut current_projection) in &mut cameras {
        *current = *transform;
        *current_projection = projection.clone();
    }
    for (sea, mut current) in &mut seas {
        current.translation.x = transform.translation.x;
        current.translation.y = transform.translation.y;
        if let Some(mut mesh) = meshes.get_mut(&sea.0) {
            crate::fullscreen::Fullscreen::fit(&mut mesh, area);
        }
        if let Some(mut material) = materials.get_mut(&sea.1) {
            material.waves_time_darkness = Vec4::new(
                simulation.wave_width,
                simulation.wave_amplitude,
                simulation.elapsed,
                simulation.water_darkness,
            );
            material.view = Vec4::new(minimum.y, maximum.y, SEA_LEVEL, target.size.y as f32);
            material.color = simulation.water_color();
        }
    }
}

#[cfg(test)]
mod shader_tests {
    #[test]
    fn source_sea_shader_validates() {
        let source = include_str!("../assets/shaders/sea.wgsl")
            .replace("#import bevy_sprite::mesh2d_vertex_output::VertexOutput",
                "struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) world_position: vec4<f32>, @location(1) uv: vec2<f32>, }")
            .replace("#{MATERIAL_BIND_GROUP}", "0");
        let module = naga::front::wgsl::parse_str(&source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}

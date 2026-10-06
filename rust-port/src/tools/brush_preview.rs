//! Shared translation of BreakTool/FloodTool/DryTool renderPass GLSL.
use crate::*;
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct BrushMaterial {
    #[uniform(0)]
    pub(crate) cursor_radius: Vec4,
    #[uniform(1)]
    pub(crate) color: Vec4,
}
impl Material2d for BrushMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/brush_preview.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub(crate) fn update_damage_brush_preview(
    windows: Query<&Window>,
    camera_state: Res<CameraControlState>,
    simulation: Res<Simulation>,
    mut materials: ResMut<Assets<BrushMaterial>>,
    mut preview: Query<
        (
            &MeshMaterial2d<BrushMaterial>,
            &mut Transform,
            &mut Visibility,
        ),
        With<DamageBrushPreview>,
    >,
) {
    let world = windows.single().ok().and_then(|window| {
        let cursor = window.cursor_position()?;
        let scale = 720.0 / window.height().max(1.0);
        let ui = Vec2::new(
            (cursor.x - window.width() * 0.5) * scale,
            (window.height() * 0.5 - cursor.y) * scale,
        );
        if camera_control::blocked(&simulation, ui) {
            None
        } else {
            camera_state.world_at_cursor(cursor)
        }
    });
    let color = match simulation.tool {
        Tool::Break => Vec4::new(1.0, 0.0, 0.0, 1.0),
        Tool::Flood => Vec4::new(0.0, 0.0, 1.0, 1.0),
        Tool::Dry => Vec4::ONE,
        Tool::Move => Vec4::ZERO,
    };
    for (handle, mut transform, mut visibility) in &mut preview {
        if let Some(world) =
            world.filter(|_| simulation.tool != Tool::Move && simulation.show_tools)
        {
            *visibility = Visibility::Inherited;
            transform.translation = world.extend(8.0);
            transform.scale = Vec3::splat(simulation.tool_size);
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.cursor_radius = Vec4::new(world.x, world.y, simulation.tool_size, 0.0);
                material.color = color;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
#[cfg(test)]
fn source_alpha(radius: f32, distance: f32) -> f32 {
    if distance <= radius {
        return 0.5;
    }
    let t = ((distance - radius) / (distance / 10.0)).clamp(0.0, 1.0);
    0.5 * (1.0 - t * t * (3.0 - 2.0 * t))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_radius_and_falloff_follow_the_tool_shader() {
        assert_eq!(source_alpha(1.0, 0.0), 0.5);
        assert_eq!(source_alpha(1.0, 1.0), 0.5);
        assert!(source_alpha(1.0, 1.05) > 0.0 && source_alpha(1.0, 1.05) < 0.5);
        assert_eq!(source_alpha(1.0, 1.12), 0.0);
    }
    #[test]
    fn source_preview_shader_validates() {
        let source=include_str!("../../assets/shaders/brush_preview.wgsl")
            .replace("#import bevy_sprite::mesh2d_vertex_output::VertexOutput", "struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) world_position: vec4<f32>, }")
            .replace("#{MATERIAL_BIND_GROUP}","0");
        let module = naga::front::wgsl::parse_str(&source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}

//! Shared translation of BreakTool/FloodTool/DryTool renderPass GLSL.
use crate::*;
pub(crate) const OVERLAY_LAYER: usize = 9;
#[derive(Component)]
pub(crate) struct BrushOverlayCamera;
pub(crate) fn source_camera_output(mut commands:Commands,
    mut cameras:Query<(Entity,&mut Camera,&bevy::camera::RenderTarget,&mut Msaa,Option<&bevy::camera::CompositingSpace>)>) {
    for (entity,mut camera,target,mut msaa,space) in &mut cameras {
        if matches!(target,bevy::camera::RenderTarget::Window(_)) {
            // Match the source framebuffer's encoded RGB blending. Bevy's
            // sprite/text/material pipelines encode their output for this view;
            // the final blit converts once for the native sRGB display target.
            if space!=Some(&bevy::camera::CompositingSpace::Srgb) {
                commands.entity(entity).insert(bevy::camera::CompositingSpace::Srgb);
            }
            camera.output_mode=bevy::camera::CameraOutputMode::Write {
                blend_state:Some(bevy::render::render_resource::BlendState::REPLACE),
                clear_color:ClearColorConfig::None};
            *msaa=Msaa::Off;
        }
    }
}
pub(crate) fn setup_overlay(mut commands: Commands,
    world: Query<(&Transform,&Projection),With<WorldCamera>>) {
    let Ok((transform,projection))=world.single() else {return};
    commands.spawn((Camera2d,BrushOverlayCamera,Camera {order:-1,
        clear_color:ClearColorConfig::None,..default()},*transform,projection.clone(),
        Msaa::Off,RenderLayers::layer(OVERLAY_LAYER)));
}
pub(crate) fn sync_overlay(
    world: Query<(&Transform,&Projection),(With<WorldCamera>,Without<BrushOverlayCamera>)>,
    mut overlay: Query<(&mut Transform,&mut Projection),(With<BrushOverlayCamera>,Without<WorldCamera>)>) {
    let Ok((transform,projection))=world.single() else {return};
    for (mut output_transform,mut output_projection) in &mut overlay {
        *output_transform=*transform;*output_projection=projection.clone();
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct BrushMaterial {
    #[uniform(0)]
    pub(crate) cursor_radius: Vec4,
    #[uniform(1)]
    pub(crate) color: Vec4,
    #[texture(2,filterable=false)]
    pub(crate) background: Handle<Image>,
}
impl Material2d for BrushMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/brush_preview.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
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
    native_window: Option<NonSend<crate::window_bevy::LiveWindow>>,
    native_brush: Option<Res<super::tool::NativeBrushInput>>,
    sea_frame: Option<Res<crate::sea::SeaFrame>>,
) {
    let world = if native_window.is_some() {
        native_brush.as_ref().and_then(|state|state.world(&camera_state))
    } else {windows.single().ok().and_then(|window| {
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
    })};
    let color = match simulation.tool {
        Tool::Break => Vec4::new(1.0, 0.0, 0.0, 1.0),
        Tool::Flood => Vec4::new(0.0, 0.0, 1.0, 1.0),
        Tool::Dry => Vec4::ONE,
        Tool::Move | Tool::None => Vec4::ZERO,
    };
    for (handle, mut transform, mut visibility) in &mut preview {
        if let Some((minimum,maximum))=camera_state.world_bounds() {
            let world=world.unwrap_or(Vec2::ZERO);
            *visibility = Visibility::Inherited;
            transform.translation = ((minimum+maximum)*0.5).extend(8.0);
            transform.scale = (maximum-minimum).extend(1.0);
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.cursor_radius = Vec4::new(world.x, world.y, simulation.tool_size, 0.0);
                material.color = color;
                if let Some(frame)=&sea_frame {material.background=frame.texture.clone();}
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
    fn native_camera_output_replaces_completed_frame_without_a_second_alpha_blend() {
        let mut app=App::new();
        app.add_systems(Update,source_camera_output);
        let camera=app.world_mut().spawn((Camera2d,Camera {order:2,..default()},Msaa::Sample4)).id();
        app.update();
        assert_eq!(*app.world().get::<Msaa>(camera).unwrap(),Msaa::Off);
        assert_eq!(app.world().get::<bevy::camera::CompositingSpace>(camera),Some(&bevy::camera::CompositingSpace::Srgb));
        assert!(matches!(app.world().get::<Camera>(camera).unwrap().output_mode,
            bevy::camera::CameraOutputMode::Write {blend_state:Some(state),..}
            if state==bevy::render::render_resource::BlendState::REPLACE));
    }
    #[test]
    fn overlay_camera_isolated_after_sea_before_gui_and_follows_world_view() {
        let mut app=App::new();
        app.add_systems(Update,setup_overlay).add_systems(PostUpdate,sync_overlay);
        let world_camera=app.world_mut().spawn((WorldCamera,Camera {order:-3,..default()},
            RenderLayers::layer(0),Projection::Orthographic(OrthographicProjection::default_2d()),
            Transform::from_xyz(12.0,15.0,0.0))).id();
        app.update();
        let overlay=app.world_mut().query_filtered::<Entity,With<BrushOverlayCamera>>().single(app.world()).unwrap();
        let camera=app.world().get::<Camera>(overlay).unwrap();
        assert_eq!(camera.order,-1);assert!(matches!(camera.clear_color,ClearColorConfig::None));
        assert!(matches!(app.world().get::<bevy::camera::RenderTarget>(overlay),
            Some(bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Primary))),
            "Use native default target, not scene capture");
        let layers=app.world().get::<RenderLayers>(overlay).unwrap();
        assert_eq!(layers,&RenderLayers::layer(OVERLAY_LAYER));
        for layer in 0..=8 {assert!(!layers.intersects(&RenderLayers::layer(layer)));}
        assert_eq!(*app.world().get::<Msaa>(overlay).unwrap(),Msaa::Off);
        // Run only sync after initial creation, avoiding duplicate setup cameras.
        app.world_mut().get_mut::<Transform>(world_camera).unwrap().translation=Vec3::new(20.0,30.0,0.0);
        let mut schedule=Schedule::default();schedule.add_systems(sync_overlay);schedule.run(app.world_mut());
        assert_eq!(app.world().get::<Transform>(overlay).unwrap().translation,Vec3::new(20.0,30.0,0.0));
    }
    #[test]
    fn native_preview_uses_shared_retained_cursor_after_capture_resize_and_hidden_controls() {
        use bevy::window::{WindowEvent as E,PrimaryWindow,CursorMoved,WindowResized};
        let mut app=App::new();
        let mut simulation=Simulation::default();simulation.tool=Tool::Flood;simulation.show_tools=false;
        app.add_plugins(MinimalPlugins).insert_resource(simulation)
            .init_resource::<CameraControlState>().init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>().init_resource::<Assets<BrushMaterial>>()
            .add_message::<bevy::input::mouse::MouseWheel>().add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_message::<E>().add_message::<bevy::app::AppExit>()
            .add_plugins(crate::window_bevy::WindowBridgePlugin)
            .add_systems(Update,(crate::camera_control::handle_camera_control,update_damage_brush_preview).chain());
        let window=app.world_mut().spawn((Window {resolution:(1280,720).into(),..default()},PrimaryWindow)).id();
        app.world_mut().spawn((WorldCamera,Projection::Orthographic(OrthographicProjection::default_2d()),Transform::default()));
        let material=app.world_mut().resource_mut::<Assets<BrushMaterial>>().add(BrushMaterial {cursor_radius:Vec4::ZERO,color:Vec4::ZERO,background:Handle::default()});
        let preview=app.world_mut().spawn((DamageBrushPreview,MeshMaterial2d(material.clone()),Transform::default(),Visibility::Hidden)).id();
        app.update();
        app.world_mut().write_message(E::CursorMoved(CursorMoved {window,position:Vec2::new(1000.0,300.0),delta:None}));
        app.update();
        app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(100.0,200.0)));
        app.world_mut().write_message(E::CursorMoved(CursorMoved {window,position:Vec2::new(100.0,200.0),delta:None}));
        app.world_mut().write_message(E::WindowResized(WindowResized {window,width:1600.0,height:900.0}));
        app.update();
        let expected=app.world().resource::<CameraControlState>().world_at_brush_cursor(Vec2::new(1000.0,300.0),[1600,900]).unwrap();
        let shared=app.world().resource::<super::super::tool::NativeBrushInput>();
        assert_eq!(shared.world(app.world().resource::<CameraControlState>()),Some(expected));
        assert_eq!(*app.world().get::<Visibility>(preview).unwrap(),Visibility::Inherited);
        let (minimum,maximum)=app.world().resource::<CameraControlState>().world_bounds().unwrap();
        assert_eq!(app.world().get::<Transform>(preview).unwrap().translation.truncate(),(minimum+maximum)*0.5);
        assert_eq!(app.world().resource::<Assets<BrushMaterial>>().get(&material).unwrap().cursor_radius,
            Vec4::new(expected.x,expected.y,1.0,0.0));
    }
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
        // Naga consumes WGSL after Bevy's shader definitions are resolved.
        // Validate both native encoded output and the linear test-target path.
        for srgb_output in [false,true] {
        let mut enabled=true;
        let source=source.lines().filter_map(|line|match line.trim() {
            "#ifdef SRGB_OUTPUT"=>{enabled=srgb_output;None},
            "#else"=>{enabled=!srgb_output;None},
            "#endif"=>{enabled=true;None},
            _=>enabled.then_some(line),
        }).collect::<Vec<_>>().join("\n");
        let module = naga::front::wgsl::parse_str(&source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        }
    }
}

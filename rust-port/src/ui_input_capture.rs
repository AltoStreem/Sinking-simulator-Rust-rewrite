//! Live capture for the port's ordinary editor window and active text fields.
//! GUI.java stops forwarding captured mouse/key callbacks to tools and camera.
use crate::*;
#[derive(Resource, Default)]
pub(crate) struct Capture {
    pub(crate) keyboard: bool,
    editor: Option<Rect>,
}
impl Capture {
    pub(crate) fn editor_point(&self, point: Vec2) -> bool {
        self.editor.is_some_and(|rect| rect.contains(point))
    }
    pub(crate) fn mouse(&self, simulation: &Simulation, point: Vec2) -> bool {
        camera_control::blocked(simulation, point) || self.editor_point(point)
    }
    pub(crate) fn packet_mouse(
        &self,
        simulation: &Simulation,
        packet: &window_bevy::OrderedInput,
    ) -> bool {
        let [width, height] = packet.screen;
        let [x, y] = packet.cursor;
        width <= 0
            || height <= 0
            || self.mouse(
                simulation,
                Vec2::new(
                    (x as f32 - width as f32 * 0.5) * 720.0 / height as f32,
                    (height as f32 * 0.5 - y as f32) * 720.0 / height as f32,
                ),
            )
    }
}
pub(crate) fn update(
    mut capture: ResMut<Capture>,
    simulation: Option<Res<Simulation>>,
    upload: Option<Res<ship_upload::SourceShipUpload>>,
    ui: Option<Res<ShipUploadUiState>>,
    layout: Option<Res<ship_upload_layout::Layout>>,
) {
    let open = upload.is_some_and(|upload| upload.window_open());
    capture.keyboard = simulation.is_some_and(|simulation| simulation.ship_search_active)
        || open && ui.is_some_and(|ui| ui.focus.is_some());
    capture.editor = open.then(|| {
        layout.map_or(Rect::from_center_size(Vec2::ZERO,Vec2::new(560.0,700.0)),|layout|layout.window_rect())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Simulation>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ShipUploadUiState>()
            .init_resource::<ship_upload_layout::Layout>()
            .init_resource::<camera_control::CameraControlState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<bevy::input::mouse::MouseWheel>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_message::<bevy::window::WindowEvent>()
            .add_message::<bevy::app::AppExit>()
            .add_plugins(window_bevy::WindowBridgePlugin)
            .add_systems(
                Update,
                (handle_controls, camera_control::handle_camera_control).chain(),
            );
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (1280, 720).into(),
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((
                WorldCamera,
                Projection::Orthographic(OrthographicProjection::default_2d()),
                Transform::default(),
            ))
            .id();
        app.update();
        (app, window, camera)
    }
    fn packet(event: window::WindowEvent, cursor: [f64; 2]) -> window_bevy::OrderedInput {
        window_bevy::OrderedInput {
            event,
            cursor,
            screen: [1280, 720],
            frame_screen: [1280, 720],
        }
    }
    #[test]
    fn editor_text_capture_prevents_tool_shortcuts_and_camera_key_zoom_then_releases() {
        let (mut app, _, camera) = app();
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        app.world_mut().resource_mut::<ShipUploadUiState>().focus = Some(ShipUploadField::Name);
        let original = app.world().get::<Projection>(camera).unwrap().clone();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit2);
        app.world_mut().write_message(packet(
            window::WindowEvent::Key {
                key: 61,
                scancode: 13,
                action: 1,
                mods: 0,
            },
            [1000.0, 300.0],
        ));
        app.update();
        assert!(app.world().resource::<Simulation>().tool == Tool::None);
        let Projection::Orthographic(before) = original else {
            panic!()
        };
        let Projection::Orthographic(after) = app.world().get::<Projection>(camera).unwrap() else {
            panic!()
        };
        assert_eq!(
            before.scale, after.scale,
            "Editor text must capture camera keys outside the editor too"
        );
        app.world_mut().resource_mut::<ShipUploadUiState>().focus = None;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit2);
        app.world_mut().write_message(packet(
            window::WindowEvent::Key {
                key: 61,
                scancode: 13,
                action: 1,
                mods: 0,
            },
            [1000.0, 300.0],
        ));
        app.update();
        assert!(app.world().resource::<Simulation>().tool == Tool::Flood);
        let Projection::Orthographic(after) = app.world().get::<Projection>(camera).unwrap() else {
            panic!()
        };
        assert_ne!(before.scale, after.scale);
    }
    #[test]
    fn ordinary_editor_hover_captures_camera_and_brush_but_outside_scene_still_works() {
        let (mut app, _, camera) = app();
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        let before = *app.world().get::<Transform>(camera).unwrap();
        for event in [
            packet(
                window::WindowEvent::MouseButton {
                    button: 0,
                    action: 1,
                    mods: 0,
                },
                [640.0, 360.0],
            ),
            packet(
                window::WindowEvent::MouseButton {
                    button: 1,
                    action: 1,
                    mods: 0,
                },
                [640.0, 360.0],
            ),
            packet(
                window::WindowEvent::CursorPos {
                    xpos: 700.0,
                    ypos: 360.0,
                },
                [700.0, 360.0],
            ),
            packet(
                window::WindowEvent::Scroll { x: 0.0, y: 1.0 },
                [700.0, 360.0],
            ),
        ] {
            app.world_mut().write_message(event);
        }
        app.update();
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), before);
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(
                !app.world()
                    .resource::<tools::tool::NativeBrushInput>()
                    .active(tool),
                "Editor click must not activate any source brush"
            );
        }
        let mut messages =
            bevy::ecs::message::MessageCursor::<camera_control::WorldInput>::default();
        let captured: Vec<_> = messages
            .read(
                app.world()
                    .resource::<Messages<camera_control::WorldInput>>(),
            )
            .collect();
        assert_eq!(captured.len(), 4);
        assert!(captured.iter().all(|event| event.blocked));
        for event in [
            packet(
                window::WindowEvent::MouseButton {
                    button: 0,
                    action: 1,
                    mods: 0,
                },
                [1100.0, 300.0],
            ),
            packet(
                window::WindowEvent::MouseButton {
                    button: 1,
                    action: 1,
                    mods: 0,
                },
                [1100.0, 300.0],
            ),
            packet(
                window::WindowEvent::CursorPos {
                    xpos: 1200.0,
                    ypos: 350.0,
                },
                [1200.0, 350.0],
            ),
            packet(
                window::WindowEvent::MouseButton {
                    button: 1,
                    action: 0,
                    mods: 0,
                },
                [1200.0, 350.0],
            ),
        ] {
            app.world_mut().write_message(event);
        }
        app.update();
        assert_ne!(
            *app.world().get::<Transform>(camera).unwrap(),
            before,
            "Editor is an ordinary window, not a scene-wide modal"
        );
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(
                app.world()
                    .resource::<tools::tool::NativeBrushInput>()
                    .active(tool),
                "Outside click still reaches every source brush"
            );
        }
    }
    fn click(app: &mut App, window: Entity, point: Vec2) {
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(640.0 + point.x, 360.0 - point.y)));
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.reset_all();
        mouse.press(MouseButton::Left);
        app.update();
    }
    #[test]
    fn moved_editor_occludes_panel_and_search_but_uncovered_controls_remain_live() {
        let (mut app, window, _) = app();
        app.insert_resource(ShipCatalog(Vec::new(), Vec::new()))
            .insert_resource(music_player::MusicPlayer::new(Vec::new()))
            .add_systems(
                Update,
                (
                    select_tool_from_panel,
                    select_ship_layer,
                    select_toolbox_tab_and_settings,
                    select_ship_from_panel,
                )
                    .chain(),
            );
        let offset = app.world().resource::<Simulation>().tool_panel_offset;
        let icon = Vec2::new(-220.0, 250.0) + offset;
        let layer = Vec2::new(-145.0, 310.0) + offset;
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = Vec2::new(-450.0, 0.0);
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        click(&mut app, window, icon);
        assert!(app.world().resource::<Simulation>().tool == Tool::None);
        click(&mut app, window, layer);
        assert!(!app.world().resource::<Simulation>().layer_dropdown_open);
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = Vec2::new(-450.0, 0.0);
        click(&mut app, window, Vec2::new(-450.0, 339.0));
        assert!(!app.world().resource::<Simulation>().toolbox_collapsed);
        click(&mut app, window, Vec2::new(-510.0, 250.0));
        assert!(!app.world().resource::<Simulation>().ship_search_active);
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = Vec2::new(450.0, 0.0);
        click(&mut app, window, icon);
        assert!(app.world().resource::<Simulation>().tool == Tool::Break);
        click(&mut app, window, layer);
        assert!(app.world().resource::<Simulation>().layer_dropdown_open);
        click(&mut app, window, Vec2::new(-510.0, 250.0));
        assert!(app.world().resource::<Simulation>().ship_search_active);
        click(&mut app, window, Vec2::new(-450.0, 339.0));
        assert!(app.world().resource::<Simulation>().toolbox_collapsed);
    }
    #[test]
    fn captured_release_keeps_source_brush_activation_until_an_uncaptured_release() {
        let (mut app, _, _) = app();
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        app.world_mut().write_message(packet(
            window::WindowEvent::MouseButton {
                button: 0,
                action: 1,
                mods: 0,
            },
            [1100.0, 300.0],
        ));
        app.update();
        app.world_mut().write_message(packet(
            window::WindowEvent::MouseButton {
                button: 0,
                action: 0,
                mods: 0,
            },
            [640.0, 360.0],
        ));
        app.update();
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(
                app.world()
                    .resource::<tools::tool::NativeBrushInput>()
                    .active(tool)
            );
        }
        app.world_mut().write_message(packet(
            window::WindowEvent::MouseButton {
                button: 0,
                action: 0,
                mods: 0,
            },
            [1100.0, 300.0],
        ));
        app.update();
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(
                !app.world()
                    .resource::<tools::tool::NativeBrushInput>()
                    .active(tool)
            );
        }
    }

    #[test]
    fn editor_capture_only_blocks_wheel_for_covered_toolbox_settings() {
        use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
        let (mut app, window, _) = app();
        app.add_systems(Update, toolbox_viewport::scroll_settings);
        {
            let mut simulation = app.world_mut().resource_mut::<Simulation>();
            simulation.active_tab = ToolboxTab::Physics;
            simulation.settings_scroll_max[1] = 200.0;
        }
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = Vec2::new(-450.0, 0.0);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(140.0, 160.0)));
        let event = MouseWheel {
            window,
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            phase: bevy::input::touch::TouchPhase::Moved,
        };
        app.world_mut().write_message(event.clone());
        app.update();
        assert_eq!(app.world().resource::<Simulation>().settings_scroll[1], 0.0);
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = Vec2::new(450.0, 0.0);
        app.world_mut().write_message(event);
        app.update();
        assert!(app.world().resource::<Simulation>().settings_scroll[1] > 0.0);
    }
}

#[cfg(test)] mod source_geometry_tests {
    use super::*;
    #[test] fn capture_tracks_source_resize_and_releases_collapsed_body() {
        let gui=source_ui_metrics::GuiMetrics::new(1.25,[1920,1080],[1920,1080]);
        let viewport=source_ui_metrics::UiViewport::new(Vec2::new(1920.0,1080.0),Vec2::new(1536.0,864.0),UVec2::new(1920,1080)).unwrap();
        let mut layout=ship_upload_layout::Layout::default();
        layout.source_window=Some(ship_upload_layout::SourceWindow {origin:Vec2::new(100.0,100.0),size:Vec2::new(600.0,400.0),collapsed:false,gui,viewport});
        let mut app=App::new();app.init_resource::<Capture>().init_resource::<ship_upload::SourceShipUpload>()
            .insert_resource(layout).add_systems(Update,update);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();app.update();
        let body=viewport.source_to_world(Vec2::new(650.0,400.0));
        assert!(app.world().resource::<Capture>().editor_point(body));
        app.world_mut().resource_mut::<ship_upload_layout::Layout>().source_window.as_mut().unwrap().size.x=400.0;app.update();
        assert!(!app.world().resource::<Capture>().editor_point(body));
        app.world_mut().resource_mut::<ship_upload_layout::Layout>().source_window.as_mut().unwrap().collapsed=true;app.update();
        let c=app.world().resource::<Capture>();
        assert!(!c.editor_point(viewport.source_to_world(Vec2::new(200.0,200.0))));
        assert!(c.editor_point(viewport.source_to_world(Vec2::new(200.0,110.0))));
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(false);app.update();
        assert!(!app.world().resource::<Capture>().editor_point(viewport.source_to_world(Vec2::new(200.0,110.0))));
    }
}

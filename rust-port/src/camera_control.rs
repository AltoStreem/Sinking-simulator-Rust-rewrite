//! CameraControl.java translated onto the Camera2D matrix implementation.
//! Source-native initial pixel scale, resize anchors, pan and zoom. The scene
//! sea offset is applied consistently to the source world origin.
use crate::{
    Simulation, WorldCamera, camera_2d::Camera2D, point_in_tool_panel, point_in_toolbox_header,
};
use bevy::{
    input::{
        ButtonState,
        keyboard::KeyboardInput,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
};
#[derive(Resource, Default)]
pub(super) struct CameraControlState {
    camera: Option<Camera2D>,
    initial_pixels_per_unit: f32,
    dragging: bool,
    last_pos: Option<Vec2>,
}
impl CameraControlState {
    /// World bounds of the source inverse camera matrix.
    pub(crate) fn world_bounds(&self) -> Option<(Vec2, Vec2)> {
        self.camera.as_ref().map(|camera| {
            (
                camera.world_at(Vec2::splat(-1.0)),
                camera.world_at(Vec2::splat(1.0)),
            )
        })
    }
    /// Use the just-updated source matrix, not last frame's GlobalTransform.
    pub(crate) fn world_at_cursor(&self, cursor: Vec2) -> Option<Vec2> {
        self.camera.as_ref().map(|camera| {
            let size = camera.size.as_vec2();
            let clip = Vec2::new(cursor.x / size.x * 2.0 - 1.0, 1.0 - cursor.y / size.y * 2.0);
            camera.world_at(clip)
        })
    }
}
pub(crate) fn blocked(simulation: &Simulation, point: Vec2) -> bool {
    point_in_toolbox_header(point)
        || (!simulation.toolbox_collapsed
            && (-640.0..=-290.0).contains(&point.x)
            && (-350.0..=350.0).contains(&point.y))
        || (simulation.show_tools && point_in_tool_panel(point))
}
pub(super) fn handle_camera_control(
    mut wheel: MessageReader<MouseWheel>,
    mut keyboard: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    simulation: Res<Simulation>,
    windows: Query<&Window>,
    mut cameras: Query<(&mut Projection, &mut Transform), With<WorldCamera>>,
    mut control: ResMut<CameraControlState>,
) {
    let scroll: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        })
        .sum();
    let zoom_keys: Vec<_> = keyboard
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .filter_map(|event| match event.key_code {
            KeyCode::Equal | KeyCode::NumpadAdd => Some(1.0),
            KeyCode::Minus | KeyCode::NumpadSubtract => Some(-1.0),
            _ => None,
        })
        .collect();
    let Ok(window) = windows.single() else {
        return;
    };
    if window.width() <= 0.0 || window.height() <= 0.0 {
        return;
    }
    let Ok((mut projection, mut transform)) = cameras.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = &mut *projection else {
        return;
    };
    let size = UVec2::new(
        window.width().round() as u32,
        window.height().round() as u32,
    );
    if control.camera.is_none() {
        control.initial_pixels_per_unit = 16.0;
        control.camera = Some(Camera2D::from_view(
            size,
            Vec2::new(0.0, crate::SEA_LEVEL),
            16.0,
        ));
    }
    if control.camera.as_ref().unwrap().size != size {
        control.camera.as_mut().unwrap().resize(size);
    }
    let cursor = window.cursor_position().unwrap_or(size.as_vec2() * 0.5);
    let ui_scale = 720.0 / window.height();
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * ui_scale,
        (window.height() * 0.5 - cursor.y) * ui_scale,
    );
    let input_blocked = blocked(&simulation, point);
    // Shift makes the selected source tool decline a scene click, allowing CameraControl to pan.
    if mouse.just_pressed(MouseButton::Left)
        && !input_blocked
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
    {
        control.dragging = window.cursor_position().is_some();
        control.last_pos = control.dragging.then_some(cursor);
    }
    if mouse.just_released(MouseButton::Left) {
        control.dragging = false;
        control.last_pos = None;
    }
    // Source right-button press starts dragging even when a UI handler blocked it.
    if mouse.just_pressed(MouseButton::Right) {
        control.dragging = window.cursor_position().is_some();
        control.last_pos = control.dragging.then_some(cursor);
    }
    if mouse.just_released(MouseButton::Right) {
        control.dragging = false;
        control.last_pos = None;
    }
    if control.dragging {
        if let Some(previous) = control.last_pos {
            control
                .camera
                .as_mut()
                .unwrap()
                .translate(cursor.x - previous.x, previous.y - cursor.y);
            control.last_pos = Some(cursor);
        }
    } else {
        if scroll != 0.0 && window.cursor_position().is_some() && !input_blocked {
            let relative = Vec4::new(
                cursor.x / window.width() * 2.0 - 1.0,
                1.0 - cursor.y / window.height() * 2.0,
                0.0,
                1.0,
            );
            control
                .camera
                .as_mut()
                .unwrap()
                .zoom(2.0f32.powf(scroll * 0.2), relative);
        }
        if !simulation.ship_search_active {
            for notches in zoom_keys {
                control
                    .camera
                    .as_mut()
                    .unwrap()
                    .zoom(2.0f32.powf(notches * 0.2), Vec4::new(0.0, 0.0, 0.0, 1.0));
            }
            // Retain the port's advertised reset shortcut, absent from CameraControl.java.
            if keys.just_pressed(KeyCode::Digit0) {
                control.camera = Some(Camera2D::from_view(
                    size,
                    Vec2::new(0.0, crate::SEA_LEVEL),
                    control.initial_pixels_per_unit,
                ));
            }
        }
    }
    let camera = control.camera.as_ref().unwrap();
    let center = camera.world_at(Vec2::ZERO);
    ortho.scaling_mode = bevy::camera::ScalingMode::WindowSize;
    ortho.scale = 1.0 / camera.scale;
    ortho.area = Rect::from_center_size(Vec2::ZERO, size.as_vec2() / camera.scale);
    transform.translation.x = center.x;
    transform.translation.y = center.y;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_camera_active_controls_keep_pointer_anchor_and_center_key_zoom() {
        let mut app = App::new();
        app.insert_resource(Simulation::default())
            .init_resource::<CameraControlState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseWheel>()
            .add_message::<KeyboardInput>()
            .add_systems(Update, handle_camera_control);
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(1000.0, 500.0)));
        let window = app.world_mut().spawn(window).id();
        app.world_mut().spawn((
            WorldCamera,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                    viewport_height: 720.0,
                },
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
        ));
        app.update();
        let pointer = Vec2::new(1000.0, 500.0);
        let before = app
            .world()
            .resource::<CameraControlState>()
            .world_at_cursor(pointer)
            .unwrap();
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -30.0,
            window,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        let state = app.world().resource::<CameraControlState>();
        let after = state.world_at_cursor(pointer).unwrap();
        assert!(before.distance(after) < 0.01);
        let camera = state.camera.as_ref().unwrap();
        assert!(
            (camera.scale - 16.0 / 64.0).abs() < 0.00001,
            "source zoom has no four-unit clamp"
        );
        let center = camera.world_at(Vec2::ZERO);
        let scale = camera.scale;
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Equal,
            logical_key: bevy::input::keyboard::Key::Character("=".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: true,
            window,
        });
        app.update();
        let camera = app
            .world()
            .resource::<CameraControlState>()
            .camera
            .as_ref()
            .unwrap();
        assert!(camera.world_at(Vec2::ZERO).distance(center) < 0.01);
        assert!((camera.scale / scale - 2.0f32.powf(0.2)).abs() < 0.00001);
    }
    #[test]
    fn source_camera_collapsed_toolbox_only_blocks_visible_header() {
        let mut sim = Simulation::default();
        sim.show_tools = false;
        assert!(blocked(&sim, Vec2::new(-465.0, 0.0)));
        sim.toolbox_collapsed = true;
        assert!(!blocked(&sim, Vec2::new(-465.0, 0.0)));
        assert!(blocked(&sim, Vec2::new(-465.0, 333.0)));
        assert!(!blocked(&sim, Vec2::new(-700.0, 0.0)));
    }
}

/// CameraControl.java's original object and InputHandler path.
pub(crate) struct SourceCameraControl {
    last_pos: std::rc::Rc<std::cell::RefCell<[f64; 2]>>,
    dragging: bool,
    window: std::rc::Rc<crate::window::SourceWindow>,
    camera: std::rc::Rc<crate::camera_2d::SourceCamera2D>,
}
impl SourceCameraControl {
    pub fn new(window: std::rc::Rc<crate::window::SourceWindow>) -> Self {
        let size = window.screen_size();
        Self::with_camera(
            window,
            std::rc::Rc::new(crate::camera_2d::SourceCamera2D::new(size[0], size[1])),
        )
    }
    pub fn with_camera(
        window: std::rc::Rc<crate::window::SourceWindow>,
        camera: std::rc::Rc<crate::camera_2d::SourceCamera2D>,
    ) -> Self {
        Self {
            last_pos: std::rc::Rc::new(std::cell::RefCell::new([0.0, 0.0])),
            dragging: false,
            window,
            camera,
        }
    }
    pub fn last_pos(&self) -> std::rc::Rc<std::cell::RefCell<[f64; 2]>> {
        self.last_pos.clone()
    }
    pub fn dragging(&self) -> bool {
        self.dragging
    }
    pub fn window(&self) -> std::rc::Rc<crate::window::SourceWindow> {
        self.window.clone()
    }
    pub fn camera(&self) -> std::rc::Rc<crate::camera_2d::SourceCamera2D> {
        self.camera.clone()
    }
}
impl crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceCameraControl {
    fn on_size(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        height: i32,
        width: i32,
    ) -> bool {
        if width != 0 && height != 0 {
            self.camera.resize([width, height]);
        }
        blocked
    }
    fn on_scroll(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        _x: f64,
        y: f64,
    ) -> bool {
        if !self.dragging && !blocked {
            let mouse = self.window.mouse_position_relative();
            let mut relative = Vec4::new(mouse[0] as f32, (-mouse[1]) as f32, 0.0, 1.0);
            self.camera
                .scale(2.0f64.powf(y * 0.2) as f32, &mut relative);
        }
        blocked
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        button: i32,
        action: i32,
        _mods: i32,
    ) -> bool {
        match button {
            0 => {
                if action == 1 && !blocked {
                    *self.last_pos.borrow_mut() = self.window.mouse_position();
                    self.dragging = true;
                } else if action == 0 {
                    self.dragging = false;
                }
            }
            1 => {
                if action == 1 {
                    *self.last_pos.borrow_mut() = self.window.mouse_position();
                    self.dragging = true;
                } else if action == 0 {
                    self.dragging = false;
                }
            }
            _ => {}
        }
        blocked
    }
    fn on_cursor_pos(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        x: f64,
        y: f64,
    ) -> bool {
        if self.dragging {
            let last = *self.last_pos.borrow();
            self.camera
                .translate((x - last[0]) as f32, (last[1] - y) as f32);
            *self.last_pos.borrow_mut() = [x, y];
        }
        blocked
    }
    fn on_key(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        key: i32,
        _scan: i32,
        action: i32,
        _mods: i32,
    ) -> bool {
        if !self.dragging && !blocked && (action == 1 || action == 2) {
            let power = match key {
                45 => Some(-0.2),
                61 => Some(0.2),
                _ => None,
            };
            if let Some(power) = power {
                let mut relative = Vec4::new(0.0, 0.0, 0.0, 1.0);
                self.camera.scale(2.0f64.powf(power) as f32, &mut relative);
            }
        }
        blocked
    }
}
#[cfg(test)]
mod source_handler_tests {
    use super::*;
    use crate::input_handler::InputHandler;
    use std::rc::Rc;
    #[test]
    fn source_camera_control_preserves_object_getters_and_drag_event_protocol() {
        let (window, _, log) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        log.lock().unwrap().clear();
        let mut control = SourceCameraControl::with_camera(window.clone(), camera.clone());
        assert!(log.lock().unwrap().is_empty());
        assert!(Rc::ptr_eq(&control.window(), &window));
        assert!(Rc::ptr_eq(&control.camera(), &camera));
        let last = control.last_pos();
        assert!(Rc::ptr_eq(&last, &control.last_pos()));
        assert_eq!(*last.borrow(), [0.0, 0.0]);
        assert!(!control.on_mouse_button(false, &window, 0, 1, 1)); // Shift is not checked here.
        assert!(control.dragging());
        assert_eq!(*last.borrow(), [300.0, 50.0]);
        assert!(control.on_cursor_pos(true, &window, 310.0, 60.0));
        assert_eq!(*last.borrow(), [310.0, 60.0]);
        assert!((camera.matrix.borrow().w_axis.x - 0.05).abs() < 0.000001);
        assert!((camera.matrix.borrow().w_axis.y + 0.1).abs() < 0.000001);
        assert!(control.on_mouse_button(true, &window, 1, 0, 0));
        assert!(!control.dragging());
        assert!(control.on_mouse_button(true, &window, 1, 1, 0));
        assert!(control.dragging());
        assert!(control.on_mouse_button(true, &window, 0, 1, 0));
        assert!(control.dragging());
        assert!(control.on_mouse_button(true, &window, 0, 0, 0)); // Either release clears the shared flag.
        assert!(!control.dragging());
        *last.borrow_mut() = [123.0, 456.0];
        assert_eq!(*control.last_pos().borrow(), [123.0, 456.0]);
    }
    #[test]
    fn source_camera_control_default_resize_scroll_and_key_conditions() {
        let (window, _, log) = crate::window::tests::fixture();
        log.lock().unwrap().clear();
        let mut control = SourceCameraControl::new(window.clone());
        assert_eq!(*log.lock().unwrap(), ["screen"]);
        assert_eq!(*control.camera().size.borrow(), [400, 200]);
        let camera = control.camera();
        log.lock().unwrap().clear();
        assert!(control.on_scroll(true, &window, 1.0, 2.0));
        assert!(log.lock().unwrap().is_empty());
        assert_eq!(camera.scale_value(), 16.0);
        assert!(!control.on_scroll(false, &window, 999.0, 2.0));
        assert_eq!(*log.lock().unwrap(), ["mouse", "screen"]);
        assert_eq!(camera.scale_value(), 16.0 * (2.0f64.powf(0.4) as f32));
        let initial = camera.scale_value();
        control.on_key(false, &window, 61, 0, 0, 0);
        assert_eq!(camera.scale_value(), initial);
        control.on_key(true, &window, 61, 0, 1, 0);
        assert_eq!(camera.scale_value(), initial);
        control.on_key(false, &window, 61, 0, 2, 0);
        assert_eq!(camera.scale_value(), initial * (2.0f64.powf(0.2) as f32));
        control.on_size(true, &window, 0, 100);
        assert_eq!(*camera.size.borrow(), [400, 200]);
        control.on_size(true, &window, 100, -200);
        assert_eq!(*camera.size.borrow(), [-200, 100]);
        control.on_mouse_button(true, &window, 1, 1, 0);
        let initial = camera.scale_value();
        log.lock().unwrap().clear();
        control.on_scroll(false, &window, 0.0, 1.0);
        control.on_key(false, &window, 45, 0, 1, 0);
        assert_eq!(camera.scale_value(), initial);
        assert!(log.lock().unwrap().is_empty());
        assert!(control.on_focus(true, &window, false));
        assert!(control.dragging());
    }
}

#[cfg(test)]
mod active_left_drag_tests {
    use super::*;
    #[test]
    fn active_source_camera_pans_after_shift_left_tool_decline_and_stops_on_release() {
        let mut app = App::new();
        app.insert_resource(Simulation::default())
            .init_resource::<CameraControlState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseWheel>()
            .add_message::<KeyboardInput>()
            .add_systems(Update, handle_camera_control);
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(1000.0, 500.0)));
        let window = app.world_mut().spawn(window).id();
        app.world_mut().spawn((
            WorldCamera,
            Projection::Orthographic(OrthographicProjection::default_2d()),
            Transform::default(),
        ));
        app.update();
        let before = app
            .world()
            .resource::<CameraControlState>()
            .camera
            .as_ref()
            .unwrap()
            .world_at(Vec2::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<CameraControlState>().dragging);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(1100.0, 550.0)));
        app.update();
        let after = app
            .world()
            .resource::<CameraControlState>()
            .camera
            .as_ref()
            .unwrap()
            .world_at(Vec2::ZERO);
        assert!((after - (before + Vec2::new(-6.25, 3.125))).length() < 0.001);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert!(!app.world().resource::<CameraControlState>().dragging);
    }
}

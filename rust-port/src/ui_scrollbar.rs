//! Numeric vertical-scrollbar behavior from imgui.internal.api.widgets.scrollbarEx.
//! Frame dimensions are supplied by the active renderer; exact DPI/style sizing is pending.
use crate::*;
#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub top: f32,
    pub size: f32,
    pub grab: f32,
    pub maximum: f32,
}
impl Geometry {
    pub fn new(top: f32, bottom: f32, available: f32, contents: f32, grab_min: f32) -> Self {
        let inset = (((top - bottom - 2.0) * 0.5) as i32 as f32).clamp(0.0, 3.0);
        let size = (top - bottom - 2.0 * inset).max(0.0);
        let window_size = contents.max(available).max(1.0);
        let grab = (size * (available / window_size)).max(grab_min).min(size);
        Self {
            top: top - inset,
            size,
            grab,
            maximum: (contents - available).max(1.0),
        }
    }
    fn grab_norm(self) -> f32 {
        self.grab / self.size
    }
    fn start_norm(self, scroll: f32) -> f32 {
        (scroll / self.maximum).clamp(0.0, 1.0) * (self.size - self.grab) / self.size
    }
    pub fn center(self, scroll: f32) -> f32 {
        self.top - self.start_norm(scroll) * self.size - self.grab * 0.5
    }
    fn clicked_norm(self, y: f32) -> f32 {
        ((self.top - y) / self.size).clamp(0.0, 1.0)
    }
    pub fn activate(self, scroll: f32, y: f32) -> (f32, f32) {
        let clicked = self.clicked_norm(y);
        let start = self.start_norm(scroll);
        let grab_norm = self.grab_norm();
        let absolute = clicked < start || clicked > start + grab_norm;
        let mut offset = if absolute {
            0.0
        } else {
            clicked - start - grab_norm * 0.5
        };
        let result = self.drag(y, offset);
        if absolute {
            offset = clicked - self.start_norm(result) - grab_norm * 0.5;
        }
        (result, offset)
    }
    pub fn drag(self, y: f32, offset: f32) -> f32 {
        let grab_norm = self.grab_norm();
        if self.size <= 0.0 || grab_norm >= 1.0 {
            return 0.0;
        }
        let ratio =
            ((self.clicked_norm(y) - offset - grab_norm * 0.5) / (1.0 - grab_norm)).clamp(0.0, 1.0);
        // Generic_helpersKt.round uses (value+0.5).toInt().toFloat(), not Math.round.
        ui_numeric::round(ratio * self.maximum)
    }
}
pub(crate) fn active_geometry(simulation: &Simulation) -> Geometry {
    let available = 311.0 - (toolbox_viewport::bottom(simulation) + 4.0);
    let maximum =
        simulation.settings_scroll_max[toolbox_viewport::tab_index(simulation.active_tab)];
    Geometry::new(
        311.0,
        toolbox_viewport::bottom(simulation) + 4.0,
        available,
        available + maximum,
        10.0,
    )
}
pub(crate) fn wheel_step(font_size: f32, inner_height: f32) -> f32 {
    // Generic_helpersKt.floor also truncates through the JVM integer conversion.
    ui_numeric::floor(f32::min(5.0 * font_size, 0.67 * inner_height))
}
pub(crate) fn handle(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    upload: Res<ship_upload::SourceShipUpload>,
    mut simulation: ResMut<Simulation>,
    mut dragging: Local<Option<(ToolboxTab, f32)>>,
) {
    let index = toolbox_viewport::tab_index(simulation.active_tab);
    if !mouse.pressed(MouseButton::Left)
        || upload.window_open()
        || simulation.toolbox_collapsed
        || simulation.active_tab == ToolboxTab::Ships
        || simulation.settings_scroll_max[index] <= 0.0
    {
        *dragging = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );
    let geometry = active_geometry(&simulation);
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = None;
        if (-301.0..=-293.0).contains(&point.x)
            && (geometry.top - geometry.size..=geometry.top).contains(&point.y)
        {
            let (scroll, offset) = geometry.activate(simulation.settings_scroll[index], point.y);
            simulation.settings_scroll[index] = scroll;
            *dragging = Some((simulation.active_tab, offset));
        }
    } else if let Some((tab, offset)) = *dragging {
        if tab != simulation.active_tab {
            *dragging = None;
            return;
        }
        simulation.settings_scroll[index] = geometry.drag(point.y, offset);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_wheel_step_and_modifier_rules_reach_active_scroll_state() {
        let mut app = App::new();
        let mut s = Simulation::default();
        s.active_tab = ToolboxTab::Physics;
        s.settings_scroll_max[1] = 173.0;
        app.insert_resource(s)
            .insert_resource(ship_upload::SourceShipUpload::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_message::<MouseWheel>()
            .add_systems(Update, toolbox_viewport::scroll_settings);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(126.0, 260.0)));
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll[1],
            90.0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll[1],
            90.0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ControlLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll[1],
            90.0
        );
    }
    #[test]
    fn source_thumb_offset_absolute_seek_and_rounding_share_draw_geometry() {
        let g = Geometry::new(300.0, 0.0, 300.0, 900.0, 10.0);
        assert_eq!(
            (g.top, g.size, g.grab, g.maximum),
            (297.0, 294.0, 98.0, 600.0)
        );
        let y = g.center(200.0) + 20.0;
        let (scroll, offset) = g.activate(200.0, y);
        assert_eq!(scroll, 200.0);
        assert!((g.drag(y - 49.0, offset) - 350.0).abs() <= 1.0);
        let (seek, delta) = g.activate(0.0, 10.0);
        assert_eq!(seek, 600.0);
        assert_eq!(g.drag(10.0, delta), 600.0);
        assert_eq!(wheel_step(18.0, 500.0), 90.0);
        assert_eq!(wheel_step(18.0, 40.0), 26.0);
    }
    #[test]
    fn active_thumb_drag_updates_scroll_and_modal_cancels_capture() {
        let mut app = App::new();
        let mut s = Simulation::default();
        s.active_tab = ToolboxTab::Physics;
        s.settings_scroll_max[1] = 173.0;
        app.insert_resource(s)
            .insert_resource(ship_upload::SourceShipUpload::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .add_systems(Update, handle);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        let g = active_geometry(app.world().resource::<Simulation>());
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(343.0, 360.0 - g.center(0.0))));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Simulation>().settings_scroll[1], 0.0);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(343.0, 710.0)));
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll[1],
            173.0
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(true);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(343.0, 52.0)));
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll[1],
            173.0
        );
    }
}

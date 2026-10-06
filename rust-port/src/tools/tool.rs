use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    Break,
    Dry,
    Flood,
    Move,
}

pub(crate) fn handle_ship_tool(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut activations: Local<BrushActivations>,
    windows: Query<&Window>,
    camera_state: Res<CameraControlState>,
    ships: Query<&Transform, With<ShipSprite>>,
    markers: Query<(Entity, &LeakMarker)>,
    simulation: Res<Simulation>,
    snapshot: Res<GpuShipPhysicsSnapshot>,
    mut physics: ResMut<GpuShipPhysicsAssets>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let Some(active) = activations.for_tool(simulation.tool) else {
        return;
    };
    if mouse.just_released(MouseButton::Left) {
        active.on_mouse_button(false, 0, 0, 0);
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        let on_game_view = windows
            .single()
            .ok()
            .and_then(|window| {
                window.cursor_position().map(|cursor| {
                    let scale = 720.0 / window.height().max(1.0);
                    let ui_point = Vec2::new(
                        (cursor.x - window.width() * 0.5) * scale,
                        (window.height() * 0.5 - cursor.y) * scale,
                    );
                    !camera_control::blocked(&simulation, ui_point)
                })
            })
            .unwrap_or(false);
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        active.on_mouse_button(!on_game_view, 0, 1, i32::from(shift));
    }
    if !active.active {
        return;
    }
    let (Ok(window), Ok(ship)) = (windows.single(), ships.single()) else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Some(world) = camera_state.world_at_cursor(cursor) else {
        return;
    };
    let local =
        (ship.rotation.inverse() * (world - ship.translation.truncate()).extend(0.0)).truncate();

    match simulation.tool {
        Tool::Break => {
            crate::tools::break_tool::apply(&mut physics, local, simulation.tool_size);
            // Preserve the port HUD's leak markers, without throttling source
            // destruction while dragging or imposing the old hull rectangle.
            if !markers
                .iter()
                .any(|(_, marker)| marker.0.distance(local) < simulation.tool_size * 1.6)
            {
                commands.spawn((
                    LeakMarker(local),
                    Transform::from_xyz(world.x, world.y, 3.0),
                ));
            }
        }
        Tool::Flood | Tool::Dry => tools::apply_water_brush(
            simulation.tool,
            local,
            simulation.tool_size,
            &snapshot,
            &mut physics,
            &mut buffers,
        ),
        Tool::Move => {}
    }
}

/// Tool.java's input-handler contract; ordinary callbacks inherit InputHandler defaults.
pub(crate) trait SourceTool<W>: crate::input_handler::InputHandler<W> {
    type Texture;
    fn texture(&self) -> &Self::Texture;
    fn active_texture(&self) -> &Self::Texture;
    fn name(&self) -> &str;
    fn update(&mut self);
}

/// Break/Flood/Dry hold their activation until a left-button release.
#[derive(Default)]
pub(crate) struct BrushActivation {
    pub active: bool,
}
impl BrushActivation {
    pub fn on_mouse_button(&mut self, blocked: bool, button: i32, action: i32, mods: i32) -> bool {
        if button == 0 {
            if action == 1 && !blocked && mods & 1 == 0 {
                self.active = true;
                return true;
            }
            if action == 0 {
                self.active = false;
            }
        }
        blocked
    }
}

#[derive(Default)]
pub(crate) struct BrushActivations([BrushActivation; 3]);
impl BrushActivations {
    pub fn for_tool(&mut self, tool: Tool) -> Option<&mut BrushActivation> {
        let index = match tool {
            Tool::Break => 0,
            Tool::Flood => 1,
            Tool::Dry => 2,
            Tool::Move => return None,
        };
        Some(&mut self.0[index])
    }
}
#[cfg(test)]
mod activation_tests {
    use super::*;
    #[test]
    fn each_source_brush_retains_its_own_activation_when_tool_selection_changes() {
        let mut states = BrushActivations::default();
        states
            .for_tool(Tool::Break)
            .unwrap()
            .on_mouse_button(false, 0, 1, 0);
        assert!(!states.for_tool(Tool::Flood).unwrap().active);
        assert!(!states.for_tool(Tool::Dry).unwrap().active);
        assert!(states.for_tool(Tool::Move).is_none());
        states
            .for_tool(Tool::Flood)
            .unwrap()
            .on_mouse_button(false, 0, 0, 0);
        assert!(states.for_tool(Tool::Break).unwrap().active);
        states
            .for_tool(Tool::Break)
            .unwrap()
            .on_mouse_button(true, 0, 0, 0);
        assert!(!states.for_tool(Tool::Break).unwrap().active);
    }
}

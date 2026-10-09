use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    None,
    Break,
    Dry,
    Flood,
    Move,
}

impl Tool {
    pub(crate) fn toggle(&mut self, selected: Self) {
        *self = if *self == selected {
            Self::None
        } else {
            selected
        };
    }
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
    native_window: Option<NonSend<crate::window_bevy::LiveWindow>>,
    native_brush: Option<Res<NativeBrushInput>>,
    capture:Option<Res<crate::ui_input_capture::Capture>>,
) {
    if native_window.is_none() && (mouse.just_pressed(MouseButton::Left) || mouse.just_released(MouseButton::Left)) {
        let captured = windows
            .single()
            .ok()
            .and_then(|window| {
                window.cursor_position().map(|cursor| {
                    let scale = 720.0 / window.height().max(1.0);
                    let ui_point = Vec2::new(
                        (cursor.x - window.width() * 0.5) * scale,
                        (window.height() * 0.5 - cursor.y) * scale,
                    );
                    capture.as_ref().map_or_else(||camera_control::blocked(&simulation,ui_point),|capture|capture.mouse(&simulation,ui_point))
                })
            })
            .unwrap_or(true);
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        activations.mouse_event(
            captured,
            false,
            0,
            i32::from(mouse.just_pressed(MouseButton::Left)),
            i32::from(shift),
        );
    }
    let active=if native_window.is_some() {
        native_brush.as_ref().is_some_and(|state|state.active(simulation.tool))
    } else {activations.active(simulation.tool)};
    if !active {
        return;
    }
    let (Ok(window), Ok(ship)) = (windows.single(), ships.single()) else {
        return;
    };
    let cursor=if native_window.is_some() {native_brush.as_ref().map(|state|state.states.cursor)} else {window.cursor_position()};
    let Some(cursor) = cursor else {
        return;
    };
    let world=if native_window.is_some() {
        native_brush.as_ref().and_then(|state|camera_state.world_at_brush_cursor(cursor,state.screen))
    } else {camera_state.world_at_cursor(cursor)};
    let Some(world) = world else {
        return;
    };
    let local =
        (ship.rotation.inverse() * (world - ship.translation.truncate()).extend(0.0)).truncate();
    // Source brush shaders compare the camera-world cursor to live position
    // texels. Ship's display-only drag displacement is not part of that test.
    let native_cursor = source_brush_cursor(world);

    match simulation.tool {
        Tool::Break => {
            crate::tools::break_tool::apply(&mut physics, native_cursor, simulation.tool_size);
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
            native_cursor,
            simulation.tool_size,
            &snapshot,
            &mut physics,
            &mut buffers,
        ),
        Tool::Move | Tool::None => {}
    }
}

fn source_brush_cursor(world: Vec2) -> Vec2 {
    world - Vec2::new(0.0, SEA_LEVEL)
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
pub(crate) struct BrushActivations {
    states: [BrushActivation; 3],
    cursor: Vec2,
}
impl BrushActivations {
    fn active(&self, tool: Tool) -> bool {
        match tool {Tool::Break=>self.states[0].active,Tool::Flood=>self.states[1].active,
            Tool::Dry=>self.states[2].active,Tool::Move|Tool::None=>false}
    }
    fn native_events(&mut self, events: &[crate::window_bevy::OrderedInput], simulation: &Simulation) {
        self.native_events_captured(events,simulation,None);
    }
    fn native_events_captured(&mut self, events:&[crate::window_bevy::OrderedInput],simulation:&Simulation,capture:Option<&crate::ui_input_capture::Capture>) {
        use crate::window::WindowEvent as E;
        for packet in events {
            let [width,height]=packet.screen;
            let [x,y]=packet.cursor;
            let captured=capture.is_some_and(|capture|capture.packet_mouse(simulation,packet)) || width<=0 || height<=0 || camera_control::blocked(simulation,
                Vec2::new((x as f32-width as f32*0.5)*720.0/height as f32,
                    (height as f32*0.5-y as f32)*720.0/height as f32));
            match packet.event {
                E::MouseButton {button,action,mods} => {self.mouse_event(captured,false,button,action,mods);},
                E::CursorPos {xpos,ypos} if !captured => {self.cursor=Vec2::new(xpos as f32,ypos as f32);},
                _=>{},
            }
        }
    }
    /// GUI forwards to every tool with the same blocked flag, independent of
    /// selection. Mouse capture prevents forwarding even button releases.
    pub fn mouse_event(
        &mut self,
        captured: bool,
        blocked: bool,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        if captured {
            return true;
        }
        let mut result = false;
        for activation in &mut self.states {
            result |= activation.on_mouse_button(blocked, button, action, mods);
        }
        result
    }
    pub fn for_tool(&mut self, tool: Tool) -> Option<&mut BrushActivation> {
        let index = match tool {
            Tool::Break => 0,
            Tool::Flood => 1,
            Tool::Dry => 2,
            Tool::Move | Tool::None => return None,
        };
        Some(&mut self.states[index])
    }
}
#[derive(Resource,Default)]
pub(crate) struct NativeBrushInput {
    states: BrushActivations,
    screen: [i32;2],
    initialized: bool,
}
impl NativeBrushInput {
    pub(crate) fn active(&self,tool:Tool)->bool {self.states.active(tool)}
    pub(crate) fn world(&self,camera:&CameraControlState) -> Option<Vec2> {
        camera.world_at_brush_cursor(self.states.cursor,self.screen)
    }
}
pub(crate) fn native_brush_input(mut events: MessageReader<crate::window_bevy::OrderedInput>,
    live: Option<NonSend<crate::window_bevy::LiveWindow>>, simulation: Option<Res<Simulation>>,
    mut state: ResMut<NativeBrushInput>,capture:Option<Res<crate::ui_input_capture::Capture>>) {
    let packets:Vec<_>=events.read().cloned().collect();
    let Some(live)=live else {return};
    let Some(simulation)=simulation else {return};
    if !state.initialized {
        state.screen=packets.first().map(|packet|packet.frame_screen).unwrap_or_else(||live.source.screen_size());
        state.initialized=true;
    }
    for packet in &packets {
        if let crate::window::WindowEvent::Size {width,height}=packet.event {state.screen=[width,height];}
    }
    state.states.native_events_captured(&packets,&simulation,capture.as_deref());
}
#[cfg(test)]
mod activation_tests {
    use super::*;
    fn packet(event: crate::window::WindowEvent, cursor: [f64;2]) -> crate::window_bevy::OrderedInput {
        crate::window_bevy::OrderedInput {event,cursor,screen:[1280,720],frame_screen:[1280,720]}
    }
    #[test]
    fn native_brush_preserves_event_modifiers_and_capture_retained_cursor_and_release() {
        use crate::window::WindowEvent as E;
        let mut states=BrushActivations::default();
        let simulation=Simulation::default();
        states.native_events(&[
            packet(E::CursorPos {xpos:1000.0,ypos:300.0},[1000.0,300.0]),
            packet(E::MouseButton {button:0,action:1,mods:1},[1000.0,300.0]),
        ],&simulation);
        assert!(!states.for_tool(Tool::Break).unwrap().active);
        states.native_events(&[
            packet(E::MouseButton {button:0,action:1,mods:0},[1000.0,300.0]),
            packet(E::CursorPos {xpos:100.0,ypos:200.0},[100.0,200.0]),
            packet(E::MouseButton {button:0,action:0,mods:0},[100.0,200.0]),
        ],&simulation);
        assert_eq!(states.cursor,Vec2::new(1000.0,300.0));
        for tool in [Tool::Break,Tool::Flood,Tool::Dry] {assert!(states.for_tool(tool).unwrap().active);}
        states.native_events(&[
            packet(E::MouseButton {button:0,action:0,mods:0},[1000.0,300.0]),
            packet(E::MouseButton {button:0,action:1,mods:0},[1000.0,300.0]),
            packet(E::MouseButton {button:0,action:0,mods:0},[1000.0,300.0]),
        ],&simulation);
        for tool in [Tool::Break,Tool::Flood,Tool::Dry] {assert!(!states.for_tool(tool).unwrap().active);}
    }
    #[test]
    fn selected_tool_toggles_to_the_source_null_slot() {
        let mut selected = Tool::None;
        for tool in [Tool::Break, Tool::Flood, Tool::Dry, Tool::Move] {
            selected.toggle(tool);
            assert!(selected == tool);
            selected.toggle(tool);
            assert!(selected == Tool::None);
        }
        selected.toggle(Tool::Flood);
        selected.toggle(Tool::Dry);
        assert!(selected == Tool::Dry);
        assert!(BrushActivations::default().for_tool(Tool::None).is_none());
    }
    #[test]
    fn source_brush_uses_live_position_coordinates_without_display_drag_offset() {
        assert_eq!(
            source_brush_cursor(Vec2::new(12.0, SEA_LEVEL + 34.0)),
            Vec2::new(12.0, 34.0)
        );
    }
    #[test]
    fn broadcast_allows_held_tool_switch_and_ui_capture_retains_activation() {
        let mut states = BrushActivations::default();
        assert!(states.mouse_event(false, false, 0, 1, 0));
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(states.for_tool(tool).unwrap().active);
        }
        assert!(states.mouse_event(true, false, 0, 0, 0));
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(states.for_tool(tool).unwrap().active);
        }
        assert!(!states.mouse_event(false, false, 0, 0, 0));
        for tool in [Tool::Break, Tool::Flood, Tool::Dry] {
            assert!(!states.for_tool(tool).unwrap().active);
        }
        assert!(states.mouse_event(false, true, 0, 1, 0));
        assert!(!states.for_tool(Tool::Break).unwrap().active);
        assert!(!states.mouse_event(false, false, 0, 1, 1));
        assert!(!states.for_tool(Tool::Flood).unwrap().active);
    }
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

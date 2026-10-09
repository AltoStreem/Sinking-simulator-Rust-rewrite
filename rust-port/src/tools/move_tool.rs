//! MoveTool.java and Ship's drag event protocol, adapted to Bevy input.
use crate::*;

#[derive(Resource)]
pub(crate) struct MoveDragState {
    pub(crate) dragging: bool,
    events: crate::ship::SourceShipEvents,
    pub(crate) offset: Vec2,
    pub(crate) pending_translation: Vec2,
}
impl Default for MoveDragState {
    fn default() -> Self {
        let events = crate::ship::SourceShipEvents::new(Mat4::IDENTITY, [1, 1]);
        Self {
            dragging: true,
            events,
            offset: Vec2::ZERO,
            pending_translation: Vec2::ZERO,
        }
    }
}
impl MoveDragState {
    fn press(&mut self, cursor: Vec2, blocked: bool, shift: bool) {
        if !blocked && !shift {
            self.dragging = true;
            self.events.cursor = (cursor - Vec2::new(0.0, SEA_LEVEL)).extend(0.0).extend(1.0);
            self.events.start_drag();
        }
    }
    fn cursor(&mut self, cursor: Vec2, blocked: bool) {
        let preview = &mut self.offset;
        self.events.on_world_cursor(
            blocked,
            (cursor - Vec2::new(0.0, SEA_LEVEL)).extend(0.0).extend(1.0),
            |offset| *preview = offset,
            |_| {},
        );
    }
    pub(crate) fn release(&mut self, blocked: bool) {
        let pending = &mut self.pending_translation;
        let preview = &mut self.offset;
        self.events.on_mouse_button(
            blocked,
            0,
            0,
            |offset| *pending += offset,
            |offset| *preview = offset,
            |_| {},
        );
        self.dragging = !self.events.moving;
    }
}

pub(crate) fn handle_ship_move(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_state: Res<CameraControlState>,
    simulation: Res<Simulation>,
    mut structure: ResMut<ShipStructure>,
    mut drag: ResMut<MoveDragState>,
    physics: Res<GpuShipPhysicsAssets>,
    mut previous_ship: Local<Option<AssetId<ShaderBuffer>>>,
    mut activation: Local<MoveActivation>,
    native_window: Option<NonSend<crate::window_bevy::LiveWindow>>,
    world_input: Option<Res<Messages<crate::camera_control::WorldInput>>>,
    mut input_cursor: Local<bevy::ecs::message::MessageCursor<crate::camera_control::WorldInput>>,
    capture:Option<Res<crate::ui_input_capture::Capture>>,
) {
    let events: Vec<_> = world_input.as_deref().map(|messages|input_cursor.read(messages).cloned().collect()).unwrap_or_default();
    let ship_id = physics.positions.id();
    if previous_ship.as_ref() != Some(&ship_id) {
        *drag = MoveDragState::default();
        *previous_ship = Some(ship_id);
    }
    if native_window.is_some() {
        process_native_events(&events,simulation.tool,&mut drag,&mut activation);
        let previous_offset=structure.manual_offset;
        structure.motion_position+=drag.offset-previous_offset;
        structure.manual_offset=drag.offset;
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let ui_scale = 720.0 / window.height().max(1.0);
    let ui_point = Vec2::new(
        (cursor.x - window.width() * 0.5) * ui_scale,
        (window.height() * 0.5 - cursor.y) * ui_scale,
    );
    let blocked = capture.as_ref().map_or_else(||camera_control::blocked(&simulation,ui_point),|capture|capture.mouse(&simulation,ui_point));
    let Some(world) = camera_state.world_at_cursor(cursor) else {
        return;
    };
    // Source MoveTool begins on any unblocked scene click, with no hull hit test
    // and no pause restriction. Ship owns the ongoing drag, even if tool changes.
    if !blocked {
        if mouse.just_pressed(MouseButton::Left) {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            activation.on_mouse_button(blocked, 0, 1, i32::from(shift));
        }
        if mouse.just_released(MouseButton::Left) {
            activation.on_mouse_button(blocked, 0, 0, 0);
        }
    }
    if simulation.tool == Tool::Move {
        activation.update(|| drag.press(world, false, false));
    }
    drag.cursor(world, blocked);
    if mouse.just_released(MouseButton::Left) {
        drag.release(blocked);
    }
    // Display-only displacement while physics is frozen. The release delta is
    // committed to the GPU position buffer, preserving point velocities.
    let previous_offset = structure.manual_offset;
    structure.motion_position += drag.offset - previous_offset;
    structure.manual_offset = drag.offset;
}

fn process_native_events(events: &[crate::camera_control::WorldInput], tool: Tool,
    drag: &mut MoveDragState, activation: &mut MoveActivation) {
    use crate::window::WindowEvent as E;
    for input in events {
        match input.event {
            E::CursorPos {..} => drag.cursor(input.world,input.blocked),
            E::MouseButton {button,action,mods} => {
                // GUI broadcasts to all tools; capture prevents forwarding.
                if !input.blocked {activation.on_mouse_button(false,button,action,mods);}
                if button==0 && action==0 {drag.release(input.blocked);}
            },
            _=>{},
        }
    }
    // Source GUI updates MoveTool after event polling. Press followed by release
    // in one poll cancels activation; startDrag uses Ship's retained cursor.
    if tool==Tool::Move {activation.update(|| {
        drag.dragging=true;
        drag.events.start_drag();
    });}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(event: crate::window::WindowEvent, world: Vec2, blocked: bool) -> crate::camera_control::WorldInput {
        crate::camera_control::WorldInput {event,world,blocked}
    }
    fn cursor(world: Vec2, blocked: bool) -> crate::camera_control::WorldInput {
        input(crate::window::WindowEvent::CursorPos {xpos:0.0,ypos:0.0},world,blocked)
    }
    fn button(action: i32, blocked: bool) -> crate::camera_control::WorldInput {
        input(crate::window::WindowEvent::MouseButton {button:0,action,mods:0},Vec2::ZERO,blocked)
    }
    #[test]
    fn native_move_receives_unselected_presses_but_captured_release_does_not_cancel_them() {
        let mut drag=MoveDragState::default();drag.release(false);
        let mut activation=MoveActivation::default();
        process_native_events(&[cursor(Vec2::new(4.0,SEA_LEVEL+6.0),false),button(1,false),
            button(0,true)],Tool::None,&mut drag,&mut activation);
        assert!(activation.active);
        assert!(!drag.dragging);
        process_native_events(&[],Tool::Move,&mut drag,&mut activation);
        assert!(drag.dragging);
        assert!(!activation.active);
        assert_eq!(drag.events.base,Vec4::new(-4.0,-6.0,0.0,-1.0));
    }
    #[test]
    fn native_move_poll_press_release_cancels_activation_before_tool_update() {
        let mut drag=MoveDragState::default();drag.release(false);
        let mut activation=MoveActivation::default();
        process_native_events(&[cursor(Vec2::new(3.0,SEA_LEVEL+4.0),false),
            button(1,false),button(0,false)],Tool::Move,&mut drag,&mut activation);
        assert!(!drag.dragging);
        assert!(drag.events.moving);
        assert!(!activation.active);
        assert_eq!(drag.pending_translation,Vec2::ZERO);
    }
    #[test]
    fn native_move_commits_release_before_later_cursor_and_keeps_blocked_preview() {
        let mut drag=MoveDragState::default();drag.release(false);
        let mut activation=MoveActivation::default();
        let start=Vec2::new(20.0,SEA_LEVEL+10.0);
        process_native_events(&[cursor(start,false),button(1,false)],Tool::Move,&mut drag,&mut activation);
        assert!(drag.dragging);
        assert_eq!(drag.events.base,Vec4::new(-20.0,-10.0,0.0,-1.0));
        process_native_events(&[cursor(start+Vec2::new(7.0,9.0),false),
            button(0,true),cursor(start+Vec2::splat(100.0),true)],Tool::Move,&mut drag,&mut activation);
        assert!(drag.dragging);
        assert_eq!(drag.offset,Vec2::new(7.0,9.0));
        process_native_events(&[button(0,false),cursor(start+Vec2::splat(200.0),false)],Tool::None,&mut drag,&mut activation);
        assert_eq!(drag.pending_translation,Vec2::new(7.0,9.0));
        assert_eq!(drag.offset,Vec2::ZERO);
        assert!(!drag.dragging);
    }

    #[test]
    fn bevy_drag_reuses_source_blocked_cursor_and_retains_preview_on_restart() {
        let mut drag = MoveDragState::default();
        drag.release(false);
        drag.press(Vec2::ZERO, false, false);
        drag.cursor(Vec2::new(3.0, 4.0), false);
        drag.cursor(Vec2::new(20.0, 30.0), true);
        assert_eq!(drag.offset, Vec2::new(3.0, 4.0));
        assert_eq!(
            drag.events.cursor,
            Vec4::new(20.0, 30.0 - SEA_LEVEL, 0.0, 1.0)
        );
        drag.press(Vec2::new(20.0, 30.0), false, false);
        assert_eq!(drag.offset, Vec2::new(3.0, 4.0));
        drag.cursor(Vec2::new(22.0, 35.0), false);
        drag.release(false);
        assert_eq!(drag.pending_translation, Vec2::new(2.0, 5.0));
        assert_eq!(
            drag.events.offset.truncate().truncate(),
            Vec2::new(2.0, 5.0)
        );
        assert_eq!(drag.offset, Vec2::ZERO);
        assert!(drag.events.moving);
    }
    #[test]
    fn move_preview_commits_once_on_release() {
        let mut d = MoveDragState::default();
        d.release(false);
        d.press(Vec2::new(100.0, 20.0), false, false);
        d.cursor(Vec2::new(120.0, 50.0), false);
        assert_eq!(d.offset, Vec2::new(20.0, 30.0));
        assert_eq!(d.pending_translation, Vec2::ZERO);
        d.release(false);
        assert!(!d.dragging);
        assert_eq!(d.offset, Vec2::ZERO);
        assert_eq!(d.pending_translation, Vec2::new(20.0, 30.0));
        d.release(false);
        assert_eq!(d.pending_translation, Vec2::new(20.0, 30.0));
    }
    #[test]
    fn blocked_events_and_shift_match_source() {
        let mut d = MoveDragState::default();
        d.release(false);
        d.press(Vec2::ZERO, true, false);
        assert!(!d.dragging);
        d.press(Vec2::ZERO, false, true);
        assert!(!d.dragging);
        d.press(Vec2::ZERO, false, false);
        d.cursor(Vec2::ONE, false);
        d.cursor(Vec2::splat(10.0), true);
        assert_eq!(d.offset, Vec2::ONE);
        d.release(true);
        assert!(d.dragging);
        d.release(false);
        assert_eq!(d.pending_translation, Vec2::ONE);
    }

    #[test]
    fn newly_loaded_ship_follows_cursor_until_unblocked_release_without_move_tool() {
        let mut drag = MoveDragState::default();
        assert!(drag.dragging);
        assert!(!drag.events.moving);
        drag.cursor(Vec2::new(40.0, SEA_LEVEL + 25.0), false);
        assert_eq!(drag.offset, Vec2::new(40.0, 25.0));
        drag.release(true);
        assert!(drag.dragging);
        assert_eq!(drag.pending_translation, Vec2::ZERO);
        drag.release(false);
        assert!(!drag.dragging);
        assert_eq!(drag.pending_translation, Vec2::new(40.0, 25.0));
        assert_eq!(drag.offset, Vec2::ZERO);
    }
}

/// MoveTool's pending click is separate from Ship's ongoing drag state.
#[derive(Default)]
pub(crate) struct MoveActivation {
    active: bool,
}
impl MoveActivation {
    pub fn on_mouse_button(&mut self, blocked: bool, button: i32, action: i32, mods: i32) -> bool {
        if button == 0 {
            if action == 1 && !blocked && mods & 1 == 0 {
                self.active = true;
                return true;
            }
            self.active = false;
        }
        blocked
    }
    pub fn update(&mut self, mut start_drag: impl FnMut()) {
        if self.active {
            self.active = false;
            start_drag();
        }
    }
}

pub(crate) struct SourceMoveTool<T> {
    texture: T,
    active_texture: T,
    activation: MoveActivation,
    start_drag: Box<dyn FnMut()>,
}
impl<T> SourceMoveTool<T> {
    fn load(
        mut load: impl FnMut(&str, bool) -> Result<T, String>,
        start_drag: impl FnMut() + 'static,
    ) -> Result<Self, String> {
        let texture = load("icons/Move.png", false)?;
        let active_texture = load("icons/Move2.png", true)?;
        Ok(Self {
            texture,
            active_texture,
            activation: MoveActivation::default(),
            start_drag: Box::new(start_drag),
        })
    }
    pub fn update(&mut self) {
        self.activation.update(&mut self.start_drag);
    }
}
impl SourceMoveTool<crate::texture_2d::SourceTexture2D> {
    /// Main.globalShip is read only after update consumes the pending click.
    /// A ship replacement between press and update must receive startDrag.
    pub fn with_current_ship(
        reader: &crate::file_reader::FileReader,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        mut current_ship: impl FnMut()
            -> std::rc::Rc<std::cell::RefCell<crate::source_ship::SourceShip>>
        + 'static,
    ) -> Result<Self, String> {
        Self::new(reader, backend, context, runtime, move || {
            current_ship().borrow().start_drag();
        })
    }
    pub fn new(
        reader: &crate::file_reader::FileReader,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        start_drag: impl FnMut() + 'static,
    ) -> Result<Self, String> {
        Self::load(
            |path, active| {
                let image = reader.read_image(std::path::Path::new(path), 0)?;
                let configure: crate::texture::Configure = if active {
                    std::sync::Arc::new(super::move_tool_active_texture::configure)
                } else {
                    std::sync::Arc::new(super::move_tool_texture::configure)
                };
                Ok(crate::texture_2d::SourceTexture2D::from_image(
                    &image,
                    32856,
                    true,
                    configure,
                    backend.clone(),
                    context.clone(),
                    runtime,
                ))
            },
            start_drag,
        )
    }
}
impl<T> crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceMoveTool<T> {
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        self.activation
            .on_mouse_button(blocked, button, action, mods)
    }
}
impl<T> super::tool::SourceTool<crate::window::SourceWindow> for SourceMoveTool<T> {
    type Texture = T;
    fn texture(&self) -> &T {
        &self.texture
    }
    fn active_texture(&self) -> &T {
        &self.active_texture
    }
    fn name(&self) -> &str {
        "Move"
    }
    fn update(&mut self) {
        SourceMoveTool::update(self);
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;

    use crate::tools::tool::SourceTool;
    use std::{cell::Cell, rc::Rc};
    #[test]
    fn source_move_constructor_and_click_protocol() {
        let mut loads = vec![];
        let calls = Rc::new(Cell::new(0));
        let target = calls.clone();
        let mut tool = SourceMoveTool::load(
            |path, active| {
                loads.push((path.to_owned(), active));
                Ok(loads.len())
            },
            move || target.set(target.get() + 1),
        )
        .unwrap();
        assert_eq!(
            loads,
            [
                ("icons/Move.png".into(), false),
                ("icons/Move2.png".into(), true)
            ]
        );
        assert_eq!(
            <SourceMoveTool<usize> as SourceTool<crate::window::SourceWindow>>::texture(&tool),
            &1
        );
        assert_eq!(
            <SourceMoveTool<usize> as SourceTool<crate::window::SourceWindow>>::active_texture(
                &tool
            ),
            &2
        );
        assert_eq!(
            <SourceMoveTool<usize> as SourceTool<crate::window::SourceWindow>>::name(&tool),
            "Move"
        );

        assert!(tool.activation.on_mouse_button(false, 0, 1, 2)); // Control is allowed; only Shift excluded.
        assert_eq!(calls.get(), 0);
        <SourceMoveTool<usize> as SourceTool<crate::window::SourceWindow>>::update(&mut tool);
        tool.update();
        assert_eq!(calls.get(), 1);
        assert!(tool.activation.on_mouse_button(false, 0, 1, 0));
        assert!(!tool.activation.on_mouse_button(false, 0, 0, 0));
        tool.update();
        assert_eq!(calls.get(), 1);
        assert!(!tool.activation.on_mouse_button(false, 0, 1, 1));
        assert!(tool.activation.on_mouse_button(true, 0, 1, 0));
        tool.update();
        assert_eq!(calls.get(), 1);
        assert!(tool.activation.on_mouse_button(false, 0, 1, 0));
        assert!(!tool.activation.on_mouse_button(false, 1, 0, 0)); // Other buttons do not clear active.
        tool.update();
        assert_eq!(calls.get(), 2);
    }
    #[test]
    fn source_move_consumes_activation_before_start_drag_panics() {
        let mut state = MoveActivation::default();
        state.on_mouse_button(false, 0, 1, 0);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || state.update(|| panic!("ship"))
            ))
            .is_err()
        );
        state.update(|| panic!("activation was not consumed"));
        let mut reads = 0;
        let tool = SourceMoveTool::<()>::load(
            |_, _| {
                reads += 1;
                Err("decode".into())
            },
            || {},
        );
        assert!(tool.is_err());
        assert_eq!(reads, 1);
    }
}

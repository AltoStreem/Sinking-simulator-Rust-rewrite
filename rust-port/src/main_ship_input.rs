//! Main.java registers [GUI, CameraControl] and the retained shipList as separate stacks.
//! Ship overrides size/cursor/mouse; its other callbacks retain InputHandler defaults.
use crate::{
    input_handler::InputHandler,
    main_globals::ShipList,
    window::{Handler, SourceWindow},
};
use std::{cell::RefCell, rc::Rc};
pub(crate) struct ShipListInput(pub ShipList);
impl InputHandler<SourceWindow> for ShipListInput {
    fn on_size(&mut self, mut blocked: bool, win: &SourceWindow, height: i32, width: i32) -> bool {
        for ship in self.0.borrow().iter() {
            blocked = ship.borrow_mut().on_size(blocked, win, height, width);
        }
        blocked
    }
    fn on_cursor_pos(&mut self, mut blocked: bool, win: &SourceWindow, x: f64, y: f64) -> bool {
        for ship in self.0.borrow().iter() {
            blocked = ship.borrow_mut().on_cursor_pos(blocked, win, x, y);
        }
        blocked
    }
    fn on_mouse_button(
        &mut self,
        mut blocked: bool,
        win: &SourceWindow,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        for ship in self.0.borrow().iter() {
            blocked = ship
                .borrow_mut()
                .on_mouse_button(blocked, win, button, action, mods);
        }
        blocked
    }
}
pub(crate) fn register(window: &SourceWindow, gui: Handler, camera: Handler, ships: ShipList) {
    let list: Handler = Rc::new(RefCell::new(ShipListInput(ships)));
    window
        .handler_stacks
        .borrow_mut()
        .extend([vec![gui, camera], vec![list]]);
}

//! Window$$special$$inlined$blockGlfwSafe$lambda$12.class.
use super::SourceWindow;

pub(super) fn invoke(window: &SourceWindow, xpos: f64, ypos: f64) {
    for callback in window.cursor_pos_callbacks.borrow().iter() {
        callback(window, xpos, ypos);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_cursor_pos(blocked, window, xpos, ypos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_handler::InputHandler;
    use std::{cell::RefCell, rc::Rc};
    struct Blocker;
    impl InputHandler<SourceWindow> for Blocker {
        fn on_mouse_button(&mut self, _: bool, _: &SourceWindow, _: i32, _: i32, _: i32) -> bool {
            true
        }
        fn on_cursor_pos(&mut self, _: bool, _: &SourceWindow, _: f64, _: f64) -> bool {
            true
        }
    }
    #[test]
    fn real_camera_right_drag_reaches_handler_even_when_preceding_handler_blocks() {
        let (window, _runtime, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        window
            .handler_stacks
            .borrow_mut()
            .push(vec![Rc::new(RefCell::new(Blocker)), control.clone()]);
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        window
            .cursor_pos_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, x, y| {
                log.borrow_mut().push((x, y));
            }));
        window.emit_mouse_button(1, 1, 0);
        assert!(control.borrow().dragging());
        window.emit_cursor_pos(310.0, 60.0);
        assert_eq!(*events.borrow(), [(310.0, 60.0)]);
        assert_eq!(*control.borrow().last_pos().borrow(), [310.0, 60.0]);
        let expected = crate::camera_2d::SourceCamera2D::new(400, 200);
        expected.translate(10.0, -10.0);
        assert_eq!(*camera.matrix.borrow(), *expected.matrix.borrow());
        window.emit_mouse_button(1, 0, 0);
        assert!(!control.borrow().dragging());
        window.emit_cursor_pos(400.0, 80.0);
        assert_eq!(*camera.matrix.borrow(), *expected.matrix.borrow());
        assert_eq!(*control.borrow().last_pos().borrow(), [310.0, 60.0]);
    }
}

//! Window$$special$$inlined$blockGlfwSafe$lambda$11.class.
use super::SourceWindow;

pub(super) fn invoke(window: &SourceWindow, button: i32, action: i32, mods: i32) {
    for callback in window.mouse_button_callbacks.borrow().iter() {
        callback(window, button, action, mods);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_mouse_button(blocked, window, button, action, mods);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_handler::InputHandler;
    use std::{cell::RefCell, rc::Rc};
    struct Handler {
        id: usize,
        result: bool,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl InputHandler<SourceWindow> for Handler {
        fn on_mouse_button(
            &mut self,
            blocked: bool,
            _: &SourceWindow,
            button: i32,
            action: i32,
            mods: i32,
        ) -> bool {
            self.log
                .borrow_mut()
                .push(format!("{}:{blocked}:{button}:{action}:{mods}", self.id));
            self.result
        }
    }
    #[test]
    fn raw_payload_and_handler_fold_are_preserved_for_press_and_release() {
        let (window, _runtime, _) = crate::window::tests::fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        let output = log.clone();
        window
            .mouse_button_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, b, a, m| {
                output.borrow_mut().push(format!("callback:{b}:{a}:{m}"));
            }));
        let handler = |id, result| -> super::super::Handler {
            Rc::new(RefCell::new(Handler {
                id,
                result,
                log: log.clone(),
            }))
        };
        window.handler_stacks.borrow_mut().extend([
            vec![handler(1, true), handler(2, false), handler(3, true)],
            vec![handler(4, false)],
        ]);
        for (button, action, mods) in [(1, 1, 0), (1, 0, 3), (i32::MIN, -1, i32::MAX)] {
            log.borrow_mut().clear();
            window.dispatch(super::super::WindowEvent::MouseButton {
                button,
                action,
                mods,
            });
            assert_eq!(
                *log.borrow(),
                vec![
                    format!("callback:{button}:{action}:{mods}"),
                    format!("1:false:{button}:{action}:{mods}"),
                    format!("2:true:{button}:{action}:{mods}"),
                    format!("3:false:{button}:{action}:{mods}"),
                    format!("4:false:{button}:{action}:{mods}"),
                ]
            );
        }
    }
    #[test]
    fn callback_failure_stops_handler_dispatch() {
        let (window, _runtime, _) = crate::window::tests::fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        window
            .handler_stacks
            .borrow_mut()
            .push(vec![Rc::new(RefCell::new(Handler {
                id: 1,
                result: false,
                log: log.clone(),
            }))]);
        window
            .mouse_button_callbacks
            .borrow_mut()
            .push(Rc::new(|_, _, _, _| panic!("callback failed")));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || window.emit_mouse_button(1, 1, 0)
            ))
            .is_err()
        );
        assert!(log.borrow().is_empty());
    }
}

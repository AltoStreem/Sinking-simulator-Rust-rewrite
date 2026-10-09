//! Window$$special$$inlined$blockGlfwSafe$lambda$14.class.
//! Raw GLFW handle is ignored; callbacks precede each independent handler fold.
use super::SourceWindow;

pub(super) fn invoke(window: &SourceWindow, x: f64, y: f64) {
    for callback in window.scroll_callbacks.borrow().iter() {
        callback(window, x, y);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_scroll(blocked, window, x, y);
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
        fn on_scroll(&mut self, blocked: bool, _: &SourceWindow, x: f64, y: f64) -> bool {
            self.log.borrow_mut().push(format!(
                "{}:{blocked}:{}:{}",
                self.id,
                x.to_bits(),
                y.to_bits()
            ));
            self.result
        }
    }
    #[test]
    fn callbacks_precede_folds_and_false_clears_the_previous_block() {
        let (window, _runtime, _) = crate::window::tests::fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        let output = log.clone();
        window
            .scroll_callbacks
            .borrow_mut()
            .push(Rc::new(move |win, _, _| {
                output.borrow_mut().push("callback".into());
                // Source callbacks may install handlers before stack iteration.
                if win.handler_stacks.borrow().is_empty() {
                    let handler = |id, result| -> super::super::Handler {
                        Rc::new(RefCell::new(Handler {
                            id,
                            result,
                            log: output.clone(),
                        }))
                    };
                    win.handler_stacks.borrow_mut().extend([
                        vec![handler(1, true), handler(2, false), handler(3, true)],
                        vec![handler(4, false)],
                    ]);
                }
            }));
        let x = -0.0_f64;
        let y = f64::INFINITY;
        window.emit_scroll(x, y);
        assert_eq!(
            *log.borrow(),
            vec![
                "callback".to_owned(),
                format!("1:false:{}:{}", x.to_bits(), y.to_bits()),
                format!("2:true:{}:{}", x.to_bits(), y.to_bits()),
                format!("3:false:{}:{}", x.to_bits(), y.to_bits()),
                format!("4:false:{}:{}", x.to_bits(), y.to_bits()),
            ]
        );
    }
}

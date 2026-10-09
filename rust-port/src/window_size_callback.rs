//! Window$$special$$inlined$blockGlfwSafe$lambda$2.class.
//! Native callbacks use width,height; InputHandler.onSize uses height,width.
use super::SourceWindow;
pub(super) fn invoke(window: &SourceWindow, width: i32, height: i32) {
    for callback in window.size_callbacks.borrow().iter() {
        callback(window, width, height);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_size(blocked, window, height, width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_handler::InputHandler;
    use std::{cell::RefCell, rc::Rc};
    struct Recorder {
        log: Rc<RefCell<Vec<String>>>,
        result: bool,
    }
    impl InputHandler<SourceWindow> for Recorder {
        fn on_size(&mut self, blocked: bool, _: &SourceWindow, height: i32, width: i32) -> bool {
            self.log
                .borrow_mut()
                .push(format!("handler:{blocked}:{height}:{width}"));
            self.result
        }
    }
    #[test]
    fn resize_routes_original_argument_order_to_actual_camera_including_zero_sizes() {
        let (window, _runtime, _) = crate::window::tests::fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        let output = log.clone();
        window
            .size_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, width, height| {
                output
                    .borrow_mut()
                    .push(format!("callback:{width}:{height}"));
            }));
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let expected = crate::camera_2d::SourceCamera2D::new(400, 200);
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        let recorder = |result| -> super::super::Handler {
            Rc::new(RefCell::new(Recorder {
                log: log.clone(),
                result,
            }))
        };
        window.handler_stacks.borrow_mut().extend([
            vec![recorder(true), control, recorder(false)],
            vec![recorder(false)],
        ]);
        for (width, height) in [(900, 350), (0, 350), (900, 0), (0, 0), (-800, 300)] {
            log.borrow_mut().clear();
            window.dispatch(super::super::WindowEvent::Size { width, height });
            if width != 0 && height != 0 {
                expected.resize([width, height]);
            }
            assert_eq!(*camera.matrix.borrow(), *expected.matrix.borrow());
            assert_eq!(
                *log.borrow(),
                vec![
                    format!("callback:{width}:{height}"),
                    format!("handler:false:{height}:{width}"),
                    format!("handler:true:{height}:{width}"),
                    format!("handler:false:{height}:{width}"),
                ]
            );
        }
    }
}

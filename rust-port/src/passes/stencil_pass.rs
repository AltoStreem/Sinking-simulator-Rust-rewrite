//! StencilPass.java disables color writes around the source fullscreen pass.
use super::{
    pass::Pass,
    shader_pass_backend::{self, ShaderPassBackend},
    stateful_pass::StatefulPass,
};
pub(crate) struct StencilPass {
    pub shader: Box<dyn ShaderPassBackend>,
}
impl StencilPass {
    pub fn new(mut shader: Box<dyn ShaderPassBackend>, src: &[String], fragment: &str) -> Self {
        shader_pass_backend::initialize(shader.as_mut(), src, &[], fragment);
        Self { shader }
    }
}
impl Pass for StencilPass {
    fn render(&mut self) {
        self.shader.stencil_test(true);
        self.shader.color_mask([false; 4]);
        let state = self.shader.current_state();
        self.shader.apply_state();
        self.shader.start_shader();
        self.shader.draw_fullscreen();
        self.shader.stop_shader();
        self.shader.restore_state(state);
        self.shader.color_mask([true; 4]);
        self.shader.stencil_test(false);
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
}
impl StatefulPass for StencilPass {
    fn set_float_arg(&mut self, name: &str, values: &[f32]) {
        shader_pass_backend::set_floats(self.shader.as_mut(), name, values);
    }
    fn set_int_arg(&mut self, name: &str, values: &[i32]) {
        shader_pass_backend::set_ints(self.shader.as_mut(), name, values);
    }
    fn set_matrix_arg(&mut self, name: &str, transposed: bool, matrix: &[f32; 16]) {
        shader_pass_backend::set_matrix(self.shader.as_mut(), name, transposed, matrix);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::standard_pass::StandardPass;
    use std::{any::Any, cell::RefCell, rc::Rc};
    struct Backend(Rc<RefCell<Vec<String>>>);
    impl ShaderPassBackend for Backend {
        fn create_shader(&mut self, fragment: &str, outputs: &[String]) {
            self.0
                .borrow_mut()
                .push(format!("compile:{fragment}:{outputs:?}"));
        }
        fn uniform_location(&mut self, name: &str) -> i32 {
            self.0.borrow_mut().push(format!("location:{name}"));
            -1
        }
        fn start_shader(&mut self) {
            self.0.borrow_mut().push("start".into());
        }
        fn stop_shader(&mut self) {
            self.0.borrow_mut().push("stop".into());
        }
        fn write_float_uniform(&mut self, location: i32, values: &[f32]) {
            self.0
                .borrow_mut()
                .push(format!("float:{location}:{values:?}"));
        }
        fn write_int_uniform(&mut self, location: i32, values: &[i32]) {
            self.0
                .borrow_mut()
                .push(format!("int:{location}:{values:?}"));
        }
        fn write_matrix_uniform(&mut self, location: i32, transposed: bool, matrix: &[f32; 16]) {
            self.0
                .borrow_mut()
                .push(format!("matrix:{location}:{transposed}:{}", matrix[15]));
        }
        fn stencil_test(&mut self, enabled: bool) {
            self.0.borrow_mut().push(format!("stencil:{enabled}"));
        }
        fn color_mask(&mut self, mask: [bool; 4]) {
            self.0.borrow_mut().push(format!("colors:{mask:?}"));
        }
        fn current_state(&mut self) -> Box<dyn Any> {
            self.0.borrow_mut().push("snapshot".into());
            Box::new(7u32)
        }
        fn apply_state(&mut self) {
            self.0.borrow_mut().push("apply".into());
        }
        fn restore_state(&mut self, state: Box<dyn Any>) {
            assert_eq!(*state.downcast::<u32>().unwrap(), 7);
            self.0.borrow_mut().push("restore".into());
        }
        fn draw_fullscreen(&mut self) {
            self.0.borrow_mut().push("fullscreen".into());
        }
    }
    #[test]
    fn standard_constructor_sampler_order_and_render_state_match_source() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut pass = StandardPass::new(
            Box::new(Backend(events.clone())),
            &["positions".into(), "masks".into()],
            &["forces".into()],
            "fragment",
        );
        assert_eq!(
            &*events.borrow(),
            &[
                "compile:fragment:[\"forces\"]",
                "location:positions",
                "start",
                "int:-1:[0]",
                "stop",
                "location:masks",
                "start",
                "int:-1:[1]",
                "stop"
            ]
        );
        events.borrow_mut().clear();
        pass.render();
        assert_eq!(
            &*events.borrow(),
            &[
                "stencil:true",
                "snapshot",
                "apply",
                "start",
                "fullscreen",
                "stop",
                "restore",
                "stencil:false"
            ]
        );
    }
    #[test]
    fn stencil_color_writes_and_uniform_calls_match_source() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut pass = StencilPass::new(Box::new(Backend(events.clone())), &[], "fragment");
        events.borrow_mut().clear();
        pass.render();
        assert_eq!(
            &*events.borrow(),
            &[
                "stencil:true",
                "colors:[false, false, false, false]",
                "snapshot",
                "apply",
                "start",
                "fullscreen",
                "stop",
                "restore",
                "colors:[true, true, true, true]",
                "stencil:false"
            ]
        );
        events.borrow_mut().clear();
        pass.set_float_arg("cursor", &[1.0, 2.0]);
        pass.set_int_arg("mask", &[4]);
        pass.set_matrix_arg("transform", true, &[3.0; 16]);
        assert_eq!(
            &*events.borrow(),
            &[
                "location:cursor",
                "start",
                "float:-1:[1.0, 2.0]",
                "stop",
                "location:mask",
                "start",
                "int:-1:[4]",
                "stop",
                "location:transform",
                "start",
                "matrix:-1:true:3",
                "stop"
            ]
        );
    }
    #[test]
    fn invalid_uniform_arity_leaves_program_started_like_source_exception() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut pass = StandardPass::new(Box::new(Backend(events.clone())), &[], &[], "fragment");
        events.borrow_mut().clear();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || pass.set_float_arg("bad", &[])
            ))
            .is_err()
        );
        assert_eq!(&*events.borrow(), &["location:bad", "start"]);
    }
}

//! StandardPass.java rendering and uniform contracts.
use super::{
    pass::Pass,
    shader_pass_backend::{self, ShaderPassBackend},
    stateful_pass::StatefulPass,
};
pub(crate) struct StandardPass {
    pub shader: Box<dyn ShaderPassBackend>,
}
impl StandardPass {
    pub fn new(
        mut shader: Box<dyn ShaderPassBackend>,
        src: &[String],
        dst: &[String],
        fragment: &str,
    ) -> Self {
        shader_pass_backend::initialize(shader.as_mut(), src, dst, fragment);
        Self { shader }
    }
}
impl Pass for StandardPass {
    fn render(&mut self) {
        self.shader.stencil_test(true);
        let state = self.shader.current_state();
        self.shader.apply_state();
        self.shader.start_shader();
        self.shader.draw_fullscreen();
        self.shader.stop_shader();
        self.shader.restore_state(state);
        self.shader.stencil_test(false);
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
}
impl StatefulPass for StandardPass {
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

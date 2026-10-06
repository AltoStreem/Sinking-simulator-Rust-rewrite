//! DirectPass.java renders all children in order, forwards only stateful children.
use super::{pass::Pass, stateful_pass::StatefulPass};
#[derive(Default)]
pub(crate) struct DirectPass {
    pub passes: Vec<Box<dyn Pass>>,
}
impl DirectPass {
    pub fn new(passes: Vec<Box<dyn Pass>>) -> Self {
        Self { passes }
    }
}
impl Pass for DirectPass {
    fn render(&mut self) {
        for pass in &mut self.passes {
            pass.render();
        }
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
}
impl StatefulPass for DirectPass {
    fn set_float_arg(&mut self, name: &str, values: &[f32]) {
        for pass in &mut self.passes {
            if let Some(stateful) = pass.stateful() {
                stateful.set_float_arg(name, &values.to_vec());
            }
        }
    }
    fn set_int_arg(&mut self, name: &str, values: &[i32]) {
        for pass in &mut self.passes {
            if let Some(stateful) = pass.stateful() {
                stateful.set_int_arg(name, &values.to_vec());
            }
        }
    }
    fn set_matrix_arg(&mut self, name: &str, transposed: bool, matrix: &[f32; 16]) {
        for pass in &mut self.passes {
            if let Some(stateful) = pass.stateful() {
                stateful.set_matrix_arg(name, transposed, matrix);
            }
        }
    }
}

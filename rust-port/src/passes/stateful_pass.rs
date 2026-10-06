//! StatefulPass.java has deliberately empty default uniform setters.
use super::pass::Pass;
pub(crate) trait StatefulPass: Pass {
    fn set_float_arg(&mut self, _name: &str, _values: &[f32]) {}
    fn set_int_arg(&mut self, _name: &str, _values: &[i32]) {}
    fn set_matrix_arg(&mut self, _name: &str, _transposed: bool, _matrix: &[f32; 16]) {}
}

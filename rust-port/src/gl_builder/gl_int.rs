//! GLInt.java scalar singleton marker.
use super::{gl_type::GlType, int_type::IntType};
#[derive(Clone, Copy)]
pub(crate) struct GlInt;
impl GlType for GlInt {
    type Value = i32;
    fn type_name(&self) -> &str {
        "int"
    }
}
impl IntType for GlInt {}

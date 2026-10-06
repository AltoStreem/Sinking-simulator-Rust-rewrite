//! GLUInt.java source typeName is literally "int", not "uint".
use super::{gl_type::GlType, uint_type::UIntType};
#[derive(Clone, Copy)]
pub(crate) struct GlUInt;
impl GlType for GlUInt {
    type Value = i32;
    fn type_name(&self) -> &str {
        "int"
    }
}
impl UIntType for GlUInt {}

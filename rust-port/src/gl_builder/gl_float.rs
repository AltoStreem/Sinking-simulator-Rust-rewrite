//! GLFloat.java scalar type name.
use super::gl_type::GlType;
#[derive(Clone, Copy)]
pub(crate) struct GlFloat;
impl GlType for GlFloat {
    type Value = f32;
    fn type_name(&self) -> &str {
        "float"
    }
}
impl super::float_type::FloatType for GlFloat {}

//! FloatType.java scalar family constraint.
use super::gl_type::GlType;
pub(crate) trait FloatType: GlType<Value = f32> {}

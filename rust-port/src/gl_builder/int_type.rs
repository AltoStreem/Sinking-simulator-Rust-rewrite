//! IntType.java scalar family constraint.
use super::gl_type::GlType;
pub(crate) trait IntType: GlType<Value = i32> {}

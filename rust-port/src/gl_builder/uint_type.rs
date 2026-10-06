//! UIntType.java uses JVM Integer values, despite its unsigned shader family.
use super::gl_type::GlType;
pub(crate) trait UIntType: GlType<Value = i32> {}

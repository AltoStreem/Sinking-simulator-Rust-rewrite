//! GLSampler2DType.java associated sampled type contract.
use super::gl_type::GlType;
pub(crate) trait GlSampler2DType: GlType {
    type Base: GlType;
    fn base_type(&self) -> &Self::Base;
}

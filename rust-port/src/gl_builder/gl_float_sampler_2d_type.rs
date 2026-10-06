//! GLFloatSampler2DType.java family-specific sampler constraint.
use super::{float_type::FloatType, gl_sampler_2d_type::GlSampler2DType};
pub(crate) trait GlFloatSampler2DType: GlSampler2DType
where
    Self::Base: FloatType,
{
}

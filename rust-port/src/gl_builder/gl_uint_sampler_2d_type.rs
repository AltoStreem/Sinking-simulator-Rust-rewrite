//! GLUIntSampler2DType.java family-specific sampler constraint.
use super::{gl_sampler_2d_type::GlSampler2DType, uint_type::UIntType};
pub(crate) trait GlUIntSampler2DType: GlSampler2DType
where
    Self::Base: UIntType,
{
}

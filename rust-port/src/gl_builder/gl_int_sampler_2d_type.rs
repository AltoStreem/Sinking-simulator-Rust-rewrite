//! GLIntSampler2DType.java family-specific sampler constraint.
use super::{gl_sampler_2d_type::GlSampler2DType, int_type::IntType};
pub(crate) trait GlIntSampler2DType: GlSampler2DType
where
    Self::Base: IntType,
{
}

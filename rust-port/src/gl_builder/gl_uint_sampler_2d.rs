//! GLUIntSampler2D.java immutable base type and shader type name.
use super::{
    gl_sampler_2d_type::GlSampler2DType, gl_type::GlType,
    gl_uint_sampler_2d_type::GlUIntSampler2DType, uint_type::UIntType,
};
#[derive(Clone)]
pub(crate) struct GlUIntSampler2D<T: UIntType> {
    pub base_type: T,
}
impl<T: UIntType> GlUIntSampler2D<T> {
    pub fn new(base_type: T) -> Self {
        Self { base_type }
    }
}
impl<T: UIntType> GlType for GlUIntSampler2D<T> {
    type Value = T;
    fn type_name(&self) -> &str {
        "usampler2D"
    }
}
impl<T: UIntType> GlSampler2DType for GlUIntSampler2D<T> {
    type Base = T;
    fn base_type(&self) -> &T {
        &self.base_type
    }
}
impl<T: UIntType> GlUIntSampler2DType for GlUIntSampler2D<T> {}

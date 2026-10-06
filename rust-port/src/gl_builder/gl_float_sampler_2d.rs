//! GLFloatSampler2D.java immutable base type and shader type name.
use super::{
    float_type::FloatType, gl_float_sampler_2d_type::GlFloatSampler2DType,
    gl_sampler_2d_type::GlSampler2DType, gl_type::GlType,
};
#[derive(Clone)]
pub(crate) struct GlFloatSampler2D<T: FloatType> {
    pub base_type: T,
}
impl<T: FloatType> GlFloatSampler2D<T> {
    pub fn new(base_type: T) -> Self {
        Self { base_type }
    }
}
impl<T: FloatType> GlType for GlFloatSampler2D<T> {
    type Value = T;
    fn type_name(&self) -> &str {
        "sampler2D"
    }
}
impl<T: FloatType> GlSampler2DType for GlFloatSampler2D<T> {
    type Base = T;
    fn base_type(&self) -> &T {
        &self.base_type
    }
}
impl<T: FloatType> GlFloatSampler2DType for GlFloatSampler2D<T> {}

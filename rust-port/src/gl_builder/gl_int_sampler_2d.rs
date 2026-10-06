//! GLIntSampler2D.java immutable base type and shader type name.
use super::{
    gl_int_sampler_2d_type::GlIntSampler2DType, gl_sampler_2d_type::GlSampler2DType,
    gl_type::GlType, int_type::IntType,
};
#[derive(Clone)]
pub(crate) struct GlIntSampler2D<T: IntType> {
    pub base_type: T,
}
impl<T: IntType> GlIntSampler2D<T> {
    pub fn new(base_type: T) -> Self {
        Self { base_type }
    }
}
impl<T: IntType> GlType for GlIntSampler2D<T> {
    type Value = T;
    fn type_name(&self) -> &str {
        "isampler2D"
    }
}
impl<T: IntType> GlSampler2DType for GlIntSampler2D<T> {
    type Base = T;
    fn base_type(&self) -> &T {
        &self.base_type
    }
}
impl<T: IntType> GlIntSampler2DType for GlIntSampler2D<T> {}

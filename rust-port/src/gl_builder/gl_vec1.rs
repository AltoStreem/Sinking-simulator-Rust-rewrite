//! Port of GLVec1.java.
#[derive(Clone, Copy)]
pub(crate) struct GlVec1;
impl GlVec1 {
    pub fn size(&self) -> usize {
        1
    }
}
impl super::gl_type::GlType for GlVec1 {
    type Value = Vec<f32>;
    fn type_name(&self) -> &str {
        "vec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlVec1 {
    type Base = super::gl_float::GlFloat;
    type Size = super::one::One;
    type Relatives = super::float_relatives::FloatRelatives;
    fn vector_size(&self) -> Self::Size {
        super::one::One
    }
    fn relatives(&self) -> Self::Relatives {
        super::float_relatives::FloatRelatives
    }
}
impl super::float_vector_type::FloatVectorType for GlVec1 {}
impl super::float_vector::FloatVector for GlVec1 {}

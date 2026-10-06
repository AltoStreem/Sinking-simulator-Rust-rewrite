//! Port of GLVec4.java.
#[derive(Clone, Copy)]
pub(crate) struct GlVec4;
impl GlVec4 {
    pub fn size(&self) -> usize {
        4
    }
}
impl super::gl_type::GlType for GlVec4 {
    type Value = Vec<f32>;
    fn type_name(&self) -> &str {
        "vec4"
    }
}
impl super::gl_vector_type::GlVectorType for GlVec4 {
    type Base = super::gl_float::GlFloat;
    type Size = super::four::Four;
    type Relatives = super::float_relatives::FloatRelatives;
    fn vector_size(&self) -> Self::Size {
        super::four::Four
    }
    fn relatives(&self) -> Self::Relatives {
        super::float_relatives::FloatRelatives
    }
}
impl super::float_vector_type::FloatVectorType for GlVec4 {}
impl super::float_vector::FloatVector for GlVec4 {}

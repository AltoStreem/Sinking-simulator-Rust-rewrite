//! Port of GLVec2.java.
#[derive(Clone, Copy)]
pub(crate) struct GlVec2;
impl GlVec2 {
    pub fn size(&self) -> usize {
        2
    }
}
impl super::gl_type::GlType for GlVec2 {
    type Value = Vec<f32>;
    fn type_name(&self) -> &str {
        "vec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlVec2 {
    type Base = super::gl_float::GlFloat;
    type Size = super::two::Two;
    type Relatives = super::float_relatives::FloatRelatives;
    fn vector_size(&self) -> Self::Size {
        super::two::Two
    }
    fn relatives(&self) -> Self::Relatives {
        super::float_relatives::FloatRelatives
    }
}
impl super::float_vector_type::FloatVectorType for GlVec2 {}
impl super::float_vector::FloatVector for GlVec2 {}

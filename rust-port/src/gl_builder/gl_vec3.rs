//! Port of GLVec3.java.
#[derive(Clone, Copy)]
pub(crate) struct GlVec3;
impl GlVec3 {
    pub fn size(&self) -> usize {
        3
    }
}
impl super::gl_type::GlType for GlVec3 {
    type Value = Vec<f32>;
    fn type_name(&self) -> &str {
        "vec3"
    }
}
impl super::gl_vector_type::GlVectorType for GlVec3 {
    type Base = super::gl_float::GlFloat;
    type Size = super::three::Three;
    type Relatives = super::float_relatives::FloatRelatives;
    fn vector_size(&self) -> Self::Size {
        super::three::Three
    }
    fn relatives(&self) -> Self::Relatives {
        super::float_relatives::FloatRelatives
    }
}
impl super::float_vector_type::FloatVectorType for GlVec3 {}
impl super::float_vector::FloatVector for GlVec3 {}

//! Port of GLIVec3.java.
#[derive(Clone, Copy)]
pub(crate) struct GlIVec3;
impl GlIVec3 {
    pub fn size(&self) -> usize {
        3
    }
}
impl super::gl_type::GlType for GlIVec3 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "ivec3"
    }
}
impl super::gl_vector_type::GlVectorType for GlIVec3 {
    type Base = super::gl_int::GlInt;
    type Size = super::three::Three;
    type Relatives = super::int_relatives::IntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::three::Three
    }
    fn relatives(&self) -> Self::Relatives {
        super::int_relatives::IntRelatives
    }
}
impl super::int_vector_type::IntVectorType for GlIVec3 {}
impl super::int_vector::IntVector for GlIVec3 {}

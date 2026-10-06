//! Port of GLIVec2.java.
#[derive(Clone, Copy)]
pub(crate) struct GlIVec2;
impl GlIVec2 {
    pub fn size(&self) -> usize {
        2
    }
}
impl super::gl_type::GlType for GlIVec2 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "ivec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlIVec2 {
    type Base = super::gl_int::GlInt;
    type Size = super::two::Two;
    type Relatives = super::int_relatives::IntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::two::Two
    }
    fn relatives(&self) -> Self::Relatives {
        super::int_relatives::IntRelatives
    }
}
impl super::int_vector_type::IntVectorType for GlIVec2 {}
impl super::int_vector::IntVector for GlIVec2 {}

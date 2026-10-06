//! Port of GLIVec4.java.
#[derive(Clone, Copy)]
pub(crate) struct GlIVec4;
impl GlIVec4 {
    pub fn size(&self) -> usize {
        4
    }
}
impl super::gl_type::GlType for GlIVec4 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "ivec4"
    }
}
impl super::gl_vector_type::GlVectorType for GlIVec4 {
    type Base = super::gl_int::GlInt;
    type Size = super::four::Four;
    type Relatives = super::int_relatives::IntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::four::Four
    }
    fn relatives(&self) -> Self::Relatives {
        super::int_relatives::IntRelatives
    }
}
impl super::int_vector_type::IntVectorType for GlIVec4 {}
impl super::int_vector::IntVector for GlIVec4 {}

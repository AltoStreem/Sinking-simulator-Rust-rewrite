//! Port of GLIVec1.java.
#[derive(Clone, Copy)]
pub(crate) struct GlIVec1;
impl GlIVec1 {
    pub fn size(&self) -> usize {
        1
    }
}
impl super::gl_type::GlType for GlIVec1 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "ivec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlIVec1 {
    type Base = super::gl_int::GlInt;
    type Size = super::one::One;
    type Relatives = super::int_relatives::IntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::one::One
    }
    fn relatives(&self) -> Self::Relatives {
        super::int_relatives::IntRelatives
    }
}
impl super::int_vector_type::IntVectorType for GlIVec1 {}
impl super::int_vector::IntVector for GlIVec1 {}

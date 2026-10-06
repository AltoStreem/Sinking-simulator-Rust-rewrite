//! Port of GLUVec1.java.
#[derive(Clone, Copy)]
pub(crate) struct GlUVec1;
impl GlUVec1 {
    pub fn size(&self) -> usize {
        1
    }
}
impl super::gl_type::GlType for GlUVec1 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "uvec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlUVec1 {
    type Base = super::gl_uint::GlUInt;
    type Size = super::one::One;
    type Relatives = super::uint_relatives::UIntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::one::One
    }
    fn relatives(&self) -> Self::Relatives {
        super::uint_relatives::UIntRelatives
    }
}
impl super::uint_vector_type::UIntVectorType for GlUVec1 {}
impl super::uint_vector::UIntVector for GlUVec1 {}

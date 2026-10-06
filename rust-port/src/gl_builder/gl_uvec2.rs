//! Port of GLUVec2.java.
#[derive(Clone, Copy)]
pub(crate) struct GlUVec2;
impl GlUVec2 {
    pub fn size(&self) -> usize {
        2
    }
}
impl super::gl_type::GlType for GlUVec2 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "uvec2"
    }
}
impl super::gl_vector_type::GlVectorType for GlUVec2 {
    type Base = super::gl_uint::GlUInt;
    type Size = super::two::Two;
    type Relatives = super::uint_relatives::UIntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::two::Two
    }
    fn relatives(&self) -> Self::Relatives {
        super::uint_relatives::UIntRelatives
    }
}
impl super::uint_vector_type::UIntVectorType for GlUVec2 {}
impl super::uint_vector::UIntVector for GlUVec2 {}

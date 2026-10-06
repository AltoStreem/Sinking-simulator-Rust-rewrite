//! Port of GLUVec4.java.
#[derive(Clone, Copy)]
pub(crate) struct GlUVec4;
impl GlUVec4 {
    pub fn size(&self) -> usize {
        4
    }
}
impl super::gl_type::GlType for GlUVec4 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "uvec4"
    }
}
impl super::gl_vector_type::GlVectorType for GlUVec4 {
    type Base = super::gl_uint::GlUInt;
    type Size = super::four::Four;
    type Relatives = super::uint_relatives::UIntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::four::Four
    }
    fn relatives(&self) -> Self::Relatives {
        super::uint_relatives::UIntRelatives
    }
}
impl super::uint_vector_type::UIntVectorType for GlUVec4 {}
impl super::uint_vector::UIntVector for GlUVec4 {}

//! Port of GLUVec3.java.
#[derive(Clone, Copy)]
pub(crate) struct GlUVec3;
impl GlUVec3 {
    pub fn size(&self) -> usize {
        3
    }
}
impl super::gl_type::GlType for GlUVec3 {
    type Value = Vec<i32>;
    fn type_name(&self) -> &str {
        "uvec3"
    }
}
impl super::gl_vector_type::GlVectorType for GlUVec3 {
    type Base = super::gl_uint::GlUInt;
    type Size = super::three::Three;
    type Relatives = super::uint_relatives::UIntRelatives;
    fn vector_size(&self) -> Self::Size {
        super::three::Three
    }
    fn relatives(&self) -> Self::Relatives {
        super::uint_relatives::UIntRelatives
    }
}
impl super::uint_vector_type::UIntVectorType for GlUVec3 {}
impl super::uint_vector::UIntVector for GlUVec3 {}

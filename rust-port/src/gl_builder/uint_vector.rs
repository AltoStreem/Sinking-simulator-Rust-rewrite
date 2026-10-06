//! Port of UIntVector.java.
pub(crate) trait UIntVector:
    super::uint_vector_type::UIntVectorType
    + super::gl_vector_type::GlVectorType<Relatives = super::uint_relatives::UIntRelatives>
{
}

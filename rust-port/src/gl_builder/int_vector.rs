//! Port of IntVector.java.
pub(crate) trait IntVector:
    super::int_vector_type::IntVectorType
    + super::gl_vector_type::GlVectorType<Relatives = super::int_relatives::IntRelatives>
{
}

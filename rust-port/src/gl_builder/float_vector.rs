//! Port of FloatVector.java.
pub(crate) trait FloatVector:
    super::float_vector_type::FloatVectorType
    + super::gl_vector_type::GlVectorType<Relatives = super::float_relatives::FloatRelatives>
{
}

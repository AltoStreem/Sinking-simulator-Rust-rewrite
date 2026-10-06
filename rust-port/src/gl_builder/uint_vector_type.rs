//! Port of UIntVectorType.java.
pub(crate) trait UIntVectorType:
    super::gl_vector_type::GlVectorType<Base = super::gl_uint::GlUInt>
    + super::gl_type::GlType<Value = Vec<i32>>
{
}

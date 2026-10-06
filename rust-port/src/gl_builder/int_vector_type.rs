//! Port of IntVectorType.java.
pub(crate) trait IntVectorType:
    super::gl_vector_type::GlVectorType<Base = super::gl_int::GlInt>
    + super::gl_type::GlType<Value = Vec<i32>>
{
}

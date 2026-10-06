//! Port of FloatVectorType.java.
pub(crate) trait FloatVectorType:
    super::gl_vector_type::GlVectorType<Base = super::gl_float::GlFloat>
    + super::gl_type::GlType<Value = Vec<f32>>
{
}

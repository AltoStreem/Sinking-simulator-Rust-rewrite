//! Port of GLVectorType.java.
use super::{
    gl_type::GlType, gl_vector_relatives::GlVectorRelatives, gl_vector_size::GlVectorSize,
};
pub(crate) trait GlVectorType: GlType<Value = Vec<<Self::Base as GlType>::Value>> {
    type Base: GlType;
    type Size: GlVectorSize;
    type Relatives: GlVectorRelatives<Base = Self::Base>;
    fn vector_size(&self) -> Self::Size;
    fn relatives(&self) -> Self::Relatives;
}

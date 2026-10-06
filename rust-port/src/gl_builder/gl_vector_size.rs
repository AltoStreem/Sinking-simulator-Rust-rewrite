//! Port of GLVectorSize.java.
pub(crate) trait GlVectorSize: super::igl_vector::IGlVector {
    fn size(&self) -> usize;
}

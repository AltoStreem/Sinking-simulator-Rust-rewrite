//! Port of FOUR.java.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Four;
impl super::igl_vector::IGlVector for Four {}
impl super::gl_vector_size::GlVectorSize for Four {
    fn size(&self) -> usize {
        4
    }
}
impl super::inc::Inc for Four {}
impl super::dec::Dec for Four {}
impl super::ix::IX for Four {}
impl super::i1::I1 for Four {}
impl super::iy::IY for Four {}
impl super::i2::I2 for Four {}
impl super::iz::IZ for Four {}
impl super::i3::I3 for Four {}
impl super::iw::IW for Four {}
impl super::i4::I4 for Four {}
impl super::dw::DW for Four {}
impl super::d4::D4 for Four {}

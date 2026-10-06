//! Port of TWO.java.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Two;
impl super::igl_vector::IGlVector for Two {}
impl super::gl_vector_size::GlVectorSize for Two {
    fn size(&self) -> usize {
        2
    }
}
impl super::inc::Inc for Two {}
impl super::dec::Dec for Two {}
impl super::ix::IX for Two {}
impl super::i1::I1 for Two {}
impl super::iy::IY for Two {}
impl super::i2::I2 for Two {}
impl super::dy::DY for Two {}
impl super::d2::D2 for Two {}
impl super::dz::DZ for Two {}
impl super::d3::D3 for Two {}
impl super::dw::DW for Two {}
impl super::d4::D4 for Two {}

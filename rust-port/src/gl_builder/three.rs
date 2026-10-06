//! Port of THREE.java.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Three;
impl super::igl_vector::IGlVector for Three {}
impl super::gl_vector_size::GlVectorSize for Three {
    fn size(&self) -> usize {
        3
    }
}
impl super::inc::Inc for Three {}
impl super::dec::Dec for Three {}
impl super::ix::IX for Three {}
impl super::i1::I1 for Three {}
impl super::iy::IY for Three {}
impl super::i2::I2 for Three {}
impl super::iz::IZ for Three {}
impl super::i3::I3 for Three {}
impl super::dz::DZ for Three {}
impl super::d3::D3 for Three {}
impl super::dw::DW for Three {}
impl super::d4::D4 for Three {}

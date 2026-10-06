//! Port of ONE.java.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct One;
impl super::igl_vector::IGlVector for One {}
impl super::gl_vector_size::GlVectorSize for One {
    fn size(&self) -> usize {
        1
    }
}
impl super::inc::Inc for One {}
impl super::dec::Dec for One {}
impl super::ix::IX for One {}
impl super::i1::I1 for One {}
impl super::dx::DX for One {}
impl super::d1::D1 for One {}
impl super::dy::DY for One {}
impl super::d2::D2 for One {}
impl super::dz::DZ for One {}
impl super::d3::D3 for One {}
impl super::dw::DW for One {}
impl super::d4::D4 for One {}

//! GLDataHolder.java texture contract, adapted to compute storage buffers.
#![allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DataTexture {
    pub width: usize,
    pub height: usize,
    pub components: usize,
    pub external_format: i32,
    pub internal_format: i32,
    pub scalar_type: i32,
    pub min_filter: i32,
    pub mag_filter: i32,
    pub wrap_s: i32,
    pub wrap_t: i32,
    pub mipmapped: bool,
}
impl DataTexture {
    pub fn new(
        width: usize,
        height: usize,
        components: usize,
        external_format: i32,
        internal_format: i32,
        scalar_type: i32,
    ) -> Self {
        Self {
            width,
            height,
            components,
            external_format,
            internal_format,
            scalar_type,
            min_filter: 9728,
            mag_filter: 9728,
            wrap_s: 33069,
            wrap_t: 33069,
            mipmapped: false,
        }
    }
}
pub(crate) trait GlDataHolder {
    fn texture(&self) -> &DataTexture;
}

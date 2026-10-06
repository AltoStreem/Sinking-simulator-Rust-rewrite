//! UInt8DataHolder.java texture allocation contract and exact source format tables.
//! Source null allocation is represented by None, not a claim of zero GPU data.
#![allow(dead_code)]
use crate::gl_data_holder::{DataTexture, GlDataHolder};
pub(crate) const INTERNAL_FORMAT: [i32; 5] = [0, 33330, 33336, 36221, 36220];
pub(crate) const FORMAT: [i32; 5] = [0, 36244, 33320, 36248, 36249];
pub(crate) struct UInt8DataHolder {
    texture: DataTexture,
    data: Option<Vec<[u32; 4]>>,
}
impl UInt8DataHolder {
    pub fn new(width: usize, height: usize, components: usize) -> Self {
        assert!(
            (1..=4).contains(&components),
            "Source integer component count must be 1..=4"
        );
        Self {
            texture: DataTexture::new(
                width,
                height,
                components,
                FORMAT[components],
                INTERNAL_FORMAT[components],
                5125,
            ),
            data: None,
        }
    }
    pub fn single_channel(width: usize, height: usize) -> Self {
        Self::new(width, height, 1)
    }
    pub fn into_storage(self) -> Option<Vec<[u32; 4]>> {
        self.data
    }
}
impl GlDataHolder for UInt8DataHolder {
    fn texture(&self) -> &DataTexture {
        &self.texture
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_null_data_default_and_integer_formats_are_preserved() {
        let holder = UInt8DataHolder::single_channel(2, 1);
        assert_eq!(
            (
                holder.texture().internal_format,
                holder.texture().external_format,
                holder.texture().scalar_type
            ),
            (33330, 36244, 5125)
        );
        assert!(holder.into_storage().is_none());
        assert_eq!(
            UInt8DataHolder::new(1, 1, 3).texture().internal_format,
            36221
        );
    }
}

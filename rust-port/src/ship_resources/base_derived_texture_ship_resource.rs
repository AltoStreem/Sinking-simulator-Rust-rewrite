use crate::materials::Materials;
use image::RgbaImage;

/// Rust counterpart of BaseDerivedTextureShipResource.genImageData.
pub(crate) struct BaseDerivedTextureShipResource;

impl BaseDerivedTextureShipResource {
    /// Clones BASE and clears pixels with missing or invisible materials.
    /// Java packs RGBA as a big-endian u32 and looks up `color >>> 8` (RGB).
    pub(crate) fn derive(base: &RgbaImage, materials: &Materials) -> RgbaImage {
        let mut derived = base.clone();
        for pixel in derived.pixels_mut() {
            let rgb =
                (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]);
            if materials.get(rgb).is_none_or(|material| material.invisible) {
                *pixel = image::Rgba([0, 0, 0, 0]);
            }
        }
        derived
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_material_appearance_clears_unknown_and_invisible_without_mutating_base() {
        let materials = Materials::from_json(
            r##"[
            {"name":"hidden","color":"#123456","invisible":true},
            {"name":"visible","color":"#abcdef"}
        ]"##,
        )
        .unwrap();
        let base = RgbaImage::from_raw(
            3,
            1,
            vec![0x12, 0x34, 0x56, 255, 0xab, 0xcd, 0xef, 128, 1, 2, 3, 255],
        )
        .unwrap();
        let appearance = BaseDerivedTextureShipResource::derive(&base, &materials);
        assert_eq!(appearance.get_pixel(0, 0).0, [0; 4]);
        assert_eq!(appearance.get_pixel(1, 0).0, [0xab, 0xcd, 0xef, 128]);
        assert_eq!(appearance.get_pixel(2, 0).0, [0; 4]);
        assert_eq!(base.get_pixel(0, 0).0, [0x12, 0x34, 0x56, 255]);
    }
}

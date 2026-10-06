//! Rust translation of `ship/ShipData.java`.
//!
//! The source class stores the base texture, its material palette, and one
//! material lookup result for every image pixel. The latter preserves the
//! source's big-endian RGBA `pixel >>> 8` color-key behavior.

use crate::materials::{Material, Materials};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct ShipData {
    pub img: image::RgbaImage,
    pub materials: Materials,
    /// Material assigned to each base-layer pixel, in row-major image order.
    pub material_buffer: Vec<Option<Arc<Material>>>,
    pub width: u32,
    pub height: u32,
}

impl ShipData {
    pub fn new(img: image::RgbaImage, materials: Materials) -> Self {
        let width = img.width();
        let height = img.height();
        // Java's materialBuffer contains shared references to palette records.
        // Share records here too, avoiding a cloned material name per texel.
        let material_lookup: HashMap<_, _> = materials
            .materials
            .iter()
            .map(|(&rgb, material)| (rgb, Arc::new(material.clone())))
            .collect();
        let material_buffer = img
            .pixels()
            .map(|pixel| {
                // Java reads an RGBA byte buffer as BIG_ENDIAN ints, then
                // shifts right eight bits to obtain the 24-bit RGB key.
                let rgb =
                    (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]);
                material_lookup.get(&rgb).cloned()
            })
            .collect();
        Self {
            img,
            materials,
            material_buffer,
            width,
            height,
        }
    }

    pub fn material_at(&self, x: u32, y: u32) -> Option<&Material> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.material_buffer
            .get(y as usize * self.width as usize + x as usize)
            .and_then(Option::as_deref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_material_physics_retains_invisible_and_zero_alpha_pixels() {
        let materials = Materials::from_json(
            r##"[
            {"name":"hidden strut","color":"#123456","mass":17,"invisible":true},
            {"name":"hull","color":"#abcdef","mass":31,"isHull":true}
        ]"##,
        )
        .unwrap();
        let img = image::RgbaImage::from_raw(
            3,
            1,
            vec![0x12, 0x34, 0x56, 255, 0xab, 0xcd, 0xef, 0, 1, 2, 3, 255],
        )
        .unwrap();
        let data = ShipData::new(img, materials);
        assert_eq!(data.material_at(0, 0).unwrap().mass, 17.0);
        assert!(data.material_at(0, 0).unwrap().invisible);
        assert_eq!(data.material_at(1, 0).unwrap().mass, 31.0);
        assert!(data.material_at(2, 0).is_none());
        assert!(data.material_at(3, 0).is_none());
    }
}

//! Rust translation of `ship/ShipData.java`.
//!
//! The source class stores the base texture, its material palette, and one
//! material lookup result for every image pixel. The latter preserves the
//! source's big-endian RGBA `pixel >>> 8` color-key behavior.

use crate::materials::{Material, Materials};
use std::{collections::HashMap, sync::Arc};

/// The source constructor retains objects rather than copying their contents.
/// Keep this representation separate from the owned Bevy image adapter below.
#[allow(dead_code)]
pub(crate) struct SourceShipData<P: SourceMaterialLookup> {
    pub img: Arc<std::sync::Mutex<crate::image_data::ImageData>>,
    pub materials: Arc<P>,
    pub material_buffer: std::rc::Rc<std::cell::RefCell<Vec<Option<P::Material>>>>,
    pub width: i32,
    pub height: i32,
}

/// Implementations return the original material reference, including aliases
/// assigned to different RGB keys, rather than manufacturing a new record.
#[allow(dead_code)]
pub(crate) trait SourceMaterialLookup {
    type Material;
    fn get(&self, rgb: u32) -> Option<Self::Material>;
}

#[allow(dead_code)]
pub(crate) trait SourceShipDataThumbnail {
    type Palette: SourceMaterialLookup;
    fn base_layer(&self) -> Result<Arc<std::sync::Mutex<crate::image_data::ImageData>>, String>;
    fn materials(&self) -> Result<Arc<Self::Palette>, String>;
}

#[allow(dead_code)]
impl<P: SourceMaterialLookup> SourceShipData<P> {
    /// ShipData's five-argument constructor does not validate dimensions or
    /// require the material list to match the image size.
    pub(crate) fn new(
        img: Arc<std::sync::Mutex<crate::image_data::ImageData>>,
        materials: Arc<P>,
        material_buffer: std::rc::Rc<std::cell::RefCell<Vec<Option<P::Material>>>>,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            img,
            materials,
            material_buffer,
            width,
            height,
        }
    }

    pub(crate) fn from_thumbnail<T: SourceShipDataThumbnail<Palette = P>>(
        thumbnail: &T,
    ) -> Result<Self, String> {
        // Capture the original byte storage before subsequent getter calls.
        // ImageData's Rust buffer currently represents the entire Java buffer;
        // positioned/limited ByteBuffer views remain an adapter limitation.
        let buffer_image = thumbnail.base_layer()?;
        let capacity = buffer_image
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .buffer
            .len()
            / 4;
        let materials = thumbnail.materials()?;
        let img = thumbnail.base_layer()?;
        let mut material_buffer = Vec::with_capacity(capacity);
        for index in 0..capacity {
            // Java evaluates getMaterials() before buf.get(i), on every pixel.
            let palette = thumbnail.materials()?;
            let pixel = {
                let image = buffer_image
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                let bytes = image
                    .buffer
                    .get(index * 4..index * 4 + 4)
                    .ok_or_else(|| "captured image buffer was shortened".to_owned())?;
                u32::from_be_bytes(bytes.try_into().unwrap())
            };
            material_buffer.push(palette.get(pixel >> 8));
        }
        let width = thumbnail
            .base_layer()?
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .width;
        let height = thumbnail
            .base_layer()?
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .height;
        Ok(Self::new(
            img,
            materials,
            std::rc::Rc::new(std::cell::RefCell::new(material_buffer)),
            width,
            height,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct ShipData {
    pub img: image::RgbaImage,
    pub materials: Materials,
    /// Material assigned to each base-layer pixel, in row-major image order.
    pub material_buffer: Vec<Option<Arc<Material>>>,
    pub width: u32,
    pub height: u32,
}

impl SourceShipData<crate::materials::SourceMaterials> {
    /// Snapshot pixels for Bevy while preserving the constructor's original
    /// material references; do not perform a second RGB lookup here.
    pub(crate) fn to_owned_adapter(&self) -> Result<ShipData, String> {
        let img = self
            .img
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
            .into_rgba()?;
        Ok(ShipData {
            img,
            materials: self.materials.to_owned_adapter(),
            material_buffer: self.material_buffer.borrow().clone(),
            width: u32::try_from(self.width)
                .map_err(|_| "negative width cannot be represented by Bevy".to_owned())?,
            height: u32::try_from(self.height)
                .map_err(|_| "negative height cannot be represented by Bevy".to_owned())?,
        })
    }
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
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    struct Palette {
        material: Rc<Cell<f32>>,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl SourceMaterialLookup for Palette {
        type Material = Rc<Cell<f32>>;
        fn get(&self, rgb: u32) -> Option<Self::Material> {
            self.log.borrow_mut().push(format!("get:{rgb:06x}"));
            (rgb == 0x123456).then(|| self.material.clone())
        }
    }

    #[test]
    fn source_primary_constructor_retains_references_and_unchecked_signed_dimensions() {
        let img = Arc::new(std::sync::Mutex::new(crate::image_data::ImageData::new(
            vec![],
            0,
            0,
            6408,
        )));
        let materials = Arc::new(Palette {
            material: Rc::new(Cell::new(17.0)),
            log: Rc::default(),
        });
        let buffer = Rc::new(RefCell::new(vec![Some(materials.material.clone())]));
        let data = SourceShipData::new(img.clone(), materials.clone(), buffer.clone(), -7, 123);
        assert!(Arc::ptr_eq(&data.img, &img));
        assert!(Arc::ptr_eq(&data.materials, &materials));
        assert!(Rc::ptr_eq(&data.material_buffer, &buffer));
        assert_eq!((data.width, data.height), (-7, 123));
        materials.material.set(42.0);
        assert_eq!(
            data.material_buffer.borrow()[0].as_ref().unwrap().get(),
            42.0
        );
        buffer.borrow_mut().clear();
        assert!(data.material_buffer.borrow().is_empty());
    }

    #[test]
    fn source_thumbnail_constructor_preserves_getter_order_and_captured_buffer() {
        struct Thumbnail {
            images: Vec<Arc<std::sync::Mutex<crate::image_data::ImageData>>>,
            base_calls: Cell<usize>,
            palette: Arc<Palette>,
        }
        impl SourceShipDataThumbnail for Thumbnail {
            type Palette = Palette;
            fn base_layer(
                &self,
            ) -> Result<Arc<std::sync::Mutex<crate::image_data::ImageData>>, String> {
                let index = self.base_calls.get();
                self.base_calls.set(index + 1);
                self.palette.log.borrow_mut().push(format!("base:{index}"));
                Ok(self.images[index].clone())
            }
            fn materials(&self) -> Result<Arc<Palette>, String> {
                self.palette.log.borrow_mut().push("materials".into());
                // Reads must observe writes to the originally captured storage.
                self.images[0].lock().unwrap().buffer[0] = 0x12;
                Ok(self.palette.clone())
            }
        }
        let image = |bytes, w, h| {
            Arc::new(std::sync::Mutex::new(crate::image_data::ImageData::new(
                bytes, w, h, 6408,
            )))
        };
        let palette = Arc::new(Palette {
            material: Rc::new(Cell::new(17.0)),
            log: Rc::default(),
        });
        let thumbnail = Thumbnail {
            images: vec![
                image(
                    vec![0, 0x34, 0x56, 0, 0x12, 0x34, 0x56, 255, 1, 2, 3, 255, 99],
                    3,
                    1,
                ),
                image(vec![9; 4], 1, 1),
                image(vec![], -3, 0),
                image(vec![], 0, 77),
            ],
            base_calls: Cell::new(0),
            palette: palette.clone(),
        };
        let data = SourceShipData::from_thumbnail(&thumbnail).unwrap();
        assert!(Arc::ptr_eq(&data.img, &thumbnail.images[1]));
        assert!(Arc::ptr_eq(&data.materials, &palette));
        assert_eq!((data.width, data.height), (-3, 77));
        let buffer = data.material_buffer.borrow();
        assert_eq!(buffer.len(), 3); // IntBuffer drops the trailing partial int.
        assert!(Rc::ptr_eq(buffer[0].as_ref().unwrap(), &palette.material));
        assert!(Rc::ptr_eq(buffer[1].as_ref().unwrap(), &palette.material));
        assert!(buffer[2].is_none());
        assert_eq!(
            *palette.log.borrow(),
            vec![
                "base:0",
                "materials",
                "base:1",
                "materials",
                "get:123456",
                "materials",
                "get:123456",
                "materials",
                "get:010203",
                "base:2",
                "base:3"
            ]
        );
    }

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

//! Rust port of SS2 PosVelDataHolder.createPositions.
use bevy::prelude::Vec2;

#[allow(dead_code)]
pub(crate) struct SourcePosVelDataHolder {
    texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>,
}
#[allow(dead_code)]
impl SourcePosVelDataHolder {
    pub(crate) fn create_positions(
        width: i32,
        height: i32,
    ) -> Result<crate::image_data::ImageData, String> {
        let count = width.wrapping_mul(height).wrapping_mul(2);
        let count =
            usize::try_from(count).map_err(|_| "negative source float array size".to_owned())?;
        let mut bytes = Vec::with_capacity(count * 4);
        for component in 0..count {
            let index = component as i32 / 2;
            let value = if component % 2 == 0 {
                if width == 0 {
                    return Err("source integer division by zero".into());
                }
                (index % width) as f32 - width as f32 * 0.5
            } else {
                height as f32 - (index / width) as f32 - height as f32 * 0.25
            };
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        let mut image = crate::image_data::ImageData::new(bytes, width, height, 33319);
        image.format = 5126;
        Ok(image)
    }
    pub(crate) fn new<P: crate::ship_data::SourceMaterialLookup>(
        dat: &crate::ship_data::SourceShipData<P>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        let positions = Self::create_positions(dat.width, dat.height)?;
        let texture = crate::texture_2d::SourceTexture2D::from_image(
            &positions,
            34836,
            false,
            std::sync::Arc::new(crate::pos_vel_data_texture_config::configure),
            backend,
            context,
            runtime,
        );
        Ok(Self {
            texture: std::sync::Arc::new(texture),
        })
    }
}
impl crate::gl_data_holder::SourceGlDataHolder for SourcePosVelDataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        self.texture.clone()
    }
}

/// Per-source-pixel rest coordinates consumed by ShipPhysics.
pub struct PosVelDataHolder {
    pub positions: Vec<Vec2>,
}

impl PosVelDataHolder {
    pub fn new(width: usize, height: usize) -> Self {
        let positions = if width == 0 || height == 0 {
            Vec::new()
        } else {
            (0..width * height)
                .map(|index| {
                    let x = index % width;
                    let y = index / width;
                    Vec2::new(
                        x as f32 - width as f32 * 0.5,
                        height as f32 - y as f32 - height as f32 * 0.25,
                    )
                })
                .collect()
        };
        Self { positions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_data_holder::SourceGlDataHolder;
    use std::sync::{Arc, Mutex};

    #[test]
    fn native_position_force_and_water_holders_preserve_uploads_identity_and_cleanup() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = crate::render_fbo::source_tests::backend(log.clone());
        let dat = crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(crate::materials::SourceMaterials::default()),
            std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            2,
            3,
        );
        let positions =
            SourcePosVelDataHolder::new(&dat, backend.clone(), context.clone(), &runtime).unwrap();
        let force = crate::force_data::SourceForceDataHolder::new(
            &dat,
            backend.clone(),
            context.clone(),
            &runtime,
        )
        .unwrap();
        let water = crate::water_data::SourceWaterDataHolder::new(
            &dat,
            backend.clone(),
            context.clone(),
            &runtime,
        )
        .unwrap();
        let p = positions.source_texture();
        let f = force.source_texture();
        let w = water.source_texture();
        assert!(Arc::ptr_eq(&p, &positions.source_texture()));
        assert!(Arc::ptr_eq(&f, &force.source_texture()));
        assert!(Arc::ptr_eq(&w, &water.source_texture()));
        assert_eq!(
            (p.width, p.height, p.format, p.internal_format),
            (2, 3, 33319, 34836)
        );
        let bytes = p.img.as_ref().unwrap().lock().unwrap();
        let floats: Vec<_> = bytes
            .chunks_exact(4)
            .map(|bytes| f32::from_ne_bytes(bytes.try_into().unwrap()))
            .collect();
        assert_eq!(
            floats,
            vec![
                -1., 2.25, 0., 2.25, -1., 1.25, 0., 1.25, -1., 0.25, 0., 0.25
            ]
        );
        drop(bytes);
        assert_eq!(
            (f.format, f.internal_format, force.base_type().components),
            (33319, 33328, 2)
        );
        assert_eq!(
            (w.format, w.internal_format, water.base_type().components),
            (6408, 34836, 4)
        );
        assert_eq!(
            (force.width, force.height, water.width, water.height),
            (2, 3, 2, 3)
        );
        for texture in [&f, &w] {
            assert_eq!(
                texture.img.as_ref().unwrap().lock().unwrap().as_slice(),
                &[0; 96]
            );
        }
        let events = log.lock().unwrap().clone();
        for parameter in [
            "parameter:3553:10240:9728",
            "parameter:3553:10241:9728",
            "parameter:3553:10242:33069",
            "parameter:3553:10243:33069",
        ] {
            assert_eq!(
                events
                    .iter()
                    .filter(|event| event.as_str() == parameter)
                    .count(),
                3
            );
        }
        assert!(!events.iter().any(|event| event.starts_with("mips:")));
        assert!(events.contains(&"image2:1:3553:34836:[2, 3]:33319:5126:false".into()));
        assert!(events.contains(&"image2:2:3553:33328:[2, 3]:33319:5126:false".into()));
        assert!(events.contains(&"image2:3:3553:34836:[2, 3]:6408:5126:false".into()));
        context.close();
        assert!(
            !log.lock()
                .unwrap()
                .iter()
                .any(|event| event.starts_with("delete_tex:"))
        );
        runtime.run_main();
        for id in 1..=3 {
            assert!(log.lock().unwrap().contains(&format!("delete_tex:{id}")));
        }
        assert!(
            crate::float_data_holder::SourceFloatDataHolder::<2>::new(
                -1, 1, backend, context, &runtime
            )
            .is_err()
        );
    }

    #[test]
    fn native_position_array_uses_signed_dimensions_and_java_integer_overflow() {
        assert!(SourcePosVelDataHolder::create_positions(-1, 1).is_err());
        assert!(
            SourcePosVelDataHolder::create_positions(0, 3)
                .unwrap()
                .buffer
                .is_empty()
        );
        // Java width*height*2 wraps to zero before the FloatArray allocation.
        assert!(
            SourcePosVelDataHolder::create_positions(1 << 30, 2)
                .unwrap()
                .buffer
                .is_empty()
        );
        let positions = SourcePosVelDataHolder::create_positions(-2, -1).unwrap();
        assert_eq!((positions.image_format, positions.format), (33319, 5126));
        let values: Vec<_> = positions
            .buffer
            .chunks_exact(4)
            .map(|bytes| f32::from_ne_bytes(bytes.try_into().unwrap()))
            .collect();
        assert_eq!(values, vec![1., -0.75, 2., -0.75]);
    }
    #[test]
    fn source_positions_preserve_pixel_spacing_and_quarter_height_draft() {
        let p = PosVelDataHolder::new(4, 4).positions;
        assert_eq!(p[0], Vec2::new(-2.0, 3.0));
        assert_eq!(p[15], Vec2::new(1.0, 0.0));
        assert_eq!(p[1] - p[0], Vec2::X);
        assert_eq!(p[4] - p[0], Vec2::NEG_Y);
        let tall = PosVelDataHolder::new(2, 8).positions;
        assert_eq!(tall.last().unwrap().y, -1.0);
    }
}

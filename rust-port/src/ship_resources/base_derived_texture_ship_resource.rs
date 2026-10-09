use super::{ShipLayer, ShipResource, ShipResourceFile, ShipResourceType};
use crate::materials::Materials;
use image::RgbaImage;
use std::sync::Arc;
pub(crate) type SourcePalette =
    Arc<dyn Fn() -> Arc<crate::materials::SourceMaterials> + Send + Sync>;

/// Rust counterpart of BaseDerivedTextureShipResource.genImageData.
#[derive(Clone, Debug)]
pub(crate) struct BaseDerivedTextureShipResource {
    pub(crate) resource: ShipResource,
    pub(crate) base_resource: ShipResourceFile,
    pub(crate) material_resource: Option<ShipResourceFile>,
    image_cache: super::image_cache::ImageCache,
    texture_cache: super::texture_cache::TextureCache,
    source_texture_cache: super::source_texture_cache::SourceTextureCache,
}

impl PartialEq for BaseDerivedTextureShipResource {
    fn eq(&self, other: &Self) -> bool {
        self.base_resource == other.base_resource
            && self.material_resource == other.material_resource
    }
}
impl Eq for BaseDerivedTextureShipResource {}

impl BaseDerivedTextureShipResource {
    pub(crate) fn set_ship(&mut self, ship: String) {
        self.resource.set_ship(ship);
    }

    pub(crate) fn new(
        base_resource: ShipResourceFile,
        material_resource: Option<ShipResourceFile>,
    ) -> Self {
        Self {
            resource: ShipResource::new(
                base_resource.ship.clone(),
                ShipLayer::default(),
                ShipResourceType::Texture,
            ),
            base_resource,
            material_resource,
            image_cache: super::image_cache::ImageCache::default(),
            texture_cache: super::texture_cache::TextureCache::default(),
            source_texture_cache: super::source_texture_cache::SourceTextureCache::default(),
        }
    }

    pub(crate) fn image_data(&self, global: &Materials) -> Result<RgbaImage, String> {
        self.image_cache.now(self.image_generator(global))
    }
    pub(crate) fn source_texture_now(
        &self,
        environment: &super::SourceTextureEnvironment,
        global: SourcePalette,
    ) -> Result<Arc<crate::texture_2d::SourceTexture2D>, String> {
        self.source_texture_cache.now(
            environment,
            || {
                self.image_cache
                    .source_poll_native(self.source_generator(global.clone()))
            },
            || {
                self.image_cache
                    .source_now_native(self.source_generator(global.clone()))
            },
            || self.image_cache.touch(),
        )
    }
    pub(crate) fn source_generator(
        &self,
        global: SourcePalette,
    ) -> super::image_cache::SourceGenerator {
        let base = self.base_resource.clone();
        let local = self.material_resource.clone();
        Arc::new(move || {
            let Some(base) = base.source_poll_image_data()? else {
                return Ok(None);
            };
            let materials = match local
                .as_ref()
                .filter(|r| r.resource_type == ShipResourceType::Materials)
            {
                Some(resource) => resource.source_materials()?,
                None => global(),
            };
            let base = base.lock().unwrap_or_else(|e| e.into_inner());
            let mut copy = base.clone();
            copy.buffer = vec![0; base.buffer.len()];
            for (pixel, out) in base
                .buffer
                .chunks_exact(4)
                .zip(copy.buffer.chunks_exact_mut(4))
            {
                let color = u32::from_be_bytes(pixel.try_into().unwrap());
                if materials.get(color >> 8).is_some_and(|m| !m.invisible) {
                    out.copy_from_slice(pixel);
                }
            }
            Ok(Some(Arc::new(std::sync::Mutex::new(copy))))
        })
    }

    pub(crate) fn poll_image_data(&self, global: &Materials) -> Result<Option<RgbaImage>, String> {
        self.image_cache.poll(self.image_generator(global))
    }

    fn image_generator(
        &self,
        global: &Materials,
    ) -> Arc<dyn Fn() -> Result<Option<RgbaImage>, String> + Send + Sync> {
        let base = self.base_resource.clone();
        let local = self.material_resource.clone();
        let global = global.clone();
        Arc::new(move || {
            // genImageData uses nonblocking BASE lookup and can return null.
            let Some(base) = base.poll_image_data()? else {
                return Ok(None);
            };
            let materials = local
                .as_ref()
                .map(|resource| resource.materials())
                .transpose()?;
            Ok(Some(Self::derive(
                &base,
                materials.as_ref().unwrap_or(&global),
            )))
        })
    }

    pub(crate) fn free_resources(&self) {
        self.texture_cache.release();
        self.source_texture_cache.release();
        self.image_cache.release();
    }

    pub(crate) fn texture_now(
        &self,
        assets: &mut bevy::prelude::Assets<bevy::prelude::Image>,
        global: &Materials,
    ) -> Result<bevy::prelude::Handle<bevy::prelude::Image>, String> {
        self.texture_cache.now(
            assets,
            || self.poll_image_data(global),
            || self.image_data(global),
            || self.image_cache.touch(),
        )
    }

    pub(crate) fn texture_current(&self,assets:&mut bevy::prelude::Assets<bevy::prelude::Image>)->Result<Option<bevy::prelude::Handle<bevy::prelude::Image>>,String> {
        let getter:SourcePalette=Arc::new(crate::main_globals::get_global_materials);
        let texture=self.texture_cache.poll(assets,||self.image_cache.source_poll_native(self.source_generator(getter))?.map(|image|image.lock().unwrap_or_else(|e|e.into_inner()).clone().into_rgba()).transpose())?;
        self.image_cache.touch();
        Ok(texture)
    }

    /// Source palette lookup happens in genImageData, on its loader thread.
    /// Cached texture/image identity bypasses the getter exactly as the source does.
    pub(crate) fn texture_now_current(
        &self,
        assets: &mut bevy::prelude::Assets<bevy::prelude::Image>,
    ) -> Result<bevy::prelude::Handle<bevy::prelude::Image>, String> {
        let getter: SourcePalette = Arc::new(crate::main_globals::get_global_materials);
        let owned = |image: super::image_cache::SourceImage| {
            image
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
                .into_rgba()
        };
        self.texture_cache.now(
            assets,
            || {
                self.image_cache
                    .source_poll_native(self.source_generator(getter.clone()))?
                    .map(owned)
                    .transpose()
            },
            || {
                owned(
                    self.image_cache
                        .source_now_native(self.source_generator(getter.clone()))?,
                )
            },
            || self.image_cache.touch(),
        )
    }

    pub(crate) fn last_used_ms(&self) -> i64 {
        self.image_cache.last_used_ms()
    }

    pub(crate) fn release_if_idle(&self, now_ms: i64) -> bool {
        let expired =
            super::ship_resource_cleanup_predicate::idle_expired(now_ms, self.last_used_ms());
        if expired {
            self.free_resources();
        }
        expired
    }

    pub(crate) fn write_to_folder(
        &self,
        folder: &std::path::Path,
        global: &Materials,
    ) -> Option<std::path::PathBuf> {
        let result = (|| -> Result<std::path::PathBuf, String> {
            if !folder.exists() {
                std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
            }
            let path = folder.join(self.resource.export_filename());
            self.image_data(global)?
                .save_with_format(&path, image::ImageFormat::Png)
                .map_err(|error| error.to_string())?;
            Ok(path)
        })();
        match result {
            Ok(path) => Some(path),
            Err(error) => {
                bevy::log::warn!("derived texture export: {error}");
                None
            }
        }
    }

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
    fn native_derivation_reads_palette_at_generation_and_preserves_raw_metadata() {
        use std::sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "LatePalette{}{}_base.png",
            std::process::id(),
            stamp
        ));
        RgbaImage::from_pixel(2, 1, image::Rgba([0x12, 0x34, 0x56, 123]))
            .save(&path)
            .unwrap();
        let base = super::super::parse_resource_path(&path).unwrap();
        let source = base.source_image_data().unwrap();
        let derived = BaseDerivedTextureShipResource::new(base.clone(), None);
        let visible = Arc::new(
            crate::materials::SourceMaterials::from_json(r##"[{"color":"#123456"}]"##).unwrap(),
        );
        let hidden = Arc::new(
            crate::materials::SourceMaterials::from_json(
                r##"[{"color":"#123456","invisible":true}]"##,
            )
            .unwrap(),
        );
        let current = Arc::new(Mutex::new(visible.clone()));
        let retained = current.clone();
        let reads = Arc::new(AtomicUsize::new(0));
        let calls = reads.clone();
        let getter: SourcePalette = Arc::new(move || {
            calls.fetch_add(1, Ordering::SeqCst);
            retained.lock().unwrap().clone()
        });
        let generate = derived.source_generator(getter.clone());
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        *current.lock().unwrap() = hidden;
        assert_eq!(
            generate().unwrap().unwrap().lock().unwrap().buffer,
            vec![0; 8]
        );
        *current.lock().unwrap() = visible;
        {
            let mut data = source.lock().unwrap();
            data.width = -5;
            data.height = 0;
            data.image_format = 33319;
            data.format = 5126;
            data.buffer.extend([8, 9]);
        }
        let copy = generate().unwrap().unwrap();
        let copy = copy.lock().unwrap();
        assert_eq!(
            (copy.width, copy.height, copy.image_format, copy.format),
            (-5, 0, 33319, 5126)
        );
        assert_eq!(
            copy.buffer,
            [vec![0x12, 0x34, 0x56, 123].repeat(2), vec![0, 0]].concat()
        );
        assert_eq!(&source.lock().unwrap().buffer[8..], &[8, 9]);
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        // A non-material local resource returns null materials and falls back globally.
        let wrong_local = BaseDerivedTextureShipResource::new(base.clone(), Some(base.clone()));
        wrong_local.source_generator(getter)().unwrap().unwrap();
        assert_eq!(reads.load(Ordering::SeqCst), 3);
        let palette_path = path.with_file_name(
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .replace("_base.png", "_materials.json"),
        );
        std::fs::write(&palette_path, r##"[{"color":"#123456","invisible":true}]"##).unwrap();
        let local = super::super::parse_resource_path(&palette_path).unwrap();
        let local_derived = BaseDerivedTextureShipResource::new(base, Some(local));
        let no_global: SourcePalette =
            Arc::new(|| panic!("local materials must suppress the global getter"));
        assert_eq!(
            local_derived.source_generator(no_global)()
                .unwrap()
                .unwrap()
                .lock()
                .unwrap()
                .buffer,
            vec![0; 10]
        );
        std::fs::remove_file(palette_path).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn derived_resource_retains_inputs_caches_until_release_and_exports_texture() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ss2-derived-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let input = root.join("Test_base.png");
        RgbaImage::from_pixel(1, 1, image::Rgba([0x12, 0x34, 0x56, 123]))
            .save(&input)
            .unwrap();
        let base = super::super::parse_resource_path(&input).unwrap();
        // Source derived generation polls BASE; callers must first finish BASE
        // loading when they require a non-null derived image immediately.
        base.image_data().unwrap();
        let resource = BaseDerivedTextureShipResource::new(base.clone(), None);
        assert_eq!(resource.base_resource, base);
        assert!(resource.resource.layer.is_default());
        assert_eq!(resource.resource.resource_type, ShipResourceType::Texture);
        let visible = Materials::from_json(r##"[{"color":"#123456"}]"##).unwrap();
        assert!(resource.poll_image_data(&visible).unwrap().is_none());
        assert_eq!(
            resource.image_data(&visible).unwrap().get_pixel(0, 0).0,
            [0x12, 0x34, 0x56, 123]
        );
        let cloned = resource.clone();
        assert_eq!(
            cloned
                .image_data(&Materials::default())
                .unwrap()
                .get_pixel(0, 0)[3],
            123
        );
        cloned.free_resources();
        assert_eq!(
            resource.image_data(&visible).unwrap().get_pixel(0, 0)[3],
            123
        );
        resource.free_resources();
        let output = resource
            .write_to_folder(&root.join("export"), &visible)
            .unwrap();
        assert_eq!(output.file_name().unwrap(), "Test_texture.png");
        assert_eq!(
            image::open(&output).unwrap().to_rgba8().get_pixel(0, 0)[3],
            123
        );
        let mut assets = bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let texture = resource.texture_now(&mut assets, &visible).unwrap();
        assert_eq!(
            resource.clone().texture_now(&mut assets, &visible).unwrap(),
            texture
        );
        assert_eq!(
            &assets.get(texture.id()).unwrap().data.as_ref().unwrap()[..4],
            &[0x12, 0x34, 0x56, 123]
        );

        // A present empty local palette overrides the global palette.
        let palette_path = root.join("Test_materials.json");
        std::fs::write(&palette_path, "[]").unwrap();
        let local = BaseDerivedTextureShipResource::new(
            base,
            Some(super::super::parse_resource_path(&palette_path).unwrap()),
        );
        assert_eq!(
            local.image_data(&visible).unwrap().get_pixel(0, 0).0,
            [0; 4]
        );
        std::fs::remove_file(output).unwrap();
        std::fs::remove_dir(root.join("export")).unwrap();
        std::fs::remove_file(palette_path).unwrap();
        std::fs::remove_file(input).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

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

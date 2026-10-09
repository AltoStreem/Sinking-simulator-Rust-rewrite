use super::ship_resource::ShipResource;
use std::{
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub(crate) struct FileShipResource {
    resource: ShipResource,
    pub(crate) path: PathBuf,
    // Kotlin's synchronized lazy retains the palette, including the empty
    // palette returned by Materials(File) after a read or parse failure.
    materials_cache: Arc<Mutex<Option<Arc<crate::materials::SourceMaterials>>>>,
    image_cache: super::image_cache::ImageCache,
    texture_cache: super::texture_cache::TextureCache,
    source_texture_cache: super::source_texture_cache::SourceTextureCache,
}

impl PartialEq for FileShipResource {
    fn eq(&self, other: &Self) -> bool {
        // Clones retain source object identity; independent constructors do not.
        Arc::ptr_eq(&self.materials_cache, &other.materials_cache)
    }
}
impl Eq for FileShipResource {}

impl FileShipResource {
    pub(crate) fn set_ship(&mut self, ship: String) {
        self.resource.set_ship(ship);
    }

    pub(crate) fn source_texture(
        &self,
        environment: &super::SourceTextureEnvironment,
    ) -> Result<Option<Arc<crate::texture_2d::SourceTexture2D>>, String> {
        self.source_texture_cache.poll(
            environment,
            || self.source_poll_image_data(),
            || self.image_cache.touch(),
        )
    }
    pub(crate) fn source_texture_now(
        &self,
        environment: &super::SourceTextureEnvironment,
    ) -> Result<Arc<crate::texture_2d::SourceTexture2D>, String> {
        self.source_texture_cache.now(
            environment,
            || self.source_poll_image_data(),
            || self.source_image_data(),
            || self.image_cache.touch(),
        )
    }
    pub(super) fn source_poll_image_data(
        &self,
    ) -> Result<Option<super::image_cache::SourceImage>, String> {
        if self.resource_type == super::ShipResourceType::Materials {
            self.image_cache.touch();
            return Ok(None);
        }
        self.image_cache.source_poll(self.image_generator())
    }
    pub(crate) fn texture_now(
        &self,
        assets: &mut bevy::prelude::Assets<bevy::prelude::Image>,
    ) -> Result<bevy::prelude::Handle<bevy::prelude::Image>, String> {
        self.texture_cache.now(
            assets,
            || self.poll_image_data(),
            || self.image_data(),
            || self.image_cache.touch(),
        )
    }

    /// ShipResource.getTexture is nonblocking, including on first access.
    pub(crate) fn texture(&self,assets:&mut bevy::prelude::Assets<bevy::prelude::Image>)->Result<Option<bevy::prelude::Handle<bevy::prelude::Image>>,String> {
        let texture=self.texture_cache.poll(assets,||self.poll_image_data())?;
        self.image_cache.touch();
        Ok(texture)
    }

    pub(crate) fn close_texture(&self, assets: &mut bevy::prelude::Assets<bevy::prelude::Image>) {
        self.texture_cache.close(assets);
        self.source_texture_cache.release();
        self.image_cache.release();
    }

    /// Source writeToFolder creates the directory and returns null on an
    /// exception. PNG pixels match; encoder bytes may differ from STB.
    pub(crate) fn write_to_folder(&self, folder: &std::path::Path) -> Option<PathBuf> {
        let result = (|| -> Result<PathBuf, String> {
            if !folder.exists() {
                std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
            }
            let target = folder.join(self.export_filename());
            if self.resource_type == super::resource_type::ResourceType::Materials {
                std::fs::write(&target, self.materials()?.to_json()?)
                    .map_err(|error| error.to_string())?;
            } else {
                self.image_data()?
                    .save_with_format(&target, image::ImageFormat::Png)
                    .map_err(|error| error.to_string())?;
            }
            Ok(target)
        })();
        match result {
            Ok(path) => Some(path),
            Err(error) => {
                bevy::log::warn!("ship resource export: {error}");
                None
            }
        }
    }

    pub(crate) fn image_data(&self) -> Result<image::RgbaImage, String> {
        if self.resource_type == super::resource_type::ResourceType::Materials {
            return Err(format!(
                "{} is a material palette, not an image",
                self.path.display()
            ));
        }
        self.image_cache.now(self.image_generator())
    }

    pub(crate) fn source_image_data(&self) -> Result<super::image_cache::SourceImage, String> {
        if self.resource_type == super::resource_type::ResourceType::Materials {
            return Err(format!(
                "{} is a material palette, not an image",
                self.path.display()
            ));
        }
        self.image_cache.source_now(self.image_generator())
    }

    /// Source getImageData: first call schedules loading and returns null;
    /// later calls publish the completed result without blocking.
    pub(crate) fn poll_image_data(&self) -> Result<Option<image::RgbaImage>, String> {
        if self.resource_type == super::resource_type::ResourceType::Materials {
            self.image_cache.touch();
            return Ok(None);
        }
        self.image_cache.poll(self.image_generator())
    }

    fn image_generator(
        &self,
    ) -> Arc<dyn Fn() -> Result<Option<image::RgbaImage>, String> + Send + Sync> {
        let path = self.path.clone();
        Arc::new(move || {
            crate::file_reader::FileReader::game()
                .read_image(&path, 4)
                .and_then(|image| image.into_rgba())
                .map(Some)
                .map_err(|error| format!("{}: {error}", path.display()))
        })
    }

    /// The source releases image/texture data without resetting lazy materials
    /// or cancelling an in-flight image loader.
    pub(crate) fn free_resources(&self) {
        self.texture_cache.release();
        self.source_texture_cache.release();
        self.image_cache.release();
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

    pub(crate) fn materials(&self) -> Result<crate::materials::Materials, String> {
        self.source_materials()
            .map(|materials| materials.to_owned_adapter())
    }

    /// Retain the native palette object and its shared color aliases. Repeated
    /// reads, including reads through resource clones, return the same object.
    pub(crate) fn source_materials(
        &self,
    ) -> Result<Arc<crate::materials::SourceMaterials>, String> {
        self.source_materials_nullable()
            .ok_or_else(|| format!("{} is not a material palette", self.path.display()))
    }

    /// Original nullable materials getter; the non-null port adapter above
    /// reports a type mismatch without changing the initializer's null result.
    pub(crate) fn source_materials_nullable(
        &self,
    ) -> Option<Arc<crate::materials::SourceMaterials>> {
        super::ship_resource_materials_initializer::invoke(
            || self.resource_type,
            || {
                let mut cache = self
                    .materials_cache
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                if let Some(materials) = cache.as_ref() {
                    return materials.clone();
                }
                let materials = crate::materials::SourceMaterials::from_file(&self.path, |error| {
                    bevy::log::warn!("material palette {}: {error}", self.path.display());
                });
                let materials = Arc::new(materials);
                *cache = Some(materials.clone());
                materials
            },
        )
    }

    pub(crate) fn new(path: PathBuf, resource: ShipResource) -> Result<Self, String> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = crate::main_flat_files::kotlin_file_extension(name);
        if !extension.eq_ignore_ascii_case(resource.resource_type.extension()) {
            return Err(format!(
                "resource {} has .{extension}; {} resources require .{}",
                path.display(),
                resource.resource_type.name(),
                resource.resource_type.extension()
            ));
        }
        Ok(Self {
            resource,
            path,
            materials_cache: Arc::new(Mutex::new(None)),
            image_cache: super::image_cache::ImageCache::default(),
            texture_cache: super::texture_cache::TextureCache::default(),
            source_texture_cache: super::source_texture_cache::SourceTextureCache::default(),
        })
    }
}

impl Deref for FileShipResource {
    type Target = ShipResource;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

#[cfg(test)]
mod tests {
    use super::super::{layer::Layer, resource_type::ResourceType};
    use super::*;

    #[test]
    fn file_resource_equality_uses_source_object_identity() {
        let metadata = ShipResource::new("Test".into(), Layer::default(), ResourceType::Base);
        let first =
            FileShipResource::new(PathBuf::from("Test_base.png"), metadata.clone()).unwrap();
        let second = FileShipResource::new(PathBuf::from("Test_base.png"), metadata).unwrap();
        assert_eq!(first, first.clone());
        assert_ne!(first, second);
        let a = super::super::BaseDerivedTextureShipResource::new(first.clone(), None);
        let b = super::super::BaseDerivedTextureShipResource::new(first, None);
        let c = super::super::BaseDerivedTextureShipResource::new(second, None);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn export_creates_folder_names_layers_and_preserves_rgba_pixels() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ss2-export-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let input = root.join("input.png");
        let image = image::RgbaImage::from_pixel(2, 1, image::Rgba([5, 17, 29, 63]));
        image.save(&input).unwrap();
        let resource = FileShipResource::new(
            input.clone(),
            ShipResource::new(
                "Titanic".into(),
                Layer::new("exterior"),
                ResourceType::Texture,
            ),
        )
        .unwrap();
        let output = resource.write_to_folder(&root.join("nested")).unwrap();
        assert_eq!(output.file_name().unwrap(), "Titanic_exterior_texture.png");
        assert_eq!(image::open(&output).unwrap().to_rgba8(), image);
        // A file supplied as the destination fails as in the source catch path.
        assert!(resource.write_to_folder(&input).is_none());
        let palette_path = root.join("input.json");
        std::fs::write(
            &palette_path,
            r##"[{"name":"steel","color":"#123456","mass":42}]"##,
        )
        .unwrap();
        let palette = FileShipResource::new(
            palette_path.clone(),
            ShipResource::new("Titanic".into(), Layer::default(), ResourceType::Materials),
        )
        .unwrap();
        let native = palette.source_materials().unwrap();
        assert!(Arc::ptr_eq(
            &native,
            &palette.clone().source_materials().unwrap()
        ));
        palette.free_resources();
        assert!(Arc::ptr_eq(&native, &palette.source_materials().unwrap()));
        let palette_output = palette.write_to_folder(&root.join("nested")).unwrap();
        assert_eq!(
            palette_output.file_name().unwrap(),
            "Titanic_materials.json"
        );
        let json = std::fs::read_to_string(&palette_output).unwrap();
        assert_eq!(
            crate::materials::Materials::from_json(&json)
                .unwrap()
                .get(0x123456)
                .unwrap()
                .mass,
            42.0
        );
        std::fs::remove_file(palette_output).unwrap();
        std::fs::remove_file(palette_path).unwrap();
        std::fs::remove_file(output).unwrap();
        std::fs::remove_dir(root.join("nested")).unwrap();
        std::fs::remove_file(input).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn palette_caches_empty_failure_and_success_across_clones() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("ss2-palette-{}-{stamp}.json", std::process::id()));
        std::fs::write(&path, "invalid json").unwrap();
        let resource = FileShipResource::new(
            path.clone(),
            ShipResource::new("Test".into(), Layer::default(), ResourceType::Materials),
        )
        .unwrap();
        assert!(resource.materials().unwrap().materials.is_empty());
        let empty = resource.source_materials().unwrap();
        std::fs::write(&path, r##"[{"name":"steel","color":"#123456","mass":42}]"##).unwrap();
        // Fixing the file does not reset the original Kotlin lazy value.
        resource.free_resources();
        assert!(Arc::ptr_eq(
            &empty,
            &resource.clone().source_materials().unwrap()
        ));
        assert!(resource.clone().materials().unwrap().materials.is_empty());
        let fresh = FileShipResource::new(
            path.clone(),
            ShipResource::new("Test".into(), Layer::default(), ResourceType::Materials),
        )
        .unwrap();
        assert_eq!(fresh.materials().unwrap().get(0x123456).unwrap().mass, 42.0);
        let retained = fresh.clone();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            retained.materials().unwrap().get(0x123456).unwrap().mass,
            42.0
        );
        let missing = FileShipResource::new(
            path.clone(),
            ShipResource::new("Test".into(), Layer::default(), ResourceType::Materials),
        )
        .unwrap();
        assert!(missing.materials().unwrap().materials.is_empty());
        std::fs::write(&path, r##"[{"color":"#123456","mass":99}]"##).unwrap();
        assert!(missing.materials().unwrap().materials.is_empty());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn image_first_poll_is_pending_and_free_reloads_shared_cache() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("ss2-image-{}-{stamp}.png", std::process::id()));
        image::RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 255]))
            .save(&path)
            .unwrap();
        let resource = FileShipResource::new(
            path.clone(),
            ShipResource::new("Test".into(), Layer::default(), ResourceType::Base),
        )
        .unwrap();
        assert!(resource.poll_image_data().unwrap().is_none());
        assert_eq!(
            resource.image_data().unwrap().get_pixel(0, 0).0,
            [1, 2, 3, 255]
        );
        let reference = resource.clone();
        image::RgbaImage::from_pixel(1, 1, image::Rgba([4, 5, 6, 255]))
            .save(&path)
            .unwrap();
        assert_eq!(
            reference.image_data().unwrap().get_pixel(0, 0).0,
            [1, 2, 3, 255]
        );
        reference.free_resources();
        assert_eq!(
            resource.image_data().unwrap().get_pixel(0, 0).0,
            [4, 5, 6, 255]
        );
        std::fs::remove_file(path).unwrap();
    }
}

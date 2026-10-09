//! ShipUpload.selectResource: resource selectors and image previews, without Steam.
use crate::{
    dslfix::{DslFix, GeometryBackend, TextureDimensions},
    ship_resources::{ShipLayer, ShipResourceType},
    ship_thumbnail::ThumbnailResource,
    ship_upload::SourceShipUpload,
};
use bevy::math::{Vec2, Vec4};
/// Active adapter lookup uses the editor's actual resource map. It must not
/// synthesize a missing selector texture through ShipThumbnail.getResource.
pub(crate) fn selected_texture_now(
    upload: &SourceShipUpload,
    kind: ShipResourceType,
    layer: &ShipLayer,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
) -> Result<Option<bevy::prelude::Handle<bevy::prelude::Image>>, String> {
    if kind == ShipResourceType::Materials {
        return Ok(None);
    }
    let Some(resource) = upload
        .layers()
        .get(layer)
        .and_then(|resources| resources.get(&kind))
    else {
        return Ok(None);
    };
    match resource {
        ThumbnailResource::File(resource) => resource.texture_now(images).map(Some),
        ThumbnailResource::BaseDerivedTexture(resource) => {
            resource.texture_now_current(images).map(Some)
        }
    }
}

pub(crate) trait ResourceBackend: GeometryBackend {
    type Texture: TextureDimensions;
    /// A null or still-loading source texture must take the selector-button path.
    fn resource_texture(&mut self, resource: &ThumbnailResource) -> Option<Self::Texture>;
    fn resource_has_materials(&mut self, resource: &ThumbnailResource) -> bool;
    /// FileShipResource.file.canonicalPath, or null for generated resources.
    fn resource_canonical_path(&mut self, resource: &ThumbnailResource) -> Option<String>;
    fn button(&mut self, label: &str, size: Vec2) -> bool;
    fn select_file(
        &mut self,
        upload: &mut SourceShipUpload,
        kind: ShipResourceType,
        layer: &ShipLayer,
    );
    fn text(&mut self, text: &str);
    fn frame_padding_y(&mut self) -> f32;
    fn texture_id(&mut self, texture: &Self::Texture) -> i32;
    fn image_button(
        &mut self,
        texture: i32,
        size: Vec2,
        uv0: Vec2,
        uv1: Vec2,
        padding: i32,
        background: Vec4,
        tint: Vec4,
    ) -> bool;
    fn description(&mut self, description: &str);
}

impl SourceShipUpload {
    pub(crate) fn render_resource<B: ResourceBackend>(
        &mut self,
        kind: ShipResourceType,
        layer: &ShipLayer,
        backend: &mut B,
    ) {
        // Retain the selected resource across picker mutations, as the source local variable does.
        let Some(resource) = self
            .layers()
            .get(layer)
            .map(|resources| resources.get(&kind).cloned())
        else {
            return;
        };
        let texture = resource
            .as_ref()
            .and_then(|resource| backend.resource_texture(resource));
        let Some(texture) = texture else {
            let label = format!(
                "{} ({})",
                kind.name(),
                if kind.is_required() {
                    "Required"
                } else {
                    "Optional"
                }
            );
            let size = Vec2::new(
                backend.window_content_region_width(),
                30.0 * backend.gui_scale(),
            );
            if backend.button(&label, size) {
                backend.select_file(self, kind, layer);
            }
            if let Some(resource) = resource
                .as_ref()
                .filter(|resource| backend.resource_has_materials(resource))
            {
                let path = backend
                    .resource_canonical_path(resource)
                    .unwrap_or_else(|| "Generated materials selected".into());
                backend.text(&path);
            }
            return;
        };
        let padding = backend.frame_padding_y() as i32;
        let pad = Vec2::splat(padding as f32);
        let maximum = 200.0 * backend.gui_scale() as f64;
        let size = DslFix::adjust_texture_size(&texture, maximum, pad, backend);
        DslFix::center_next_element(size, pad, backend);
        let id = backend.texture_id(&texture);
        if backend.image_button(
            id,
            size,
            Vec2::ZERO,
            Vec2::ONE,
            padding,
            Vec4::ZERO,
            Vec4::ONE,
        ) {
            backend.select_file(self, kind, layer);
        }
        backend.description(&format!(
            "Click to change the {} {}",
            kind.name(),
            kind.extension()
        ));
    }

    pub(crate) fn render_resource_default<B: ResourceBackend>(
        &mut self,
        kind: ShipResourceType,
        layer: Option<&ShipLayer>,
        mask: i32,
        backend: &mut B,
    ) {
        let default = ShipLayer::default();
        let layer = if mask & 2 != 0 {
            &default
        } else {
            let Some(layer) = layer else {
                return;
            };
            layer
        };
        self.render_resource(kind, layer, backend);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_resource_lookup_preserves_unselected_texture_and_independent_layer_images() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "UploadResource{}{}_base.png",
            std::process::id(),
            stamp
        ));
        image::RgbaImage::from_pixel(3, 2, image::Rgba([10, 20, 30, 255]))
            .save(&path)
            .unwrap();
        let mut upload = SourceShipUpload::default();
        upload.select_resource(
            ShipResourceType::Base,
            crate::ship_resources::parse_resource_path(&path).unwrap(),
            ShipLayer::default(),
        );
        assert!(upload.thumbnail().unwrap().is_some());
        let mut images = bevy::prelude::Assets::default();
        assert!(
            selected_texture_now(
                &upload,
                ShipResourceType::Texture,
                &ShipLayer::default(),
                &mut images
            )
            .unwrap()
            .is_none()
        );
        let base = selected_texture_now(
            &upload,
            ShipResourceType::Base,
            &ShipLayer::default(),
            &mut images,
        )
        .unwrap()
        .unwrap();
        assert_eq!(images.get(&base).unwrap().texture_descriptor.size.width, 3);
        upload.reset();
        let layer = ShipLayer::new("exterior");
        upload.select_resource(
            ShipResourceType::Texture,
            crate::ship_resources::parse_resource_path(&path).unwrap(),
            layer.clone(),
        );
        assert!(upload.thumbnail().unwrap().is_none());
        assert!(
            selected_texture_now(
                &upload,
                ShipResourceType::Texture,
                &ShipLayer::default(),
                &mut images
            )
            .unwrap()
            .is_none()
        );
        let texture = selected_texture_now(&upload, ShipResourceType::Texture, &layer, &mut images)
            .unwrap()
            .unwrap();
        assert_eq!(
            images.get(&texture).unwrap().texture_descriptor.size.height,
            2
        );
        assert!(
            selected_texture_now(&upload, ShipResourceType::Materials, &layer, &mut images)
                .unwrap()
                .is_none()
        );
        std::fs::remove_file(path).unwrap();
    }
    struct Texture;
    impl TextureDimensions for Texture {
        fn width(&self) -> i32 {
            1000
        }
        fn height(&self) -> i32 {
            500
        }
    }
    struct Backend {
        log: Vec<String>,
        texture: bool,
        materials: bool,
        generated: bool,
        clicked: bool,
        size: Option<Vec2>,
    }
    impl GeometryBackend for Backend {
        fn gui_scale(&mut self) -> f32 {
            1.25
        }
        fn window_content_region_width(&mut self) -> f32 {
            500.0
        }
        fn window_height(&mut self) -> f32 {
            1000.0
        }
        fn frame_padding(&mut self) -> Vec2 {
            Vec2::new(5.0, 3.75)
        }
        fn scroll_max_y(&mut self) -> f32 {
            1.0
        }
        fn window_width(&mut self) -> f32 {
            530.0
        }
        fn scrollbar_size(&mut self) -> f32 {
            17.0
        }
        fn set_cursor_pos_x(&mut self, x: f32) {
            self.log.push(format!("center:{x}"));
        }
    }
    impl ResourceBackend for Backend {
        type Texture = Texture;
        fn resource_texture(&mut self, _: &ThumbnailResource) -> Option<Texture> {
            self.log.push("texture".into());
            self.texture.then_some(Texture)
        }
        fn resource_has_materials(&mut self, _: &ThumbnailResource) -> bool {
            self.log.push("materials".into());
            self.materials
        }
        fn resource_canonical_path(&mut self, _: &ThumbnailResource) -> Option<String> {
            self.log.push("path".into());
            (!self.generated).then(|| "C:/ships/old_materials.json".into())
        }
        fn button(&mut self, label: &str, size: Vec2) -> bool {
            self.log.push(label.into());
            self.size = Some(size);
            self.clicked
        }
        fn select_file(
            &mut self,
            upload: &mut SourceShipUpload,
            _: ShipResourceType,
            layer: &ShipLayer,
        ) {
            self.log.push(format!("select:{}", layer.display_name()));
            upload.reset();
        }
        fn text(&mut self, text: &str) {
            self.log.push(text.into());
        }
        fn frame_padding_y(&mut self) -> f32 {
            self.log.push("padding".into());
            3.75
        }
        fn texture_id(&mut self, _: &Texture) -> i32 {
            self.log.push("id".into());
            42
        }
        fn image_button(
            &mut self,
            id: i32,
            size: Vec2,
            uv0: Vec2,
            uv1: Vec2,
            padding: i32,
            background: Vec4,
            tint: Vec4,
        ) -> bool {
            assert_eq!(id, 42);
            assert_eq!(uv0, Vec2::ZERO);
            assert_eq!(uv1, Vec2::ONE);
            assert_eq!(padding, 3);
            assert_eq!(background, Vec4::ZERO);
            assert_eq!(tint, Vec4::ONE);
            self.log.push("image".into());
            self.size = Some(size);
            self.clicked
        }
        fn description(&mut self, text: &str) {
            self.log.push(text.into());
        }
    }
    fn backend() -> Backend {
        Backend {
            log: vec![],
            texture: false,
            materials: false,
            generated: false,
            clicked: false,
            size: None,
        }
    }
    fn resource(kind: ShipResourceType) -> crate::ship_resources::ShipResourceFile {
        crate::ship_resources::ShipResourceFile::new(
            format!("old.{}", kind.extension()).into(),
            crate::ship_resources::ShipResource::new("Ship".into(), ShipLayer::default(), kind),
        )
        .unwrap()
    }
    #[test]
    fn resource_selectors_preserve_missing_layer_null_texture_and_old_materials_after_picker() {
        let mut upload = SourceShipUpload::default();
        let mut backend = backend();
        upload.render_resource(
            ShipResourceType::Base,
            &ShipLayer::new("absent"),
            &mut backend,
        );
        assert!(backend.log.is_empty());
        upload.render_resource_default(ShipResourceType::Base, None, 2, &mut backend);
        assert_eq!(backend.log, ["BASE (Required)"]);
        assert_eq!(backend.size, Some(Vec2::new(500.0, 37.5)));
        upload.select_resource(
            ShipResourceType::Materials,
            resource(ShipResourceType::Materials),
            ShipLayer::default(),
        );
        backend.log.clear();
        backend.materials = true;
        backend.clicked = true;
        upload.render_resource(
            ShipResourceType::Materials,
            &ShipLayer::default(),
            &mut backend,
        );
        assert_eq!(
            backend.log,
            [
                "texture",
                "MATERIALS (Optional)",
                "select:Default",
                "materials",
                "path",
                "C:/ships/old_materials.json"
            ]
        );
        assert!(upload.layers()[&ShipLayer::default()].is_empty());
    }
    #[test]
    fn texture_preview_uses_source_cap_padding_centering_and_picker_description_order() {
        let mut upload = SourceShipUpload::default();
        let layer = ShipLayer::new("exterior");
        upload.select_resource(
            ShipResourceType::Texture,
            resource(ShipResourceType::Texture),
            layer.clone(),
        );
        let mut backend = backend();
        backend.texture = true;
        backend.clicked = true;
        upload.render_resource(ShipResourceType::Texture, &layer, &mut backend);
        assert_eq!(backend.size, Some(Vec2::new(494.0, 244.0)));
        assert_eq!(
            backend.log,
            [
                "texture",
                "padding",
                "center:6.5",
                "id",
                "image",
                "select:exterior",
                "Click to change the TEXTURE png"
            ]
        );
    }
    #[test]
    fn generated_materials_and_loading_image_use_the_source_fallback_paths() {
        let mut upload = SourceShipUpload::default();
        upload.select_resource(
            ShipResourceType::Materials,
            resource(ShipResourceType::Materials),
            ShipLayer::default(),
        );
        let mut backend = backend();
        backend.materials = true;
        backend.generated = true;
        upload.render_resource_default(
            ShipResourceType::Materials,
            Some(&ShipLayer::new("ignored")),
            2,
            &mut backend,
        );
        assert_eq!(backend.log.last().unwrap(), "Generated materials selected");
        backend.log.clear();
        backend.materials = false;
        upload.select_resource(
            ShipResourceType::Texture,
            resource(ShipResourceType::Texture),
            ShipLayer::default(),
        );
        upload.render_resource_default(ShipResourceType::Texture, None, 2, &mut backend);
        assert_eq!(backend.log, ["texture", "TEXTURE (Optional)", "materials"]);
    }
}

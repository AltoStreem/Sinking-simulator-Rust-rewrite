//! Rust translation of `ship/ShipThumbnail.java` resource grouping and lookup.

use crate::ship_resources::{
    BaseDerivedTextureShipResource, ShipLayer, ShipResourceFile, ShipResourceType,
    parse_resource_path,
};
use std::collections::HashMap;

/// Source `ShipThumbnail` stores resources by layer and then by resource type.
/// Base-derived textures are represented explicitly because the Java class
/// synthesizes one when the default layer has no TEXTURE resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ThumbnailResource {
    File(ShipResourceFile),
    BaseDerivedTexture(BaseDerivedTextureShipResource),
}

#[derive(Clone, Debug)]
pub(crate) struct ShipThumbnail {
    layers: HashMap<ShipLayer, HashMap<ShipResourceType, ThumbnailResource>>,
    layer_order: Vec<ShipLayer>,
}

impl PartialEq for ShipThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.layers == other.layers
    }
}
impl Eq for ShipThumbnail {}

/// Bind the source global palette for the original ShipData(thumbnail) overload.
/// All getter calls still reach the resource's shared lazy/image cache.
pub(crate) struct SourceShipDataThumbnail<'a> {
    pub(crate) thumbnail: &'a ShipThumbnail,
    pub(crate) global: &'a std::sync::Arc<crate::materials::SourceMaterials>,
}

impl crate::ship_data::SourceShipDataThumbnail for SourceShipDataThumbnail<'_> {
    type Palette = crate::materials::SourceMaterials;
    fn base_layer(
        &self,
    ) -> Result<std::sync::Arc<std::sync::Mutex<crate::image_data::ImageData>>, String> {
        match self
            .thumbnail
            .get_resource(ShipResourceType::Base, &ShipLayer::default())
        {
            Some(ThumbnailResource::File(resource)) => resource.source_image_data(),
            _ => Err("ship is missing its default-layer base image".into()),
        }
    }
    fn materials(&self) -> Result<std::sync::Arc<Self::Palette>, String> {
        self.thumbnail.source_materials(self.global)
    }
}
pub(crate) struct SourceShipDataThumbnailCurrent<'a> {
    pub(crate) thumbnail: &'a ShipThumbnail,
    pub(crate) global: &'a dyn Fn() -> std::sync::Arc<crate::materials::SourceMaterials>,
}
impl crate::ship_data::SourceShipDataThumbnail for SourceShipDataThumbnailCurrent<'_> {
    type Palette = crate::materials::SourceMaterials;
    fn base_layer(
        &self,
    ) -> Result<std::sync::Arc<std::sync::Mutex<crate::image_data::ImageData>>, String> {
        match self
            .thumbnail
            .get_resource(ShipResourceType::Base, &ShipLayer::default())
        {
            Some(ThumbnailResource::File(resource)) => resource.source_image_data(),
            _ => Err("ship is missing its default-layer base image".into()),
        }
    }
    fn materials(&self) -> Result<std::sync::Arc<Self::Palette>, String> {
        self.thumbnail.source_materials_current(self.global)
    }
}

impl ShipThumbnail {
    /// Collect sibling resources using the source's exact ship-name identity.
    pub(crate) fn from_base_file(path: &std::path::Path) -> Result<Self, String> {
        let base = parse_resource_path(path)?;
        let parent = path
            .parent()
            .ok_or_else(|| "base image has no parent directory".to_owned())?;
        let entries = std::fs::read_dir(parent).map_err(|error| error.to_string())?;
        let mut resources = Vec::new();
        for entry in entries {
            let path = entry.map_err(|error| error.to_string())?.path();
            if !path.is_file() {
                continue;
            }
            if let Ok(resource) = parse_resource_path(&path) {
                if resource.ship == base.ship {
                    resources.push(resource);
                }
            }
        }
        Self::new(resources)
    }

    pub(crate) fn base_layer(&self) -> Result<image::RgbaImage, String> {
        match self.get_resource(ShipResourceType::Base, &ShipLayer::default()) {
            Some(ThumbnailResource::File(resource)) => resource.image_data(),
            _ => Err("ship is missing its default-layer base image".to_owned()),
        }
    }

    /// Java falls back to the global palette when the local resource is absent.
    /// A present but unreadable palette produces a cached empty Materials value.
    pub(crate) fn materials(
        &self,
        global: &crate::materials::Materials,
    ) -> crate::materials::Materials {
        if let Some(ThumbnailResource::File(resource)) =
            self.get_resource(ShipResourceType::Materials, &ShipLayer::default())
        {
            match resource.materials() {
                Ok(materials) => return materials,
                Err(error) => bevy::log::warn!("Could not load ship material palette: {error}"),
            }
        }
        global.clone()
    }

    /// Native ShipData consumers retain the local lazy palette object or the
    /// actual global palette object when no local resource is present.
    pub(crate) fn source_materials(
        &self,
        global: &std::sync::Arc<crate::materials::SourceMaterials>,
    ) -> Result<std::sync::Arc<crate::materials::SourceMaterials>, String> {
        self.source_materials_current(|| global.clone())
    }
    pub(crate) fn source_materials_current(
        &self,
        global: impl Fn() -> std::sync::Arc<crate::materials::SourceMaterials>,
    ) -> Result<std::sync::Arc<crate::materials::SourceMaterials>, String> {
        match self.get_resource(ShipResourceType::Materials, &ShipLayer::default()) {
            Some(ThumbnailResource::File(resource)) => resource.source_materials(),
            _ => Ok(global()),
        }
    }

    pub(crate) fn texture(
        &self,
        layer: &ShipLayer,
        global: &crate::materials::Materials,
    ) -> Result<Option<image::RgbaImage>, String> {
        match self.get_resource(ShipResourceType::Texture, layer) {
            Some(ThumbnailResource::File(resource)) => resource.image_data().map(Some),
            Some(ThumbnailResource::BaseDerivedTexture(resource)) => {
                resource.image_data(global).map(Some)
            }
            None => Ok(None),
        }
    }

    /// Group input resources. Later duplicate resource types replace earlier
    /// values, matching Kotlin `associateBy` in the decompiled constructor.
    pub(crate) fn new(
        resources: impl IntoIterator<Item = ShipResourceFile>,
    ) -> Result<Self, String> {
        let resources: Vec<_> = resources.into_iter().collect();
        let mut names = Vec::new();
        for resource in &resources {
            if !names.contains(&resource.ship) {
                names.push(resource.ship.clone());
            }
        }
        if names.len() > 1 {
            return Err(format!(
                "cannot build a ship from resources belonging to different ships: {names:?}"
            ));
        }

        let mut layers: HashMap<ShipLayer, HashMap<ShipResourceType, ThumbnailResource>> =
            HashMap::new();
        let mut layer_order = Vec::new();
        for resource in resources {
            if !layers.contains_key(&resource.layer) {
                layer_order.push(resource.layer.clone());
            }
            layers
                .entry(resource.layer.clone())
                .or_default()
                .insert(resource.resource_type, ThumbnailResource::File(resource));
        }

        let default_layer = ShipLayer::default();
        let Some(default_resources) = layers.get_mut(&default_layer) else {
            return Err("cannot build ship: missing default-layer BASE resource".to_owned());
        };
        if !default_resources.contains_key(&ShipResourceType::Base) {
            return Err("cannot build ship: missing default-layer BASE resource".to_owned());
        }
        if !default_resources.contains_key(&ShipResourceType::Texture) {
            let base = match &default_resources[&ShipResourceType::Base] {
                ThumbnailResource::File(resource) => resource.clone(),
                _ => unreachable!("BASE resources originate from files"),
            };
            let materials = match default_resources.get(&ShipResourceType::Materials) {
                Some(ThumbnailResource::File(resource)) => Some(resource.clone()),
                _ => None,
            };
            default_resources.insert(
                ShipResourceType::Texture,
                ThumbnailResource::BaseDerivedTexture(BaseDerivedTextureShipResource::new(
                    base, materials,
                )),
            );
        }

        Ok(Self {
            layers,
            layer_order,
        })
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self.get_resource(ShipResourceType::Base, &ShipLayer::default())? {
            ThumbnailResource::File(resource) => Some(&resource.ship),
            ThumbnailResource::BaseDerivedTexture(_) => None,
        }
    }

    pub(crate) fn get_resource(
        &self,
        resource_type: ShipResourceType,
        layer: &ShipLayer,
    ) -> Option<&ThumbnailResource> {
        self.layers.get(layer)?.get(&resource_type)
    }

    pub(crate) fn layers(
        &self,
    ) -> &HashMap<ShipLayer, HashMap<ShipResourceType, ThumbnailResource>> {
        &self.layers
    }

    /// Kotlin groupBy/mapValues preserve first encounter order even when a
    /// later resource replaces the value for an existing type.
    pub(crate) fn ordered_layers(&self) -> &[ShipLayer] {
        &self.layer_order
    }
}

#[cfg(test)]
mod source_palette_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn native_thumbnail_reads_current_palette_per_pixel_and_local_palette_suppresses_global() {
        use std::{cell::Cell, sync::Arc};
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!("CurrentPalette{}{}", std::process::id(), stamp);
        let path = std::env::temp_dir().join(format!("{name}_base.png"));
        let local_path = std::env::temp_dir().join(format!("{name}_materials.json"));
        image::RgbaImage::from_raw(
            3,
            1,
            vec![
                0x12, 0x34, 0x56, 0, 0x12, 0x34, 0x56, 64, 0x12, 0x34, 0x56, 255,
            ],
        )
        .unwrap()
        .save(&path)
        .unwrap();
        let base = parse_resource_path(&path).unwrap();
        base.source_image_data().unwrap();
        let thumbnail = ShipThumbnail::new([base.clone()]).unwrap();
        let palette = |mass| {
            Arc::new(
                crate::materials::SourceMaterials::from_json(&format!(
                    r##"[{{"color":"#123456","mass":{mass}}}]"##
                ))
                .unwrap(),
            )
        };
        let captured = palette(11);
        let first = palette(22);
        let empty = Arc::new(crate::materials::SourceMaterials::default());
        let last = palette(44);
        let palettes = [captured.clone(), first.clone(), empty, last.clone()];
        let calls = Cell::new(0);
        let getter = || {
            let index = calls.get();
            calls.set(index + 1);
            palettes[index].clone()
        };
        let data =
            crate::ship_data::SourceShipData::from_thumbnail(&SourceShipDataThumbnailCurrent {
                thumbnail: &thumbnail,
                global: &getter,
            })
            .unwrap();
        assert_eq!(calls.get(), 4);
        assert!(Arc::ptr_eq(&data.materials, &captured));
        let buffer = data.material_buffer.borrow();
        assert!(Arc::ptr_eq(
            buffer[0].as_ref().unwrap(),
            &first.get(0x123456).unwrap()
        ));
        assert!(buffer[1].is_none());
        assert!(Arc::ptr_eq(
            buffer[2].as_ref().unwrap(),
            &last.get(0x123456).unwrap()
        ));
        drop(buffer);
        std::fs::write(
            &local_path,
            r##"[{"color":"#123456","mass":66,"invisible":true}]"##,
        )
        .unwrap();
        let local = parse_resource_path(&local_path).unwrap();
        let thumbnail = ShipThumbnail::new([base, local.clone()]).unwrap();
        let getter = || panic!("local palette must avoid the global getter");
        let data =
            crate::ship_data::SourceShipData::from_thumbnail(&SourceShipDataThumbnailCurrent {
                thumbnail: &thumbnail,
                global: &getter,
            })
            .unwrap();
        let retained = local.source_materials().unwrap();
        assert!(Arc::ptr_eq(&data.materials, &retained));
        assert!(
            data.material_buffer
                .borrow()
                .iter()
                .all(|m| Arc::ptr_eq(m.as_ref().unwrap(), &retained.get(0x123456).unwrap()))
        );
        assert!(data.material_buffer.borrow()[0].as_ref().unwrap().invisible);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(local_path).unwrap();
    }
    #[test]
    fn original_ship_data_constructor_loads_real_thumbnail_and_retains_resource_image_and_aliases()
    {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ss2-native-data-{stamp}"));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("Identity_base.png");
        image::RgbaImage::from_raw(
            3,
            1,
            vec![0x12, 0x34, 0x56, 0, 0xab, 0xcd, 0xef, 255, 1, 2, 3, 255],
        )
        .unwrap()
        .save(&path)
        .unwrap();
        let base = parse_resource_path(&path).unwrap();
        let thumbnail = ShipThumbnail::new([base.clone()]).unwrap();
        let global = Arc::new(crate::materials::SourceMaterials::from_json(
            r##"[{"name":"hidden","color":"#123456","colors":["#abcdef"],"mass":17,"invisible":true}]"##).unwrap());
        let binding = SourceShipDataThumbnail {
            thumbnail: &thumbnail,
            global: &global,
        };
        let data = crate::ship_data::SourceShipData::from_thumbnail(&binding).unwrap();
        assert_eq!((data.width, data.height), (3, 1));
        assert!(Arc::ptr_eq(&data.img, &base.source_image_data().unwrap()));
        assert!(Arc::ptr_eq(&data.materials, &global));
        let adapter = data.to_owned_adapter().unwrap();
        assert_eq!((adapter.width, adapter.height), (3, 1));
        assert!(Arc::ptr_eq(
            adapter.material_buffer[0].as_ref().unwrap(),
            &global.get(0x123456).unwrap()
        ));
        assert!(Arc::ptr_eq(
            adapter.material_buffer[0].as_ref().unwrap(),
            adapter.material_buffer[1].as_ref().unwrap()
        ));
        {
            let materials = data.material_buffer.borrow();
            assert!(Arc::ptr_eq(
                materials[0].as_ref().unwrap(),
                &global.get(0x123456).unwrap()
            ));
            assert!(Arc::ptr_eq(
                materials[0].as_ref().unwrap(),
                materials[1].as_ref().unwrap()
            ));
            assert!(materials[0].as_ref().unwrap().invisible);
            assert!(materials[2].is_none());
        }
        data.img.lock().unwrap().buffer[0] = 9;
        assert_eq!(base.image_data().unwrap().get_pixel(0, 0)[0], 9);
        base.free_resources();
        let reloaded = crate::ship_data::SourceShipData::from_thumbnail(&binding).unwrap();
        assert!(!Arc::ptr_eq(&data.img, &reloaded.img));
        assert_eq!(data.img.lock().unwrap().buffer[0], 9);
        assert_eq!(reloaded.img.lock().unwrap().buffer[0], 0x12);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn native_palette_lookup_retains_global_or_local_object_without_fallback_after_failure() {
        let base = parse_resource_path(std::path::Path::new("Identity_base.png")).unwrap();
        let global = Arc::new(
            crate::materials::SourceMaterials::from_json(
                r##"[{"color":"#123456","colors":["#abcdef"],"mass":17}]"##,
            )
            .unwrap(),
        );
        let thumbnail = ShipThumbnail::new([base.clone()]).unwrap();
        assert!(Arc::ptr_eq(
            &global,
            &thumbnail.source_materials(&global).unwrap()
        ));
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir()
            .join(format!("ss2-absent-{stamp}"))
            .join("Identity_materials.json");
        let local = parse_resource_path(&path).unwrap();
        let thumbnail = ShipThumbnail::new([base, local.clone()]).unwrap();
        let retained = thumbnail.source_materials(&global).unwrap();
        assert!(retained.materials.is_empty());
        assert!(!Arc::ptr_eq(&retained, &global));
        assert!(Arc::ptr_eq(&retained, &local.source_materials().unwrap()));
        assert!(Arc::ptr_eq(
            &retained,
            &thumbnail.clone().source_materials(&global).unwrap()
        ));
        local.free_resources();
        assert!(Arc::ptr_eq(
            &retained,
            &thumbnail.source_materials(&global).unwrap()
        ));
    }
}

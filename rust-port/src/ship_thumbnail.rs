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
    BaseDerivedTexture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShipThumbnail {
    layers: HashMap<ShipLayer, HashMap<ShipResourceType, ThumbnailResource>>,
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

    /// Java falls back to the global palette when the local resource is absent
    /// or fails to load (FileShipResource.genMaterials returns null on error).
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

    pub(crate) fn texture(
        &self,
        layer: &ShipLayer,
        global: &crate::materials::Materials,
    ) -> Result<Option<image::RgbaImage>, String> {
        match self.get_resource(ShipResourceType::Texture, layer) {
            Some(ThumbnailResource::File(resource)) => resource.image_data().map(Some),
            Some(ThumbnailResource::BaseDerivedTexture) => {
                Ok(Some(BaseDerivedTextureShipResource::derive(
                    &self.base_layer()?,
                    &self.materials(global),
                )))
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
        for resource in resources {
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
        default_resources
            .entry(ShipResourceType::Texture)
            .or_insert(ThumbnailResource::BaseDerivedTexture);

        Ok(Self { layers })
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self.get_resource(ShipResourceType::Base, &ShipLayer::default())? {
            ThumbnailResource::File(resource) => Some(&resource.ship),
            ThumbnailResource::BaseDerivedTexture => None,
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
}

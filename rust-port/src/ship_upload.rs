//! State portion of `ShipUpload.java`, kept separate from the optional
//! ImGui/Steam frontends.

use crate::ship_resources::{
    ShipLayer as Layer, ShipResourceFile as FileShipResource,
    ShipResourceType as ResourceType,
};
use crate::ship_thumbnail::{ShipThumbnail, ThumbnailResource};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShipUploadProgress {
    pub(crate) status: String,
}

pub(crate) struct SourceShipUpload {
    layers: HashMap<Layer, HashMap<ResourceType, FileShipResource>>,
    current_layer: Layer,
    window_open: bool,
    error_string: Option<String>,
    success_string: Option<String>,
    new_layer_buf: Vec<u16>,
    ship_name_buf: Vec<u16>,
    ship_description_buf: Vec<u16>,
    current_progress: Option<ShipUploadProgress>,
}

impl Default for SourceShipUpload {
    fn default() -> Self {
        let mut upload = Self {
            layers: HashMap::new(),
            current_layer: Layer::default(),
            window_open: false,
            error_string: None,
            success_string: None,
            new_layer_buf: Vec::new(),
            ship_name_buf: Vec::new(),
            ship_description_buf: Vec::new(),
            current_progress: None,
        };
        upload.reset();
        upload
    }
}

impl SourceShipUpload {
    pub(crate) fn reset(&mut self) {
        self.layers.clear();
        self.layers.insert(Layer::default(), HashMap::new());
        self.current_layer = Layer::default();
        self.new_layer_buf = vec![0; 256];
        self.ship_name_buf = vec![0; 256];
        self.ship_description_buf = vec![0; 256];
        self.ship_description_buf[..13].copy_from_slice(&[
            65, 32, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110,
        ]);
        self.error_string = None;
        self.success_string = None;
        self.current_progress = None;
    }

    pub(crate) fn show(&mut self) {
        self.set_window_open(true);
    }

    pub(crate) fn set_window_open(&mut self, value: bool) {
        if !value {
            self.reset();
        }
        self.window_open = value;
    }

    pub(crate) fn window_open(&self) -> bool {
        self.window_open
    }

    pub(crate) fn open(&self) -> bool {
        true
    }

    pub(crate) fn current_layer(&self) -> &Layer {
        &self.current_layer
    }

    pub(crate) fn set_current_layer(&mut self, layer: Layer) {
        self.current_layer = layer;
    }

    pub(crate) fn error_string(&self) -> Option<&str> {
        self.error_string.as_deref()
    }

    pub(crate) fn success_string(&self) -> Option<&str> {
        self.success_string.as_deref()
    }

    pub(crate) fn progress(&self) -> Option<&ShipUploadProgress> {
        self.current_progress.as_ref()
    }

    pub(crate) fn set_progress(&mut self, progress: Option<ShipUploadProgress>) {
        self.current_progress = progress;
    }

    pub(crate) fn ship_name(&self) -> String {
        nul_terminated_utf16(&self.ship_name_buf)
    }

    pub(crate) fn ship_description(&self) -> String {
        nul_terminated_utf16(&self.ship_description_buf)
    }

    pub(crate) fn set_ship_name(&mut self, value: &str) {
        self.ship_name_buf = fixed_utf16(value);
    }

    pub(crate) fn set_ship_description(&mut self, value: &str) {
        self.ship_description_buf = fixed_utf16(value);
    }

    pub(crate) fn layers(
        &self,
    ) -> &HashMap<Layer, HashMap<ResourceType, FileShipResource>> {
        &self.layers
    }

    pub(crate) fn select_resource(
        &mut self,
        resource_type: ResourceType,
        resource: FileShipResource,
        layer: Layer,
    ) {
        self.layers
            .entry(layer)
            .or_default()
            .insert(resource_type, resource);
    }

    pub(crate) fn set_from_thumbnail(&mut self, thumbnail: &ShipThumbnail) {
        self.reset();
        for (layer, resources) in thumbnail.layers() {
            let target = self.layers.entry(layer.clone()).or_default();
            for (resource_type, resource) in resources {
                if let ThumbnailResource::File(resource) = resource {
                    target.insert(*resource_type, resource.clone());
                }
            }
        }
        if let Some(name) = thumbnail.name() {
            self.set_ship_name(name);
        }
    }

    pub(crate) fn thumbnail(&self) -> Result<Option<ShipThumbnail>, String> {
        let Some(default_layer) = self.layers.get(&Layer::default()) else {
            return Ok(None);
        };
        if !default_layer.contains_key(&ResourceType::Base) {
            return Ok(None);
        }
        let resources = self
            .layers
            .values()
            .flat_map(|layer| layer.values().cloned())
            .collect::<Vec<_>>();
        ShipThumbnail::new(resources).map(Some)
    }

    pub(crate) fn base_resources() -> Vec<ResourceType> {
        [ResourceType::Base, ResourceType::Materials]
            .into_iter()
            .filter(|resource| !resource.is_layered())
            .collect()
    }

    pub(crate) fn layered_maps() -> Vec<ResourceType> {
        [
            ResourceType::Base,
            ResourceType::Materials,
            ResourceType::Texture,
            ResourceType::InLights,
            ResourceType::ExLights,
        ]
        .into_iter()
        .filter(|resource_type| resource_type.is_layered())
        .collect()
    }
}

fn fixed_utf16(value: &str) -> Vec<u16> {
    let mut buffer = vec![0; 256];
    for (index, unit) in value.encode_utf16().take(255).enumerate() {
        buffer[index] = unit;
    }
    buffer
}

fn nul_terminated_utf16(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship_resources::ShipResource;
    use std::path::PathBuf;

    fn resource(kind: ResourceType, name: &str) -> FileShipResource {
        FileShipResource::new(
            PathBuf::from(format!("{name}.{}", kind.extension())),
            ShipResource::new(name.to_owned(), Layer::default(), kind),
        )
        .unwrap()
    }

    #[test]
    fn reset_matches_source_defaults() {
        let upload = SourceShipUpload::default();
        assert!(!upload.window_open());
        assert_eq!(upload.ship_description(), "A Description");
        assert_eq!(upload.ship_name(), "");
        assert!(upload.layers().contains_key(&Layer::default()));
    }

    #[test]
    fn thumbnail_requires_base_and_round_trips_name() {
        let mut upload = SourceShipUpload::default();
        assert_eq!(upload.thumbnail().unwrap(), None);
        upload.select_resource(ResourceType::Base, resource(ResourceType::Base, "Titanic"), Layer::default());
        let thumbnail = upload.thumbnail().unwrap().unwrap();
        assert_eq!(thumbnail.name(), Some("Titanic"));
        upload.set_from_thumbnail(&thumbnail);
        assert_eq!(upload.ship_name(), "Titanic");
    }

    #[test]
    fn fixed_buffer_keeps_a_nul_after_truncation() {
        let mut upload = SourceShipUpload::default();
        upload.set_ship_name(&"x".repeat(300));
        assert_eq!(upload.ship_name().len(), 255);
    }
}

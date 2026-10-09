//! State portion of `ShipUpload.java`, kept separate from the optional
//! ImGui/Steam frontends.

use crate::ship_resources::{
    ShipLayer as Layer, ShipResourceFile as FileShipResource, ShipResourceType as ResourceType,
};
use crate::ship_thumbnail::{ShipThumbnail, ThumbnailResource};
use bevy::prelude::Resource;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShipUploadProgress {
    pub(crate) status: String,
}

#[derive(Resource)]
pub(crate) struct SourceShipUpload {
    layers: HashMap<Layer, HashMap<ResourceType, ThumbnailResource>>,
    layer_order: Vec<Layer>,
    layer_table_capacity: usize,
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
            layer_order: Vec::new(),
            layer_table_capacity: 16,
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
        self.layer_order.clear();
        self.layer_order.push(Layer::default());
        self.layer_table_capacity = 16;
        self.current_layer = Layer::default();
        self.new_layer_buf = vec![0; 256];
        self.ship_name_buf = vec![0; 256];
        self.ship_description_buf = vec![0; 256];
        self.ship_description_buf[..13]
            .copy_from_slice(&[65, 32, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110]);
        self.error_string = None;
        self.success_string = None;
        self.current_progress = None;
    }

    pub(crate) fn show(&mut self) {
        self.set_window_open(true);
    }

    /// Toolbox's Edit current ship button copies the active thumbnail, then
    /// opens the upload window.
    pub(crate) fn edit_current(&mut self, thumbnail: &ShipThumbnail) {
        self.set_from_thumbnail(thumbnail);
        self.show();
    }

    /// Toolbox's Create new ship button resets the form, then opens it.
    pub(crate) fn create_new(&mut self) {
        self.reset();
        self.show();
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

    /// ShipUpload.setOpen(false) removes the currently selected layer entry.
    pub(crate) fn set_open(&mut self, value: bool) {
        if !value {
            self.layers.remove(&self.current_layer);
            self.layer_order
                .retain(|layer| layer != &self.current_layer);
        }
    }

    pub(crate) fn current_layer(&self) -> &Layer {
        &self.current_layer
    }

    pub(crate) fn set_current_layer(&mut self, layer: Layer) {
        self.current_layer = layer;
    }

    /// Layer iteration follows Java HashMap bucket order for the source UI tabs.
    pub(crate) fn layers_in_order(&self) -> &[Layer] {
        &self.layer_order
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

    pub(crate) fn ship_name_is_blank(&self) -> bool {
        self.ship_name_buf
            .iter()
            .take_while(|unit| **unit != 0)
            .all(|unit| kotlin_character_is_whitespace(*unit))
    }

    pub(crate) fn ship_description(&self) -> String {
        nul_terminated_utf16(&self.ship_description_buf)
    }

    pub(crate) fn new_layer_name(&self) -> String {
        nul_terminated_utf16(&self.new_layer_buf)
    }

    pub(crate) fn editor_units(&self, field: crate::ShipUploadField) -> Vec<u16> {
        let buffer=match field {crate::ShipUploadField::Name=>&self.ship_name_buf,
            crate::ShipUploadField::Description=>&self.ship_description_buf,
            crate::ShipUploadField::LayerName=>&self.new_layer_buf};
        buffer.iter().copied().take_while(|unit|*unit!=0).collect()
    }
    pub(crate) fn set_editor_units(&mut self, field: crate::ShipUploadField, value: &[u16]) {
        let mut buffer=vec![0u16;256];
        for (target,unit) in buffer.iter_mut().take(255).zip(value.iter().copied().take_while(|unit|*unit!=0)) {*target=unit;}
        match field {
            crate::ShipUploadField::Name=>{
                self.set_ship_name(&String::from_utf16_lossy(value));self.ship_name_buf=buffer;
            },
            crate::ShipUploadField::Description=>self.ship_description_buf=buffer,
            crate::ShipUploadField::LayerName=>self.new_layer_buf=buffer,
        }
    }

    pub(crate) fn set_ship_name(&mut self, value: &str) {
        self.ship_name_buf = fixed_utf16(value);
        let ship_name = self.ship_name();
        for resources in self.layers.values_mut() {
            for resource in resources.values_mut() {
                match resource {
                    ThumbnailResource::File(resource) => resource.set_ship(ship_name.clone()),
                    ThumbnailResource::BaseDerivedTexture(resource) => {
                        resource.set_ship(ship_name.clone());
                    }
                }
            }
        }
    }

    pub(crate) fn set_ship_description(&mut self, value: &str) {
        self.ship_description_buf = fixed_utf16(value);
    }

    pub(crate) fn set_new_layer_name(&mut self, value: &str) {
        self.new_layer_buf = fixed_utf16(value);
    }

    /// Mirrors the Add Layer button: blank names do nothing; a duplicate
    /// Layer key replaces that layer's resource map, as HashMap.put does.
    pub(crate) fn add_layer(&mut self) -> bool {
        if self
            .new_layer_buf
            .iter()
            .take_while(|unit| **unit != 0)
            .all(|unit| kotlin_character_is_whitespace(*unit))
        {
            return false;
        }
        let name = nul_terminated_utf16(&self.new_layer_buf);
        let layer = Layer::new(name);
        if !self.layers.contains_key(&layer) {
            self.layers.insert(layer.clone(), HashMap::new());
            self.layer_order.push(layer);
            self.grow_and_order_layers();
        } else {
            self.layers.insert(layer, HashMap::new());
        }
        self.new_layer_buf = vec![0; 256];
        true
    }

    fn grow_and_order_layers(&mut self) {
        while self.layers.len() > self.layer_table_capacity * 3 / 4 {
            self.layer_table_capacity *= 2;
        }
        let mask = self.layer_table_capacity - 1;
        self.layer_order.sort_by_key(|layer| {
            let hash = layer.java_hash_code() as u32;
            (hash ^ (hash >> 16)) & mask as u32
        });
    }

    pub(crate) fn layers(&self) -> &HashMap<Layer, HashMap<ResourceType, ThumbnailResource>> {
        &self.layers
    }

    pub(crate) fn select_resource(
        &mut self,
        resource_type: ResourceType,
        resource: FileShipResource,
        layer: Layer,
    ) {
        if !self.layers.contains_key(&layer) {
            self.layer_order.push(layer.clone());
            self.layers.insert(layer.clone(), HashMap::new());
            self.grow_and_order_layers();
        }
        self.layers
            .entry(layer)
            .or_default()
            .insert(resource_type, ThumbnailResource::File(resource));
    }

    /// Completes the result side of ShipUpload.selectFile after the source
    /// file picker returns a path. The resource name comes from the current
    /// fixed-width ship-name buffer and replaces the same type in that layer.
    pub(crate) fn select_file(
        &mut self,
        file: PathBuf,
        resource_type: ResourceType,
        layer: Layer,
    ) -> Result<(), String> {
        let resource = FileShipResource::new(
            file,
            crate::ship_resources::ShipResource::new(
                self.ship_name(),
                layer.clone(),
                resource_type,
            ),
        )?;
        let resources = self
            .layers
            .get_mut(&layer)
            .ok_or_else(|| format!("ship upload has no resource layer {layer:?}"))?;
        resources.insert(resource_type, ThumbnailResource::File(resource));
        Ok(())
    }

    pub(crate) fn set_from_thumbnail(&mut self, thumbnail: &ShipThumbnail) {
        self.reset();
        for (layer, resources) in thumbnail.layers() {
            if !self.layers.contains_key(layer) {
                self.layers.insert(layer.clone(), HashMap::new());
                self.layer_order.push(layer.clone());
                self.grow_and_order_layers();
            }
            let target = self.layers.entry(layer.clone()).or_default();
            // ShipUpload stores ShipResource values, so retain synthesized
            // BaseDerivedTexture resources as well as file-backed resources.
            for (resource_type, resource) in resources {
                target.insert(*resource_type, resource.clone());
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
            .flat_map(|layer| {
                layer.values().filter_map(|resource| match resource {
                    ThumbnailResource::File(resource) => Some(resource.clone()),
                    ThumbnailResource::BaseDerivedTexture(_) => None,
                })
            })
            .collect::<Vec<_>>();
        ShipThumbnail::new(resources).map(Some)
    }

    /// Mirrors ShipUpload's independent "Save Locally" action. Steam Workshop
    /// submission remains outside this operation.
    pub(crate) fn save_locally(
        &mut self,
        game_root: &Path,
        global_materials: &crate::materials::Materials,
    ) -> Result<(), String> {
        let name = self.ship_name();
        let folder = game_root.join("ships").join("custom").join(&name);
        if folder.exists() {
            self.success_string = None;
            let error = format!("Ship of name {name} already exists locally");
            self.error_string = Some(error.clone());
            return Err(error);
        }

        self.error_string = None;
        for resources in self.layers.values() {
            for resource in resources.values() {
                match resource {
                    ThumbnailResource::File(resource) => {
                        resource.write_to_folder(&folder);
                    }
                    ThumbnailResource::BaseDerivedTexture(resource) => {
                        resource.write_to_folder(&folder, global_materials);
                    }
                }
            }
        }

        self.success_string = Some(format!("{name} has been saved under ships/custom/{name}/"));
        Ok(())
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

/// Shipped CharsKt__CharJVMKt.isWhitespace combines Character.isWhitespace
/// and Character.isSpaceChar. Keep JRE 13's BMP set, including NBSP separators.
fn kotlin_character_is_whitespace(unit: u16) -> bool {
    matches!(
        unit,
        0x0009..=0x000d
            | 0x001c..=0x0020
            | 0x00a0
            | 0x1680
            | 0x2000..=0x200a
            | 0x2028
            | 0x2029
            | 0x202f
            | 0x205f
            | 0x3000
    )
}

fn nul_terminated_utf16(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
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
        upload.select_resource(
            ResourceType::Base,
            resource(ResourceType::Base, "Titanic"),
            Layer::default(),
        );
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

    #[test]
    fn blank_names_follow_shipped_kotlin_whitespace_and_space_char_union() {
        let mut upload = SourceShipUpload::default();
        for name in [
            "",
            " \t\r\n",
            "\u{00a0}\u{2007}\u{202f}",
            "\u{001c}\u{3000}",
        ] {
            upload.set_ship_name(name);
            assert!(upload.ship_name_is_blank(), "{name:?}");
            upload.set_new_layer_name(name);
            assert!(!upload.add_layer(), "blank layer {name:?} must be ignored");
        }
        for name in ["ship", "\u{0085}", "\u{180e}", "\u{200b}"] {
            upload.set_ship_name(name);
            assert!(!upload.ship_name_is_blank(), "{name:?}");
        }
    }
}

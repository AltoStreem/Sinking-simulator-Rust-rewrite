use super::{layer::Layer, resource_type::ResourceType};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShipResource {
    pub(crate) ship: String,
    pub(crate) layer: Layer,
    pub(crate) resource_type: ResourceType,
}

impl ShipResource {
    pub(crate) fn new(ship: String, layer: Layer, resource_type: ResourceType) -> Self {
        Self {
            ship,
            layer,
            resource_type,
        }
    }

    /// Mirrors ShipResource.Companion.fromFile: only recognized resource suffixes
    /// are parsed; an unrecognized suffix remains part of a default-layer BASE name.
    pub(crate) fn from_file(
        path: &Path,
    ) -> Result<super::file_ship_resource::FileShipResource, String> {
        use super::file_ship_resource::FileShipResource;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid ship resource name: {}", path.display()))?;
        let upper = stem.to_ascii_uppercase();
        let parsed = ["BASE", "MATERIALS", "TEXTURE", "INLIGHTS", "EXLIGHTS"]
            .iter()
            .find_map(|suffix| {
                upper.strip_suffix(&format!("_{suffix}")).map(|_| {
                    (
                        &stem[..stem.len() - suffix.len() - 1],
                        ResourceType::from_name(suffix).unwrap(),
                    )
                })
            });
        let (ship, layer, kind) = match parsed {
            Some((prefix, kind)) => match prefix.rsplit_once('_') {
                Some((ship, layer)) if !ship.is_empty() && !layer.is_empty() => {
                    (ship.to_owned(), Layer::new(layer), kind)
                }
                _ => (prefix.to_owned(), Layer::default(), kind),
            },
            None => (stem.to_owned(), Layer::default(), ResourceType::Base),
        };
        FileShipResource::new(path.to_path_buf(), Self::new(ship, layer, kind))
    }
}

pub(crate) fn parse_resource_path(
    path: &Path,
) -> Result<super::file_ship_resource::FileShipResource, String> {
    ShipResource::from_file(path)
}

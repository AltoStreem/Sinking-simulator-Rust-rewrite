use super::{layer::Layer, resource_type::ResourceType};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShipResource {
    pub(crate) ship: String,
    pub(crate) layer: Layer,
    pub(crate) resource_type: ResourceType,
}

impl ShipResource {
    pub(crate) fn export_filename(&self) -> String {
        let kind = self.resource_type.name().to_ascii_lowercase();
        if self.layer.is_default() {
            format!("{}_{kind}.{}", self.ship, self.resource_type.extension())
        } else {
            format!(
                "{}_{}_{kind}.{}",
                self.ship,
                self.layer.name(),
                self.resource_type.extension()
            )
        }
    }

    pub(crate) fn new(ship: String, layer: Layer, resource_type: ResourceType) -> Self {
        Self {
            ship,
            layer,
            resource_type,
        }
    }

    pub(crate) fn set_ship(&mut self, ship: String) {
        self.ship = ship;
    }

    /// Mirrors ShipResource.Companion.fromFile: only recognized resource suffixes
    /// are parsed; an unrecognized suffix remains part of a default-layer BASE name.
    pub(crate) fn from_file(
        path: &Path,
    ) -> Result<super::file_ship_resource::FileShipResource, String> {
        super::ship_resource_companion::from_file(path)
    }
}

pub(crate) fn parse_resource_path(
    path: &Path,
) -> Result<super::file_ship_resource::FileShipResource, String> {
    ShipResource::from_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotfile_stem_matches_kotlin_last_dot_rule() {
        let resource = ShipResource::from_file(Path::new(".png")).unwrap();
        assert_eq!(resource.ship, "");
        assert!(resource.layer.is_default());
        let resource = ShipResource::from_file(Path::new(".Titanic_BASE.png")).unwrap();
        assert_eq!(resource.ship, ".Titanic");
    }

    #[test]
    fn source_regex_allows_an_empty_ship_with_a_named_layer() {
        // Java: ^(.*?)_(?:([^_]+)_)?(BASE|...)$
        let resource = ShipResource::from_file(Path::new("_exterior_BASE.png")).unwrap();
        assert_eq!(resource.ship, "");
        assert_eq!(resource.layer.name(), "exterior");
        assert_eq!(resource.resource_type, ResourceType::Base);
    }

    #[test]
    fn empty_layer_segment_is_retained_in_the_ship_name() {
        let resource = ShipResource::from_file(Path::new("Titanic__BASE.png")).unwrap();
        assert_eq!(resource.ship, "Titanic_");
        assert!(resource.layer.is_default());
    }
}

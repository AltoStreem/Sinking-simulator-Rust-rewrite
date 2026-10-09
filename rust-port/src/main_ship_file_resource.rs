//! Main$main$3$files$2.class: delegate every accepted file to fromFile.
pub(crate) fn invoke(
    path: &std::path::Path,
) -> Result<crate::ship_resources::ShipResourceFile, String> {
    crate::ship_resources::parse_resource_path(path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_named_layers_and_unrecognized_suffixes_without_extension_filter() {
        let resource = invoke(std::path::Path::new("Titanic_exterior_TEXTURE.png")).unwrap();
        assert_eq!(resource.ship, "Titanic");
        assert_eq!(resource.layer.name(), "exterior");
        assert_eq!(
            resource.resource_type,
            crate::ship_resources::ShipResourceType::Texture
        );
        let other = invoke(std::path::Path::new("notes_unrecognized.png")).unwrap();
        assert_eq!(other.ship, "notes_unrecognized");
        assert_eq!(
            other.resource_type,
            crate::ship_resources::ShipResourceType::Base
        );
        assert!(invoke(std::path::Path::new("notes.unrecognized")).is_err());
    }
}

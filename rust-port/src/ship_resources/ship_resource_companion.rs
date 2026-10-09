//! ShipResource$Companion.fromFile: original filename regex and construction.
use super::{layer::Layer, resource_type::ResourceType, ship_resource::ShipResource};
use std::path::Path;

fn type_matches(value: &str, expected: &str) -> bool {
    let mut chars = value.chars();
    expected.chars().all(|letter| {
        chars.next().is_some_and(|ch| {
            ch.eq_ignore_ascii_case(&letter)
                || (letter == 'S' && ch == '\u{017f}')
                || (letter == 'I' && matches!(ch, '\u{0130}' | '\u{0131}'))
        })
    }) && chars.next().is_none()
}

/// Java Pattern's dot excludes all five line terminators; the layer's
/// negated underscore class accepts those same characters.
fn metadata(stem: &str) -> Result<ShipResource, String> {
    for (index, _) in stem.match_indices('_') {
        let ship = &stem[..index];
        if ship
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}'))
        {
            break;
        }
        let remainder = &stem[index + 1..];
        let (layer, kind) = match remainder.split_once('_') {
            Some((layer, kind)) if !layer.is_empty() && !kind.contains('_') => (layer, kind),
            None => ("", remainder),
            _ => continue,
        };
        if ResourceType::values()
            .iter()
            .any(|value| type_matches(kind, value.name()))
        {
            // The source converts the captured spelling to uppercase before
            // Enum.valueOf, rather than returning the regex alternative.
            let upper: String = kind.chars().flat_map(char::to_uppercase).collect();
            return Ok(ShipResource::new(
                ship.to_owned(),
                Layer::new(layer),
                ResourceType::value_of(&upper)?,
            ));
        }
    }
    Ok(ShipResource::new(
        stem.to_owned(),
        Layer::default(),
        ResourceType::Base,
    ))
}

pub(crate) fn from_file(
    path: &Path,
) -> Result<super::file_ship_resource::FileShipResource, String> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid ship resource name: {}", path.display()))?;
    let stem = crate::main_flat_files::kotlin_file_name_without_extension(name);
    super::file_ship_resource::FileShipResource::new(path.to_path_buf(), metadata(stem)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lazy_ship_capture_and_java_line_terminators() {
        let value = metadata("a_b_c_TEXTURE").unwrap();
        assert_eq!(value.ship, "a_b");
        assert_eq!(value.layer.name(), "c");
        for line in ['\n', '\r', '\u{0085}', '\u{2028}', '\u{2029}'] {
            let stem = format!("ship{line}_BASE");
            assert_eq!(metadata(&stem).unwrap().ship, stem);
            let value = metadata(&format!("ship_layer{line}_TEXTURE")).unwrap();
            assert_eq!(value.ship, "ship");
            assert_eq!(value.layer.name(), format!("layer{line}"));
            assert_eq!(value.resource_type, ResourceType::Texture);
        }
    }
    #[test]
    fn unicode_regex_capture_still_runs_enum_uppercase_conversion() {
        assert_eq!(
            metadata("ship_BAſE").unwrap().resource_type,
            ResourceType::Base
        );
        assert_eq!(
            metadata("ship_ınlights").unwrap().resource_type,
            ResourceType::InLights
        );
        assert!(
            metadata("ship_İNLightS")
                .unwrap_err()
                .contains("ResourceType.İNLIGHTS")
        );
        assert_eq!(metadata("ship_notbase").unwrap().ship, "ship_notbase");
    }
}

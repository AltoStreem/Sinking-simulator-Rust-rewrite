//! Rust counterpart of Kotlin's synthetic `ShipResource$WhenMappings` table.
//! Rust matches are exhaustive, but the source-generated ordinal table is
//! retained as a separate module for conversion and reflection parity.

use crate::ship_resources::ShipResourceType as ResourceType;

pub(crate) const RESOURCE_TYPE_ORDINALS: [ResourceType; 5] = [
    ResourceType::Base,
    ResourceType::Materials,
    ResourceType::Texture,
    ResourceType::InLights,
    ResourceType::ExLights,
];

pub(crate) fn ordinal(resource_type: ResourceType) -> usize {
    RESOURCE_TYPE_ORDINALS
        .iter()
        .position(|candidate| *candidate == resource_type)
        .expect("all ResourceType variants have a generated mapping")
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_preserves_source_enum_order() {
        for (index, resource_type) in RESOURCE_TYPE_ORDINALS.iter().copied().enumerate() {
            assert_eq!(ordinal(resource_type), index + 1);
        }
    }
}

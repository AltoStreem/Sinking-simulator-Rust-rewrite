//! Rust counterpart of Kotlin's synthetic `ShipResource$WhenMappings` table.
//! The synthetic switch table is indexed by enum ordinal. Only MATERIALS
//! receives a nonzero branch number in the original class initializer.

use crate::ship_resources::ShipResourceType as ResourceType;

pub(crate) const RESOURCE_TYPE_ORDINALS: [ResourceType; 5] = [
    ResourceType::Base,
    ResourceType::Materials,
    ResourceType::Texture,
    ResourceType::InLights,
    ResourceType::ExLights,
];

pub(crate) const ENUM_SWITCH_MAPPING_0: [i32; 5] = [0, 1, 0, 0, 0];

pub(crate) fn switch_mapping(resource_type: ResourceType) -> i32 {
    let ordinal = RESOURCE_TYPE_ORDINALS
        .iter()
        .position(|candidate| *candidate == resource_type)
        .expect("all ResourceType variants have a generated mapping");
    ENUM_SWITCH_MAPPING_0[ordinal]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_matches_java_static_initializer() {
        assert_eq!(switch_mapping(ResourceType::Base), 0);
        assert_eq!(switch_mapping(ResourceType::Materials), 1);
        assert_eq!(switch_mapping(ResourceType::Texture), 0);
        assert_eq!(switch_mapping(ResourceType::InLights), 0);
        assert_eq!(switch_mapping(ResourceType::ExLights), 0);
    }
}

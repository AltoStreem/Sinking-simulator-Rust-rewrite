#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ResourceType {
    Base,
    Materials,
    Texture,
    InLights,
    ExLights,
}

impl ResourceType {
    /// Port-specific case-insensitive parser used by the ship filename adapter.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "BASE" => Some(Self::Base),
            "MATERIALS" => Some(Self::Materials),
            "TEXTURE" => Some(Self::Texture),
            "INLIGHTS" => Some(Self::InLights),
            "EXLIGHTS" => Some(Self::ExLights),
            _ => None,
        }
    }
    pub(crate) const fn values() -> [Self; 5] {
        [
            Self::Base,
            Self::Materials,
            Self::Texture,
            Self::InLights,
            Self::ExLights,
        ]
    }
    pub(crate) const fn ordinal(self) -> usize {
        match self {
            Self::Base => 0,
            Self::Materials => 1,
            Self::Texture => 2,
            Self::InLights => 3,
            Self::ExLights => 4,
        }
    }
    /// Java Enum.valueOf is exact-case; Result carries its exception message across the Rust boundary.
    pub(crate) fn value_of(name: &str) -> Result<Self, String> {
        match name {
            "BASE" => Ok(Self::Base),
            "MATERIALS" => Ok(Self::Materials),
            "TEXTURE" => Ok(Self::Texture),
            "INLIGHTS" => Ok(Self::InLights),
            "EXLIGHTS" => Ok(Self::ExLights),
            _ => Err(format!(
                "No enum constant com.wicpar.sinkingsimulator.ship.resources.ResourceType.{name}"
            )),
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::Materials => "MATERIALS",
            Self::Texture => "TEXTURE",
            Self::InLights => "INLIGHTS",
            Self::ExLights => "EXLIGHTS",
        }
    }
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Materials => "json",
            Self::Base | Self::Texture | Self::InLights | Self::ExLights => "png",
        }
    }
    pub(crate) fn is_layered(self) -> bool {
        matches!(self, Self::Texture | Self::InLights | Self::ExLights)
    }
    pub(crate) fn is_required(self) -> bool {
        matches!(self, Self::Base)
    }
}

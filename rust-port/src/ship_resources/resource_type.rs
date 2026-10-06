#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ResourceType {
    Base,
    Materials,
    Texture,
    InLights,
    ExLights,
}

impl ResourceType {
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

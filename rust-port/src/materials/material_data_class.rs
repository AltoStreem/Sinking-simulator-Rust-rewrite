use super::Material;
use std::{collections::HashSet, sync::Arc};

impl Material {
    pub(crate) fn component1(&self) -> Option<String> {
        (!self.name_is_null).then(|| self.name.clone())
    }
    pub(crate) fn component2(&self) -> Option<String> {
        self.color.clone()
    }
    pub(crate) fn component3(&self) -> Option<Arc<Vec<String>>> {
        self.colors.clone()
    }
    pub(crate) fn component4(&self) -> Option<String> {
        self.colour.clone()
    }
    pub(crate) fn component5(&self) -> Option<Arc<Vec<String>>> {
        self.colours.clone()
    }
    pub(crate) fn component6(&self) -> f32 {
        self.strength
    }
    pub(crate) fn component7(&self) -> Option<f32> {
        self.tensile_strength
    }
    pub(crate) fn component8(&self) -> Option<f32> {
        self.compressive_strength
    }
    pub(crate) fn component9(&self) -> f32 {
        self.mass
    }
    pub(crate) fn component10(&self) -> f32 {
        self.rigidity
    }
    pub(crate) fn component11(&self) -> bool {
        self.is_hull
    }
    pub(crate) fn component12(&self) -> bool {
        self.is_rope
    }
    pub(crate) fn component13(&self) -> bool {
        self.is_ground
    }
    pub(crate) fn component14(&self) -> bool {
        self.invisible
    }

    pub(crate) fn copy(
        &self,
        name: String,
        color: Option<String>,
        colors: Option<Arc<Vec<String>>>,
        colour: Option<String>,
        colours: Option<Arc<Vec<String>>>,
        strength: f32,
        tensile_strength: Option<f32>,
        compressive_strength: Option<f32>,
        mass: f32,
        rigidity: f32,
        is_hull: bool,
        is_rope: bool,
        is_ground: bool,
        invisible: bool,
    ) -> Self {
        Self {
            name,
            name_is_null: false,
            color,
            colors,
            colour,
            colours,
            strength,
            tensile_strength,
            compressive_strength,
            mass,
            rigidity,
            is_hull,
            is_rope,
            is_ground,
            invisible,
        }
    }
    pub(crate) fn copy_with_mask(&self, value: &Self, mask: u32) -> Self {
        Self {
            name: if mask & 0x001 != 0 {
                self.name.clone()
            } else {
                value.name.clone()
            },
            name_is_null: if mask & 0x001 != 0 {
                self.name_is_null
            } else {
                value.name_is_null
            },
            color: if mask & 0x002 != 0 {
                self.color.clone()
            } else {
                value.color.clone()
            },
            colors: if mask & 0x004 != 0 {
                self.colors.clone()
            } else {
                value.colors.clone()
            },
            colour: if mask & 0x008 != 0 {
                self.colour.clone()
            } else {
                value.colour.clone()
            },
            colours: if mask & 0x010 != 0 {
                self.colours.clone()
            } else {
                value.colours.clone()
            },
            strength: if mask & 0x020 != 0 {
                self.strength
            } else {
                value.strength
            },
            tensile_strength: if mask & 0x040 != 0 {
                self.tensile_strength
            } else {
                value.tensile_strength
            },
            compressive_strength: if mask & 0x080 != 0 {
                self.compressive_strength
            } else {
                value.compressive_strength
            },
            mass: if mask & 0x100 != 0 {
                self.mass
            } else {
                value.mass
            },
            rigidity: if mask & 0x200 != 0 {
                self.rigidity
            } else {
                value.rigidity
            },
            is_hull: if mask & 0x400 != 0 {
                self.is_hull
            } else {
                value.is_hull
            },
            is_rope: if mask & 0x800 != 0 {
                self.is_rope
            } else {
                value.is_rope
            },
            is_ground: if mask & 0x1000 != 0 {
                self.is_ground
            } else {
                value.is_ground
            },
            invisible: if mask & 0x2000 != 0 {
                self.invisible
            } else {
                value.invisible
            },
        }
    }
    pub(crate) fn all_colors(&self) -> Result<Vec<u32>, String> {
        let mut seen = HashSet::new();
        let values = self
            .color
            .iter()
            .chain(self.colour.iter())
            .chain(self.colors.iter().flat_map(|items| items.iter()))
            .chain(self.colours.iter().flat_map(|items| items.iter()));
        let mut result = Vec::new();
        for color in values {
            if !seen.insert(color.as_str()) {
                continue;
            }
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                let name = if self.name_is_null {
                    "null"
                } else {
                    self.name.as_str()
                };
                return Err(format!(
                    "Color '{color}' in Material {name} must be of Format #RRGGBB"
                ));
            }
            result.push(
                u32::from_str_radix(&color[1..], 16).expect("six validated hexadecimal digits"),
            );
        }
        Ok(result)
    }
    pub(crate) fn java_hash_code(&self) -> i32 {
        fn string_hash(s: &str) -> i32 {
            s.encode_utf16()
                .fold(0i32, |h, u| h.wrapping_mul(31).wrapping_add(i32::from(u)))
        }
        fn array_hash(a: &Option<Arc<Vec<String>>>) -> i32 {
            a.as_ref().map_or(0, |items| {
                items
                    .iter()
                    .fold(1i32, |h, s| h.wrapping_mul(31).wrapping_add(string_hash(s)))
            })
        }
        fn float_hash(v: f32) -> i32 {
            if v.is_nan() {
                0x7fc00000u32 as i32
            } else {
                v.to_bits() as i32
            }
        }
        fn optional_float_hash(v: Option<f32>) -> i32 {
            v.map_or(0, float_hash)
        }
        let mut h = if self.name_is_null {
            0
        } else {
            string_hash(&self.name)
        };
        for part in [
            self.color.as_ref().map_or(0, |v| string_hash(v)),
            array_hash(&self.colors),
            self.colour.as_ref().map_or(0, |v| string_hash(v)),
            array_hash(&self.colours),
            float_hash(self.strength),
            optional_float_hash(self.tensile_strength),
            optional_float_hash(self.compressive_strength),
            float_hash(self.mass),
            float_hash(self.rigidity),
            i32::from(self.is_hull),
            i32::from(self.is_rope),
            i32::from(self.is_ground),
        ] {
            h = h.wrapping_mul(31).wrapping_add(part);
        }
        h.wrapping_mul(31).wrapping_add(i32::from(self.invisible))
    }
}

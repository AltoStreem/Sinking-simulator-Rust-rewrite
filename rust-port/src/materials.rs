//! Rust translation of SS2 Materials and its nested Material record.
//! Color spellings and lookup behavior match the decompiled Gson-backed loader.
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub name: String,
    pub strength: f32,
    pub tensile_strength: Option<f32>,
    pub compressive_strength: Option<f32>,
    pub mass: f32,
    pub rigidity: f32,
    pub is_hull: bool,
    pub is_rope: bool,
    pub is_ground: bool,
    pub invisible: bool,
}

/// Rust counterpart of the source `Materials` object and its color lookup map.
#[derive(Clone, Debug, Default)]
pub struct Materials {
    pub materials: HashMap<u32, Material>,
}

impl Materials {
    pub fn from_json(json: &str) -> Result<Self, String> {
        Ok(Self {
            materials: parse_palette(json)?,
        })
    }

    pub fn get(&self, rgb: u32) -> Option<&Material> {
        self.materials.get(&rgb)
    }
}

/// Decode material JSON and index every declared RGB key to its source record.
pub fn parse_palette(json: &str) -> Result<HashMap<u32, Material>, String> {
    let values = serde_json::from_str::<Vec<Value>>(json).map_err(|error| error.to_string())?;
    let mut palette = HashMap::new();
    for value in values {
        let object = value
            .as_object()
            .ok_or_else(|| "material entry must be a JSON object".to_owned())?;
        let material = Material {
            name: object
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            strength: number(object.get("strength")),
            tensile_strength: optional_number(object.get("tensileStrength")),
            compressive_strength: optional_number(object.get("compressiveStrength")),
            mass: number(object.get("mass")),
            rigidity: number(object.get("rigidity")),
            is_hull: boolean(object.get("isHull")),
            is_rope: boolean(object.get("isRope")),
            is_ground: boolean(object.get("isGround")),
            invisible: boolean(object.get("invisible")),
        };
        let mut color_names = Vec::new();
        for key in ["color", "colour"] {
            if let Some(color) = object.get(key).and_then(Value::as_str) {
                color_names.push(color.to_owned());
            }
        }
        for key in ["colors", "colours"] {
            if let Some(colors) = object.get(key).and_then(Value::as_array) {
                color_names.extend(colors.iter().filter_map(Value::as_str).map(str::to_owned));
            }
        }
        let mut seen = HashSet::new();
        for color in color_names
            .into_iter()
            .filter(|color| seen.insert(color.clone()))
        {
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(format!(
                    "color '{color}' in material '{}' must use #RRGGBB",
                    material.name
                ));
            }
            let rgb = u32::from_str_radix(&color[1..], 16)
                .map_err(|error| format!("invalid RGB color '{color}': {error}"))?;
            // Materials.java groups equal color keys and selects the first
            // material in file order when there is a collision.
            palette.entry(rgb).or_insert_with(|| material.clone());
        }
    }
    Ok(palette)
}

fn number(value: Option<&Value>) -> f32 {
    value.and_then(Value::as_f64).unwrap_or(0.0) as f32
}

fn optional_number(value: Option<&Value>) -> Option<f32> {
    value.and_then(Value::as_f64).map(|value| value as f32)
}

fn boolean(value: Option<&Value>) -> bool {
    value.and_then(Value::as_bool).unwrap_or(false)
}

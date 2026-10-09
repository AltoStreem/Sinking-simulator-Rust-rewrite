//! Rust translation of SS2 Materials and its nested Material record.
//! Color spellings and lookup behavior match the decompiled Gson-backed loader.
mod material;
mod material_constructor;
mod material_data_class;
pub use material::Material;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Rust counterpart of the source `Materials` object and its color lookup map.
#[derive(Clone, Debug, Default)]
pub struct Materials {
    pub materials: HashMap<u32, Material>,
    color_order: Vec<u32>,
}

/// Materials.java stores the same immutable Material object for every color
/// declared by that record. The owned adapter above supports Bevy callers;
/// this representation retains source record identity across aliases.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceMaterials {
    pub(crate) materials: HashMap<u32, Arc<Material>>,
    color_order: Vec<u32>,
}

#[allow(dead_code)]
impl SourceMaterials {
    pub(crate) fn from_json(json: &str) -> Result<Self, String> {
        let (materials, color_order) = parse_shared_palette_ordered(json)?;
        Ok(Self {
            materials,
            color_order,
        })
    }

    pub(crate) fn get(&self, rgb: u32) -> Option<Arc<Material>> {
        self.materials.get(&rgb).cloned()
    }

    pub(crate) fn to_owned_adapter(&self) -> Materials {
        Materials {
            materials: self
                .materials
                .iter()
                .map(|(&rgb, material)| (rgb, (**material).clone()))
                .collect(),
            color_order: self.color_order.clone(),
        }
    }

    pub(crate) fn to_json(&self) -> Result<String, String> {
        self.to_owned_adapter().to_json()
    }
}

impl crate::ship_data::SourceMaterialLookup for SourceMaterials {
    type Material = Arc<Material>;
    fn get(&self, rgb: u32) -> Option<Self::Material> {
        SourceMaterials::get(self, rgb)
    }
}

impl Materials {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let (materials, color_order) = parse_palette_ordered(json)?;
        Ok(Self {
            materials,
            color_order,
        })
    }

    pub fn get(&self, rgb: u32) -> Option<&Material> {
        self.materials.get(&rgb)
    }
    /// Materials.java preserves first color encounter order in its LinkedHashMap.
    /// PaletteGenKt applies distinct() to values, retaining the first occurrence.
    pub(crate) fn source_palette_rows(&self) -> Result<Vec<(&Material, Vec<u32>)>, String> {
        let mut rows: Vec<(&Material, Vec<u32>)> = Vec::new();
        for rgb in &self.color_order {
            let Some(material) = self.materials.get(rgb) else {
                continue;
            };
            if rows
                .iter()
                .any(|(previous, _)| previous.source_equals(material))
            {
                continue;
            }
            rows.push((material, material.all_colors()?));
        }
        Ok(rows)
    }

    /// Source values().toSet(): first color encounter order, with Java's
    /// array identity equality retained by shared Arc arrays.
    pub fn to_json(&self) -> Result<String, String> {
        let mut unique: Vec<&Material> = Vec::new();
        for color in &self.color_order {
            if let Some(material) = self.materials.get(color) {
                if !unique
                    .iter()
                    .any(|previous| previous.source_equals(material))
                {
                    unique.push(material);
                }
            }
        }
        let mut records = Vec::new();
        for material in unique {
            for value in [
                Some(material.strength),
                material.tensile_strength,
                material.compressive_strength,
                Some(material.mass),
                Some(material.rigidity),
            ]
            .into_iter()
            .flatten()
            {
                if !value.is_finite() {
                    return Err(format!("{value} is not a valid JSON floating point value"));
                }
            }
            let value = material.json_value();
            let object = value.as_object().unwrap();
            let mut fields = Vec::new();
            // Gson's reflective adapter emits fields in declaration order.
            for key in [
                "name",
                "color",
                "colors",
                "colour",
                "colours",
                "strength",
                "tensileStrength",
                "compressiveStrength",
                "mass",
                "rigidity",
                "isHull",
                "isRope",
                "isGround",
                "invisible",
            ] {
                if let Some(value) = object.get(key) {
                    let encoded = match key {
                        "strength" => serde_json::to_string(&material.strength),
                        "mass" => serde_json::to_string(&material.mass),
                        "rigidity" => serde_json::to_string(&material.rigidity),
                        "tensileStrength" => {
                            serde_json::to_string(&material.tensile_strength.unwrap())
                        }
                        "compressiveStrength" => {
                            serde_json::to_string(&material.compressive_strength.unwrap())
                        }
                        _ => serde_json::to_string(value),
                    }
                    .map_err(|error| error.to_string())?;
                    // Pretty Gson arrays also put each element on its own line.
                    let encoded = if let Value::Array(array) = value {
                        if array.is_empty() {
                            "[]".into()
                        } else {
                            let items: Result<Vec<_>, _> =
                                array.iter().map(serde_json::to_string).collect();
                            format!(
                                "[\n      {}\n    ]",
                                items.map_err(|error| error.to_string())?.join(",\n      ")
                            )
                        }
                    } else {
                        encoded
                    };
                    fields.push(format!("    \"{key}\": {encoded}"));
                }
            }
            records.push(format!("  {{\n{}\n  }}", fields.join(",\n")));
        }
        let json = if records.is_empty() {
            "[]".into()
        } else {
            format!("[\n{}\n]", records.join(",\n"))
        };
        // Default Gson HTML-safe escaping, plus JavaScript line separators.
        Ok(json
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026")
            .replace('=', "\\u003d")
            .replace('\'', "\\u0027")
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029"))
    }
}

impl Material {
    fn source_equals(&self, other: &Self) -> bool {
        fn arrays(a: &Option<Arc<Vec<String>>>, b: &Option<Arc<Vec<String>>>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        fn bits(value: f32) -> u32 {
            if value.is_nan() {
                0x7fc00000
            } else {
                value.to_bits()
            }
        }
        self.name == other.name
            && self.name_is_null == other.name_is_null
            && self.color == other.color
            && self.colour == other.colour
            && arrays(&self.colors, &other.colors)
            && arrays(&self.colours, &other.colours)
            && bits(self.strength) == bits(other.strength)
            && self.tensile_strength.map(bits) == other.tensile_strength.map(bits)
            && self.compressive_strength.map(bits) == other.compressive_strength.map(bits)
            && bits(self.mass) == bits(other.mass)
            && bits(self.rigidity) == bits(other.rigidity)
            && self.is_hull == other.is_hull
            && self.is_rope == other.is_rope
            && self.is_ground == other.is_ground
            && self.invisible == other.invisible
    }

    fn json_value(&self) -> Value {
        let mut value = serde_json::json!({"name": self.name, "strength": self.strength,
            "mass": self.mass, "rigidity": self.rigidity, "isHull": self.is_hull,
            "isRope": self.is_rope, "isGround": self.is_ground, "invisible": self.invisible});
        let object = value.as_object_mut().unwrap();
        if self.name_is_null {
            object.remove("name");
        }
        for (key, text) in [("color", &self.color), ("colour", &self.colour)] {
            if let Some(text) = text {
                object.insert(key.into(), Value::String(text.clone()));
            }
        }
        for (key, array) in [("colors", &self.colors), ("colours", &self.colours)] {
            if let Some(array) = array {
                object.insert(key.into(), serde_json::json!(array.as_ref()));
            }
        }
        for (key, number) in [
            ("tensileStrength", self.tensile_strength),
            ("compressiveStrength", self.compressive_strength),
        ] {
            if let Some(number) = number {
                object.insert(key.into(), serde_json::json!(number));
            }
        }
        value
    }
}

/// Decode material JSON and index every declared RGB key to its source record.
pub fn parse_palette(json: &str) -> Result<HashMap<u32, Material>, String> {
    parse_palette_ordered(json).map(|(palette, _)| palette)
}

fn parse_palette_ordered(json: &str) -> Result<(HashMap<u32, Material>, Vec<u32>), String> {
    parse_shared_palette_ordered(json).map(|(palette, order)| {
        (
            palette
                .into_iter()
                .map(|(rgb, material)| (rgb, (*material).clone()))
                .collect(),
            order,
        )
    })
}

fn parse_shared_palette_ordered(
    json: &str,
) -> Result<(HashMap<u32, Arc<Material>>, Vec<u32>), String> {
    parse_shared_palette_with_warnings(json, |warning| println!("{warning}"))
}

fn parse_shared_palette_with_warnings(
    json: &str,
    mut warning: impl FnMut(String),
) -> Result<(HashMap<u32, Arc<Material>>, Vec<u32>), String> {
    let values = serde_json::from_str::<Vec<Value>>(json).map_err(|error| error.to_string())?;
    let mut palette = HashMap::new();
    let mut color_order = Vec::new();
    let mut grouped: HashMap<u32, Vec<Arc<Material>>> = HashMap::new();
    let mut records = Vec::with_capacity(values.len());
    // Gson finishes deserializing the complete array before Kotlin asks any
    // record for its colors or prints warnings.
    for value in values {
        let object = value
            .as_object()
            .ok_or_else(|| "material entry must be a JSON object".to_owned())?;
        let material = Arc::new(Material {
            color: string(object.get("color"))?,
            colour: string(object.get("colour"))?,
            colors: string_array(object.get("colors"))?,
            colours: string_array(object.get("colours"))?,
            name: string(object.get("name"))?.unwrap_or_default(),
            name_is_null: object.get("name").is_none_or(Value::is_null),
            strength: optional_number(object.get("strength"))?.unwrap_or(0.0),
            tensile_strength: optional_number(object.get("tensileStrength"))?,
            compressive_strength: optional_number(object.get("compressiveStrength"))?,
            mass: optional_number(object.get("mass"))?.unwrap_or(0.0),
            rigidity: optional_number(object.get("rigidity"))?.unwrap_or(0.0),
            is_hull: boolean(object.get("isHull"))?,
            is_rope: boolean(object.get("isRope"))?,
            is_ground: boolean(object.get("isGround"))?,
            invisible: boolean(object.get("invisible"))?,
        });
        records.push(material);
    }
    for material in records {
        let colors = material.all_colors()?;
        if colors.is_empty() {
            let name = if material.name_is_null {
                "null"
            } else {
                &material.name
            };
            warning(format!("Warning: {name} has no associated color"));
        }
        for rgb in colors {
            grouped.entry(rgb).or_default().push(material.clone());
            // Materials.java groups duplicate keys and keeps the first record in file order.
            if let std::collections::hash_map::Entry::Vacant(entry) = palette.entry(rgb) {
                color_order.push(rgb);
                entry.insert(material.clone());
            }
        }
    }
    // Kotlin groupBy retains first key encounter order and counts repeated
    // colors from the same record as well as colors shared by different records.
    for rgb in &color_order {
        let records = &grouped[rgb];
        if records.len() > 1 {
            let names: Vec<_> = records
                .iter()
                .map(|record| {
                    if record.name_is_null {
                        "null"
                    } else {
                        record.name.as_str()
                    }
                })
                .collect();
            warning(format!(
                "Warning: materials [{}] are requesting the same color: #{rgb:x}, choosing first: {}",
                names.join(", "),
                names[0]
            ));
        }
    }
    Ok((palette, color_order))
}

fn string(value: Option<&Value>) -> Result<Option<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(Value::Bool(value)) => Ok(Some(value.to_string())),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        _ => Err("expected scalar string value".into()),
    }
}

fn string_array(value: Option<&Value>) -> Result<Option<Arc<Vec<String>>>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(array)) => {
            let values: Result<Vec<_>, _> = array
                .iter()
                .map(|value| {
                    string(Some(value))?.ok_or_else(|| "null material color is invalid".to_owned())
                })
                .collect();
            Ok(Some(Arc::new(values?)))
        }
        _ => Err("expected material color array".into()),
    }
}

fn optional_number(value: Option<&Value>) -> Result<Option<f32>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_f64()
            .map(|value| Some(value as f32))
            .ok_or_else(|| "invalid numeric material value".into()),
        Some(Value::String(value)) => value
            .trim_matches(|ch: char| ch <= '\u{20}')
            .parse::<f64>()
            .map(|value| Some(value as f32))
            .map_err(|error| error.to_string()),
        _ => Err("expected numeric material value".into()),
    }
}

fn boolean(value: Option<&Value>) -> Result<bool, String> {
    match value {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(Value::String(value)) => Ok(value.eq_ignore_ascii_case("true")),
        _ => Err("expected boolean material value".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization_finishes_before_color_validation_or_warnings() {
        let mut warnings = Vec::new();
        let result = parse_shared_palette_with_warnings(
            r##"[{"name":"empty"},{"name":"bad-color","color":"invalid"},{"mass":true}]"##,
            |message| warnings.push(message),
        );
        assert_eq!(result.unwrap_err(), "expected numeric material value");
        assert!(warnings.is_empty());

        let result = parse_shared_palette_with_warnings(
            r##"[{"name":"empty"},{"name":"bad-color","color":"invalid"}]"##,
            |message| warnings.push(message),
        );
        assert!(result.unwrap_err().contains("Color 'invalid'"));
        assert_eq!(warnings, ["Warning: empty has no associated color"]);
    }

    #[test]
    fn duplicate_warnings_follow_group_order_and_include_repeated_aliases() {
        let mut warnings = Vec::new();
        let (palette, order) = parse_shared_palette_with_warnings(
            r##"[{"name":"first","colors":["#00000A","#000001","#00000a"]},{"name":"empty"},{"color":"#000001"}]"##,
            |message| warnings.push(message),
        ).unwrap();
        assert_eq!(order, vec![10, 1]);
        assert_eq!(
            warnings,
            vec![
                "Warning: empty has no associated color",
                "Warning: materials [first, first] are requesting the same color: #a, choosing first: first",
                "Warning: materials [first, null] are requesting the same color: #1, choosing first: first",
            ]
        );
        assert!(Arc::ptr_eq(&palette[&10], &palette[&1]));
    }

    #[test]
    fn source_palette_preserves_alias_identity_first_winner_and_distinct_records() {
        let palette = SourceMaterials::from_json(
            r##"[
          {"name":"first","color":"#123456","colors":["#abcdef","#123456"]},
          {"name":"second","color":"#123456","colour":"#010203"},
          {"name":"first","color":"#555555"}
        ]"##,
        )
        .unwrap();
        let first = palette.get(0x123456).unwrap();
        assert!(Arc::ptr_eq(&first, &palette.get(0xabcdef).unwrap()));
        assert_eq!(first.name, "first");
        assert_eq!(palette.get(0x010203).unwrap().name, "second");
        assert!(!Arc::ptr_eq(&first, &palette.get(0x555555).unwrap()));
        assert!(Arc::ptr_eq(&first, &palette.clone().get(0x123456).unwrap()));
        assert!(palette.get(0xffffff).is_none());
        let records: Vec<Value> = serde_json::from_str(&palette.to_json().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn gson_scalar_coercion_null_name_and_invalid_type_failure() {
        let palette = Materials::from_json(r##"[
          {"color":"#123456","name":true,"mass":" 42.5 ","isHull":"TrUe","isRope":" true ","strength":null},
          {"color":"#abcdef","tensileStrength":null}
        ]"##).unwrap();
        let first = palette.get(0x123456).unwrap();
        assert_eq!(first.name, "true");
        assert_eq!(first.mass, 42.5);
        assert!(first.is_hull);
        assert!(!first.is_rope);
        assert_eq!(first.strength, 0.0);
        let values: Vec<Value> = serde_json::from_str(&palette.to_json().unwrap()).unwrap();
        assert!(values[1].get("name").is_none());
        assert!(values[1].get("tensileStrength").is_none());
        for invalid in [
            r##"[{"color":"#123456","mass":true}]"##,
            r##"[{"color":"#123456","mass":"invalid"}]"##,
            r##"[{"color":"#123456","isHull":1}]"##,
            r##"[{"color":"#123456","colors":"#abcdef"}]"##,
            r##"[{"color":"#123456","colors":[null]}]"##,
            r##"[{"color":"#123456","name":{}}]"##,
        ] {
            assert!(Materials::from_json(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn export_preserves_spelling_order_aliases_and_first_color_winner() {
        let palette = Materials::from_json(
            r##"[
          {"name":"first","color":"#010203","colours":["#040506"],"mass":7},
          {"name":"hidden","color":"#010203","mass":99},
          {"name":"last","colour":"#070809","tensileStrength":12}
        ]"##,
        )
        .unwrap();
        let json = palette.to_json().unwrap();
        let values: Vec<Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0]["name"], "first");
        assert_eq!(values[0]["colours"][0], "#040506");
        assert!(values[0].get("compressiveStrength").is_none());
        assert_eq!(values[1]["colour"], "#070809");
        assert_eq!(
            Materials::from_json(&json)
                .unwrap()
                .get(0x040506)
                .unwrap()
                .mass,
            7.0
        );
        assert_eq!(Materials::default().to_json().unwrap(), "[]");
    }

    #[test]
    fn material_equality_retains_java_array_identity_and_signed_zero() {
        let a = Material {
            colors: Some(Arc::new(vec!["#010203".into()])),
            ..Default::default()
        };
        assert!(a.source_equals(&a.clone()));
        let b = Material {
            colors: Some(Arc::new(vec!["#010203".into()])),
            ..Default::default()
        };
        assert!(!a.source_equals(&b));
        let mut c = a.clone();
        c.mass = -0.0;
        assert!(!a.source_equals(&c));
    }

    #[test]
    fn export_uses_gson_field_order_html_escaping_and_rejects_nonfinite_floats() {
        let mut palette = Materials::from_json(
            r##"[{"name":"<&='>","color":"#123456","colors":["#abcdef"],"mass":0.1}]"##,
        )
        .unwrap();
        let json = palette.to_json().unwrap();
        assert!(json.contains(r#""name": "\u003c\u0026\u003d\u0027\u003e""#));
        assert!(json.find("\"name\"").unwrap() < json.find("\"color\"").unwrap());
        assert!(json.contains("\"colors\": [\n      \"#abcdef\"\n    ]"));
        assert!(json.contains("\"mass\": 0.1"));
        palette.materials.get_mut(&0x123456).unwrap().mass = f32::INFINITY;
        assert!(palette.to_json().is_err());
        palette.materials.get_mut(&0x123456).unwrap().mass = f32::NAN;
        assert!(palette.to_json().is_err());
    }
}

//! Materials$Material: retained palette record and its nullable Gson fields.
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub name: String,
    pub(super) name_is_null: bool,
    pub color: Option<String>,
    pub colors: Option<Arc<Vec<String>>>,
    pub colour: Option<String>,
    pub colours: Option<Arc<Vec<String>>>,
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

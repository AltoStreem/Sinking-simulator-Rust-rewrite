//! GLReference.java typed expression text.
use super::gl_type::GlType;
#[derive(Clone)]
pub(crate) struct GlReference<T: GlType> {
    pub gl_type: T,
    pub value: String,
}
impl<T: GlType> GlReference<T> {
    pub fn new(gl_type: T, value: impl Into<String>) -> Self {
        Self {
            gl_type,
            value: value.into(),
        }
    }
}
impl<T: GlType> std::fmt::Display for GlReference<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.value)
    }
}

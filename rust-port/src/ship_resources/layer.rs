use std::fmt;

/// Kotlin data class `Layer(name: String)`, including its retained default instance.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Layer(String);

impl Layer {
    pub(super) const fn empty() -> Self {
        Self(String::new())
    }
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    /// Source `getName()` returns the stored value verbatim (empty for DEFAULT).
    pub(crate) fn name(&self) -> &str {
        &self.0
    }
    /// UI-only label; the JVM model itself does not substitute a display string.
    pub(crate) fn display_name(&self) -> &str {
        if self.0.is_empty() {
            "Default"
        } else {
            &self.0
        }
    }
    pub(crate) fn is_default(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn component1(&self) -> String {
        self.0.clone()
    }
    pub(crate) fn copy(&self, name: impl Into<String>) -> Self {
        Self::new(name)
    }
    /// Java/Kotlin String.hashCode over UTF-16 code units, used by source HashMap behavior.
    pub(crate) fn java_hash_code(&self) -> i32 {
        self.0.encode_utf16().fold(0i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
    pub(crate) fn default_layer() -> &'static Self {
        super::layer_companion::get_default()
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::default_layer().clone()
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Layer(name={})", self.0)
    }
}

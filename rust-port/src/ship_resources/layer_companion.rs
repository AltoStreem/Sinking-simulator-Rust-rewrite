//! Recovered Layer$Companion.class shared DEFAULT getter.
use super::layer::Layer;
static DEFAULT: Layer = Layer::empty();
pub(crate) fn get_default() -> &'static Layer {
    &DEFAULT
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_getter_retains_identity_and_empty_source_name() {
        let first = get_default();
        let second = get_default();
        assert!(std::ptr::eq(first, second));
        assert!(std::ptr::eq(first, Layer::default_layer()));
        assert_eq!(first.name(), "");
        assert_eq!(first.java_hash_code(), 0);
        assert_eq!(first.to_string(), "Layer(name=)");
        let copied = first.copy("");
        assert_eq!(&copied, first);
        assert!(!std::ptr::eq(&copied, first));
    }
}

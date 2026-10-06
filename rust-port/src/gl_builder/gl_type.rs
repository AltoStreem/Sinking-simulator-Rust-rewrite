//! GLType.java generic shader-language type contract.
pub(crate) trait GlType {
    type Value;
    fn type_name(&self) -> &str;
}

//! Recovered MaskStrutsDataHolder.createMask.3.1 from the original JAR class.
pub(crate) fn invoke(material: Option<&crate::materials::Material>) -> bool {
    material.is_some_and(|material| !material.is_rope)
}

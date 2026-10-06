//! TypedDataHolder.java: expose the source vector type in addition to texture metadata.
#![allow(dead_code)]
use crate::gl_data_holder::GlDataHolder;
pub(crate) trait TypedDataHolder: GlDataHolder {
    type BaseType;
    fn base_type(&self) -> Self::BaseType;
}

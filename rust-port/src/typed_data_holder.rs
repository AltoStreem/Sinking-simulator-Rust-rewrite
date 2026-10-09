//! TypedDataHolder.java: preserve the relationship between a holder's GL type and value type.
#![allow(dead_code)]
use crate::{
    gl_builder::gl_type::GlType,
    gl_data_holder::{GlDataHolder, SourceGlDataHolder},
};

pub(crate) trait TypedDataHolder: GlDataHolder {
    type Value;
    type BaseType: GlType<Value = Self::Value>;
    fn base_type(&self) -> Self::BaseType;
}

/// Native-texture counterpart used by the source-facing GL holder adapters.
pub(crate) trait SourceTypedDataHolder: SourceGlDataHolder {
    type Value;
    type BaseType: GlType<Value = Self::Value>;
    fn base_type(&self) -> Self::BaseType;
}

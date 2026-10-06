//! FloatDataHolder.java allocation and component formats.
//! Storage is padded to vec4 for WGSL; unused components remain zero.
#![allow(dead_code)]
use crate::{
    gl_data_holder::{DataTexture, GlDataHolder},
    typed_data_holder::TypedDataHolder,
};
pub(crate) const INTERNAL_FORMAT: [i32; 5] = [0, 33326, 33328, 34837, 34836];
pub(crate) const FORMAT: [i32; 5] = [0, 6403, 33319, 6407, 6408];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FloatVectorType {
    pub components: usize,
}
pub(crate) struct FloatDataHolder<const N: usize> {
    texture: DataTexture,
    storage: Vec<[f32; 4]>,
}
impl<const N: usize> FloatDataHolder<N> {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            (1..=4).contains(&N),
            "Source float vector size must be 1..=4"
        );
        let count = width
            .checked_mul(height)
            .expect("Data texture dimensions overflow");
        Self {
            texture: DataTexture::new(width, height, N, FORMAT[N], INTERNAL_FORMAT[N], 5126),
            storage: vec![[0.0; 4]; count],
        }
    }
    pub fn into_storage(self) -> Vec<[f32; 4]> {
        self.storage
    }
    pub fn into_state_planes(self, planes: usize) -> Vec<[f32; 4]> {
        let mut storage = Vec::with_capacity(
            self.storage
                .len()
                .checked_mul(planes)
                .expect("Data plane size overflow"),
        );
        for _ in 0..planes {
            storage.extend_from_slice(&self.storage);
        }
        storage
    }
}
impl<const N: usize> GlDataHolder for FloatDataHolder<N> {
    fn texture(&self) -> &DataTexture {
        &self.texture
    }
}
impl<const N: usize> TypedDataHolder for FloatDataHolder<N> {
    type BaseType = FloatVectorType;
    fn base_type(&self) -> FloatVectorType {
        FloatVectorType { components: N }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_formats_zero_initialization_and_component_type_are_preserved() {
        let holder = FloatDataHolder::<2>::new(3, 2);
        assert_eq!(holder.base_type().components, 2);
        let texture = holder.texture();
        assert_eq!(
            (
                texture.external_format,
                texture.internal_format,
                texture.scalar_type
            ),
            (33319, 33328, 5126)
        );
        assert_eq!(
            (
                texture.min_filter,
                texture.mag_filter,
                texture.wrap_s,
                texture.wrap_t
            ),
            (9728, 9728, 33069, 33069)
        );
        assert!(!texture.mipmapped);
        assert_eq!(holder.into_storage(), vec![[0.0; 4]; 6]);
        assert_eq!(
            FloatDataHolder::<4>::new(3, 2).into_state_planes(3),
            vec![[0.0; 4]; 18]
        );
        assert_eq!(FloatDataHolder::<1>::new(0, 0).into_storage().len(), 0);
        assert_eq!(
            FloatDataHolder::<3>::new(1, 1).texture().internal_format,
            34837
        );
    }
}

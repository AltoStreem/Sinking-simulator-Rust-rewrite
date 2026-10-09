//! FloatDataHolder.java allocation and component formats.
//! Storage is padded to vec4 for WGSL; unused components remain zero.
#![allow(dead_code)]
use crate::{
    gl_data_holder::{DataTexture, GlDataHolder},
    typed_data_holder::TypedDataHolder,
};
pub(crate) const INTERNAL_FORMAT: [i32; 5] = [0, 33326, 33328, 34837, 34836];
pub(crate) const FORMAT: [i32; 5] = [0, 6403, 33319, 6407, 6408];

pub(crate) struct SourceFloatDataHolder<const N: usize> {
    pub(crate) width: i32,
    pub(crate) height: i32,
    texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>,
}
impl<const N: usize> SourceFloatDataHolder<N> {
    pub(crate) fn new(
        width: i32,
        height: i32,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        let format = *FORMAT
            .get(N)
            .ok_or_else(|| "source vector size is outside format table".to_owned())?;
        let internal = INTERNAL_FORMAT[N];
        // BufferUtils.createByteBuffer(width * height * 4 * 4), regardless of N.
        let capacity = width.wrapping_mul(height).wrapping_mul(4).wrapping_mul(4);
        let capacity = usize::try_from(capacity)
            .map_err(|_| "negative source byte buffer capacity".to_owned())?;
        let pixels = std::sync::Arc::new(std::sync::Mutex::new(vec![0; capacity]));
        let texture = crate::texture_2d::SourceTexture2D::new(
            Some(pixels),
            [width, height],
            format,
            internal,
            5126,
            false,
            std::sync::Arc::new(crate::float_data_texture_config::configure),
            backend,
            context,
            runtime,
        );
        Ok(Self {
            width,
            height,
            texture: std::sync::Arc::new(texture),
        })
    }
    pub(crate) fn base_type(&self) -> FloatVectorType {
        FloatVectorType { components: N }
    }
}
impl<const N: usize> crate::typed_data_holder::SourceTypedDataHolder for SourceFloatDataHolder<N> {
    type Value = Vec<f32>;
    type BaseType = FloatVectorType;
    fn base_type(&self) -> FloatVectorType {
        FloatVectorType { components: N }
    }
}
impl<const N: usize> crate::gl_data_holder::SourceGlDataHolder for SourceFloatDataHolder<N> {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        self.texture.clone()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FloatVectorType {
    pub components: usize,
}
impl crate::gl_builder::gl_type::GlType for FloatVectorType {
    type Value = Vec<f32>;
    fn type_name(&self) -> &str {
        match self.components {
            // The original GLVec1 singleton reports "vec2" despite having size ONE.
            1 | 2 => "vec2",
            3 => "vec3",
            4 => "vec4",
            _ => panic!("Source GL vector size is outside 1..=4"),
        }
    }
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
    type Value = Vec<f32>;
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

//! UInt8DataHolder.java texture allocation contract and exact source format tables.
//! Source null allocation is represented by None, not a claim of zero GPU data.
#![allow(dead_code)]
use crate::gl_data_holder::{DataTexture, GlDataHolder};
pub(crate) const INTERNAL_FORMAT: [i32; 5] = [0, 33330, 33336, 36221, 36220];
pub(crate) const FORMAT: [i32; 5] = [0, 36244, 33320, 36248, 36249];

/// Native Texture2D-backed counterpart of the original GLDataHolder.
pub(crate) struct SourceUInt8DataHolder {
    pub(crate) width: i32,
    pub(crate) height: i32,
    texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>,
}
impl SourceUInt8DataHolder {
    pub(crate) fn new(
        width: i32,
        height: i32,
        components: i32,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        // Java indexes format first and internalFormat second. Preserve that order
        // and accept table entry zero, as the source constructor does.
        let index = usize::try_from(components).map_err(|_| {
            format!("Source UInt8DataHolder format array index out of bounds: {components}")
        })?;
        let external_format = *FORMAT.get(index).ok_or_else(|| {
            format!("Source UInt8DataHolder format array index out of bounds: {components}")
        })?;
        let internal_format = *INTERNAL_FORMAT.get(index).ok_or_else(|| {
            format!("Source UInt8DataHolder internalFormat array index out of bounds: {components}")
        })?;
        let texture = crate::texture_2d::SourceTexture2D::new(
            None,
            [width, height],
            external_format,
            internal_format,
            5125,
            false,
            std::sync::Arc::new(crate::uint8_data_texture_config::configure),
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
}
impl crate::gl_data_holder::SourceGlDataHolder for SourceUInt8DataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        self.texture.clone()
    }
}

pub(crate) struct UInt8DataHolder {
    texture: DataTexture,
    data: Option<Vec<[u32; 4]>>,
}
impl UInt8DataHolder {
    pub fn new(width: usize, height: usize, components: usize) -> Self {
        // The Java constructor does not validate the component count. It indexes
        // `format[components]` before `internalFormat[components]`, so index 0 is
        // a valid (if unusual) zero-format texture and an out-of-range index
        // fails at the first array lookup.
        let external_format = *FORMAT
            .get(components)
            .expect("Source UInt8DataHolder format array index out of bounds");
        let internal_format = *INTERNAL_FORMAT
            .get(components)
            .expect("Source UInt8DataHolder internalFormat array index out of bounds");
        Self {
            texture: DataTexture::new(
                width,
                height,
                components,
                external_format,
                internal_format,
                5125,
            ),
            data: None,
        }
    }
    pub fn single_channel(width: usize, height: usize) -> Self {
        Self::new(width, height, 1)
    }
    pub fn into_storage(self) -> Option<Vec<[u32; 4]>> {
        self.data
    }
}
impl GlDataHolder for UInt8DataHolder {
    fn texture(&self) -> &DataTexture {
        &self.texture
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_null_data_default_and_integer_formats_are_preserved() {
        let holder = UInt8DataHolder::single_channel(2, 1);
        assert_eq!(
            (
                holder.texture().internal_format,
                holder.texture().external_format,
                holder.texture().scalar_type
            ),
            (33330, 36244, 5125)
        );
        assert!(holder.into_storage().is_none());
        assert_eq!(
            UInt8DataHolder::new(1, 1, 3).texture().internal_format,
            36221
        );
    }
}

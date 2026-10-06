//! ScreenFBO.java's window-sized render target, adapted to Bevy image targets.
//! The scene target is copied into a regenerated mip chain before Sea renders.
use bevy::{prelude::*, render::render_resource::TextureFormat};
#[derive(Resource, Clone, bevy::render::extract_resource::ExtractResource)]
pub(crate) struct ScreenFbo {
    pub texture: Handle<Image>,
    pub filtered: Handle<Image>,
    pub size: UVec2,
}
impl ScreenFbo {
    pub fn new(size: UVec2, images: &mut Assets<Image>) -> Self {
        Self {
            texture: images.add(Self::image(size)),
            filtered: images.add(Self::mip_image(size)),
            size,
        }
    }
    fn image(size: UVec2) -> Image {
        let mut image = Image::new_target_texture(
            size.x.max(1),
            size.y.max(1),
            TextureFormat::Rgba8Unorm,
            None,
        );
        image.data = None;
        image.copy_on_resize = false;
        image.texture_descriptor.usage |= bevy::render::render_resource::TextureUsages::COPY_SRC;
        image.sampler = bevy::image::ImageSampler::linear();
        image
    }
    fn mip_image(size: UVec2) -> Image {
        let mut image = Self::image(size);
        image.texture_descriptor.mip_level_count = mip_count(size);
        image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;
        image
    }
    pub fn resize(&mut self, size: UVec2, images: &mut Assets<Image>) {
        if self.size != size {
            if let Some(mut image) = images.get_mut(&self.texture) {
                *image = Self::image(size);
            }
            if let Some(mut image) = images.get_mut(&self.filtered) {
                *image = Self::mip_image(size);
            }
            self.size = size;
        }
    }
}

fn mip_count(size: UVec2) -> u32 {
    32 - size.x.max(size.y).max(1).leading_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framebuffer_mip_chain_and_resize_keep_source_dimensions() {
        let mut images = Assets::default();
        let mut target = ScreenFbo::new(UVec2::new(2554, 1378), &mut images);
        let original = target.texture.clone();
        let filtered = target.filtered.clone();
        assert_eq!(
            images
                .get(&filtered)
                .unwrap()
                .texture_descriptor
                .mip_level_count,
            12
        );
        target.resize(UVec2::new(3, 7), &mut images);
        assert_eq!(target.texture, original);
        assert_eq!(target.filtered, filtered);
        assert_eq!(
            images
                .get(&filtered)
                .unwrap()
                .texture_descriptor
                .mip_level_count,
            3
        );
        assert_eq!(
            images.get(&filtered).unwrap().texture_descriptor.size.width,
            3
        );
    }
    #[test]
    fn framebuffer_compute_shader_validates() {
        let shader = naga::front::wgsl::parse_str(include_str!(
            "../assets/shaders/framebuffer_mipmaps.wgsl"
        ))
        .unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&shader)
        .unwrap();
    }
}

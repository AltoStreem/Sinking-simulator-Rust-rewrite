//! Texture2D configuration used by ShipResource.texture and textureNow.
//! Source GL constants: MAG NEAREST (9728), MIN NEAREST_MIPMAP_LINEAR
//! (9986), and CLAMP_TO_BORDER (33069). Border sampling is handled in
//! ship_texture.wgsl so this does not require optional GPU sampler features.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use image::RgbaImage;

pub(crate) fn ship_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::nearest()
    })
}

/// Keep mip colors in original RGBA code space, matching the source RGBA8
/// texture rather than averaging Bevy's decoded linear-light colors.
pub(crate) fn ship_texture(rgba: RgbaImage) -> Image {
    let width = rgba.width();
    let height = rgba.height();
    assert!(width > 0 && height > 0);
    let mut texture = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.as_raw().clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let mut levels = 1;
    let mut data = rgba.as_raw().clone();
    let mut previous = rgba;
    while previous.width() > 1 || previous.height() > 1 {
        let next = image::imageops::resize(
            &previous,
            (previous.width() / 2).max(1),
            (previous.height() / 2).max(1),
            image::imageops::FilterType::Triangle,
        );
        data.extend_from_slice(next.as_raw());
        previous = next;
        levels += 1;
    }
    texture.data = Some(data);
    texture.texture_descriptor.mip_level_count = levels;
    texture.sampler = ship_sampler();
    texture
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_texture_has_complete_mips_and_pixel_filtering() {
        let texture = ship_texture(RgbaImage::from_pixel(8, 4, image::Rgba([20, 40, 80, 255])));
        assert_eq!(texture.texture_descriptor.mip_level_count, 4);
        assert_eq!(texture.data.as_ref().unwrap().len(), (32 + 8 + 2 + 1) * 4);
        for pixel in texture.data.unwrap().chunks_exact(4) {
            assert_eq!(pixel, [20, 40, 80, 255]);
        }
        let ImageSampler::Descriptor(sampler) = texture.sampler else {
            panic!("missing source sampler")
        };
        assert_eq!(sampler.mag_filter, ImageFilterMode::Nearest);
        assert_eq!(sampler.min_filter, ImageFilterMode::Nearest);
        assert_eq!(sampler.mipmap_filter, ImageFilterMode::Linear);
    }
    #[test]
    fn source_texture_non_square_mips_reach_one_pixel() {
        let texture = ship_texture(RgbaImage::from_pixel(1, 7, image::Rgba([255, 0, 0, 255])));
        assert_eq!(texture.texture_descriptor.mip_level_count, 3);
        assert_eq!(texture.data.unwrap().len(), (7 + 3 + 1) * 4);
    }
}

/// Texture2D.java upload/resource path; existing Bevy image conversion remains above.
#[allow(dead_code)]
pub(crate) struct SourceTexture2D {
    pub texture: crate::texture::Texture,
    pub img: Option<crate::texture::PixelBuffer>,
    pub width: i32,
    pub height: i32,
    pub format: i32,
    pub internal_format: i32,
}
#[allow(dead_code)]
impl SourceTexture2D {
    pub fn new(
        img: Option<crate::texture::PixelBuffer>,
        size: [i32; 2],
        format: i32,
        internal_format: i32,
        data_format: i32,
        mipmap: bool,
        conf: crate::texture::Configure,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let texture = crate::texture::Texture::new(3553, conf, backend, context, runtime);
        texture.bind();
        {
            let pixels = img.as_ref().map(|buffer| buffer.lock().unwrap());
            texture.backend.lock().unwrap().image_2d(
                3553,
                internal_format,
                size,
                format,
                data_format,
                pixels.as_deref().map(|pixels| pixels.as_slice()),
            );
        }
        if mipmap {
            texture
                .backend
                .lock()
                .unwrap()
                .generate_mipmaps(texture.target);
        }
        texture.unbind();
        texture.reconfigure();
        Self {
            texture,
            img,
            width: size[0],
            height: size[1],
            format,
            internal_format,
        }
    }
    pub fn from_image(
        dat: &crate::image_data::ImageData,
        internal_format: i32,
        mipmap: bool,
        conf: crate::texture::Configure,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let pixels = std::sync::Arc::new(std::sync::Mutex::new(dat.buffer.clone()));
        Self::new(
            Some(pixels),
            [dat.width, dat.height],
            dat.image_format,
            internal_format,
            dat.format,
            mipmap,
            conf,
            backend,
            context,
            runtime,
        )
    }
}
impl crate::framebuffer_target::FramebufferTarget for SourceTexture2D {
    fn texture(&self) -> Option<&crate::texture::Texture> {
        Some(&self.texture)
    }
    fn bind_to_framebuffer(&self, attachment: i32) {
        self.texture.bind_to_framebuffer(attachment);
    }
}

//! Sky$1.class, recovered independently with CFR: sky-map texture parameters.
#[allow(dead_code)]
pub(crate) fn configure(texture: &crate::texture::Texture) {
    for (parameter, value) in PARAMETERS {
        texture.set_parameter(parameter, value);
    }
}
const PARAMETERS: [(i32, i32); 4] = [(10240, 9729), (10241, 9987), (10242, 33071), (10243, 33071)];

/// Active backend of Sky(String, CameraControl): source FileReader resolution,
/// four channels, RGBA8 and a complete mip chain before use by the material.
pub(crate) fn load(
    reader: &crate::file_reader::FileReader,
    path: &std::path::Path,
) -> Result<bevy::prelude::Image, String> {
    let data = reader.read_image(path, 4)?;
    let mut image = crate::texture_2d::ship_texture(data.into_rgba()?);
    image.texture_descriptor.format = bevy::render::render_resource::TextureFormat::Rgba8Unorm;
    image.sampler = sampler();
    Ok(image)
}

pub(crate) fn sampler() -> bevy::image::ImageSampler {
    use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn active_sky_loader_resolves_bundled_map_with_complete_rgba8_mips() {
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let reader = crate::file_reader::FileReader {
            ss_home: assets.join("absent-home"),
            ss_resources: assets.join("absent-resources"),
            bundled_resources: assets,
        };
        let path = std::path::Path::new("config/sky.png");
        let source = reader.read_image(path, 4).unwrap();
        let image = super::load(&reader, path).unwrap();
        assert_eq!(
            image.texture_descriptor.format,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm
        );
        assert_eq!(image.texture_descriptor.size.width, source.width as u32);
        assert_eq!(image.texture_descriptor.size.height, source.height as u32);
        assert_eq!(
            &image.data.as_ref().unwrap()[..source.buffer.len()],
            source.buffer
        );
        let levels = 32 - (source.width.max(source.height) as u32).leading_zeros();
        assert_eq!(image.texture_descriptor.mip_level_count, levels);
        assert!(
            super::load(
                &reader,
                std::path::Path::new("config/nonexistent-sky-map.png")
            )
            .is_err()
        );
    }
    #[test]
    fn source_sky_map_parameters_and_bevy_sampler_agree() {
        use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler};
        assert_eq!(
            super::PARAMETERS,
            [(10240, 9729), (10241, 9987), (10242, 33071), (10243, 33071)]
        );
        let ImageSampler::Descriptor(sampler) = super::sampler() else {
            panic!("descriptor")
        };
        assert_eq!(sampler.mag_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.min_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.mipmap_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);
    }
}

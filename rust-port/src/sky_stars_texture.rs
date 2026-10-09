//! Sky$makeStars$stars$1.class, recovered independently with CFR.
#[allow(dead_code)]
pub(crate) fn configure(texture: &crate::texture::Texture) {
    for (parameter, value) in PARAMETERS {
        texture.set_parameter(parameter, value);
    }
}
const PARAMETERS: [(i32, i32); 4] = [(10240, 9729), (10241, 9729), (10242, 33071), (10243, 33071)];

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
    fn baked_stars_use_linear_nonmipmapped_clamp_to_edge() {
        assert_eq!(
            super::PARAMETERS,
            [(10240, 9729), (10241, 9729), (10242, 33071), (10243, 33071)]
        );
        use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler};
        let ImageSampler::Descriptor(sampler) = super::sampler() else {
            panic!("descriptor")
        };
        assert_eq!(sampler.mag_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.min_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);
    }
}

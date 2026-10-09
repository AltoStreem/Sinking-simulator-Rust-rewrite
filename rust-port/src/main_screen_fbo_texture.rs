//! Main$main$screenFB$1.class: scene texture filtering and transparent border.
#[allow(dead_code)]
pub(crate) fn configure(texture: &crate::texture::Texture) {
    for (parameter, value) in PARAMETERS {
        texture.set_parameter(parameter, value);
    }
}
const PARAMETERS: [(i32, i32); 4] = [(10240, 9729), (10241, 9987), (10242, 33069), (10243, 33069)];

pub(crate) fn sampler(native_border: bool) -> bevy::image::ImageSampler {
    use bevy::image::{
        ImageAddressMode, ImageSampler, ImageSamplerBorderColor, ImageSamplerDescriptor,
    };
    let address = if native_border {
        ImageAddressMode::ClampToBorder
    } else {
        ImageAddressMode::ClampToEdge
    };
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: address,
        address_mode_v: address,
        border_color: native_border.then_some(ImageSamplerBorderColor::TransparentBlack),
        ..ImageSamplerDescriptor::linear()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn original_scene_texture_parameters_and_native_sampler_match() {
        use bevy::image::{
            ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerBorderColor,
        };
        assert_eq!(
            super::PARAMETERS,
            [(10240, 9729), (10241, 9987), (10242, 33069), (10243, 33069)]
        );
        let ImageSampler::Descriptor(native) = super::sampler(true) else {
            panic!()
        };
        assert_eq!(native.address_mode_u, ImageAddressMode::ClampToBorder);
        assert_eq!(native.address_mode_v, ImageAddressMode::ClampToBorder);
        assert_eq!(
            native.border_color,
            Some(ImageSamplerBorderColor::TransparentBlack)
        );
        assert_eq!(native.mag_filter, ImageFilterMode::Linear);
        assert_eq!(native.min_filter, ImageFilterMode::Linear);
        assert_eq!(native.mipmap_filter, ImageFilterMode::Linear);
        let ImageSampler::Descriptor(fallback) = super::sampler(false) else {
            panic!()
        };
        assert_eq!(fallback.address_mode_u, ImageAddressMode::ClampToEdge);
        assert_eq!(fallback.border_color, None);
    }
}

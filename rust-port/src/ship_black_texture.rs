//! Ship$Companion$blackTexture$1.class's one-pixel transparent RGBA8 upload.
#[derive(bevy::prelude::Resource)]
pub(crate) struct SharedBlackTexture(pub bevy::prelude::Handle<bevy::prelude::Image>);
impl bevy::prelude::FromWorld for SharedBlackTexture {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        Self(
            world
                .resource_mut::<bevy::prelude::Assets<bevy::prelude::Image>>()
                .add(image()),
        )
    }
}
fn image() -> bevy::prelude::Image {
    use bevy::{
        asset::RenderAssetUsages,
        image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    let mut image = bevy::prelude::Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; 4],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::nearest()
    });
    image
}
pub(crate) fn create(
    backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: &crate::resource::ResourceRuntime,
) -> crate::texture_2d::SourceTexture2D {
    crate::texture_2d::SourceTexture2D::new(
        Some(std::sync::Arc::new(std::sync::Mutex::new(vec![0; 4]))),
        [1, 1],
        6408,
        32856,
        5121,
        false,
        std::sync::Arc::new(crate::ship_black_texture_config::configure),
        backend,
        context,
        runtime,
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn active_black_texture_is_transparent_rgba8_nearest_repeat_and_shared() {
        use bevy::prelude::*;
        let mut world = World::new();
        world.insert_resource(Assets::<Image>::default());
        world.init_resource::<super::SharedBlackTexture>();
        let first = world.resource::<super::SharedBlackTexture>().0.clone();
        world.init_resource::<super::SharedBlackTexture>();
        assert_eq!(first, world.resource::<super::SharedBlackTexture>().0);
        let assets = world.resource::<Assets<Image>>();
        assert_eq!(assets.len(), 1);
        let image = assets.get(&first).unwrap();
        assert_eq!(image.data.as_ref().unwrap(), &[0; 4]);
        assert_eq!(
            image.texture_descriptor.format,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm
        );
        assert_eq!(image.texture_descriptor.mip_level_count, 1);
        let bevy::image::ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("descriptor")
        };
        assert_eq!(
            sampler.address_mode_u,
            bevy::image::ImageAddressMode::Repeat
        );
        assert_eq!(
            sampler.address_mode_v,
            bevy::image::ImageAddressMode::Repeat
        );
        assert_eq!(sampler.min_filter, bevy::image::ImageFilterMode::Nearest);
        assert_eq!(sampler.mag_filter, bevy::image::ImageFilterMode::Nearest);
    }
}

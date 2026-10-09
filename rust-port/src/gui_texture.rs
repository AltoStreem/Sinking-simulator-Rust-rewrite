//! Encoded GUI texture views for source ImGui sampling/filtering.
//! Original ship images retain their world-rendering format and handles.
use bevy::{prelude::*,camera::visibility::RenderLayers,render::render_resource::TextureFormat};
use std::collections::{HashMap,HashSet};

struct View {source:Handle<Image>,encoded:Handle<Image>,icon:bool,revision:u64}
#[derive(Resource,Default)]
pub(crate) struct GuiTextures(HashMap<AssetId<Image>,View>);

fn encoded_view(image:&Image,icon:bool,native_border:bool)->Option<Image> {
    if image.data.is_none() {return None;}
    let format=match image.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb=>TextureFormat::Rgba8Unorm,
        TextureFormat::Bgra8UnormSrgb=>TextureFormat::Bgra8Unorm,
        _=>return None,
    };
    let mut image=image.clone();image.texture_descriptor.format=format;
    image.texture_descriptor.view_formats=&[];
    if let Some(view)=&mut image.texture_view_descriptor {view.format=Some(format);}
    if icon && format==TextureFormat::Rgba8Unorm {
        let size=image.texture_descriptor.size;
        let base=image::RgbaImage::from_raw(size.width,size.height,
            image.data.as_ref().unwrap()[..(size.width*size.height*4) as usize].to_vec())?;
        let mip_image=crate::texture_2d::ship_texture(base);
        image.data=mip_image.data;
        image.texture_descriptor.mip_level_count=mip_image.texture_descriptor.mip_level_count;
        image.sampler=crate::main_screen_fbo_texture::sampler(native_border);
        image.texture_descriptor.usage|=bevy::render::render_resource::TextureUsages::STORAGE_BINDING
            |bevy::render::render_resource::TextureUsages::COPY_SRC;
    }
    Some(image)
}

pub(crate) fn sync(mut textures:ResMut<GuiTextures>,mut images:ResMut<Assets<Image>>,
    mut mip_requests:ResMut<crate::gui_icon_mips::IconMipRequests>,
    mut events:MessageReader<AssetEvent<Image>>,
    server:Option<Res<AssetServer>>,
    device:Option<Res<bevy::render::renderer::RenderDevice>>,
    mut sprites:Query<(&mut Sprite,&RenderLayers)>) {
    let native_border=device.is_some_and(|device|device.features().contains(
        bevy::render::settings::WgpuFeatures::ADDRESS_MODE_CLAMP_TO_BORDER));
    let changed:HashSet<_>=events.read().filter_map(|event|match event {
        AssetEvent::Modified {id}|AssetEvent::Added {id}|AssetEvent::Removed {id}=>Some(*id),_=>None,
    }).collect();
    let reverse:HashMap<_,_>=textures.0.iter().map(|(id,view)|(view.encoded.id(),*id)).collect();
    let mut used=HashSet::new();
    for (mut sprite,layers) in &mut sprites {
        if !layers.iter().any(|layer|(1..=6).contains(&layer)||(12..=16).contains(&layer)) {continue;}
        let id=reverse.get(&sprite.image.id()).copied().unwrap_or(sprite.image.id());
        let icon=server.as_ref().and_then(|server|server.get_path(id)).is_some_and(|path|
            path.path().starts_with("icons"));
        if let Some(view)=textures.0.get_mut(&id) {
            if changed.contains(&id) {
                if let Some(image)=images.get(&view.source).and_then(|image|encoded_view(image,icon,native_border)) {
                    images.insert(view.encoded.id(),image).expect("Retained GUI image handle");
                    view.revision=view.revision.wrapping_add(1);
                } else {
                    // A closed resource, live target or already raw image no
                    // longer needs a copied view; expose its actual handle.
                    sprite.image=view.source.clone();continue;
                }
            }
            used.insert(id);
            sprite.image=view.encoded.clone();continue;
        }
        let Some(image)=images.get(&sprite.image).and_then(|image|encoded_view(image,icon,native_border)) else {continue};
        let icon=icon && image.texture_descriptor.format==TextureFormat::Rgba8Unorm;
        let source=sprite.image.clone();let encoded=images.add(image);
        sprite.image=encoded.clone();textures.0.insert(id,View {source,encoded,icon,revision:1});used.insert(id);
    }
    // Strong handles keep source/alias data alive while a GUI consumer uses it.
    // Removed controls release both via normal Bevy asset maintenance.
    textures.0.retain(|id,_|used.contains(id));
    mip_requests.0.clear();
    for view in textures.0.values().filter(|view|view.icon) {
        mip_requests.0.insert(view.encoded.id(),view.revision);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icons_have_complete_encoded_mips_and_source_linear_transparent_border_sampler() {
        use bevy::image::{ImageAddressMode,ImageSampler,ImageFilterMode,ImageSamplerBorderColor};
        let mut original=crate::texture_2d::ship_texture(image::RgbaImage::from_pixel(8,4,image::Rgba([64,128,192,255])));
        original.texture_descriptor.mip_level_count=1;original.data.as_mut().unwrap().truncate(8*4*4);
        let icon=encoded_view(&original,true,true).unwrap();
        assert_eq!(icon.texture_descriptor.format,TextureFormat::Rgba8Unorm);
        assert_eq!(icon.texture_descriptor.mip_level_count,4);
        let ImageSampler::Descriptor(sampler)=icon.sampler else {panic!("Icon sampler")};
        assert_eq!(sampler.mag_filter,ImageFilterMode::Linear);
        assert_eq!(sampler.min_filter,ImageFilterMode::Linear);
        assert_eq!(sampler.mipmap_filter,ImageFilterMode::Linear);
        assert_eq!(sampler.address_mode_u,ImageAddressMode::ClampToBorder);
        assert_eq!(sampler.address_mode_v,ImageAddressMode::ClampToBorder);
        assert_eq!(sampler.border_color,Some(ImageSamplerBorderColor::TransparentBlack));
        assert_eq!(original.texture_descriptor.mip_level_count,1,"Source/world image is unchanged");
    }
    #[test]
    fn gui_views_preserve_bytes_mips_sampler_world_format_updates_and_release() {
        let mut app=App::new();app.add_plugins(MinimalPlugins).init_resource::<Assets<Image>>()
            .init_resource::<crate::gui_icon_mips::IconMipRequests>()
            .init_resource::<GuiTextures>().add_message::<AssetEvent<Image>>().add_systems(Update,sync);
        let original=crate::texture_2d::ship_texture(image::RgbaImage::from_pixel(4,2,image::Rgba([51,102,153,128])));
        let source=app.world_mut().resource_mut::<Assets<Image>>().add(original.clone());
        let gui=app.world_mut().spawn((Sprite::from_image(source.clone()),RenderLayers::layer(4))).id();
        let world=app.world_mut().spawn((Sprite::from_image(source.clone()),RenderLayers::layer(0))).id();
        app.update();
        let alias=app.world().get::<Sprite>(gui).unwrap().image.clone();
        assert_ne!(alias,source);
        assert_eq!(app.world().get::<Sprite>(world).unwrap().image,source);
        let images=app.world().resource::<Assets<Image>>();
        assert_eq!(images.get(&source).unwrap().texture_descriptor.format,TextureFormat::Rgba8UnormSrgb);
        let encoded=images.get(&alias).unwrap();
        assert_eq!(encoded.texture_descriptor.format,TextureFormat::Rgba8Unorm);
        assert_eq!(encoded.data,original.data);
        assert_eq!(encoded.texture_descriptor.mip_level_count,original.texture_descriptor.mip_level_count);
        assert_eq!(format!("{:?}",encoded.sampler),format!("{:?}",original.sampler));
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&source).unwrap().data.as_mut().unwrap()[0]=99;
        app.world_mut().write_message(AssetEvent::Modified {id:source.id()});app.update();
        assert_eq!(app.world().get::<Sprite>(gui).unwrap().image,alias);
        assert_eq!(app.world().resource::<Assets<Image>>().get(&alias).unwrap().data.as_ref().unwrap()[0],99);
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&source).unwrap().texture_descriptor.format=TextureFormat::Rgba8Unorm;
        app.world_mut().write_message(AssetEvent::Modified {id:source.id()});app.update();
        assert_eq!(app.world().get::<Sprite>(gui).unwrap().image,source);
        assert!(app.world().resource::<GuiTextures>().0.is_empty());
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&source).unwrap().texture_descriptor.format=TextureFormat::Rgba8UnormSrgb;
        app.world_mut().write_message(AssetEvent::Modified {id:source.id()});app.update();
        assert_ne!(app.world().get::<Sprite>(gui).unwrap().image,source);
        app.world_mut().despawn(gui);app.update();
        assert!(app.world().resource::<GuiTextures>().0.is_empty());
    }
}

//! Full mip pixel comparison with direct original OpenGL readbacks.
use super::*;
use bevy::{app::PluginsState,camera::{RenderTarget,visibility::RenderLayers},
    render::{extract_resource::ExtractResourcePlugin,gpu_readback::{Readback,ReadbackComplete},
        render_resource::*,renderer::RenderContext},window::{WindowPlugin,ExitCondition}};

#[derive(Clone)]
struct Region {level:u32,width:u32,height:u32,y:u32,offset:usize}
#[derive(Clone)]
struct Icon {path:String,regions:Vec<Region>}
#[derive(Component)]
struct IconIndex(usize);
#[derive(Resource,Clone,ExtractResource)]
struct Atlas {texture:Handle<Image>,icons:Vec<Icon>,entries:Vec<(usize,Handle<Image>)>}
#[derive(Resource,Default)]
struct Captured {bytes:Vec<u8>,revision:u64}
fn capture(event:On<ReadbackComplete>,mut pixels:ResMut<Captured>) {
    pixels.bytes=event.data.clone();pixels.revision+=1;
}
fn collect(mut atlas:ResMut<Atlas>,sprites:Query<(&IconIndex,&Sprite)>,images:Res<Assets<Image>>) {
    atlas.entries=sprites.iter().filter(|(_,sprite)|images.get(&sprite.image).is_some_and(|image|
        image.texture_descriptor.usage.contains(TextureUsages::STORAGE_BINDING)))
        .map(|(index,sprite)|(index.0,sprite.image.clone())).collect();
}
fn copy_atlas(atlas:Res<Atlas>,images:Res<RenderAssets<GpuImage>>,
    pipeline:Res<crate::render_fbo::MipmapPipeline>,cache:Res<PipelineCache>,mut context:RenderContext) {
    if !pipeline.is_ready(&cache) || atlas.entries.len()!=atlas.icons.len() {return;}
    let Some(destination)=images.get(&atlas.texture) else {return};
    if atlas.entries.iter().any(|(_,image)|images.get(image).is_none()) {return;}
    for (index,image) in &atlas.entries {
        let source=images.get(image).unwrap();
        for region in &atlas.icons[*index].regions {
            context.command_encoder().copy_texture_to_texture(
                TexelCopyTextureInfo {texture:&source.texture,mip_level:region.level,origin:Origin3d::ZERO,aspect:TextureAspect::All},
                TexelCopyTextureInfo {texture:&destination.texture,mip_level:0,origin:Origin3d {x:0,y:region.y,z:0},aspect:TextureAspect::All},
                Extent3d {width:region.width,height:region.height,depth_or_array_layers:1});
        }
    }
}

#[test]
#[ignore="Requires actual GPU; run explicitly with --ignored"]
fn gpu_all_icon_mip_pixels_match_original_gl_readbacks() {
    let metadata:serde_json::Value=serde_json::from_str(include_str!("../tools/fixtures/source-icon-gl-mips.json")).unwrap();
    let expected=include_bytes!("../tools/fixtures/source-icon-gl-mips.rgba");
    let mut y=0;let mut icons=Vec::new();
    for icon in metadata.as_array().unwrap() {
        let mut regions=Vec::new();
        for (index,level) in icon["Levels"].as_array().unwrap().iter().enumerate() {
            let height=level["Height"].as_u64().unwrap() as u32;
            regions.push(Region {level:index as u32+1,width:level["Width"].as_u64().unwrap() as u32,
                height,y,offset:level["Offset"].as_u64().unwrap() as usize});y+=height;
        }
        icons.push(Icon {path:icon["Path"].as_str().unwrap().to_owned(),regions});
    }
    let mut app=App::new();app.add_plugins(DefaultPlugins
        .set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
        .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
        .add_plugins((crate::gui_gpu_blend::SourceGuiBlendPlugin,ExtractResourcePlugin::<Atlas>::default()))
        .init_resource::<Captured>().add_systems(PostUpdate,collect.after(crate::gui_texture::sync));
    while app.plugins_state()!=PluginsState::Ready {
        bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));
    }
    app.finish();app.cleanup();
    let mut image=Image::new_target_texture(128,y,TextureFormat::Rgba8Unorm,None);
    image.copy_on_resize=false;image.texture_descriptor.usage|=TextureUsages::COPY_SRC;
    let texture=app.world_mut().resource_mut::<Assets<Image>>().add(image);
    for (index,icon) in icons.iter().enumerate() {
        let handle=app.world().resource::<AssetServer>().load(icon.path.clone());
        app.world_mut().spawn((IconIndex(index),Sprite::from_image(handle),Visibility::Hidden,RenderLayers::layer(1)));
    }
    let camera_image=app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(2,2,TextureFormat::Rgba8Unorm,None));
    app.world_mut().spawn((Camera2d,Msaa::Off,RenderTarget::Image(camera_image.into()),RenderLayers::layer(0)));
    app.world_mut().insert_resource(Atlas {texture:texture.clone(),icons,entries:Vec::new()});
    app.world_mut().spawn(Readback::texture(texture)).observe(capture);
    app.sub_app_mut(RenderApp).add_systems(Core2d,copy_atlas.after(super::generate_icon_mips).before(Core2dSystems::MainPass));
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    let mut last=0;let mut stable=0;let mut previous=Vec::new();
    loop {
        app.update();let pixels=app.world().resource::<Captured>();
        if pixels.bytes.len()==y as usize*512 && pixels.revision!=last && pixels.bytes.iter().any(|&byte|byte!=0) {
            last=pixels.revision;stable=if pixels.bytes==previous {stable+1}else {0};previous=pixels.bytes.clone();
            if stable>=3 {
                let atlas=app.world().resource::<Atlas>();let mut total=0;let mut differences=0;let mut maximum=0;
                for icon in &atlas.icons {
                    let mut count=0;let mut max=0;
                    for region in &icon.regions {
                        for row in 0..region.height as usize {
                            let src=(region.y as usize+row)*512;
                            let reference=region.offset+row*region.width as usize*4;
                            for (&actual,&expected) in pixels.bytes[src..src+region.width as usize*4].iter()
                                .zip(&expected[reference..reference+region.width as usize*4]) {
                                let diff=actual.abs_diff(expected);max=max.max(diff);total+=1;if diff!=0 {count+=1;}
                            }
                        }
                    }
                    println!("Native GPU versus original GL {}: max difference {max}, {count} different bytes",icon.path);
                    maximum=maximum.max(max);differences+=count;
                }
                println!("Compared {total} original GL mip bytes: maximum {maximum}, different {differences}");
                if let Ok(destination)=std::env::var("SS2_GPU_ICON_ATLAS_OUTPUT") {
                    std::fs::write(destination,&pixels.bytes).unwrap();
                }
                assert_eq!(differences,0,"Full original GL icon mip pixel parity is not achieved");break;
            }
        }
        assert!(std::time::Instant::now()<deadline,"GPU icon atlas never stabilized");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

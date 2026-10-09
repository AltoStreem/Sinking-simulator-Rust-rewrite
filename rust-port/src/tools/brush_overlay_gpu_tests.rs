//! Actual backend evidence for brush/GUI encoded RGB blending and capture isolation.
use crate::*;
use bevy::{app::PluginsState,camera::RenderTarget,render::{gpu_readback::{Readback,ReadbackComplete},
    render_resource::{TextureFormat,TextureUsages}},sprite_render::Material2dPlugin,
    window::{ExitCondition,WindowPlugin}};
#[derive(Resource,Default)]
struct Pixels {scene:Vec<u8>,output:Vec<u8>,revisions:[u64;2]}
#[derive(Component)]
struct Capture(usize);
fn capture(event:On<ReadbackComplete>,labels:Query<&Capture>,mut pixels:ResMut<Pixels>) {
    let Ok(label)=labels.get(event.entity) else {return};
    pixels.revisions[label.0]+=1;
    if label.0==0 {pixels.scene=event.data.clone();} else {pixels.output=event.data.clone();}
}
fn pixel(bytes:&[u8],x:usize,y:usize)->[u8;4] {bytes[y*256+x*4..y*256+x*4+4].try_into().unwrap()}
#[test]
#[ignore="Requires actual GPU; run explicitly with --ignored"]
fn gpu_brush_overlay_follows_sea_precedes_gui_and_never_enters_scene_capture() {
    use super::brush_preview::{BrushMaterial,BrushOverlayCamera,OVERLAY_LAYER};
    let mut app=App::new();
    app.add_plugins(DefaultPlugins
        .set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
        .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
        .add_plugins((Material2dPlugin::<BrushMaterial>::default(),crate::gui_gpu_blend::SourceGuiBlendPlugin)).init_resource::<Pixels>();
    while app.plugins_state()!=PluginsState::Ready {
        bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));
    }
    app.finish();app.cleanup();
    let mut target=Image::new_target_texture(32,32,TextureFormat::Rgba8UnormSrgb,None);
    target.texture_descriptor.usage|=TextureUsages::COPY_SRC;
    let scene=app.world_mut().resource_mut::<Assets<Image>>().add(target.clone());
    let output=app.world_mut().resource_mut::<Assets<Image>>().add(target);
    let background=app.world_mut().resource_mut::<Assets<Image>>().add(
        Image::new_target_texture(32,32,TextureFormat::Rgba8Unorm,None));
    app.world_mut().spawn((Camera2d,WorldCamera,Camera {order:-3,
        clear_color:ClearColorConfig::Custom(Color::WHITE),..default()},Msaa::Off,
        RenderTarget::Image(scene.clone().into()),RenderLayers::layer(0)));
    let background_camera=app.world_mut().spawn((Camera2d,Camera {order:-2,
        clear_color:ClearColorConfig::Custom(Color::srgb(0.0,0.0,1.0)),..default()},Msaa::Off,
        RenderTarget::Image(background.clone().into()),RenderLayers::layer(7))).id();
    let mut setup=Schedule::default();setup.add_systems(super::brush_preview::setup_overlay);setup.run(app.world_mut());
    let overlay=app.world_mut().query_filtered::<Entity,With<BrushOverlayCamera>>().single(app.world()).unwrap();
    app.world_mut().entity_mut(overlay).insert((RenderTarget::Image(output.clone().into()),bevy::camera::CompositingSpace::Srgb));
    let mesh=app.world_mut().resource_mut::<Assets<Mesh>>().add(Rectangle::new(32.0,32.0));
    let material=app.world_mut().resource_mut::<Assets<BrushMaterial>>().add(BrushMaterial {
        cursor_radius:Vec4::new(0.0,0.0,8.0,0.0),color:Vec4::new(1.0,0.0,0.0,1.0),background});
    app.world_mut().spawn((DamageBrushPreview,Mesh2d(mesh),MeshMaterial2d(material.clone()),
        Transform::default(),RenderLayers::layer(OVERLAY_LAYER)));
    app.world_mut().spawn((Camera2d,Camera {order:1,clear_color:ClearColorConfig::None,..default()},Msaa::Off,
        RenderTarget::Image(output.clone().into()),RenderLayers::layer(1),bevy::camera::CompositingSpace::Srgb));
    app.world_mut().spawn((Sprite::from_color(Color::srgb(0.0,1.0,0.0),Vec2::splat(2.0)),RenderLayers::layer(1)));
    app.world_mut().spawn((Sprite::from_color(Color::srgba(1.0,1.0,0.0,0.5),Vec2::splat(2.0)),
        Transform::from_xyz(-6.0,0.0,1.0),RenderLayers::layer(1)));
    let gui_mesh=app.world_mut().resource_mut::<Assets<Mesh>>().add(Rectangle::new(2.0,2.0));
    let gui_material=app.world_mut().resource_mut::<Assets<ColorMaterial>>().add(ColorMaterial::from(Color::srgba(1.0,1.0,0.0,0.5)));
    app.world_mut().spawn((Mesh2d(gui_mesh),MeshMaterial2d(gui_material),
        Transform::from_xyz(-6.0,-4.0,1.0),RenderLayers::layer(1)));
    let tint_texture=app.world_mut().resource_mut::<Assets<Image>>().add(crate::texture_2d::ship_texture(
        image::RgbaImage::from_pixel(2,2,image::Rgba([64,128,192,255]))));
    let mut tinted=Sprite::from_image(tint_texture);tinted.custom_size=Some(Vec2::splat(2.0));
    tinted.color=Color::srgba(0.5,0.25,0.75,0.5);
    app.world_mut().spawn((tinted,Transform::from_xyz(6.0,-4.0,1.0),RenderLayers::layer(1)));
    let mut filter_texture=crate::texture_2d::ship_texture(image::RgbaImage::from_fn(2,1,|x,_|
        if x==0 {image::Rgba([0,0,0,255])}else {image::Rgba([255,255,255,255])}));
    filter_texture.sampler=bevy::image::ImageSampler::linear();
    // Force base-level filtering, so a pre-averaged mip cannot conceal an
    // incorrect sRGB hardware filtering path (which would produce ~188).
    filter_texture.texture_descriptor.mip_level_count=1;
    filter_texture.data.as_mut().unwrap().truncate(8);
    let filter_texture=app.world_mut().resource_mut::<Assets<Image>>().add(filter_texture);
    let mut filtered=Sprite::from_image(filter_texture);filtered.custom_size=Some(Vec2::ONE);
    app.world_mut().spawn((filtered,Transform::from_xyz(6.5,-8.5,1.0),RenderLayers::layer(1)));
    let mut gradient=Mesh::from(Rectangle::new(1.0,1.0));
    let bevy::mesh::VertexAttributeValues::Float32x3(positions)=gradient.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else {panic!("Rectangle positions")};
    let colors:Vec<[f32;4]>=positions.iter().map(|p|{let value=p[0]+0.5;[value,value,value,1.0]}).collect();
    gradient.insert_attribute(Mesh::ATTRIBUTE_COLOR,colors);
    let gradient=app.world_mut().resource_mut::<Assets<Mesh>>().add(gradient);
    let gradient_material=app.world_mut().resource_mut::<Assets<ColorMaterial>>().add(ColorMaterial::default());
    app.world_mut().spawn((Mesh2d(gradient),MeshMaterial2d(gradient_material),
        Transform::from_xyz(-6.5,8.5,1.0),RenderLayers::layer(1)));
    let icon_source:Handle<Image>=app.world().resource::<AssetServer>().load("icons/Break.png");
    let mut icon=Sprite::from_image(icon_source.clone());icon.custom_size=Some(Vec2::ONE);
    let icon_entity=app.world_mut().spawn((icon,Transform::from_xyz(0.5,10.5,1.0),RenderLayers::layer(1))).id();
    let picker_material=app.world_mut().resource_mut::<Assets<ColorMaterial>>().add(ColorMaterial::default());
    for (mesh,position) in [
        (crate::gui_color_picker::saturation_value(Vec2::splat(4.0),Vec3::new(0.0,1.0/3.0,1.0)),Vec3::new(-10.0,5.0,1.0)),
        (crate::gui_color_picker::hue(Vec2::new(2.0,12.0)),Vec3::new(-13.0,0.0,1.0)),
        (crate::gui_color_picker::alpha(Vec2::new(2.0,8.0),Vec3::new(0.1,0.2,0.3)),Vec3::new(-10.0,-10.0,1.0)),
    ] {
        let mesh=app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        app.world_mut().spawn((Mesh2d(mesh),MeshMaterial2d(picker_material.clone()),
            Transform::from_translation(position),RenderLayers::layer(1)));
    }
    // The test substitutes image targets for native windows. Request the same
    // final replace used by source_camera_output, bypassing Bevy's auto blend.
    let mut cameras=app.world_mut().query::<&mut Camera>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        camera.output_mode=bevy::camera::CameraOutputMode::Write {
            blend_state:Some(bevy::render::render_resource::BlendState::REPLACE),
            clear_color:ClearColorConfig::None};
    }
    app.world_mut().spawn((Readback::texture(scene),Capture(0))).observe(capture);
    app.world_mut().spawn((Readback::texture(output),Capture(1))).observe(capture);
    for (name,background_color,brush_color,expected,expected_background,expected_translucent_gui,expected_tinted) in [
        ("red over blue",Color::linear_rgba(0.0,0.0,1.0,1.0),Vec4::new(1.0,0.0,0.0,1.0),[128u8,0,128,192],[0u8,0,255,255],[192u8,128,64,160],[80u8,16,136,160]),
        ("white over mixed RGB and alpha",Color::linear_rgba(0.2,0.4,0.6,0.25),Vec4::ONE,[153,179,204,96],[51,102,153,64],[204,217,102,112],[93,105,174,112]),
        ("no selected brush",Color::linear_rgba(0.2,0.4,0.6,0.25),Vec4::ZERO,[51,102,153,64],[51,102,153,64],[153,179,77,96],[42,67,149,96]),
    ] {
    app.world_mut().get_mut::<Camera>(background_camera).unwrap().clear_color=ClearColorConfig::Custom(background_color);
    app.world_mut().resource_mut::<Assets<BrushMaterial>>().get_mut(&material).unwrap().color=brush_color;
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    let mut last=[0,0];let mut stable=0;
    loop {
        app.update();let pixels=app.world().resource::<Pixels>();
        if pixels.scene.len()==8192 && pixels.output.len()==8192 && pixels.revisions[0]!=last[0] && pixels.revisions[1]!=last[1] {
            last=pixels.revisions;
            let brush=pixel(&pixels.output,22,16);let gui=pixel(&pixels.output,16,16);
            let translucent_gui=pixel(&pixels.output,10,16);
            let translucent_mesh=pixel(&pixels.output,10,20);
            let tinted=pixel(&pixels.output,22,20);let filtered=pixel(&pixels.output,22,24);
            let gradient=pixel(&pixels.output,9,7);
            let minified_icon=pixel(&pixels.output,16,5);
            let sv_picker=pixel(&pixels.output,6,10);
            let hue_picker=pixel(&pixels.output,3,15);
            let alpha_picker=pixel(&pixels.output,6,25);
            // Original GL final Break mip is [165,117,5,254]. Compose that
            // source pixel with the case background using the source blend.
            let expected_icon: [u8;4]=std::array::from_fn(|channel| {
                let source=[165u8,117,5,254];let alpha=254.0/255.0;
                (source[channel] as f32*alpha+expected_background[channel] as f32*(1.0-alpha)).round() as u8
            });
            let background=pixel(&pixels.output,4,4);let untouched=pixel(&pixels.scene,22,16);
            let matches=brush.iter().zip(expected).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                background.iter().zip(expected_background).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                translucent_gui.iter().zip(expected_translucent_gui).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                translucent_mesh.iter().zip(expected_translucent_gui).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                tinted.iter().zip(expected_tinted).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                filtered.iter().zip([128u8,128,128,255]).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                gradient.iter().zip([128u8,128,128,255]).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                minified_icon.iter().zip(expected_icon).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                sv_picker.iter().zip([60u8,93,159,195]).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                hue_picker.iter().zip([0u8,255,191,255]).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                alpha_picker.iter().zip([71u8,85,99,192]).all(|(&actual,expected)|actual.abs_diff(expected)<=1) &&
                gui[0]<3 && gui[1]>250 && gui[2]<3 && untouched[0]>250 && untouched[1]>250 && untouched[2]>250;
            stable=if matches {stable+1}else {0};
            if stable>=3 {
                let alias=&app.world().get::<Sprite>(icon_entity).unwrap().image;
                assert_ne!(alias,&icon_source,"GUI icon uses encoded view");
                let image=app.world().resource::<Assets<Image>>().get(alias).unwrap();
                assert_eq!(image.texture_descriptor.mip_level_count,9);
                assert_eq!(image.texture_descriptor.format,TextureFormat::Rgba8Unorm);
                println!("Actual GPU overlay {name}: brush={brush:?}, GUI={gui:?}, translucent GUI={translucent_gui:?}, translucent mesh={translucent_mesh:?}, tinted={tinted:?}, filtered={filtered:?}, gradient={gradient:?}, minified icon={minified_icon:?}, background={background:?}, scene={untouched:?}, SV picker={sv_picker:?}, hue picker={hue_picker:?}, alpha picker={alpha_picker:?}");break;
            }
            if !matches && stable==0 && pixels.revisions[1]%100==0 {
                println!("Pending GUI pixels: tint={tinted:?}, filter={filtered:?}, gradient={gradient:?}, icon={minified_icon:?}, SV={sv_picker:?}, hue={hue_picker:?}, alpha={alpha_picker:?}");
            }
        }
        assert!(std::time::Instant::now()<deadline,"Overlay GPU check timed out: output length {}, brush {:?}, GUI {:?}, background {:?}, scene {:?}",
            pixels.output.len(),if pixels.output.len()==8192 {Some(pixel(&pixels.output,22,16))} else {None},
            if pixels.output.len()==8192 {Some(pixel(&pixels.output,16,16))} else {None},
            if pixels.output.len()==8192 {Some(pixel(&pixels.output,4,4))} else {None},
            if pixels.scene.len()==8192 {Some(pixel(&pixels.scene,22,16))} else {None});
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    }
}

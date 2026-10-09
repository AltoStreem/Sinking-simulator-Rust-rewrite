//! Source inputText cursor-follow arithmetic and clipped native field cameras.
//! Exact source child-window sizing, scrollbar and DPI behavior remain pending.
use crate::*;
use bevy::camera::Viewport;
#[derive(Component)]
pub(crate) struct FieldCamera(pub ShipUploadField);
pub(crate) fn index(field:ShipUploadField)->usize {match field {ShipUploadField::Name=>0,ShipUploadField::Description=>1,ShipUploadField::LayerName=>2}}
pub(crate) fn layer(field:ShipUploadField)->usize {12+index(field)}
pub(crate) fn spawn(mut commands:Commands) {
    for field in [ShipUploadField::Name,ShipUploadField::Description,ShipUploadField::LayerName] {
        commands.spawn((Camera2d,Camera {order:6+index(field) as isize,is_active:false,
            clear_color:ClearColorConfig::None,..default()},FieldCamera(field),
            Projection::Orthographic(OrthographicProjection::default_2d()),RenderLayers::layer(layer(field)),
            bevy::camera::CompositingSpace::Srgb));
    }
}
pub(crate) fn field_padding(layout:&ship_upload_layout::Layout)->Vec2 {
    layout.source_window.as_ref().map_or(Vec2::new(4.0,3.0),|s|s.gui.frame_padding*s.viewport.source_length_to_world(1.0))
}
pub(crate) fn source_pixel(layout:&ship_upload_layout::Layout)->f32 {
    layout.source_window.as_ref().map_or(1.0,|s|s.viewport.source_length_to_world(1.0))
}
pub(crate) fn follow(editor:&mut text_edit_state::TextEditState,inner:Vec2,multiline:bool,no_horizontal:bool) {follow_at_pixel(editor,inner,multiline,no_horizontal,1.0)}
fn follow_at_pixel(editor:&mut text_edit_state::TextEditState,inner:Vec2,multiline:bool,no_horizontal:bool,pixel:f32) {
    if !editor.cursor_follow {return;}
    let cursor=ui_text_visuals::offset_at_font(&editor.buffer.units,editor.cursor,editor.font_height());
    if no_horizontal {editor.scroll_x=0.0;}
    else {
        let increment=inner.x*0.25;
        if cursor.x<editor.scroll_x {editor.scroll_x=ui_numeric::floor((cursor.x-increment).max(0.0)/pixel)*pixel;}
        else if cursor.x-inner.x>=editor.scroll_x {editor.scroll_x=ui_numeric::floor((cursor.x-inner.x+increment)/pixel)*pixel;}
    }
    if multiline {
        if cursor.y-editor.font_height()<editor.scroll_y {editor.scroll_y=(cursor.y-editor.font_height()).max(0.0);}
        else if cursor.y-inner.y>=editor.scroll_y {editor.scroll_y=cursor.y-inner.y;}
    }
    editor.cursor_follow=false;
}
pub(crate) fn update(
    windows:Query<&Window>,upload:Res<ship_upload::SourceShipUpload>,layout:Res<ship_upload_layout::Layout>,
    mut ui:ResMut<ShipUploadUiState>,
    fields:Query<(&ShipUploadFieldControl,&Transform,&Sprite),(Without<Text2d>,Without<FieldCamera>)>,
    mut texts:Query<(&ship_upload_layout::Role,&mut Transform),(With<Text2d>,Without<ShipUploadFieldControl>,Without<FieldCamera>)>,
    mut cameras:Query<(&FieldCamera,&mut Camera,&mut Projection,&mut Transform),(Without<Text2d>,Without<ShipUploadFieldControl>)>,
) {
    for (_,mut camera,_,_) in &mut cameras {camera.is_active=false;}
    let Ok(window)=windows.single() else {return;};
    if !upload.window_open() || layout.is_collapsed() || window.physical_height()==0 {return;}
    let scale=window.physical_height() as f32/720.0;
    let width=window.physical_width() as f32/scale;
    let parent=layout.body_rect();
    for (control,transform,sprite) in &fields {
        let field=control.0;let center=transform.translation.truncate();let size=sprite.custom_size.unwrap_or(Vec2::ZERO);
        let active=ui.focus==Some(field);
        let bar=if field==ShipUploadField::Description {ui.description_scroll.scrollbar_width}else {0.0};
        let inner=Vec2::new(size.x-bar,size.y);
        let editor=&mut ui.editors[index(field)];
        if active && editor.error.is_none() {follow_at_pixel(editor,inner,field==ShipUploadField::Description,false,source_pixel(&layout));}
        let scroll=Vec2::new(if active {editor.scroll_x}else {0.0},if field==ShipUploadField::Description {editor.scroll_y}else {0.0});
        if field==ShipUploadField::Description {
            let units=if active {editor.buffer.units.clone()}else {upload.editor_units(field)};
            ui.description_scroll.record_contents(&units,inner.x,scroll.y,active);
        }
        for (role,mut transform) in &mut texts {
            let matching=matches!((field,role),(ShipUploadField::Name,ship_upload_layout::Role::Name)
                |(ShipUploadField::Description,ship_upload_layout::Role::Description)
                |(ShipUploadField::LayerName,ship_upload_layout::Role::LayerName));
            if matching {transform.translation.x-=scroll.x;transform.translation.y+=scroll.y;}
        }
        let field_rect=Rect::from_center_size(center-Vec2::new(bar*0.5,0.0),inner);
        let min=field_rect.min.max(parent.min-Vec2::new(0.0,layout.scroll));
        let max=field_rect.max.min(parent.max-Vec2::new(0.0,layout.scroll));
        if !max.cmpgt(min).all() {continue;}
        let clip=Rect::from_corners(min+layout.position+Vec2::new(0.0,layout.scroll),max+layout.position+Vec2::new(0.0,layout.scroll));
        let physical=UVec2::new(window.physical_width(),window.physical_height());
        let start=(Vec2::new((clip.min.x+width*0.5)*scale,(360.0-clip.max.y)*scale).round().max(Vec2::ZERO).as_uvec2()).min(physical);
        let end=(Vec2::new((clip.max.x+width*0.5)*scale,(360.0-clip.min.y)*scale).round().max(Vec2::ZERO).as_uvec2()).min(physical);
        if !end.cmpgt(start).all() {continue;}
        for (marker,mut camera,mut projection,mut transform) in &mut cameras {
            if marker.0!=field {continue;}
            camera.is_active=true;
            camera.viewport=Some(Viewport {physical_position:start,physical_size:end-start,..default()});
            transform.translation.x=(start.x+end.x) as f32*0.5/scale-width*0.5-layout.position.x;
            transform.translation.y=360.0-(start.y+end.y) as f32*0.5/scale-layout.position.y-layout.scroll;
            if let Projection::Orthographic(p)=&mut *projection {p.scaling_mode=ScalingMode::FixedVertical {viewport_height:(end.y-start.y) as f32/scale};}
        }
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn follow_uses_source_quarter_width_floor_and_vertical_baseline() {
        let mut s=text_edit_state::TextEditState::default();s.begin_focus(&vec![87;20],true);s.cursor_follow=true;
        let cursor=ui_text_visuals::offset(&s.buffer.units,s.cursor);
        follow(&mut s,Vec2::new(100.0,30.0),false,false);
        assert_eq!(s.scroll_x,(cursor.x-75.0).floor());assert!(!s.cursor_follow);
        s.cursor=0;s.cursor_follow=true;follow(&mut s,Vec2::new(100.0,30.0),false,false);assert_eq!(s.scroll_x,0.0);
        s.scroll_x=50.0;s.cursor_follow=true;follow(&mut s,Vec2::new(100.0,30.0),false,true);assert_eq!(s.scroll_x,0.0);
        s.begin_focus(&[97,10,97,10,97,10,97],false);s.cursor_follow=true;
        follow(&mut s,Vec2::new(100.0,30.0),true,false);assert_eq!(s.scroll_y,42.0);
        s.cursor=0;s.cursor_follow=true;follow(&mut s,Vec2::new(100.0,30.0),true,false);assert_eq!(s.scroll_y,0.0);
    }
    #[test] fn native_field_camera_clips_text_and_hides_when_editor_closes() {
        let mut app=App::new();app.init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ShipUploadUiState>().init_resource::<ship_upload_layout::Layout>()
            .add_systems(Startup,spawn).add_systems(Update,update);
        app.world_mut().spawn(Window::default());
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();
        {let mut ui=app.world_mut().resource_mut::<ShipUploadUiState>();ui.focus=Some(ShipUploadField::Name);
            ui.editors[0].begin_focus(&vec![87;20],true);ui.editors[0].cursor_follow=true;}
        app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Name),Transform::default(),
            Sprite::from_color(Color::WHITE,Vec2::new(100.0,30.0))));
        let text=app.world_mut().spawn((ship_upload_layout::Role::Name,Text2d::new("W".repeat(20)),Transform::from_xyz(-46.0,0.0,83.0))).id();
        app.update();
        let scroll=app.world().resource::<ShipUploadUiState>().editors[0].scroll_x;assert!(scroll>0.0);
        assert_eq!(app.world().get::<Transform>(text).unwrap().translation.x,-46.0-scroll);
        let world=app.world_mut();let mut query=world.query::<(&FieldCamera,&Camera)>();
        for (field,camera) in query.iter(world) {
            assert_eq!(camera.is_active,field.0==ShipUploadField::Name);
            if camera.is_active {assert_eq!(camera.viewport.as_ref().unwrap().physical_size,UVec2::new(100,30));}
        }
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(false);app.update();
        let world=app.world_mut();let mut query=world.query::<&Camera>();assert!(query.iter(world).all(|camera|!camera.is_active));
    }
    #[test] fn field_layers_are_disjoint_from_world_brush_and_existing_gui_views() {
        for field in [ShipUploadField::Name,ShipUploadField::Description,ShipUploadField::LayerName] {
            assert!(layer(field)>6);assert_ne!(layer(field),tools::brush_preview::OVERLAY_LAYER);
        }
        assert_eq!([layer(ShipUploadField::Name),layer(ShipUploadField::Description),layer(ShipUploadField::LayerName)],[12,13,14]);
    }
    #[derive(Resource,Default)] struct Pixels(Vec<u8>);
    #[test]
    #[ignore="Requires actual GPU; run explicitly with --ignored"]
    fn gpu_field_viewport_clips_large_primitive_and_excludes_brush_layer() {
        use bevy::{app::PluginsState,camera::RenderTarget,render::{gpu_readback::{Readback,ReadbackComplete},
            render_resource::{TextureFormat,TextureUsages}},window::{ExitCondition,WindowPlugin}};
        let mut app=App::new();
        app.add_plugins(DefaultPlugins.set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
            .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
            .disable::<bevy::winit::WinitPlugin>().disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
            .add_plugins(gui_gpu_blend::SourceGuiBlendPlugin)
            .init_resource::<ship_upload::SourceShipUpload>().init_resource::<ShipUploadUiState>()
            .init_resource::<ship_upload_layout::Layout>().init_resource::<Pixels>()
            .add_systems(Startup,spawn).add_systems(Update,update);
        while app.plugins_state()!=PluginsState::Ready {bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));}
        app.finish();app.cleanup();
        app.world_mut().spawn(Window {resolution:(64,64).into(),..default()});
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();
        let field=app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Name),Transform::default(),
            Sprite::from_color(Color::WHITE,Vec2::splat(180.0)))).id();
        let mut image=Image::new_target_texture(64,64,TextureFormat::Rgba8UnormSrgb,None);
        image.texture_descriptor.usage|=TextureUsages::COPY_SRC;
        let image=app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.world_mut().spawn((Camera2d,Camera {order:0,clear_color:ClearColorConfig::Custom(Color::WHITE),..default()},
            RenderTarget::Image(image.clone().into()),RenderLayers::layer(0),Msaa::Off));
        let primitive=app.world_mut().spawn((Sprite::from_color(Color::srgb(1.0,0.0,0.0),Vec2::splat(512.0)),
            RenderLayers::layer(layer(ShipUploadField::Name)))).id();
        app.world_mut().spawn((Sprite::from_color(Color::srgb(0.0,1.0,0.0),Vec2::splat(512.0)),
            Transform::from_xyz(0.0,0.0,10.0),RenderLayers::layer(tools::brush_preview::OVERLAY_LAYER)));
        // Active popup primitives must never leak through either editor-field camera.
        // The former popup layers 12/13 collide with Name/Description and fail this readback.
        for (index,layer) in [source_tools_popup::BACKGROUND_LAYER,source_tools_popup::ENTRIES_LAYER].into_iter().enumerate() {
            app.world_mut().spawn((Sprite::from_color(Color::srgb(0.0,0.0,1.0),Vec2::splat(512.0)),
                Transform::from_xyz(0.0,0.0,20.0+index as f32),RenderLayers::layer(layer)));
        }
        app.world_mut().spawn(Readback::texture(image.clone())).observe(|event:On<ReadbackComplete>,mut pixels:ResMut<Pixels>|{pixels.0=event.data.clone();});
        app.update();
        let world=app.world_mut();let mut query=world.query_filtered::<Entity,With<FieldCamera>>();let cameras:Vec<_>=query.iter(world).collect();
        for camera in cameras {app.world_mut().entity_mut(camera).insert((RenderTarget::Image(image.clone().into()),Msaa::Off));}
        let mut stable=0;let mut case=0;
        for _ in 0..240 {
            app.update();bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));
            let pixels=&app.world().resource::<Pixels>().0;
            if pixels.len()!=64*256 {continue;}
            let matched=(0..64).all(|y|(0..64).all(|x|{
                let p=&pixels[y*256+x*4..y*256+x*4+4];
                let right=if case==0 {40}else {39};
                let expected=if (24..right).contains(&x)&&(24..40).contains(&y) {[255,0,0,255]}else {[255,255,255,255]};
                p==expected
            }));
            stable=if matched {stable+1}else {0};if stable==3 {
                if case==1 {return;}case=1;stable=0;
                app.world_mut().get_mut::<ShipUploadFieldControl>(field).unwrap().0=ShipUploadField::Description;
                *app.world_mut().get_mut::<RenderLayers>(primitive).unwrap()=RenderLayers::layer(layer(ShipUploadField::Description));
                let text="a\n".repeat(16);let units=text.encode_utf16().collect::<Vec<_>>();
                app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_description(&text);
                let mut ui=app.world_mut().resource_mut::<ShipUploadUiState>();
                ui.description_scroll.begin(Vec2::splat(180.0),0.0);
                ui.description_scroll.record_contents(&units,180.0,0.0,false);
                ui.description_scroll.begin(Vec2::splat(180.0),0.0);
                assert_eq!(ui.description_scroll.scrollbar_width,14.0);
            }
        }
        panic!("GPU field viewport failed Name/Description clipping, scrollbar reservation, brush or popup isolation");
    }
}

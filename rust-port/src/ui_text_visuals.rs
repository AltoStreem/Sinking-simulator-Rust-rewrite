//! Source inputText caret/selection geometry with a live Bevy sprite adapter.
//! Full child scrollbars, DPI and DrawList line antialiasing remain pending.
use crate::*;
#[derive(Component)]
pub(crate) struct Overlay(pub usize);
pub(crate) fn spawn(mut commands:Commands) {
    for index in 0..257 {
        commands.spawn((Overlay(index),Sprite::from_color(Color::WHITE,Vec2::ONE),
            Transform::default(),RenderLayers::layer(5),Visibility::Hidden));
    }
}
pub(crate) fn blink_visible(animation:f32,enabled:bool)->bool {
    !enabled || animation<=0.0 || animation-1.2*(animation/1.2).floor()<=0.8
}
pub(crate) fn offset(text:&[u16],index:usize)->Vec2 {offset_at_font(text,index,18.0)}
pub(crate) fn offset_at_font(text:&[u16],index:usize,font:f32)->Vec2 {
    let index=index.min(text.len());let mut first=index;
    while first>0 && text[first-1]!=10 {first-=1;}
    let mut width=0.0;
    for &unit in &text[first..index] {if unit!=13 {width+=source_font_advances::advance(unit)*(font/18.0);}}
    let mut lines=0;let mut found=None;
    for (i,&unit) in text.iter().enumerate() {
        if unit==0 {break;}
        if unit==10 {lines+=1;if i+1>=index {found=Some(lines);break;}}
    }
    // Preserve source search's inclusive newline boundary, including index
    // immediately after LF, rather than recomputing a conventional line number.
    Vec2::new(width,found.unwrap_or(lines+1) as f32*font)
}
fn clipped(rect:Rect,clip:Rect)->Option<Rect> {
    let min=rect.min.max(clip.min);let max=rect.max.min(clip.max);
    max.cmpgt(min).all().then(||Rect::from_corners(min,max))
}
pub(crate) fn geometry(editor:&text_edit_state::TextEditState,multiline:bool,clip:Rect)->(Option<Rect>,Vec<Rect>) {geometry_at_font(editor,multiline,clip,18.0,1.0)}
pub(crate) fn geometry_at_font(editor:&text_edit_state::TextEditState,multiline:bool,clip:Rect,font:f32,pixel:f32)->(Option<Rect>,Vec<Rect>) {
    let text=&editor.buffer.units;
    let scroll=Vec2::new(editor.scroll_x,editor.scroll_y);
    let cursor=offset_at_font(text,editor.cursor,font)-scroll;
    let caret=blink_visible(editor.cursor_anim,true).then(||Rect::from_corners(
        cursor+Vec2::new(0.0,-font+0.5*pixel),cursor+Vec2::new(pixel,-1.5*pixel))).and_then(|r|clipped(r,clip));
    let mut selections=Vec::new();
    let mut index=editor.select_start.min(editor.select_end).min(text.len());
    let end=editor.select_start.max(editor.select_end).min(text.len());
    let mut position=offset_at_font(text,index,font)-scroll;
    while index<end {
        let mut width=0.0;
        while index<end {
            let unit=text[index];index+=1;
            if unit==10 {break;}
            if unit!=13 {width+=source_font_advances::advance(unit)*(font/18.0);}
        }
        if width<=0.0 {width=ui_numeric::floor(source_font_advances::advance(32)*(font/18.0)*0.5/pixel)*pixel;}
        let up=if multiline {0.0}else {-pixel};let down=if multiline {0.0}else {2.0*pixel};
        if let Some(rect)=clipped(Rect::from_corners(position+Vec2::new(0.0,up-font),
            position+Vec2::new(width,down)),clip) {selections.push(rect);}
        position.x= -scroll.x;position.y+=font;
    }
    (caret,selections)
}
pub(crate) fn update(
    time:Res<Time>,upload:Res<ship_upload::SourceShipUpload>,mut ui:ResMut<ShipUploadUiState>,
    layout:Option<Res<ship_upload_layout::Layout>>,
    fields:Query<(&ShipUploadFieldControl,&Transform,&Sprite),Without<Overlay>>,
    mut overlays:Query<(&Overlay,&mut Sprite,&mut Transform,&mut Visibility,&mut RenderLayers)>,
) {
    for (_,_,_,mut visibility,_) in &mut overlays {*visibility=Visibility::Hidden;}
    if !upload.window_open()||layout.as_ref().is_some_and(|layout|layout.is_collapsed()) {return;}
    let Some(field)=ui.focus else {return;};
    let Some((_,transform,sprite))=fields.iter().find(|(control,_,_)|control.0==field) else {return;};
    let index=match field {ShipUploadField::Name=>0,ShipUploadField::Description=>1,ShipUploadField::LayerName=>2};
    let editor=&mut ui.editors[index];if editor.error.is_some() {return;}
    editor.cursor_anim+=time.delta_secs();
    let center=transform.translation.truncate();let size=sprite.custom_size.unwrap_or(Vec2::ZERO);
    let multiline=field==ShipUploadField::Description;
    let padding=layout.as_ref().map_or(Vec2::new(4.0,3.0),|layout|ui_text_viewport::field_padding(layout));
    let pixel=layout.as_ref().map_or(1.0,|layout|ui_text_viewport::source_pixel(layout));
    let font=editor.font_height();
    let top=if multiline {center.y+size.y*0.5-padding.y}else {center.y+font*0.5};
    let origin=Vec2::new(center.x-size.x*0.5+padding.x,top);
    let clip=Rect::from_corners(Vec2::new(center.x-size.x*0.5-origin.x,top-center.y-size.y*0.5),
        Vec2::new(center.x+size.x*0.5-origin.x,top-center.y+size.y*0.5));
    let (caret,selection)=geometry_at_font(editor,multiline,clip,font,pixel);
    for (overlay,mut sprite,mut transform,mut visibility,mut layers) in &mut overlays {
        *layers=RenderLayers::layer(ui_text_viewport::layer(field));
        let rect=if overlay.0==0 {caret}else {selection.get(overlay.0-1).copied()};
        if let Some(rect)=rect {
            let position=rect.center();
            transform.translation=Vec3::new(origin.x+position.x,origin.y-position.y,
                if overlay.0==0 {84.0}else {82.5});
            sprite.custom_size=Some(rect.size());
            sprite.color=if overlay.0==0 {Color::srgba(230.0/255.0,230.0/255.0,230.0/255.0,1.0)}
                else {Color::srgba(0.0,0.0,1.0,89.0/255.0)};
            *visibility=Visibility::Inherited;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn editor(text:&[u16])->text_edit_state::TextEditState {
        let mut s=text_edit_state::TextEditState::default();s.begin_focus(text,false);s
    }
    #[test] fn blink_has_source_reset_delay_period_and_inclusive_visible_boundary() {
        assert!(blink_visible(-0.3,true));assert!(blink_visible(0.8,true));
        assert!(!blink_visible(0.81,true));assert!(blink_visible(1.2,true));
        assert!(blink_visible(0.9,false));
    }
    #[test] fn caret_offsets_preserve_inclusive_source_newline_boundary() {
        let mut s=editor(&[87,105,10,105,108]);s.cursor=3;
        let clip=Rect::from_corners(Vec2::splat(-100.0),Vec2::splat(100.0));
        assert_eq!(geometry(&s,true,clip).0.unwrap(),Rect::from_corners(Vec2::new(0.0,0.5),Vec2::new(1.0,16.5)));
        s.cursor=4;assert_eq!(geometry(&s,true,clip).0.unwrap().min.y,18.5);
    }
    #[test] fn selection_draws_empty_newline_width_and_clips_each_line() {
        let mut s=editor(&[87,105,10,10,105]);s.select_start=0;s.select_end=5;
        let clip=Rect::from_corners(Vec2::ZERO,Vec2::new(10.0,40.0));
        let (_,r)=geometry(&s,true,clip);assert_eq!(r.len(),3);
        assert_eq!(r[0].max.x,10.0);assert_eq!(r[1].width(),(source_font_advances::advance(32)*0.5).floor());
        assert_eq!(r[2].max.y,40.0);
    }
    #[test] fn caret_and_selection_share_scrolled_text_coordinates() {
        let mut s=editor(&[87,105,10,105,108]);s.cursor=4;s.select_start=3;s.select_end=5;
        let clip=Rect::from_corners(Vec2::splat(-100.0),Vec2::splat(100.0));
        let before=geometry(&s,true,clip);s.scroll_x=2.0;s.scroll_y=18.0;
        let after=geometry(&s,true,clip);
        assert_eq!(after.0.unwrap().min,before.0.unwrap().min-Vec2::new(2.0,18.0));
        assert_eq!(after.1[0].min,before.1[0].min-Vec2::new(2.0,18.0));
    }
    #[test] fn live_overlays_show_for_focused_editor_and_hide_after_deactivation() {
        let mut app=App::new();app.add_plugins(MinimalPlugins)
            .init_resource::<ship_upload::SourceShipUpload>().init_resource::<ShipUploadUiState>()
            .add_systems(Startup,spawn).add_systems(Update,update);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();
        {let mut ui=app.world_mut().resource_mut::<ShipUploadUiState>();ui.focus=Some(ShipUploadField::Name);
            ui.editors[0].begin_focus(&[97,98],true);ui.editors[0].select_all();}
        app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Name),Transform::default(),
            Sprite::from_color(Color::WHITE,Vec2::new(100.0,24.0))));
        app.update();
        let world=app.world_mut();let mut query=world.query::<(&Overlay,&Visibility)>();
        assert_eq!(query.iter(world).filter(|(_,v)|**v!=Visibility::Hidden).count(),2);
        app.world_mut().resource_mut::<ShipUploadUiState>().focus=None;app.update();
        let world=app.world_mut();let mut query=world.query::<(&Overlay,&Visibility)>();
        assert!(query.iter(world).all(|(_,v)|*v==Visibility::Hidden));
    }
}

#[cfg(test)] mod projection_tests {
    use super::*;
    #[test] fn source_pixel_caret_selection_and_empty_line_width_survive_projection() {
        let mut e=text_edit_state::TextEditState::default();e.begin_focus(&[87,10,10,105],false);
        e.cursor=3;e.select_start=0;e.select_end=4;
        let clip=Rect::from_corners(Vec2::splat(-1000.0),Vec2::splat(1000.0));
        let a=geometry_at_font(&e,true,clip,22.5,1.0);
        let b=geometry_at_font(&e,true,clip,11.25,0.5);
        let ca=a.0.unwrap();let cb=b.0.unwrap();
        assert_eq!(ca.width(),1.0);assert_eq!(cb.width(),0.5);
        assert_eq!(cb.min,ca.min*0.5);assert_eq!(cb.max,ca.max*0.5);
        assert_eq!(a.1.len(),b.1.len());
        for (a,b) in a.1.iter().zip(&b.1) {
            assert_eq!(b.min,a.min*0.5);assert_eq!(b.max,a.max*0.5);
        }
    }
}

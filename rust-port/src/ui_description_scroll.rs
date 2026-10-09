//! InputText multiline child-window sizing and deferred vertical scroll targets.
//! F18/default style adapter; rounded primitives and wheel locks pending.
use crate::*;
#[derive(Debug,Default)]
pub(crate) struct DescriptionScroll {
    pub window:ui_window_scroll::WindowScroll,
    pub scrollbar_width:f32,
    initialized:bool,
    source_style:Option<(source_ui_metrics::GuiMetrics,f32)>,
    pub dragging:Option<f32>,pub hovered:bool,pub held:bool,
}
impl DescriptionScroll {
    fn pixel(&self)->f32 {self.source_style.map_or(1.0,|(_,pixel)|pixel)}
    fn font(&self)->f32 {self.source_style.map_or(18.0,|(gui,pixel)|gui.font18*pixel)}
    pub(crate) fn configure(&mut self,gui:source_ui_metrics::GuiMetrics,pixel:f32) {
        let ratio=(gui.font18*pixel)/self.font();
        if ratio!=1.0 {
            self.window.content_size*=ratio;self.window.scroll*=ratio;
            for axis in 0..2 {if self.window.target[axis]<f32::MAX {self.window.target[axis]*=ratio;}}
        }
        self.source_style=Some((gui,pixel));
    }
    fn inset(&self,extent:f32)->f32 {ui_numeric::floor((extent/self.pixel()-2.0)*0.5).clamp(0.0,3.0)*self.pixel()}

    pub fn begin(&mut self,size:Vec2,current_scroll:f32) -> f32 {
        let pixel=self.pixel();
        let (padding,spacing,bar)=self.source_style.map_or((Vec2::new(4.0,3.0),Vec2::new(8.0,4.0),14.0),|(gui,pixel)|(gui.frame_padding*pixel,gui.item_spacing*pixel,gui.scrollbar_size*pixel));
        let w=&mut self.window;w.scroll.y=current_scroll;w.size_full=size;
        w.padding=padding;w.item_spacing=spacing;
        // Window.begin selects bars using previous content; a new window uses zero needed size.
        let needed=if self.initialized {w.content_size.y+w.padding.y*2.0}else {0.0};
        self.scrollbar_width=if needed>size.y {bar}else {0.0};
        w.scrollbar_sizes=Vec2::new(self.scrollbar_width,0.0);
        w.maximum=Vec2::new(ui_numeric::max(0.0,w.content_size.x+w.padding.x*2.0-(size.x-self.scrollbar_width)),
            ui_numeric::max(0.0,w.content_size.y+w.padding.y*2.0-size.y));
        // Window's scroll-target floors belong to original source pixels.
        let mut source=w.clone();source.scroll/=pixel;source.maximum/=pixel;source.size_full/=pixel;
        source.scrollbar_sizes/=pixel;source.content_size/=pixel;source.padding/=pixel;source.item_spacing/=pixel;
        source.decoration_up_height/=pixel;
        for axis in 0..2 {if source.target[axis]<f32::MAX {source.target[axis]/=pixel;}}
        source.begin_apply();w.scroll=source.scroll*pixel;w.target=source.target;
        self.initialized=true;w.scroll.y
    }
    pub fn record_contents(&mut self,units:&[u16],inner_width:f32,current_scroll:f32,active:bool) {
        // Inactive inputTextCalcTextLenAndLineCount checks the NEXT unit.
        // This ignores a leading LF; the terminated buffer still adds one line.
        let end=units.iter().position(|&unit|unit==0).unwrap_or(units.len());
        let first=if active {0}else {1.min(end)};
        let lines=1+units[first..end].iter().filter(|&&unit|unit==10).count();
        // inputText emits dummy(textSize + (0,fontSize)); itemSize's max excludes item spacing.
        let pixel=self.pixel();let source_font=self.source_style.map_or(18.0,|(gui,_)|gui.font18);
        self.window.content_size=Vec2::new(ui_numeric::floor(inner_width/pixel)*pixel,ui_numeric::floor((lines as f32+1.0)*source_font)*pixel);
        self.window.scroll.y=current_scroll;
    }
    pub fn key_scroll(&mut self,down:bool) {
        let y=if down {ui_numeric::min(self.window.scroll.y+self.font(),self.window.maximum.y)}
            else {ui_numeric::max(self.window.scroll.y-self.font(),0.0)};
        self.window.set_scroll_y(y);
    }
    pub fn alpha(&self)->f32 {((self.window.size_full.y-self.font())/(6.0*self.pixel())).clamp(0.0,1.0)}
    pub fn bounds(&self,center:Vec2,size:Vec2)->Rect {
        Rect::from_corners(center+Vec2::new(size.x*0.5-self.scrollbar_width,-size.y*0.5),center+size*0.5)
    }
    pub fn geometry(&self,center:Vec2,size:Vec2)->ui_scrollbar::Geometry {
        let pixel=self.pixel();
        let mut g=ui_scrollbar::Geometry::new((center.y+size.y*0.5)/pixel,(center.y-size.y*0.5)/pixel,size.y/pixel,
            (self.window.content_size.y+self.window.padding.y*2.0)/pixel,self.source_style.map_or(10.0,|(gui,_)|(10.0*gui.gui_scale).floor()));
        g.top*=pixel;g.size*=pixel;g.grab*=pixel;g.maximum*=pixel;g
    }
    pub fn pointer(&mut self,point:Vec2,down:bool,clicked:bool,center:Vec2,size:Vec2)->bool {
        if !down {self.dragging=None;self.held=false;}
        if self.scrollbar_width<=0.0 || self.alpha()<=0.0 {self.dragging=None;self.hovered=false;self.held=false;return false;}
        let frame=self.bounds(center,size);
        let inset=Vec2::new(self.inset(frame.width()),
            self.inset(frame.height()));
        let hit=Rect::from_corners(frame.min+inset,frame.max-inset);
        self.hovered=point.cmpge(hit.min).all()&&point.cmplt(hit.max).all();
        if clicked&&self.hovered {self.held=true;}
        let geometry=self.geometry(center,size);
        if clicked&&self.hovered&&self.alpha()>=1.0&&geometry.grab<geometry.size {
            let (scroll,offset)=geometry.activate(self.window.scroll.y,point.y);
            self.window.scroll.y=scroll;self.dragging=Some(offset);
        }else if down {if let Some(offset)=self.dragging {self.window.scroll.y=geometry.drag(point.y,offset);}}
        self.held
    }
}
#[derive(Component)] pub(crate) struct Bar(pub bool);
pub(crate) fn spawn(mut commands:Commands) {
    for thumb in [false,true] {commands.spawn((Bar(thumb),Sprite::from_color(Color::WHITE,Vec2::ONE),
        Transform::from_xyz(0.0,0.0,85.0),RenderLayers::layer(5),Visibility::Hidden));}
}
fn packed_color(rgba:[f32;4])->Color {
    let c=rgba.map(|v|((v*255.0+0.5) as u32).min(255) as f32/255.0);Color::srgba(c[0],c[1],c[2],c[3])
}
pub(crate) fn draw(upload:Res<ship_upload::SourceShipUpload>,ui:Res<ShipUploadUiState>,
    layout:Option<Res<ship_upload_layout::Layout>>,
    fields:Query<(&ShipUploadFieldControl,&Transform,&Sprite),Without<Bar>>,
    mut bars:Query<(&Bar,&mut Transform,&mut Sprite,&mut Visibility)>) {
    for (_,_,_,mut visibility) in &mut bars {*visibility=Visibility::Hidden;}
    let s=&ui.description_scroll;if !upload.window_open()||layout.as_ref().is_some_and(|layout|layout.is_collapsed())||s.scrollbar_width<=0.0||s.alpha()<=0.0 {return;}
    let Some((_,transform,sprite))=fields.iter().find(|(field,_,_)|field.0==ShipUploadField::Description) else {return;};
    let size=sprite.custom_size.unwrap_or(Vec2::ZERO);let center=transform.translation.truncate();let frame=s.bounds(center,size);
    let g=s.geometry(center,size);let inset_x=s.inset(frame.width());
    for (bar,mut transform,mut sprite,mut visibility) in &mut bars {
        let pos=if bar.0 {Vec2::new(frame.center().x,g.center(s.window.scroll.y))}else {frame.center()};
        transform.translation=pos.extend(if bar.0 {85.1}else {85.0});
        sprite.custom_size=Some(if bar.0 {Vec2::new(frame.width()-2.0*inset_x,g.grab)}else {frame.size()});
        sprite.color=if !bar.0 {packed_color([0.2,0.25,0.3,0.6])}else {
            let color=if s.held {[0.41,0.39,0.8,0.6*s.alpha()]}
                else if s.hovered {[0.4,0.4,0.8,0.4*s.alpha()]}else {[0.4,0.4,0.8,0.3*s.alpha()]};packed_color(color)
        };*visibility=Visibility::Inherited;
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn previous_content_drives_bar_width_and_targets_apply_on_next_begin() {
        let mut s=DescriptionScroll::default();let size=Vec2::new(536.0,144.0);
        assert_eq!(s.begin(size,0.0),0.0);assert_eq!(s.scrollbar_width,0.0);
        let units="a\na\na\na\na\na\na\na".encode_utf16().collect::<Vec<_>>();
        s.record_contents(&units,536.0,0.0,true);assert_eq!(s.scrollbar_width,0.0);
        assert_eq!(s.begin(size,0.0),0.0);assert_eq!(s.scrollbar_width,14.0);assert_eq!(s.window.maximum.y,24.0);
        s.key_scroll(true);assert_eq!(s.window.scroll.y,0.0);assert_eq!(s.begin(size,0.0),18.0);
        s.key_scroll(true);assert_eq!(s.begin(size,18.0),24.0);
        s.key_scroll(false);assert_eq!(s.begin(size,24.0),6.0);
    }
    #[test] fn original_jvm_inactive_line_count_snapshots_preserve_leading_lf_bug() {
        for (units,lines) in [(vec![],1),(vec![10],1),(vec![10,97],1),(vec![97,10],2),(vec![97,10,97],2),(vec![10,10,97],2)] {
            let mut s=DescriptionScroll::default();s.record_contents(&units,536.0,0.0,false);
            assert_eq!(s.window.content_size.y,(lines+1) as f32*18.0);
        }
        let mut s=DescriptionScroll::default();s.record_contents(&[10,97],536.0,0.0,true);
        assert_eq!(s.window.content_size.y,54.0);
    }
    #[test] fn track_hit_insets_fade_and_drag_capture_follow_source_rules() {
        let mut s=DescriptionScroll::default();let size=Vec2::new(536.0,144.0);let center=Vec2::new(0.0,203.0);
        s.begin(size,0.0);s.record_contents(&vec![10;8],536.0,0.0,true);s.begin(size,0.0);
        assert!(!s.pointer(Vec2::new(255.0,135.0),true,true,center,size)); // Outside inset X hit box.
        assert!(s.pointer(Vec2::new(261.0,135.0),true,true,center,size));assert!(s.window.scroll.y>0.0);
        assert!(s.pointer(Vec2::new(1000.0,272.0),true,false,center,size));assert_eq!(s.window.scroll.y,0.0);
        assert!(!s.pointer(Vec2::ZERO,false,false,center,size));
        s.window.size_full.y=21.0;let tiny=Vec2::new(536.0,21.0);
        assert_eq!(s.alpha(),0.5);assert!(s.pointer(Vec2::new(261.0,203.0),true,true,center,tiny));
        assert!(s.dragging.is_none()); // Source button still holds while interaction is faded out.
    }
    #[test] fn native_bar_draw_uses_source_bounds_quantized_colors_and_cleanup() {
        let mut app=App::new();app.init_resource::<ship_upload::SourceShipUpload>().init_resource::<ShipUploadUiState>()
            .add_systems(Startup,spawn).add_systems(Update,draw);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();
        app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Description),Transform::from_xyz(0.0,203.0,82.0),
            Sprite::from_color(Color::WHITE,Vec2::new(536.0,144.0))));
        {let mut ui=app.world_mut().resource_mut::<ShipUploadUiState>();let s=&mut ui.description_scroll;
            s.begin(Vec2::new(536.0,144.0),0.0);s.record_contents(&vec![10;8],536.0,0.0,true);s.begin(Vec2::new(536.0,144.0),0.0);}
        app.update();
        let world=app.world_mut();let mut q=world.query::<(&Bar,&Sprite,&Transform,&Visibility)>();
        for (bar,sprite,transform,visibility) in q.iter(world) {
            assert_ne!(*visibility,Visibility::Hidden);assert_eq!(transform.translation.x,261.0);
            if bar.0 {assert_eq!(sprite.custom_size.unwrap().x,8.0);assert_eq!(sprite.color,packed_color([0.4,0.4,0.8,0.3]));}
            else {assert_eq!(sprite.custom_size.unwrap(),Vec2::new(14.0,144.0));assert_eq!(sprite.color,packed_color([0.2,0.25,0.3,0.6]));}
        }
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(false);app.update();
        let world=app.world_mut();let mut q=world.query_filtered::<&Visibility,With<Bar>>();assert!(q.iter(world).all(|v|*v==Visibility::Hidden));
    }
}

#[cfg(test)] mod projection_tests {
    use super::*;
    #[test] fn child_scroll_floors_in_source_pixels_and_rescales_pending_targets() {
        let gui=source_ui_metrics::GuiMetrics::new(1.25,[1440,900],[1440,900]);
        let mut full=DescriptionScroll::default();full.configure(gui,1.0);
        let mut half=DescriptionScroll::default();half.configure(gui,0.5);
        let units=vec![10;12];
        for (s,pixel) in [(&mut full,1.0),(&mut half,0.5)] {
            s.begin(Vec2::new(400.0,100.0)*pixel,0.0);
            s.record_contents(&units,390.75*pixel,0.0,true);
            s.begin(Vec2::new(400.0,100.0)*pixel,0.0);
            assert_eq!(s.scrollbar_width,17.0*pixel);
            assert_eq!(s.window.content_size.x,390.0*pixel);
            s.key_scroll(true);
        }
        let a=full.begin(Vec2::new(400.0,100.0),0.0);
        let b=half.begin(Vec2::new(200.0,50.0),0.0);
        assert_eq!(b,a*0.5);
        full.key_scroll(true);let target=full.window.target.y;
        full.configure(gui,0.5);
        assert_eq!(full.window.target.y,target*0.5);
        assert_eq!(full.window.content_size,half.window.content_size);
    }
}

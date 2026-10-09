//! Live lower-left Tools geometry in original GLFW display pixels.
//! One set of rectangles drives rendering, hover, clicks and camera capture.
use crate::*;
use crate::source_ui_metrics::{GuiMetrics,UiViewport,SourceUiMetrics};
use bevy::sprite::Anchor;
use bevy::text::LineHeight;
#[derive(Component,Clone,Copy)]
pub(crate) enum Role {Button(Tool),Image(Tool),Combo,ComboValue,ComboLabel,Arrow,
    Size,SizeValue,SizeLabel,Hint,Popup,Option(usize),OptionLabel(usize)}
#[derive(Clone,Copy,Debug)]
pub(crate) struct Layout {
    pub gui:GuiMetrics,pub viewport:UiViewport,pub window:Rect,pub popup_scroll:f32,pub layer_count:usize,
    pub combo:Rect,pub arrow:Rect,pub slider:Rect,pub buttons:[Rect;4],pub images:[Rect;4],
    pub source_combo:Rect,pub combo_value:Vec2,pub combo_label:Vec2,pub size_label:Vec2,pub hint:Vec2,
}
fn advance(text:&str,height:f32)->f32 {
    let size=crate::source_font_text::Font {font_size:18.0,advances:crate::source_font_text::F18}
        .measure(height,f32::MAX,0.0,&text.encode_utf16().collect::<Vec<_>>(),-1).expect("ASCII source tool label").size.x;
    crate::ui_numeric::floor(size+0.95)
}
impl Layout {
    pub(crate) fn new(gui:GuiMetrics,viewport:UiViewport)->Self {
        let field=gui.default_item_width;
        let label_width=advance("Show Layer",gui.font18).max(advance("Tool Size",gui.font18));
        let content_width=(field+gui.inner_spacing.x+label_width).max(gui.tool_row_width(4))
            .max(advance("You can hide the tools in the graphics settings",gui.font12));
        let size=Vec2::new((content_width+gui.window_padding.x*2.0).ceil(),gui.tools_window_height());
        // SetNextWindowPos((padding, int(displayY)-padding), Always, pivot=(0,1)).
        // Original begin floors window position after applying pivot.
        let origin=Vec2::new(gui.layout_padding,viewport.source_size.y as i32 as f32-gui.layout_padding-size.y).floor();
        let cursor=origin+gui.window_padding;
        let source_combo=Rect::from_corners(cursor,cursor+Vec2::new(field,gui.frame_height));
        let image_y=(cursor.y+gui.frame_height+gui.item_spacing.y).floor();
        let buttons=std::array::from_fn(|index| {
            let p=Vec2::new(cursor.x+index as f32*(gui.tool_button_side+gui.item_spacing.x),image_y);
            viewport.source_rect_to_world(Rect::from_corners(p,p+Vec2::splat(gui.tool_button_side)))
        });
        let images=std::array::from_fn(|index| {
            let p=Vec2::new(cursor.x+index as f32*(gui.tool_button_side+gui.item_spacing.x),image_y)
                +Vec2::splat(gui.tool_button_padding as f32);
            viewport.source_rect_to_world(Rect::from_corners(p,p+Vec2::splat(gui.tool_image_side)))
        });
        let slider_y=(image_y+gui.tool_button_side+gui.item_spacing.y).floor();
        let source_slider=Rect::from_corners(Vec2::new(cursor.x,slider_y),Vec2::new(cursor.x+field,slider_y+gui.frame_height));
        let arrow=Rect::from_corners(Vec2::new(source_combo.max.x-gui.frame_height,source_combo.min.y),source_combo.max);
        Self {gui,viewport,popup_scroll:0.0,layer_count:1,window:viewport.source_rect_to_world(Rect::from_corners(origin,origin+size)),
            combo:viewport.source_rect_to_world(source_combo),arrow:viewport.source_rect_to_world(arrow),
            slider:viewport.source_rect_to_world(source_slider),buttons,images,source_combo,
            combo_value:viewport.source_to_world(cursor+gui.frame_padding),
            combo_label:viewport.source_to_world(Vec2::new(source_combo.max.x+gui.inner_spacing.x,cursor.y+gui.frame_padding.y)),
            size_label:viewport.source_to_world(Vec2::new(source_slider.max.x+gui.inner_spacing.x,slider_y+gui.frame_padding.y)),
            hint:viewport.source_to_world(Vec2::new(cursor.x,(source_slider.max.y+gui.item_spacing.y).floor()))}
    }
    pub(crate) fn index(tool:Tool)->Option<usize> {match tool {Tool::Break=>Some(0),Tool::Flood=>Some(1),Tool::Dry=>Some(2),Tool::Move=>Some(3),Tool::None=>None}}
    pub(crate) fn font(&self,hint:bool)->f32 {self.viewport.source_length_to_world(if hint {self.gui.font12}else {self.gui.font18})}
    pub(crate) fn source_popup(&self,count:usize)->Rect {
        let content_height=count.max(1) as f32*(self.gui.font18+self.gui.item_spacing.y).floor()-self.gui.item_spacing.y+self.gui.window_padding.y*2.0;
        let maximum=(self.gui.font18+self.gui.item_spacing.y)*8.0-self.gui.item_spacing.y+self.gui.window_padding.y*2.0;
        let height=content_height.ceil().min(maximum);
        let size=Vec2::new(self.source_combo.width(),height);
        let pad=(Vec2::splat(3.0)*self.gui.gui_scale).floor();
        let outer=Rect::from_corners(pad,self.viewport.source_size-pad);
        // Original combo-box policy: Down, Right, Left, Up; default prefer Down.
        let candidates=[Vec2::new(self.source_combo.min.x,self.source_combo.max.y),
            Vec2::new(self.source_combo.min.x,self.source_combo.min.y-height),
            Vec2::new(self.source_combo.max.x-size.x,self.source_combo.max.y),
            Vec2::new(self.source_combo.max.x-size.x,self.source_combo.min.y-height)];
        let min=candidates.into_iter().find(|p|p.x>=outer.min.x&&p.y>=outer.min.y&&( *p+size).cmple(outer.max).all())
            .unwrap_or(self.source_combo.min.max(outer.min).min((outer.max-size).max(outer.min)));
        Rect::from_corners(min.floor(),min.floor()+size)
    }
    pub(crate) fn popup(&self,count:usize)->Rect {self.viewport.source_rect_to_world(self.source_popup(count))}
    pub(crate) fn scroll_max(&self,count:usize)->f32 {(count as f32*(self.gui.font18+self.gui.item_spacing.y).floor()-self.gui.item_spacing.y+self.gui.window_padding.y*2.0-self.source_popup(count).height()).max(0.0)}
    pub(crate) fn option(&self,index:usize,count:usize)->Rect {
        // beginCombo overrides horizontal popup padding with FramePadding.x.
        let popup=self.source_popup(count);
        let p=popup.min+Vec2::new(self.gui.frame_padding.x,self.gui.window_padding.y+index as f32*(self.gui.font18+self.gui.item_spacing.y).floor()-self.popup_scroll);
        let half=(self.gui.item_spacing*0.5).floor();
        let width=popup.width()-self.gui.frame_padding.x*2.0-if count>8 {self.gui.scrollbar_size}else {0.0};
        self.viewport.source_rect_to_world(Rect::from_corners(p-half,p+Vec2::new(width,self.gui.font18)+self.gui.item_spacing-half))
    }
    pub(crate) fn option_label(&self,index:usize,count:usize)->Vec2 {
        let popup=self.source_popup(count);
        self.viewport.source_to_world(popup.min+Vec2::new(self.gui.frame_padding.x,
            self.gui.window_padding.y+index as f32*(self.gui.font18+self.gui.item_spacing.y).floor()-self.popup_scroll))
    }
    pub(crate) fn captures(&self,point:Vec2,popup_open:bool,count:usize)->bool {
        self.window.contains(point)||(popup_open&&self.popup(count).contains(point))
    }
}
/// Before all input adapters; SourceUiMetrics retains constructor style across resizing.
pub(crate) fn update(metrics:Res<SourceUiMetrics>,mut simulation:ResMut<Simulation>,catalog:Res<ShipCatalog>,
    preview:Option<Res<ship_upload_preview::ActivePreview>>,retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>) {
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    simulation.source_tools=metrics.gui.zip(metrics.viewport).map(|(gui,viewport)| {
        let mut layout=Layout::new(gui,viewport);
        simulation.layer_popup_scroll=simulation.layer_popup_scroll.clamp(0.0,layout.scroll_max(count));
        layout.popup_scroll=simulation.layer_popup_scroll;layout.layer_count=count;layout
    });
}
/// After dropdown/legacy position sync, before text layout and transform extraction.
pub(crate) fn sync(mut commands:Commands,simulation:Res<Simulation>,
    catalog:Res<ShipCatalog>,preview:Option<Res<ship_upload_preview::ActivePreview>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut widgets:Query<(Entity,&Role,&mut Transform,Option<&mut Sprite>,Option<&mut TextFont>,
        Option<&mut crate::ui_font::SourceSize>,Option<&mut LineHeight>,Option<&mut Anchor>,Option<&mut TextLayout>)>) {
    let Some(layout)=simulation.source_tools else {return};
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    for (entity,role,mut transform,sprite,font,source_size,line_height,anchor,text_layout) in &mut widgets {
        let has_sprite=sprite.is_some();
        let (position,rect,text_anchor)=match *role {
            Role::Button(tool)=>{let r=layout.buttons[Layout::index(tool).unwrap()];(r.center(),Some(r),None)},
            Role::Image(tool)=>{let r=layout.images[Layout::index(tool).unwrap()];(r.center(),Some(r),None)},
            Role::Combo=>(layout.combo.center(),Some(layout.combo),None),
            Role::Arrow=>{
                let source_arrow=Rect::from_corners(layout.viewport.world_to_source(layout.arrow.min),layout.viewport.world_to_source(layout.arrow.max));
                let centre=if has_sprite {layout.arrow.center()}else {layout.viewport.source_to_world(source_arrow.min+layout.gui.frame_padding+Vec2::splat(layout.gui.font18*0.5))};
                (centre,Some(layout.arrow),None)
            },
            Role::ComboValue=>(layout.combo_value,None,Some(Anchor::TOP_LEFT)),
            Role::ComboLabel=>(layout.combo_label,None,Some(Anchor::TOP_LEFT)),
            Role::Size=>(layout.slider.center(),Some(layout.slider),None),
            Role::SizeValue=>(layout.slider.center(),None,Some(Anchor::CENTER)),
            Role::SizeLabel=>(layout.size_label,None,Some(Anchor::TOP_LEFT)),
            Role::Hint=>(layout.hint,None,Some(Anchor::TOP_LEFT)),
            Role::Popup=>{let r=layout.popup(count);(r.center(),Some(r),None)},
            Role::Option(index)=>{let r=layout.option(index,count);(r.center(),Some(r),None)},
            Role::OptionLabel(index)=>(layout.option_label(index,count),None,Some(Anchor::TOP_LEFT)),
        };
        transform.translation.x=position.x;transform.translation.y=position.y;
        if let (Some(mut sprite),Some(rect))=(sprite,rect) {sprite.custom_size=Some(rect.size());}
        if let Some(wanted)=text_anchor {
            if let Some(mut anchor)=anchor {if *anchor!=wanted {*anchor=wanted;}}
            else {commands.entity(entity).insert(wanted);}
        }
        if let Some(mut font)=font {
            let height=layout.font(matches!(role,Role::Hint));
            let wanted=FontSize::Px(crate::ui_font::em_size(height));
            if font.font_size!=wanted {font.font_size=wanted;if let Some(mut layout)=text_layout {layout.set_changed();}}
            if let Some(mut source)=source_size {source.0=FontSize::Px(height);}
            else {commands.entity(entity).insert(crate::ui_font::SourceSize(FontSize::Px(height)));}
            if let Some(mut line)=line_height {*line=LineHeight::Px(height);}
            else {commands.entity(entity).insert(LineHeight::Px(height));}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_panel_pixel_sizes_do_not_grow_when_window_height_doubles() {
        let gui=GuiMetrics::new(1.25,[1001,701],[1001,701]);
        let make=|w,h|Layout::new(gui,UiViewport::new(Vec2::new(w,h),Vec2::new(w/1.25,h/1.25),UVec2::new(w as u32,h as u32)).unwrap());
        let a=make(1001.0,701.0);let b=make(2002.0,1402.0);
        for layout in [a,b] {
            let scale=layout.viewport.source_size.y/720.0;
            assert!((layout.buttons[0].width()*scale-84.5).abs()<0.0001);
            assert!((layout.images[0].width()*scale-80.5).abs()<0.0001);
            assert!((layout.combo.width()*scale-360.0).abs()<0.0001);
            assert!((layout.font(false)*scale-22.5).abs()<0.00001);
            let next=layout.viewport.world_to_source(layout.buttons[1].min).x;
            let first=layout.viewport.world_to_source(layout.buttons[0].max).x;
            assert!((next-first-10.0).abs()<0.0001);
        }
        assert!(a.buttons[0].width()>b.buttons[0].width());
    }
    #[test]
    fn source_window_capture_includes_popup_and_excludes_old_oversized_area() {
        let gui=GuiMetrics::new(1.0,[1280,720],[1280,720]);
        let layout=Layout::new(gui,UiViewport::new(Vec2::new(1280.0,720.0),Vec2::new(1280.0,720.0),UVec2::new(1280,720)).unwrap());
        for r in layout.buttons {assert!(layout.captures(r.center(),false,2));}
        assert!(layout.captures(layout.option(0,2).center(),true,2));
        assert_eq!(layout.slider.width(),288.0);
        assert_eq!(layout.window.height(),160.0);
        assert_eq!(layout.viewport.world_to_source(layout.hint).x,18.0);
        assert_eq!(layout.viewport.world_to_source(layout.window.max).x,layout.window.width()+10.0);
    }
    #[test]
    fn live_source_roles_clicks_and_camera_capture_remain_aligned_after_resize() {
        let mut app=App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(ShipCatalog(Vec::new(),Vec::new()))
            .add_systems(Update,select_tool_from_panel)
            .add_systems(PostUpdate,sync);
        let window=app.world_mut().spawn(Window::default()).id();
        let icon=app.world_mut().spawn((Role::Image(Tool::Dry),Transform::default(),
            Sprite::from_color(Color::WHITE,Vec2::ONE))).id();
        let label=app.world_mut().spawn((Role::ComboValue,Transform::default(),
            Text2d::new("Default"),TextFont::default(),TextLayout::default(),Anchor::CENTER)).id();
        let gui=GuiMetrics::new(1.25,[1001,701],[1001,701]);
        for (width,height) in [(1001,701),(2002,1402)] {
            let viewport=UiViewport::new(Vec2::new(width as f32,height as f32),
                Vec2::new(width as f32/1.25,height as f32/1.25),UVec2::new(width,height)).unwrap();
            let layout=Layout::new(gui,viewport);
            {let mut simulation=app.world_mut().resource_mut::<Simulation>();
                simulation.source_tools=Some(layout);simulation.tool=Tool::None;simulation.toolbox_collapsed=true;}
            {let mut w=app.world_mut().get_mut::<Window>(window).unwrap();
                w.resolution.set_physical_resolution(width,height);w.resolution.set_scale_factor(1.25);
                w.set_cursor_position(Some(viewport.source_to_logical(viewport.world_to_source(layout.buttons[2].center()))));}
            {let mut mouse=app.world_mut().resource_mut::<ButtonInput<MouseButton>>();*mouse=ButtonInput::default();mouse.press(MouseButton::Left);}
            app.update();
            let simulation=app.world().resource::<Simulation>();
            assert!(simulation.tool==Tool::Dry,"actual live click must select Dry at {width}x{height}");
            assert!(camera_control::blocked(simulation,layout.buttons[2].center()));
            let transform=app.world().get::<Transform>(icon).unwrap();
            assert!((transform.translation.truncate()-layout.images[2].center()).abs().max_element()<0.0001);
            assert_eq!(app.world().get::<Sprite>(icon).unwrap().custom_size,Some(layout.images[2].size()));
            assert_eq!(app.world().get::<Anchor>(label),Some(&Anchor::TOP_LEFT));
            assert_eq!(app.world().get::<crate::ui_font::SourceSize>(label).unwrap().0,FontSize::Px(layout.font(false)));
            assert_eq!(app.world().get::<TextFont>(label).unwrap().font_size,FontSize::Px(crate::ui_font::em_size(layout.font(false))));
        }
    }
    #[test]
    fn original_cpu_imgui_settled_tools_rectangles_match() {
        let capture:serde_json::Value=serde_json::from_str(include_str!("../tools/fixtures/source-tools-layout-capture.json")).unwrap();
        let gui=GuiMetrics::new(1.25,[1001,701],[1001,701]);
        let layout=Layout::new(gui,UiViewport::new(Vec2::new(1001.0,701.0),Vec2::new(800.8,560.8),UVec2::new(1001,701)).unwrap());
        let to_source=|r:Rect|Rect::from_corners(layout.viewport.world_to_source(r.min),layout.viewport.world_to_source(r.max));
        let window=to_source(layout.window);
        assert!((window.min-Vec2::new(12.0,498.0)).abs().max_element()<0.0001);
        assert!((window.size()-Vec2::new(482.0,190.0)).abs().max_element()<0.0001);
        let mut checked=0;
        for frame in capture["frames"].as_array().unwrap().iter().filter(|frame|frame["frame"].as_u64()==Some(2)) {
            let item=frame["item"].as_str().unwrap();
            let actual=match item {
                "combo"=>layout.source_combo,
                "drag"=>to_source(layout.slider),
                "hint"=>{let min=layout.viewport.world_to_source(layout.hint);Rect::from_corners(min,min+Vec2::new(264.0,15.0))},
                _ if item.starts_with("image")=>to_source(layout.buttons[item[5..].parse::<usize>().unwrap()]),
                _=>continue,
            };
            let v=|key:&str|Vec2::new(frame[key][0].as_f64().unwrap() as f32,frame[key][1].as_f64().unwrap() as f32);
            let mut expected=Rect::from_corners(v("itemMin"),v("itemMax"));
            // Original itemRect for combo/drag includes the right-hand label.
            if matches!(item,"combo"|"drag") {expected.max.x=expected.min.x+gui.default_item_width;}
            assert!((actual.min-expected.min).abs().max_element()<0.0002,"{item} min {actual:?} {expected:?}");
            assert!((actual.max-expected.max).abs().max_element()<0.0002,"{item} max {actual:?} {expected:?}");
            checked+=1;
        }
        assert_eq!(checked,7,"All settled tool item rectangles must be covered");
    }

}

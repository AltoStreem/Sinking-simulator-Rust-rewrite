//! GUI constructor, bundled Style.scaleAllSizes and Toolbox Tools dimensions.
//! Windows GLFW uses physical client pixels; Winit logical pixels are an adapter
//! coordinate system, not the original GUI's framebuffer/window scale.
use bevy::prelude::*;

#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) struct GuiMetrics {
    pub fontscale:f32,
    pub fbscale:f32,
    pub gui_scale:f32,
    pub font_global_scale:f32,
    pub font18_atlas_size:f32,
    pub font12_atlas_size:f32,
    pub font18:f32,
    pub font12:f32,
    pub window_padding:Vec2,
    pub frame_padding:Vec2,
    pub item_spacing:Vec2,
    pub inner_spacing:Vec2,
    pub window_rounding:f32,
    pub window_border:f32,
    pub tab_rounding:f32,
    pub scrollbar_size:f32,
    pub scrollbar_rounding:f32,
    pub frame_height:f32,
    pub default_item_width:f32,
    pub layout_padding:f32,
    pub tool_image_side:f32,
    pub tool_button_padding:i32,
    pub tool_button_side:f32,
}
impl GuiMetrics {
    /// Pure original float rules, including x-only fbscale and JVM-cast edge
    /// cases. Validation/defer logic belongs to the native initialization adapter.
    pub(crate) fn new(fontscale:f32,source_screen:[i32;2],framebuffer:[i32;2])->Self {
        let fbscale=framebuffer[0] as f32/source_screen[0] as f32;
        let gui_scale=fontscale/fbscale;
        let font_global_scale=1.0/fbscale;
        let font18_atlas_size=18.0*fontscale;
        let font12_atlas_size=12.0*fontscale;
        let font18=font18_atlas_size*font_global_scale;
        let font12=font12_atlas_size*font_global_scale;
        let window_padding=(Vec2::splat(8.0)*gui_scale).floor();
        let frame_padding=(Vec2::new(4.0,3.0)*gui_scale).floor();
        let item_spacing=(Vec2::new(8.0,4.0)*gui_scale).floor();
        let inner_spacing=(Vec2::splat(4.0)*gui_scale).floor();
        let frame_height=font18+frame_padding.y*2.0;
        let tool_image_side=frame_height*3.0-4.0*gui_scale;
        let tool_button_padding=(2.0*gui_scale) as i32;
        Self {fontscale,fbscale,gui_scale,font_global_scale,font18_atlas_size,font12_atlas_size,font18,font12,
            window_padding,frame_padding,item_spacing,inner_spacing,
            window_rounding:(7.0*gui_scale).floor(),window_border:1.0,
            tab_rounding:(4.0*gui_scale).floor(),scrollbar_size:(14.0*gui_scale).floor(),
            scrollbar_rounding:(9.0*gui_scale).floor(),frame_height,
            default_item_width:(font18*16.0).floor(),layout_padding:10.0*gui_scale,
            tool_image_side,tool_button_padding,tool_button_side:tool_image_side+tool_button_padding as f32*2.0}
    }
    pub(crate) fn tool_row_width(&self,count:usize)->f32 {
        self.tool_button_side*count as f32+self.item_spacing.x*count.saturating_sub(1) as f32
    }
    pub(crate) fn tools_content_height(&self)->f32 {
        // Original itemSize floors each cursor advance, and cursorMaxPos excludes the final spacing.
        2.0*(self.frame_height+self.item_spacing.y).floor() + (self.tool_button_side+self.item_spacing.y).floor() + (self.font12+self.item_spacing.y).floor() - self.item_spacing.y
    }
    pub(crate) fn tools_window_height(&self)->f32 {(self.tools_content_height()+self.window_padding.y*2.0).ceil()}
}

/// Conversion at the existing 720-unit rendering boundary. Source geometry is
/// kept in actual original display pixels, never scaled by the window's height.
#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) struct UiViewport {
    pub source_size:Vec2,
    pub winit_logical_size:Vec2,
    pub framebuffer_size:UVec2,
}
impl UiViewport {
    pub(crate) fn new(source_size:Vec2,winit_logical_size:Vec2,framebuffer_size:UVec2)->Option<Self> {
        (source_size.min_element()>0.0&&source_size.is_finite()&&winit_logical_size.min_element()>0.0
            &&winit_logical_size.is_finite()&&framebuffer_size.min_element()>0)
            .then_some(Self {source_size,winit_logical_size,framebuffer_size})
    }
    pub(crate) fn source_to_world(&self,point:Vec2)->Vec2 {
        Vec2::new(point.x-self.source_size.x*0.5,self.source_size.y*0.5-point.y)*(720.0/self.source_size.y)
    }
    pub(crate) fn world_to_source(&self,point:Vec2)->Vec2 {
        let p=point*(self.source_size.y/720.0);
        Vec2::new(p.x+self.source_size.x*0.5,self.source_size.y*0.5-p.y)
    }
    pub(crate) fn logical_to_source(&self,point:Vec2)->Vec2 {point*self.source_size/self.winit_logical_size}
    pub(crate) fn source_to_logical(&self,point:Vec2)->Vec2 {point*self.winit_logical_size/self.source_size}
    pub(crate) fn source_length_to_world(&self,length:f32)->f32 {length*(720.0/self.source_size.y)}
    pub(crate) fn source_rect_to_world(&self,rect:Rect)->Rect {
        Rect::from_corners(self.source_to_world(rect.min),self.source_to_world(rect.max))
    }
}

/// Frozen GUI-constructor style plus current display/cursor projection. Moving
/// or resizing the window does not rerun GUI's styleAllSizes/font construction.
#[derive(Resource,Default,Debug)]
pub(crate) struct SourceUiMetrics {
    pub gui:Option<GuiMetrics>,
    pub viewport:Option<UiViewport>,
    captured_window:Option<Entity>,
}
impl SourceUiMetrics {
    pub(crate) fn update(&mut self,entity:Entity,fontscale:f32,source_screen:[i32;2],framebuffer:[i32;2],logical:Vec2) {
        self.viewport=UiViewport::new(Vec2::new(source_screen[0] as f32,source_screen[1] as f32),logical,
            UVec2::new(framebuffer[0].max(0) as u32,framebuffer[1].max(0) as u32));
        if self.viewport.is_some()&&(self.gui.is_none()||self.captured_window!=Some(entity)) {
            self.gui=Some(GuiMetrics::new(fontscale,source_screen,framebuffer));
            self.captured_window=Some(entity);
        }
    }
}

/// Init this resource, then schedule before all live UI layout/input systems.
/// Original GLFW constructor chooses the PRIMARY monitor, not whichever monitor
/// currently contains the window. Its content scale is captured only once.
pub(crate) fn capture(windows:Query<(Entity,&Window),With<bevy::window::PrimaryWindow>>,
    native:Option<NonSend<bevy::winit::WinitWindows>>,mut metrics:ResMut<SourceUiMetrics>) {
    let Ok((entity,window))=windows.single() else {return};
    let framebuffer=[window.physical_width() as i32,window.physical_height() as i32];
    let logical=Vec2::new(window.width(),window.height());
    // Verified against the bundled Windows GLFW DLL: glfwGetWindowSize and
    // glfwGetFramebufferSize report identical physical client sizes at DPI1.25.
    let screen=if cfg!(target_os="windows") {framebuffer}else {[window.width() as i32,window.height() as i32]};
    let fontscale=if let Some(gui)=metrics.gui.filter(|_|metrics.captured_window==Some(entity)) {gui.fontscale}
        else if let Some(native)=native.as_ref() {
            let Some(scale)=native.get_window(entity).and_then(|window|window.primary_monitor()).map(|monitor|monitor.scale_factor() as f32) else {return};
            scale
        } else {
            // Headless fixtures have no monitor handle; use their explicit base
            // scale. Native presentation always takes the primary-monitor path.
            window.resolution.base_scale_factor()
        };
    metrics.update(entity,fontscale,screen,framebuffer,logical);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_windows_glfw_probe_yields_source_125_percent_metrics() {
        let m=GuiMetrics::new(1.25,[1001,701],[1001,701]);
        assert_eq!((m.fbscale,m.gui_scale,m.font18,m.font12),(1.0,1.25,22.5,15.0));
        assert_eq!(m.window_padding,Vec2::splat(10.0));assert_eq!(m.frame_padding,Vec2::new(5.0,3.0));
        assert_eq!(m.item_spacing,Vec2::new(10.0,5.0));assert_eq!(m.inner_spacing,Vec2::splat(5.0));
        assert_eq!((m.frame_height,m.default_item_width,m.tool_image_side,m.tool_button_padding,m.tool_button_side),(28.5,360.0,80.5,2,84.5));
        assert_eq!(m.tool_row_width(4),368.0);assert_eq!(m.tools_content_height(),170.0);assert_eq!(m.tools_window_height(),190.0);
        assert_eq!(m.window_border,1.0,"Style.scaleAllSizes does not scale borders");
    }
    #[test]
    fn source_floors_sizes_uses_x_framebuffer_ratio_and_does_not_round_image_padding() {
        let m=GuiMetrics::new(1.75,[800,600],[1200,1200]);
        assert_eq!(m.fbscale,1.5);assert_eq!(m.gui_scale,1.75/1.5);
        assert_eq!(m.frame_padding,(Vec2::new(4.0,3.0)*(1.75/1.5)).floor());
        assert_eq!(m.font18,(18.0*1.75)*(1.0/1.5));
        assert_eq!(m.tool_button_padding,2);
        let zero=GuiMetrics::new(1.0,[800,600],[0,600]);
        assert!(zero.gui_scale.is_infinite());assert_eq!(zero.tool_button_padding,i32::MAX);
    }
    #[test]
    fn resizing_changes_adapter_projection_without_changing_source_pixel_extents() {
        let mut m=SourceUiMetrics::default();let entity=Entity::from_bits(1);
        m.update(entity,1.25,[1000,700],[1000,700],Vec2::new(800.0,560.0));let original=m.gui.unwrap();
        let v=m.viewport.unwrap();let p=Vec2::new(18.0,680.0);
        assert!((v.world_to_source(v.source_to_world(p))-p).abs().max_element()<0.0001);
        assert_eq!(v.logical_to_source(Vec2::new(14.4,544.0)),p);
        m.update(entity,2.0,[2000,1400],[2000,1400],Vec2::new(1000.0,700.0));
        assert_eq!(m.gui,Some(original));assert_eq!(m.gui.unwrap().tool_button_side,84.5);
        let larger=m.viewport.unwrap();
        assert_eq!(larger.source_length_to_world(84.5),v.source_length_to_world(84.5)*0.5);
        m.update(entity,2.0,[0,0],[0,0],Vec2::ZERO);assert!(m.viewport.is_none());assert_eq!(m.gui,Some(original));
    }
    #[test]
    fn headless_window_adapter_captures_windows_physical_pixels_not_winit_logical_ratio() {
        let mut app=App::new();app.init_resource::<SourceUiMetrics>().add_systems(Update,capture);
        let mut window=Window::default();window.resolution.set_physical_resolution(1000,700);window.resolution.set_scale_factor(1.25);
        app.world_mut().spawn((window,bevy::window::PrimaryWindow));app.update();
        let metrics=app.world().resource::<SourceUiMetrics>();
        if cfg!(target_os="windows") {assert_eq!(metrics.gui.unwrap().fbscale,1.0);}
        assert_eq!(metrics.gui.unwrap().fontscale,1.25);
    }
}



#[cfg(test)]
#[path = "source_tools_popup_evidence_tests.rs"]
mod popup_evidence_tests;

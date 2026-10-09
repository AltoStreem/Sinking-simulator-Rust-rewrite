//! Layer-combo popup: bundled ImGui inner clipping, wheel step and scrollbar.
//! Entries keep their shared source Tools geometry; cameras clip partial rows.
use crate::*;
use crate::source_tools_layout::{Layout,Role};
use bevy::camera::Viewport;
// Editor text owns 12..=14; popup passes must be disjoint from every field camera.
pub(crate) const BACKGROUND_LAYER:usize=15;
pub(crate) const ENTRIES_LAYER:usize=16;

#[derive(Component)]
pub(crate) struct PopupCamera(pub bool); // true = selectable/text inner clip
#[derive(Component)]
pub(crate) struct Scrollbar(pub bool); // true = grab

/// windows.begin InnerRect/InnerClipRect, with beginCombo's FramePadding.x override.
pub(crate) fn inner_clip(layout:&Layout,count:usize)->Rect {
    let popup=layout.source_popup(count);
    let border=layout.gui.window_border;
    let inset=(layout.gui.frame_padding.x*0.5).floor().max(border);
    let scrollbar=if count>8 {layout.gui.scrollbar_size}else {0.0};
    let source=Rect::from_corners(
        (popup.min+Vec2::new(inset,border)+Vec2::splat(0.5)).floor(),
        (popup.max-Vec2::new(scrollbar+inset,border)+Vec2::splat(0.5)).floor());
    let source=Rect::from_corners(source.min.max(Vec2::ZERO),source.max.min(layout.viewport.source_size));
    layout.viewport.source_rect_to_world(source)
}
/// Selectable hit testing must use this intersection, not only its unclipped row.
pub(crate) fn option_contains(layout:&Layout,count:usize,index:usize,point:Vec2)->bool {
    inner_clip(layout,count).contains(point)&&layout.option(index,count).contains(point)
}
fn wheel_step(layout:&Layout,count:usize)->f32 {
    // MiscKt.updateMouseWheel: uses InnerRect.height, NOT InnerClipRect.height.
    ui_scrollbar::wheel_step(layout.gui.font18,layout.source_popup(count).height())
}
fn scrollbar_geometry(layout:&Layout,count:usize)->(Rect,ui_scrollbar::Geometry) {
    let popup=layout.source_popup(count);
    let border=layout.gui.window_border;
    // Coordinates for Geometry are source pixels with upward-positive Y.
    let track=Rect::from_corners(Vec2::new(popup.max.x-layout.gui.scrollbar_size,popup.min.y+border),popup.max-Vec2::splat(border));
    let available=popup.height();
    let geometry=ui_scrollbar::Geometry::new(-track.min.y,-track.max.y,available,
        available+layout.scroll_max(count),(10.0*layout.gui.gui_scale).floor());
    (track,geometry)
}
fn cursor(window:&Window,layout:&Layout)->Option<Vec2> {
    window.cursor_position().map(|p|layout.viewport.source_to_world(layout.viewport.logical_to_source(p)))
}

pub(crate) fn setup(mut commands:Commands) {
    for inner in [false,true] {
        commands.spawn((Camera2d,Camera {order:if inner {10}else {9},is_active:false,
            clear_color:ClearColorConfig::None,..default()},
            Projection::Orthographic(OrthographicProjection::default_2d()),PopupCamera(inner),
            RenderLayers::layer(if inner {ENTRIES_LAYER}else {BACKGROUND_LAYER})));
    }
    for grab in [false,true] {
        commands.spawn((Scrollbar(grab),Sprite::from_color(ui_classic_appearance::rgba([0.2,0.25,0.3,0.6]),Vec2::ONE),
            Transform::from_xyz(0.0,0.0,if grab {43.0}else {42.0}),Visibility::Hidden,RenderLayers::layer(BACKGROUND_LAYER)));
    }
}
pub(crate) fn route_layers(mut rows:Query<(&Role,&mut RenderLayers),Without<Camera>>) {
    for (role,mut layers) in &mut rows {
        match role {
            Role::Popup=>*layers=RenderLayers::layer(BACKGROUND_LAYER),
            Role::Option(_)|Role::OptionLabel(_)=>*layers=RenderLayers::layer(ENTRIES_LAYER),
            _=>{},
        }
    }
}
pub(crate) fn scroll(mut wheel:MessageReader<MouseWheel>,keys:Res<ButtonInput<KeyCode>>,
    windows:Query<&Window>,catalog:Res<ShipCatalog>,preview:Option<Res<ship_upload_preview::ActivePreview>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,mut simulation:ResMut<Simulation>) {
    // Read every frame so closed-popup events cannot replay when it opens.
    let deltas:Vec<_>=wheel.read().map(|event|(event.window,match event.unit {
        MouseScrollUnit::Line=>event.y,MouseScrollUnit::Pixel=>event.y/40.0,
    })).collect(); // /40 is the existing Winit-to-GLFW adapter, not an ImGui constant.
    if !simulation.layer_dropdown_open||!simulation.show_tools
        ||keys.pressed(KeyCode::ControlLeft)||keys.pressed(KeyCode::ControlRight)
        ||keys.pressed(KeyCode::ShiftLeft)||keys.pressed(KeyCode::ShiftRight) {return;}
    let Some(layout)=simulation.source_tools else {return};
    let Ok(window)=windows.single() else {return};
    let Some(point)=cursor(window,&layout) else {return};
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    if !layout.popup(count).contains(point) {return;}
    for (_,delta) in deltas {
        simulation.layer_popup_scroll=(simulation.layer_popup_scroll-delta*wheel_step(&layout,count)).clamp(0.0,layout.scroll_max(count));
    }
    let scroll=simulation.layer_popup_scroll;
    if let Some(layout)=simulation.source_tools.as_mut() {layout.popup_scroll=scroll;}
}
pub(crate) fn drag(mouse:Res<ButtonInput<MouseButton>>,windows:Query<&Window>,catalog:Res<ShipCatalog>,
    preview:Option<Res<ship_upload_preview::ActivePreview>>,retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut simulation:ResMut<Simulation>,mut dragging:Local<Option<f32>>) {
    if !mouse.pressed(MouseButton::Left)||!simulation.layer_dropdown_open||!simulation.show_tools {*dragging=None;return;}
    let Some(layout)=simulation.source_tools else {return};
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    if count<=8 {*dragging=None;return;}
    let Ok(window)=windows.single() else {return};
    let Some(point)=cursor(window,&layout) else {return};
    let source=layout.viewport.world_to_source(point);
    let (track,geometry)=scrollbar_geometry(&layout,count);
    if mouse.just_pressed(MouseButton::Left) {
        *dragging=None;
        if track.contains(source) {
            let (scroll,offset)=geometry.activate(simulation.layer_popup_scroll,-source.y);
            simulation.layer_popup_scroll=scroll.clamp(0.0,layout.scroll_max(count));*dragging=Some(offset);
        }
    } else if let Some(offset)=*dragging {
        simulation.layer_popup_scroll=geometry.drag(-source.y,offset).clamp(0.0,layout.scroll_max(count));
    }
    let scroll=simulation.layer_popup_scroll;
    if let Some(layout)=simulation.source_tools.as_mut() {layout.popup_scroll=scroll;}
}
/// After dropdown widgets exist and the shared layout has applied scroll positions.
pub(crate) fn sync(windows:Query<&Window>,simulation:Res<Simulation>,catalog:Res<ShipCatalog>,
    preview:Option<Res<ship_upload_preview::ActivePreview>>,retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut cameras:Query<(&PopupCamera,&mut Camera,&mut Projection,&mut Transform)>,
    mut bars:Query<(&Scrollbar,&mut Sprite,&mut Transform,&mut Visibility),Without<PopupCamera>>) {
    let Ok(window)=windows.single() else {return};
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    let open=simulation.layer_dropdown_open&&simulation.show_tools;
    for (tag,mut camera,mut projection,mut transform) in &mut cameras {
        camera.is_active=false;
        let Some(layout)=simulation.source_tools.filter(|_|open) else {continue};
        let source_rect=if tag.0 {
            let r=inner_clip(&layout,count);
            Rect::from_corners(layout.viewport.world_to_source(r.min),layout.viewport.world_to_source(r.max))
        } else {layout.source_popup(count)};
        let ratio=window.physical_size().as_vec2()/layout.viewport.source_size;
        let start=(source_rect.min*ratio).round().max(Vec2::ZERO).as_uvec2();
        let end=(source_rect.max*ratio).round().max(Vec2::ZERO).as_uvec2().min(window.physical_size());
        if !end.cmpgt(start).all() {continue;}
        camera.is_active=true;
        camera.viewport=Some(Viewport {physical_position:start,physical_size:end-start,..default()});
        let source_min=start.as_vec2()/ratio;let source_max=end.as_vec2()/ratio;
        let center=layout.viewport.source_to_world((source_min+source_max)*0.5);
        transform.translation.x=center.x;transform.translation.y=center.y;
        if let Projection::Orthographic(p)=&mut *projection {
            p.scaling_mode=ScalingMode::FixedVertical {viewport_height:layout.viewport.source_length_to_world(source_max.y-source_min.y)};
            p.scale=1.0;
        }
    }
    for (tag,mut sprite,mut transform,mut visibility) in &mut bars {
        *visibility=Visibility::Hidden;
        let Some(layout)=simulation.source_tools.filter(|_|open&&count>8) else {continue};
        let (track,geometry)=scrollbar_geometry(&layout,count);
        let rect=if tag.0 {
            let center=-geometry.center(simulation.layer_popup_scroll);
            let inset=((layout.gui.scrollbar_size-2.0)*0.5).floor().clamp(0.0,3.0);
            Rect::from_center_size(Vec2::new(track.center().x,center),Vec2::new((track.width()-2.0*inset).max(0.0),geometry.grab))
        } else {track};
        let rect=layout.viewport.source_rect_to_world(rect);
        sprite.custom_size=Some(rect.size());transform.translation.x=rect.center().x;transform.translation.y=rect.center().y;
        let hovered=cursor(window,&layout).is_some_and(|point|rect.contains(point));
        sprite.color=if !tag.0 {ui_classic_appearance::rgba([0.2,0.25,0.3,0.6])}
            else if hovered {ui_classic_appearance::rgba([0.4,0.4,0.8,0.4])}else {ui_classic_appearance::rgba([0.4,0.4,0.8,0.3])};
        *visibility=Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn layout()->Layout {
        Layout::new(source_ui_metrics::GuiMetrics::new(1.0,[1280,720],[1280,720]),
            source_ui_metrics::UiViewport::new(Vec2::new(1280.0,720.0),Vec2::new(1280.0,720.0),UVec2::new(1280,720)).unwrap())
    }
    #[test]
    fn bundled_popup_clip_excludes_scrollbar_and_uses_half_horizontal_padding() {
        let layout=layout();let popup=layout.source_popup(12);let inner=inner_clip(&layout,12);
        let min=layout.viewport.world_to_source(Vec2::new(inner.min.x,inner.max.y));
        let max=layout.viewport.world_to_source(Vec2::new(inner.max.x,inner.min.y));
        assert_eq!(min,popup.min+Vec2::new(2.0,1.0));
        assert_eq!(max,popup.max-Vec2::new(16.0,1.0));
        assert_eq!(popup.height(),188.0);assert_eq!(wheel_step(&layout,12),90.0);
        assert_eq!(layout.scroll_max(12),88.0);
    }
    #[test]
    fn last_layer_becomes_visible_at_scroll_max_without_truncating_partial_rows() {
        let mut layout=layout();let count=20;
        assert!(!option_contains(&layout,count,19,layout.option(19,count).center()));
        layout.popup_scroll=layout.scroll_max(count);
        assert!(option_contains(&layout,count,19,layout.option(19,count).center()));
        assert!(!option_contains(&layout,count,0,layout.option(0,count).center()));
        layout.popup_scroll=11.0;
        let row=layout.option(0,count);let clip=inner_clip(&layout,count);
        assert!(row.min.y<clip.max.y&&row.max.y>clip.max.y,"first row is partially clipped");
    }
    #[test]
    fn fractional_font_rows_use_parent_source_flooring_and_raw_eight_row_height_cap() {
        let mut layout=Layout::new(source_ui_metrics::GuiMetrics::new(1.25,[1280,720],[1280,720]),
            source_ui_metrics::UiViewport::new(Vec2::new(1280.0,720.0),Vec2::new(1280.0,720.0),UVec2::new(1280,720)).unwrap());
        assert_eq!(layout.source_popup(8).height(),231.0);
        assert_eq!(layout.source_popup(20).height(),235.0);
        assert_eq!(layout.scroll_max(20),320.0);
        assert_eq!(wheel_step(&layout,20),112.0);
        layout.popup_scroll=layout.scroll_max(20);
        assert!(option_contains(&layout,20,19,layout.option(19,20).center()));
        let (_,geometry)=scrollbar_geometry(&layout,20);
        assert_eq!(geometry.maximum,320.0);
    }
    #[test]
    fn popup_camera_layers_do_not_draw_ship_editor_field_primitives() {
        for field in [ShipUploadField::Name,ShipUploadField::Description,ShipUploadField::LayerName] {
            let editor=RenderLayers::layer(ui_text_viewport::layer(field));
            assert!(!editor.intersects(&RenderLayers::layer(BACKGROUND_LAYER)));
            assert!(!editor.intersects(&RenderLayers::layer(ENTRIES_LAYER)));
        }
        assert!(!RenderLayers::layer(BACKGROUND_LAYER).intersects(&RenderLayers::layer(ENTRIES_LAYER)));
    }
    #[test]
    fn popup_roles_use_separate_native_background_and_inner_passes() {
        let mut app=App::new();app.add_systems(Update,route_layers);
        let backdrop=app.world_mut().spawn((Role::Popup,RenderLayers::layer(1))).id();
        let row=app.world_mut().spawn((Role::Option(9),RenderLayers::layer(1))).id();
        app.update();
        assert_eq!(app.world().get::<RenderLayers>(backdrop).unwrap(),&RenderLayers::layer(BACKGROUND_LAYER));
        assert_eq!(app.world().get::<RenderLayers>(row).unwrap(),&RenderLayers::layer(ENTRIES_LAYER));
    }
}



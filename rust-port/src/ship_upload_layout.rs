//! Source editor window sizing, movement/collapse, resource reflow and clipping.
//! Source atlas rasterization, complete tab overflow and resize-grip drawing remain pending.
use crate::*;
use bevy::{camera::Viewport, sprite::Anchor, text::TextBounds};
use crate::source_ui_metrics::{GuiMetrics,UiViewport,SourceUiMetrics};
use bevy::text::LineHeight;
#[derive(Clone,Copy,Debug)]
pub(crate) struct SourceWindow {pub origin:Vec2,pub size:Vec2,pub collapsed:bool,pub gui:GuiMetrics,pub viewport:UiViewport}
impl SourceWindow {
    fn scale(self)->f32 {720.0/self.viewport.source_size.y}
    fn title_height(self)->f32 {self.gui.font18+self.gui.frame_padding.y*2.0}
    fn local(self,p:Vec2)->Vec2 {Vec2::new(p.x-self.size.x*0.5,self.size.y*0.5-p.y)*self.scale()}
    fn local_rect(self,r:Rect)->Rect {Rect::from_corners(self.local(r.min),self.local(r.max))}
    fn visible_size(self)->Vec2 {Vec2::new(self.size.x,if self.collapsed {self.title_height()}else {self.size.y})}
    fn minimum(self)->Vec2 {(Vec2::splat(32.0)*self.gui.gui_scale).floor().max(Vec2::new(0.0,self.title_height()+(self.gui.window_rounding-1.0).max(0.0)))}
}
impl Layout {
    pub(crate) fn is_collapsed(&self)->bool {self.source_window.is_some_and(|w|w.collapsed)}
    pub(crate) fn frame_padding(&self)->Vec2 {self.source_window.map_or(Vec2::new(4.0,3.0),|w|w.gui.frame_padding*w.scale())}
    pub(crate) fn pixel_size(&self)->f32 {self.source_window.map_or(1.0,|w|w.scale())}
    pub(crate) fn text_height(&self)->f32 {self.source_window.map_or(18.0,|w|w.gui.font18*w.scale())}
    pub(crate) fn window_rect(&self)->Rect {
        self.source_window.map_or(Rect::from_center_size(self.position,Vec2::new(560.0,700.0)),|w|
            w.viewport.source_rect_to_world(Rect::from_corners(w.origin,w.origin+w.visible_size())))
    }
    pub(crate) fn body_rect(&self)->Rect {
        self.source_window.map_or(rect(),|w| {
            let inset=(w.gui.window_padding.x*0.5).floor().max(w.gui.window_border);
            let scrollbar=if self.maximum>0.0 {w.gui.scrollbar_size}else {0.0};
            w.local_rect(Rect::from_corners(Vec2::new((inset+0.5).floor(),(w.title_height()+0.5).floor()),
                Vec2::new((w.size.x-scrollbar-inset+0.5).floor(),(w.size.y-w.gui.window_border+0.5).floor())))
        })
    }
    pub(crate) fn chrome_point(&self,point:Vec2)->Option<Vec2> {
        let local=point-self.position;
        if let Some(w)=self.source_window {
            w.local_rect(Rect::from_corners(Vec2::ZERO,Vec2::new(w.size.x,w.title_height()))).contains(local).then_some(local)
        } else {(local.y>311.0&&self.window_rect().contains(point)).then_some(local)}
    }
    fn refresh_source_position(&mut self) {
        if let Some(w)=self.source_window {self.position=w.viewport.source_to_world(w.origin+w.size*0.5);}
    }
}
fn source_settings()->Option<(Vec2,Vec2,bool)> {
    let manifest=std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [std::path::PathBuf::from("imgui.ini"),manifest.join("../SS2/imgui.ini")] {
        let Ok(text)=std::fs::read_to_string(path) else {continue};
        let mut found=false;let mut origin=None;let mut size=None;let mut collapsed=false;
        for line in text.lines() {
            if line.starts_with('[') {if found {break;}found=line=="[Window][Edit Ship]";continue;}
            if !found {continue;}
            let vector=|value:&str|->Option<Vec2>{let (x,y)=value.split_once(',')?;Some(Vec2::new(x.parse().ok()?,y.parse().ok()?))};
            if let Some(v)=line.strip_prefix("Pos=") {origin=vector(v);}
            if let Some(v)=line.strip_prefix("Size=") {size=vector(v);}
            if let Some(v)=line.strip_prefix("Collapsed=") {collapsed=v.trim()=="1";}
        }
        if let (Some(origin),Some(size))=(origin,size) {if origin.is_finite()&&size.min_element()>0.0 {return Some((origin,size,collapsed));}}
    }None
}
/// Before capture and editor inputs. Constructor style persists; display projection updates.
pub(crate) fn prepare(metrics:Option<Res<SourceUiMetrics>>,mut layout:ResMut<Layout>) {
    let Some((gui,viewport))=metrics.as_ref().and_then(|m|m.gui.zip(m.viewport)) else {return};
    if let Some(mut w)=layout.source_window {
        let source_scroll=layout.scroll/w.scale();w.viewport=viewport;
        layout.scroll=source_scroll*w.scale();layout.source_window=Some(w);
    } else {
        let settings=source_settings();
        let (origin,size,collapsed)=settings.unwrap_or((Vec2::splat(60.0),Vec2::new(gui.default_item_width+gui.window_padding.x*2.0,400.0),false));
        layout.source_window=Some(SourceWindow {origin,size,collapsed,gui,viewport});
        layout.source_auto_fit=if settings.is_none() {2}else {0};
    }
    layout.refresh_source_position();
}
struct NativeStack {slots:Vec<Slot>,name:Rect,description:Rect,layer:Rect,add:Rect,tabs:f32,tail:f32,content_height:f32,width:f32}
fn source_stack(w:SourceWindow,textures:&[(ShipResourceType,Option<Handle<Image>>,Vec2)],scrollbar:bool,
    materials_path:bool,test:Option<(Handle<Image>,Vec2)>,ready:bool,status:bool)->NativeStack {
    let g=w.gui;let width=w.size.x-g.window_padding.x*2.0-if scrollbar {g.scrollbar_size}else {0.0};
    let mut y=(w.title_height()+g.window_padding.y).floor();let x=g.window_padding.x;
    let field=|y,height|Rect::from_corners(Vec2::new(x,y),Vec2::new(x+(width-1.0).floor().max(1.0),y+height));
    let name=field(y,g.frame_height);y=(y+g.frame_height+g.item_spacing.y).floor();
    let description=field(y,g.font18*8.0+g.frame_padding.y*2.0);y=(description.max.y+g.item_spacing.y).floor();

    let mut slots=vec![];let mut add=Rect::default();let mut layer=Rect::default();let mut tabs=0.0;
    for (index,(kind,handle,dimensions)) in textures.iter().enumerate() {
        if index==2 {
            y=(y+g.item_spacing.y).floor(); // separator itemSize(0,0)
            let add_width=source_text_width("Add Layer",g.font18)+g.frame_padding.x*2.0;
            add=Rect::from_corners(Vec2::new(x,y),Vec2::new(x+add_width,y+g.frame_height));
            let layer_x=add.max.x+g.item_spacing.x;
            layer=Rect::from_corners(Vec2::new(layer_x,y),Vec2::new((x+width-1.0).floor().max(layer_x+1.0),y+g.frame_height));
            y=(y+g.frame_height+g.item_spacing.y).floor();tabs=y;
            y=(y+g.frame_height+g.item_spacing.y).floor();
        }
        let (size,frame,cx)=if handle.is_some() {
            let pad=Vec2::splat(g.frame_padding.y as i32 as f32);
            let size=dslfix::DslFix::adjust_texture_size(&Dimensions(dimensions.x as u32,dimensions.y as u32),
                (200.0*g.gui_scale) as f64,pad,&mut NativeGeometry {w,width,scrollbar,x:0.0});
            let frame=size+pad*2.0;let cx=(w.size.x-if scrollbar {g.scrollbar_size}else {0.0})*0.5;
            (size,frame,cx)
        }else {(Vec2::ZERO,Vec2::new(width,30.0*g.gui_scale),x+width*0.5)};
        slots.push(Slot {kind:*kind,center:w.local(Vec2::new(cx,y+frame.y*0.5)),frame:frame*w.scale(),image:handle.clone(),size:size*w.scale()});
        y=(y+frame.y+g.item_spacing.y).floor();
        if *kind==ShipResourceType::Materials&&materials_path {y=(y+g.font18+g.item_spacing.y).floor();}
    }

    y=(y+g.item_spacing.y).floor();
    if let Some((handle,dimensions))=test {
        let pad=Vec2::splat(g.frame_padding.y as i32 as f32);
        let size=dslfix::DslFix::adjust_texture_size_default(&Dimensions(dimensions.x as u32,dimensions.y as u32),0.0,pad,2,
            &mut NativeGeometry {w,width,scrollbar,x:0.0});
        let frame=size+pad*2.0;let cx=(w.size.x-if scrollbar {g.scrollbar_size}else {0.0})*0.5;
        slots.push(Slot {kind:ShipResourceType::Texture,center:w.local(Vec2::new(cx,y+frame.y*0.5)),frame:frame*w.scale(),image:Some(handle),size:size*w.scale()});
        y=(y+frame.y+g.item_spacing.y).floor();
    }
    let tail=y;
    if status {y=(y+g.font18+g.item_spacing.y).floor();}
    if ready {y=(y+30.0*g.gui_scale+g.item_spacing.y).floor();}
    NativeStack {slots,name,description,layer,add,tabs,tail,content_height:(y-g.item_spacing.y-(w.title_height()+g.window_padding.y).floor()).floor(),width}
}
fn source_text_width(text:&str,height:f32)->f32 {
    crate::source_font_text::Font {font_size:18.0,advances:crate::source_font_text::F18}
        .measure(height,f32::MAX,0.0,&text.encode_utf16().collect::<Vec<_>>(),-1).map_or(0.0,|m|(m.size.x+0.95).floor())
}
struct NativeGeometry {w:SourceWindow,width:f32,scrollbar:bool,x:f32}
impl dslfix::GeometryBackend for NativeGeometry {
    fn gui_scale(&mut self)->f32 {self.w.gui.gui_scale}
    fn window_content_region_width(&mut self)->f32 {self.width}
    fn window_height(&mut self)->f32 {self.w.size.y}
    fn frame_padding(&mut self)->Vec2 {self.w.gui.frame_padding}
    fn scroll_max_y(&mut self)->f32 {if self.scrollbar {1.0}else {0.0}}
    fn window_width(&mut self)->f32 {self.w.size.x}
    fn scrollbar_size(&mut self)->f32 {self.w.gui.scrollbar_size}
    fn set_cursor_pos_x(&mut self,x:f32) {self.x=x;}
}

#[derive(Default)]
pub(crate) struct SourceGesture {held:Option<(Vec2,Vec2,Vec2,Vec2)>,last_title:Option<(std::time::Instant,Vec2)>}
fn source_gesture(mouse:&ButtonInput<MouseButton>,window:&Window,layout:&mut Layout,state:&mut SourceGesture)->bool {
    let Some(mut w)=layout.source_window else {return false};
    let Some(cursor)=window.cursor_position().map(|p|w.viewport.logical_to_source(p)) else {state.held=None;return true};
    if !mouse.pressed(MouseButton::Left) {state.held=None;return true;}
    if let Some((start,origin,size,edge))=state.held {
        let delta=cursor-start;
        if edge==Vec2::ZERO {w.origin=(origin+delta).floor();}
        else {
            for axis in 0..2 {
                if edge[axis]>0.0 {w.size[axis]=(size[axis]+delta[axis]).max(w.minimum()[axis]);}
                else if edge[axis]<0.0 {w.size[axis]=(size[axis]-delta[axis]).max(w.minimum()[axis]);w.origin[axis]=(origin[axis]+size[axis]-w.size[axis]).floor();}
            }
        }
        layout.source_window=Some(w);layout.refresh_source_position();return true;
    }
    if !mouse.just_pressed(MouseButton::Left) {return true;}
    let p=cursor-w.origin;let title=w.title_height();
    if Rect::from_corners(Vec2::ZERO,Vec2::new(title,title)).contains(p) {
        w.collapsed=!w.collapsed;layout.source_window=Some(w);layout.refresh_source_position();return true;
    }
    let now=std::time::Instant::now();
    if p.x>=0.0&&p.x<=w.size.x-title&&p.y>=0.0&&p.y<title {
        if state.last_title.is_some_and(|(when,pos)|now.duration_since(when).as_secs_f32()<0.3&&cursor.distance_squared(pos)<36.0) {
            w.collapsed=!w.collapsed;layout.source_window=Some(w);state.last_title=None;
        }else {state.last_title=Some((now,cursor));state.held=Some((cursor,w.origin,w.size,Vec2::ZERO));}
        return true;
    }
    if !w.collapsed {
        let draw=(w.gui.font18*1.35).max(w.gui.window_rounding+1.0+w.gui.font18*0.2).floor();
        let grip=(draw*0.75).floor();let outer=4.0;
        let mut edge=Vec2::ZERO;
        if p.y>=w.size.y-grip&&p.y<=w.size.y+outer {
            if p.x>=w.size.x-grip&&p.x<=w.size.x+outer {edge=Vec2::ONE;}
            else if p.x>=-outer&&p.x<=grip {edge=Vec2::new(-1.0,1.0);}
        }
        if edge==Vec2::ZERO {
            if p.y>=0.0&&p.y<=w.size.y {if p.x.abs()<=outer {edge.x=-1.0;}else if (p.x-w.size.x).abs()<=outer {edge.x=1.0;}}
            if p.x>=0.0&&p.x<=w.size.x {if p.y.abs()<=outer {edge.y=-1.0;}else if (p.y-w.size.y).abs()<=outer {edge.y=1.0;}}
        }
        if edge!=Vec2::ZERO {layout.source_auto_fit=0;state.held=Some((cursor,w.origin,w.size,edge));return true;}
    }
    false
}

#[derive(Component)]
pub(crate) struct EditorCamera;
#[derive(Component, Clone)]
pub(crate) enum Role {
    Field(ShipUploadField),
    Name,
    Description,
    LayerName,
    Resource(ShipResourceType),
    ResourceLabel(ShipResourceType),
    Image(usize),
    TestFrame,
    Path,
    Add,
    AddLabel,
    Tab,
    TabLabel,
    Save,
    SaveLabel,
    Status,
    Hidden,
    Chrome(Vec2),
}
#[derive(Component)]
pub(crate) struct Bar(bool);
#[derive(Resource, Default)]
pub(crate) struct Layout {
    pub scroll: f32,
    pub position: Vec2,
    pub source_window:Option<SourceWindow>,
    source_auto_fit:u8,
    maximum: f32,
    open: bool,
    slots: Vec<Slot>,
}
impl Layout {
    pub(crate) fn description_targets(
        &self,
    ) -> impl Iterator<Item = (usize, ShipResourceType, &Handle<Image>, Rect)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.image.as_ref().map(|image| {
                (
                    index,
                    slot.kind,
                    image,
                    Rect::from_center_size(slot.center, slot.frame),
                )
            })
        })
    }
    pub(crate) fn test_bounds(&self) -> Option<Rect> {
        self.slots
            .get(5)
            .map(|slot| Rect::from_center_size(slot.center, slot.frame))
    }
}
struct Slot {
    kind: ShipResourceType,
    center: Vec2,
    frame: Vec2,
    image: Option<Handle<Image>>,
    size: Vec2,
}
struct Geometry {
    width: f32,
    scrollbar: bool,
    x: f32,
}
impl dslfix::GeometryBackend for Geometry {
    fn gui_scale(&mut self) -> f32 {
        1.0
    }
    fn window_content_region_width(&mut self) -> f32 {
        self.width
    }
    fn window_height(&mut self) -> f32 {
        700.0
    }
    fn frame_padding(&mut self) -> Vec2 {
        Vec2::new(4.0, 3.0)
    }
    fn scroll_max_y(&mut self) -> f32 {
        if self.scrollbar { 1.0 } else { 0.0 }
    }
    fn window_width(&mut self) -> f32 {
        560.0
    }
    fn scrollbar_size(&mut self) -> f32 {
        14.0
    }
    fn set_cursor_pos_x(&mut self, x: f32) {
        self.x = x;
    }
}
struct Dimensions(u32, u32);
impl dslfix::TextureDimensions for Dimensions {
    fn width(&self) -> i32 {
        self.0 as i32
    }
    fn height(&self) -> i32 {
        self.1 as i32
    }
}
pub(crate) fn rect() -> Rect {
    Rect::from_corners(Vec2::new(-268.0, -342.0), Vec2::new(268.0, 311.0))
}
pub(crate) fn content_point(layout: &Layout, point: Vec2) -> Option<Vec2> {
    let point = point - layout.position;
    (!layout.is_collapsed()&&layout.body_rect().contains(point))
        .then(|| point - Vec2::new(0.0, layout.scroll))
}
pub(crate) fn spawn(mut commands: Commands) {
    for kind in [false,true] {for row in 0..7 {commands.spawn((Decoration {kind,row},
        Sprite::from_color(if kind {Color::srgba(0.35,0.4,0.61,0.62)}else {Color::srgba(0.9,0.9,0.9,1.0)},Vec2::ONE),
        Transform::from_xyz(0.0,0.0,86.0),RenderLayers::layer(3),Visibility::Hidden));}}

    commands.spawn((
        Role::Chrome(Vec2::new(0.0, 333.0)),
        Sprite::from_color(Color::srgb(0.28, 0.29, 0.53), Vec2::new(560.0, 34.0)),
        Transform::from_xyz(0.0, 333.0, 81.5),
        RenderLayers::layer(3),
        Visibility::Hidden,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 5,
            clear_color: ClearColorConfig::None,
            is_active: false,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection::default_2d()),
        EditorCamera,
        RenderLayers::layer(5),
    ));
    for index in 0..6 {
        commands.spawn((
            Role::Image(index),
            Sprite::default(),
            Transform::from_xyz(0.0, 0.0, 84.0),
            RenderLayers::layer(5),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        Role::TestFrame,
        Sprite::from_color(Color::srgb(0.24, 0.28, 0.43), Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 82.0),
        RenderLayers::layer(5),
        Visibility::Hidden,
    ));
    commands.spawn((
        Role::Path,
        Text2d::new(""),
        TextFont::from_font_size(18.0),
        TextLayout::no_wrap(),
        Anchor::TOP_LEFT,
        Transform::from_xyz(-264.0, 0.0, 84.0),
        RenderLayers::layer(5),
        Visibility::Hidden,
    ));
    for thumb in [false, true] {
        commands.spawn((
            Bar(thumb),
            Sprite::from_color(
                if thumb {
                    Color::srgb(0.23, 0.28, 0.43)
                } else {
                    Color::srgb(0.12, 0.14, 0.20)
                },
                Vec2::ONE,
            ),
            Transform::from_xyz(273.0, 0.0, 85.0),
            RenderLayers::layer(3),
            Visibility::Hidden,
        ));
    }
}
pub(crate) fn route(
    mut commands: Commands,
    entities: Query<
        (
            Entity,
            Option<&ShipUploadButton>,
            Option<&ShipUploadFieldControl>,
            Option<&ShipUploadNameText>,
            Option<&ShipUploadDescriptionText>,
            Option<&ShipUploadLayerText>,
            Option<&ShipUploadLayerTab>,
            Option<&ShipUploadLayerTabLabel>,
            Option<&ShipUploadStatusText>,
            Option<&ShipUploadResourcePreview>,
            Option<&Text2d>,
            &Transform,
            Option<&Sprite>,
        ),
        (With<ShipUploadModalContent>, Without<Role>),
    >,
) {
    for (
        entity,
        button,
        field,
        name,
        description,
        layer_name,
        tab,
        tab_label,
        status,
        old_preview,
        text,
        transform,
        sprite,
    ) in &entities
    {
        let role = if let Some(field) = field {
            Some(Role::Field(field.0))
        } else if name.is_some() {
            Some(Role::Name)
        } else if description.is_some() {
            Some(Role::Description)
        } else if layer_name.is_some() {
            Some(Role::LayerName)
        } else if tab.is_some() {
            Some(Role::Tab)
        } else if tab_label.is_some() {
            Some(Role::TabLabel)
        } else if status.is_some() {
            Some(Role::Status)
        } else if old_preview.is_some() {
            Some(Role::Hidden)
        } else if let Some(button) = button {
            match button.0 {
                ShipUploadAction::SelectResource(kind) => Some(Role::Resource(kind)),
                ShipUploadAction::AddLayer => Some(Role::Add),
                ShipUploadAction::SaveLocally => Some(Role::Save),
                ShipUploadAction::CycleLayer => Some(Role::Hidden),
                ShipUploadAction::Close if transform.translation.y < 0.0 => Some(Role::Hidden),
                ShipUploadAction::Close => Some(Role::Chrome(Vec2::new(260.0, 333.0))),
                _ => None,
            }
        } else if let Some(text) = text {
            if let Some(kind) = ShipResourceType::values()
                .into_iter()
                .find(|kind| text.0.starts_with(&format!("{} (", kind.name())))
            {
                Some(Role::ResourceLabel(kind))
            } else {
                match text.0.as_str() {
                    "Edit Ship" => Some(Role::Chrome(Vec2::new(-228.0, 333.0))),
                    "×" => Some(Role::Chrome(Vec2::new(260.0, 333.0))),
                    "Add Layer" => Some(Role::AddLabel),
                    "Save Locally" => Some(Role::SaveLabel),
                    "Ship name" | "Description" | "New layer" | "Next Layer" | "Close" => {
                        Some(Role::Hidden)
                    }
                    _ => None,
                }
            }
        } else {
            match sprite.and_then(|sprite| sprite.custom_size) {
                Some(size) if size == Vec2::new(1280.0, 720.0) => Some(Role::Hidden),
                Some(size) if size == Vec2::new(560.0, 700.0) => Some(Role::Chrome(Vec2::ZERO)),
                _ => None,
            }
        };
        if let Some(role) = role {
            if matches!(role, Role::Name | Role::LayerName | Role::Status)
                || matches!(&role, Role::Chrome(_) if text.is_some_and(|text|text.0=="Edit Ship"))
            {
                commands.entity(entity).insert(Anchor::CENTER_LEFT);
            }
            if matches!(role, Role::Description) {
                commands
                    .entity(entity)
                    .insert(Anchor::TOP_LEFT);
            }
            let field=match role {Role::Name=>Some(ShipUploadField::Name),Role::Description=>Some(ShipUploadField::Description),Role::LayerName=>Some(ShipUploadField::LayerName),_=>None};
            if field.is_some() {commands.entity(entity).insert((TextBounds::default(),TextLayout::no_wrap()));}
            let layer = if let Some(field)=field {ui_text_viewport::layer(field)}else if matches!(role, Role::Chrome(_)) {
                3
            } else {
                5
            };
            commands
                .entity(entity)
                .insert((role, RenderLayers::layer(layer)));
        }
    }
}
fn stack(
    textures: &[(ShipResourceType, Option<Handle<Image>>, Vec2)],
    scrollbar: bool,
    materials_path: bool,
    test: Option<(Handle<Image>, Vec2)>,
) -> (Vec<Slot>, f32, f32, f32) {
    let mut geometry = Geometry {
        width: 536.0 - if scrollbar { 14.0 } else { 0.0 },
        scrollbar,
        x: 0.0,
    };
    let mut y = 127.0;
    let mut slots = vec![];
    let mut add_y = 0.0;
    let mut tabs_y = 0.0;
    for (index, (kind, handle, dimensions)) in textures.iter().enumerate() {
        if index == 2 {
            y -= 8.0;
            add_y = y - 12.0;
            y -= 28.0;
            tabs_y = y - 12.0;
            y -= 28.0;
        }
        let (size, frame, center_x) = if handle.is_some() {
            let size = dslfix::DslFix::adjust_texture_size(
                &Dimensions(dimensions.x as u32, dimensions.y as u32),
                200.0,
                Vec2::splat(3.0),
                &mut geometry,
            );
            dslfix::DslFix::center_next_element(size, Vec2::splat(3.0), &mut geometry);
            (
                size,
                size + Vec2::splat(6.0),
                -280.0 + geometry.x + (size.x + 6.0) * 0.5,
            )
        } else {
            (
                Vec2::ZERO,
                Vec2::new(geometry.width, 30.0),
                if scrollbar { -7.0 } else { 0.0 },
            )
        };
        slots.push(Slot {
            kind: *kind,
            center: Vec2::new(center_x, y - frame.y * 0.5),
            frame,
            image: handle.clone(),
            size,
        });
        y -= frame.y + 4.0;
        if *kind == ShipResourceType::Materials && materials_path {
            y -= 22.0;
        }
    }
    y -= 8.0;
    if let Some((handle, dimensions)) = test {
        let size = dslfix::DslFix::adjust_texture_size(
            &Dimensions(dimensions.x as u32, dimensions.y as u32),
            350.0,
            Vec2::splat(3.0),
            &mut geometry,
        );
        dslfix::DslFix::center_next_element(size, Vec2::splat(3.0), &mut geometry);
        let frame = size + Vec2::splat(6.0);
        slots.push(Slot {
            kind: ShipResourceType::Texture,
            center: Vec2::new(-280.0 + geometry.x + frame.x * 0.5, y - frame.y * 0.5),
            frame,
            image: Some(handle),
            size,
        });
        y -= frame.y + 4.0;
    }
    (slots, add_y, tabs_y, y)
}
pub(crate) fn sync(
    mut commands:Commands,
    upload: Res<ship_upload::SourceShipUpload>,
    ui: Res<ShipUploadUiState>,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
    mut layout: ResMut<Layout>,
    mut entities: Query<
        (Entity,&Role,&mut Transform,Option<&mut Sprite>,Option<&mut Text2d>,&mut Visibility,
            Option<&mut TextFont>,Option<&mut crate::ui_font::SourceSize>,Option<&mut LineHeight>,Option<&mut Anchor>,Option<&mut TextLayout>,Option<&ShipUploadLayerTab>,Option<&ShipUploadLayerTabLabel>),
        (Without<EditorCamera>, Without<Bar>),
    >,
    mut cameras: Query<
        (&mut Camera, &mut Projection, &mut Transform),
        (With<EditorCamera>, Without<Role>, Without<Bar>),
    >,
    mut bars: Query<
        (&Bar, &mut Sprite, &mut Transform, &mut Visibility),
        (Without<Role>, Without<EditorCamera>),
    >,
) {
    let opened = upload.window_open();
    if !opened || !layout.open {
        layout.scroll = 0.0;
    }
    layout.open = opened;
    let mut textures = vec![];
    for kind in ShipResourceType::values() {
        let layer = ship_upload_resource_layer(kind, &ui.active_layer);
        let texture = if opened {
            ship_upload_resources::selected_texture_now(&upload, kind, &layer, &mut images)
                .ok()
                .flatten()
        } else {
            None
        };
        let size = texture
            .as_ref()
            .and_then(|handle| images.get(handle))
            .map(|image| {
                Vec2::new(
                    image.texture_descriptor.size.width as f32,
                    image.texture_descriptor.size.height as f32,
                )
            })
            .unwrap_or(Vec2::ZERO);
        textures.push((kind, texture, size));
    }
    let materials_path = upload
        .layers()
        .get(&ShipLayer::default())
        .and_then(|resources| resources.get(&ShipResourceType::Materials))
        .and_then(|resource| match resource {
            ship_thumbnail::ThumbnailResource::File(file) => Some(
                std::fs::canonicalize(&file.path)
                    .unwrap_or_else(|_| file.path.clone())
                    .to_string_lossy()
                    .into_owned(),
            ),
            _ => None,
        });
    let ready = !upload.ship_name_is_blank() && upload.thumbnail().ok().flatten().is_some();
    let test = if opened && ready {
        upload
            .thumbnail()
            .ok()
            .flatten()
            .and_then(|thumbnail| {
                thumbnail
                    .get_resource(ShipResourceType::Texture, &ShipLayer::default())
                    .cloned()
            })
            .and_then(|resource| {
                let handle = match resource {
                    ship_thumbnail::ThumbnailResource::File(file) => {
                        file.texture_now(&mut images).ok()
                    }
                    ship_thumbnail::ThumbnailResource::BaseDerivedTexture(resource) => {
                        resource.texture_now_current(&mut images).ok()
                    }
                }?;
                let image = images.get(&handle)?;
                let size = Vec2::new(
                    image.texture_descriptor.size.width as f32,
                    image.texture_descriptor.size.height as f32,
                );
                Some((handle, size))
            })
    } else {
        None
    };
    if let Some(mut w)=layout.source_window {
        let status=!ready||ui.notice.is_some()||upload.error_string().is_some();
        let mut stack=source_stack(w,&textures,false,materials_path.is_some(),test.clone(),ready,status);
        if layout.source_auto_fit>0&&opened {
            let max=(w.viewport.source_size-Vec2::splat(6.0*w.gui.gui_scale)).max(w.minimum());
            w.size=Vec2::new(w.size.x,(stack.content_height+w.gui.window_padding.y*2.0+w.title_height()).min(max.y)).max(w.minimum());
            layout.source_auto_fit-=1;layout.source_window=Some(w);layout.refresh_source_position();
            stack=source_stack(w,&textures,false,materials_path.is_some(),test.clone(),ready,status);
        }
        let available=w.size.y-w.title_height();
        let mut max=(stack.content_height+w.gui.window_padding.y*2.0-available).max(0.0);
        if max>0.0 {stack=source_stack(w,&textures,true,materials_path.is_some(),test,ready,status);max=(stack.content_height+w.gui.window_padding.y*2.0-available).max(0.0);}
        layout.maximum=max*w.scale();layout.scroll=layout.scroll.clamp(0.0,layout.maximum);layout.slots=stack.slots;

        for (entity,role,mut transform,mut sprite,mut text,mut visibility,font,source_size,line_height,anchor,text_layout,tab,tab_label) in &mut entities {
            let mut shown=opened&&!w.collapsed;let mut rect=None;let mut p=None;let mut wanted_anchor=None;
            match role {
                Role::Field(field)=>rect=Some(match field {ShipUploadField::Name=>stack.name,ShipUploadField::Description=>stack.description,ShipUploadField::LayerName=>stack.layer}),
                Role::Name=>{p=Some(w.local(stack.name.min+w.gui.frame_padding));wanted_anchor=Some(Anchor::TOP_LEFT);},
                Role::Description=>{p=Some(w.local(stack.description.min+w.gui.frame_padding));wanted_anchor=Some(Anchor::TOP_LEFT);},
                Role::LayerName=>{p=Some(w.local(stack.layer.min+w.gui.frame_padding));wanted_anchor=Some(Anchor::TOP_LEFT);},
                Role::Resource(kind)|Role::ResourceLabel(kind)=>{
                    if let Some(slot)=layout.slots.iter().find(|slot|slot.kind==*kind) {p=Some(slot.center);if let Some(ref mut sprite)=sprite {sprite.custom_size=Some(slot.frame);}if matches!(role,Role::ResourceLabel(_)) {shown&=slot.image.is_none();}}
                },
                Role::Image(index)=>{if let Some(slot)=layout.slots.get(*index) {p=Some(slot.center);shown&=slot.image.is_some();if let (Some(mut sprite),Some(handle))=(sprite,slot.image.as_ref()) {sprite.image=handle.clone();sprite.custom_size=Some(slot.size);}}else {shown=false;}*visibility=if shown {Visibility::Inherited}else {Visibility::Hidden};if let Some(p)=p {transform.translation.x=p.x;transform.translation.y=p.y;}continue;},
                Role::TestFrame=>{if let Some(slot)=layout.slots.get(5) {p=Some(slot.center);if let Some(ref mut sprite)=sprite {sprite.custom_size=Some(slot.frame);}}else {shown=false;}},
                Role::Path=>{shown&=materials_path.is_some();p=layout.slots.get(1).map(|s|Vec2::new(w.local(Vec2::new(w.gui.window_padding.x,0.0)).x,s.center.y-s.frame.y*0.5-w.gui.item_spacing.y*w.scale()));wanted_anchor=Some(Anchor::TOP_LEFT);if let(Some(ref mut text),Some(path))=(text,materials_path.as_ref()) {text.0=path.clone();}},
                Role::Add|Role::AddLabel=>rect=Some(stack.add),
                Role::Tab|Role::TabLabel=>{
                    let layer=tab.map(|t|&t.0).or_else(||tab_label.map(|t|&t.0));
                    let mut x=w.gui.window_padding.x;
                    if let Some(layer)=layer {
                        for candidate in upload.layers().keys() {
                            let label=candidate.display_name();
                            let width=source_text_width(label,w.gui.font18)+w.gui.frame_padding.x*2.0;
                            if candidate==layer {rect=Some(Rect::from_corners(Vec2::new(x,stack.tabs),Vec2::new(x+width,stack.tabs+w.gui.frame_height)));break;}
                            x+=width+w.gui.inner_spacing.x;
                        }
                    }
                    shown&=*visibility!=Visibility::Hidden;
                },
                Role::Save|Role::SaveLabel=>{shown&=ready;let y=stack.tail+if status {(w.gui.font18+w.gui.item_spacing.y).floor()}else {0.0};rect=Some(Rect::from_corners(Vec2::new(w.gui.window_padding.x,y),Vec2::new(w.gui.window_padding.x+stack.width,y+30.0*w.gui.gui_scale)));},
                Role::Status=>{shown&=status;p=Some(w.local(Vec2::new(w.gui.window_padding.x,stack.tail)));wanted_anchor=Some(Anchor::TOP_LEFT);},
                Role::Hidden=>shown=false,
                Role::Chrome(local)=>{
                    shown=opened;
                    if *local==Vec2::ZERO {rect=Some(Rect::from_corners(Vec2::ZERO,w.visible_size()));}
                    else if local.x==0.0 {rect=Some(Rect::from_corners(Vec2::ZERO,Vec2::new(w.size.x,w.title_height())));}
                    else if local.x>0.0 {p=Some(w.local(Vec2::new(w.size.x-w.gui.frame_padding.x-w.gui.font18*0.5,w.title_height()*0.5)));if let Some(ref mut sprite)=sprite {sprite.custom_size=Some(Vec2::splat(w.gui.font18*w.scale()));}}
                    else {p=Some(w.local(Vec2::new(w.gui.font18+w.gui.frame_padding.x*2.0,w.title_height()*0.5)));wanted_anchor=Some(Anchor::CENTER_LEFT);}
                },
            }
            if let Some(rect)=rect {let r=w.local_rect(rect);p=Some(r.center());if let Some(mut sprite)=sprite {sprite.custom_size=Some(r.size());}}
            if let Some(mut p)=p {if matches!(role,Role::Chrome(_)) {p+=layout.position;}transform.translation.x=p.x;transform.translation.y=p.y;}
            if let Some(wanted)=wanted_anchor {if let Some(mut anchor)=anchor {if *anchor!=wanted {*anchor=wanted;}}else {commands.entity(entity).insert(wanted);}}
            if let Some(mut font)=font {
                let height=w.gui.font18*w.scale();let wanted=FontSize::Px(crate::ui_font::em_size(height));
                if font.font_size!=wanted {font.font_size=wanted;if let Some(mut text_layout)=text_layout {text_layout.set_changed();}}
                if let Some(mut source)=source_size {if source.0!=FontSize::Px(height) {source.0=FontSize::Px(height);}}else {commands.entity(entity).insert(crate::ui_font::SourceSize(FontSize::Px(height)));}
                if let Some(mut line)=line_height {if *line!=LineHeight::Px(height) {*line=LineHeight::Px(height);}}else {commands.entity(entity).insert(LineHeight::Px(height));}
            }
            *visibility=if shown {Visibility::Inherited}else {Visibility::Hidden};
        }
        let Ok(window)=windows.single() else {return};
        let clip=layout.body_rect();let global=Rect::from_corners(clip.min+layout.position,clip.max+layout.position);
        let source=Rect::from_corners(w.viewport.world_to_source(global.min),w.viewport.world_to_source(global.max));
        let ratio=window.physical_size().as_vec2()/w.viewport.source_size;
        let start=(source.min*ratio).round().max(Vec2::ZERO).as_uvec2();let end=(source.max*ratio).round().max(Vec2::ZERO).as_uvec2().min(window.physical_size());
        for (mut camera,mut projection,mut transform) in &mut cameras {
            camera.is_active=opened&&!w.collapsed&&end.cmpgt(start).all();if !camera.is_active {continue;}
            camera.viewport=Some(Viewport {physical_position:start,physical_size:end-start,..default()});
            let center=w.viewport.source_to_world((start.as_vec2()/ratio+end.as_vec2()/ratio)*0.5)-layout.position-Vec2::new(0.0,layout.scroll);
            transform.translation.x=center.x;transform.translation.y=center.y;
            if let Projection::Orthographic(p)=&mut *projection {p.scaling_mode=ScalingMode::FixedVertical {viewport_height:(end.y-start.y) as f32/ratio.y*w.scale()};p.scale=1.0;}
        }
        let geometry=ui_scrollbar::Geometry::new(clip.max.y,clip.min.y,clip.height(),clip.height()+layout.maximum,(10.0*w.gui.gui_scale).floor()*w.scale());
        for (bar,mut sprite,mut transform,mut visibility) in &mut bars {
            *visibility=if opened&&!w.collapsed&&max>0.0 {Visibility::Inherited}else {Visibility::Hidden};
            let width=w.gui.scrollbar_size*w.scale();let inset=((w.gui.scrollbar_size-2.0)*0.5).floor().clamp(0.0,3.0)*w.scale();
            sprite.custom_size=Some(Vec2::new(if bar.0 {(width-inset*2.0).max(0.0)}else {width},if bar.0 {geometry.grab}else {clip.height()}));
            transform.translation.x=w.local(Vec2::new(w.size.x-w.gui.scrollbar_size*0.5,0.0)).x+layout.position.x;
            transform.translation.y=(if bar.0 {geometry.center(layout.scroll)}else {clip.center().y})+layout.position.y;
        }
        return;
    }

    let (mut slots, mut add_y, mut tabs_y, mut tail) =
        stack(&textures, false, materials_path.is_some(), test.clone());
    let status_shown = !ready || ui.notice.is_some() || upload.error_string().is_some();
    let tail_height = if ready {
        34.0 + if status_shown { 22.0 } else { 0.0 }
    } else {
        22.0
    };
    let mut end = tail - tail_height;
    if end < rect().min.y {
        (slots, add_y, tabs_y, tail) = stack(&textures, true, materials_path.is_some(), test);
        end = tail - tail_height;
    }
    layout.maximum = (rect().min.y - end).max(0.0);
    layout.scroll = layout.scroll.clamp(0.0, layout.maximum);
    layout.slots = slots;
    for (_,role, mut transform, sprite, text, mut visibility,_,_,_,_,_,_,_) in &mut entities {
        let mut shown = opened;
        let mut center = None;
        let mut size = None;
        match role {
            Role::Field(field) => {
                let (x, y, w, h) = match field {
                    ShipUploadField::Name => (0.0, 291.0, 536.0, 24.0),
                    ShipUploadField::Description => (0.0, 203.0, 536.0, 144.0),
                    ShipUploadField::LayerName => (46.0, add_y, 444.0, 24.0),
                };
                center = Some(Vec2::new(x, y));
                size = Some(Vec2::new(w, h));
            }
            Role::Name => center = Some(Vec2::new(-264.0, 291.0)),
            Role::Description => center = Some(Vec2::new(-264.0, 272.0)),
            Role::LayerName => center = Some(Vec2::new(-172.0, add_y)),
            Role::Resource(kind) | Role::ResourceLabel(kind) => {
                if let Some(slot) = layout.slots.iter().find(|slot| slot.kind == *kind) {
                    center = Some(slot.center);
                    size = Some(slot.frame);
                    if matches!(role, Role::ResourceLabel(_)) {
                        shown &= slot.image.is_none();
                    }
                }
            }
            Role::Image(index) => {
                if let Some(slot) = layout.slots.get(*index) {
                    shown &= slot.image.is_some();
                    center = Some(slot.center);
                    size = Some(slot.size);
                    if let (Some(mut sprite), Some(handle)) = (sprite, slot.image.as_ref()) {
                        sprite.image = handle.clone();
                        sprite.custom_size = size;
                    }
                } else {
                    shown = false;
                }
                *visibility = if shown {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if let Some(center) = center {
                    transform.translation.x = center.x;
                    transform.translation.y = center.y;
                }
                continue;
            }
            Role::TestFrame => {
                if let Some(slot) = layout.slots.get(5) {
                    center = Some(slot.center);
                    size = Some(slot.frame);
                } else {
                    shown = false;
                }
            }
            Role::Path => {
                shown &= materials_path.is_some();
                center = layout
                    .slots
                    .get(1)
                    .map(|slot| Vec2::new(-264.0, slot.center.y - slot.frame.y * 0.5 - 4.0));
                if let (Some(mut text), Some(path)) = (text, materials_path.as_ref()) {
                    text.0 = path.clone();
                }
            }
            Role::Add | Role::AddLabel => {
                center = Some(Vec2::new(-226.0, add_y));
                size = Some(Vec2::new(84.0, 24.0));
            }
            Role::Tab | Role::TabLabel => {
                transform.translation.y = tabs_y;
                shown &= *visibility != Visibility::Hidden;
            }
            Role::Save | Role::SaveLabel => {
                shown &= ready;
                center = Some(Vec2::new(
                    0.0,
                    tail - 15.0 - if status_shown { 22.0 } else { 0.0 },
                ));
                size = Some(Vec2::new(536.0, 30.0));
            }
            Role::Status => {
                shown &= status_shown;
                center = Some(Vec2::new(-264.0, tail - 9.0));
            }
            Role::Hidden => shown = false,
            Role::Chrome(local) => center = Some(*local + layout.position),
        }
        if let Some(center) = center {
            transform.translation.x = center.x;
            transform.translation.y = center.y;
        }
        if let Some(mut sprite) = sprite {
            if let Some(size) = size {
                sprite.custom_size = Some(size);
            }
            if matches!(role, Role::Resource(_)) {
                sprite.color = Color::srgb(0.24, 0.28, 0.43);
            }
        }
        *visibility = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.physical_height() as f32 / 720.0;
    let width = window.physical_width() as f32 / scale.max(0.001);
    let clip = Rect::from_corners(rect().min + layout.position, rect().max + layout.position);
    let start = Vec2::new(
        (clip.min.x + width * 0.5) * scale,
        (360.0 - clip.max.y) * scale,
    )
    .round()
    .max(Vec2::ZERO)
    .as_uvec2();
    let end = Vec2::new(
        (clip.max.x + width * 0.5) * scale,
        (360.0 - clip.min.y) * scale,
    )
    .round()
    .max(Vec2::ZERO)
    .as_uvec2()
    .min(UVec2::new(
        window.physical_width(),
        window.physical_height(),
    ));
    for (mut camera, mut projection, mut transform) in &mut cameras {
        camera.is_active = opened && end.cmpgt(start).all();
        if camera.is_active {
            camera.viewport = Some(Viewport {
                physical_position: start,
                physical_size: end - start,
                ..default()
            });
            transform.translation.x =
                (start.x + end.x) as f32 * 0.5 / scale - width * 0.5 - layout.position.x;
            transform.translation.y =
                360.0 - (start.y + end.y) as f32 * 0.5 / scale - layout.scroll - layout.position.y;
            if let Projection::Orthographic(projection) = &mut *projection {
                projection.scaling_mode = ScalingMode::FixedVertical {
                    viewport_height: (end.y - start.y) as f32 / scale,
                };
            }
        }
    }
    let geometry = ui_scrollbar::Geometry::new(
        clip.max.y,
        clip.min.y,
        clip.height(),
        clip.height() + layout.maximum,
        10.0,
    );
    for (bar, mut sprite, mut transform, mut visibility) in &mut bars {
        transform.translation.x = 273.0 + layout.position.x;
        *visibility = if opened && layout.maximum > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        sprite.custom_size = Some(Vec2::new(
            if bar.0 { 8.0 } else { 14.0 },
            if bar.0 { geometry.grab } else { clip.height() },
        ));
        transform.translation.y = if bar.0 {
            geometry.center(layout.scroll)
        } else {
            clip.center().y
        };
    }
}
pub(crate) fn scroll(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    upload: Res<ship_upload::SourceShipUpload>,
    mut wheel: MessageReader<MouseWheel>,
    mut layout: ResMut<Layout>,
    mut dragging: Local<Option<f32>>,
) {
    let events: Vec<_> = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        })
        .collect();
    if !upload.window_open() {
        *dragging = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let point = Vec2::new(
        cursor.x - window.width() * 0.5,
        window.height() * 0.5 - cursor.y,
    ) * (720.0 / window.height().max(1.0))
        - layout.position;
    if !mouse.pressed(MouseButton::Left) {
        *dragging = None;
    }
    if layout.is_collapsed() {*dragging=None;return;}
    let clip = layout.body_rect();
    let geometry = ui_scrollbar::Geometry::new(
        clip.max.y,
        clip.min.y,
        clip.height(),
        clip.height() + layout.maximum,
        10.0,
    );
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = None;
        if layout.maximum > 0.0
            && layout.source_window.map_or((269.0..=277.0).contains(&point.x),|w| {
                let x=w.local(Vec2::new(w.size.x-w.gui.scrollbar_size*0.5,0.0)).x;
                (point.x-x).abs()<=w.gui.scrollbar_size*w.scale()*0.5
            })
            && (geometry.top - geometry.size..=geometry.top).contains(&point.y)
        {
            let (scroll, offset) = geometry.activate(layout.scroll, point.y);
            layout.scroll = scroll.clamp(0.0, layout.maximum);
            *dragging = Some(offset);
            return;
        }
    } else if let Some(offset) = *dragging {
        layout.scroll = geometry.drag(point.y, offset).clamp(0.0, layout.maximum);
        return;
    }
    if !clip.contains(point)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
    {
        return;
    }
    for delta in events {
        layout.scroll = (layout.scroll - delta * ui_scrollbar::wheel_step(layout.text_height()/layout.pixel_size(), clip.height()/layout.pixel_size())*layout.pixel_size())
            .clamp(0.0, layout.maximum);
    }
}

/// newFrame.updateMouseMovingWindowNewFrame retains the initial click offset,
/// continues outside the window, and releases when mouseDown[0] becomes false.
pub(crate) fn move_window(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    upload: Res<ship_upload::SourceShipUpload>,
    mut layout: ResMut<Layout>,
    items: Query<(
        &Role,
        &Transform,
        &Sprite,
        &Visibility,
        Option<&ShipUploadButton>,
    )>,
    mut dragging: Local<Option<Vec2>>,
    mut was_open: Local<bool>,
    mut source_gesture_state:Local<SourceGesture>,
) {
    let open = upload.window_open();
    if !open || !*was_open {
        *dragging = None;
        *was_open = open;
        return;
    }
    if !mouse.pressed(MouseButton::Left) {
        *dragging = None;
        return;
    }
    let Ok(window) = windows.single() else { return };
    if source_gesture(&mouse,window,&mut layout,&mut source_gesture_state) {return;}
    if let Some(w)=layout.source_window {
        if !w.collapsed {if let Some(cursor)=window.cursor_position().map(|p|w.viewport.logical_to_source(p)) {
            let point=w.viewport.source_to_world(cursor);
            let content=content_point(&layout,point);
            let occupied=items.iter().any(|(role,transform,sprite,visible,_)| {
                *visible!=Visibility::Hidden&&!matches!(role,Role::Chrome(_)|Role::Hidden)&&content.is_some_and(|p|
                    Rect::from_center_size(transform.translation.truncate(),sprite.custom_size.unwrap_or(Vec2::ZERO)).contains(p))
            });
            if !occupied&&content.is_some() {source_gesture_state.held=Some((cursor,w.origin,w.size,Vec2::ZERO));}
        }}return;
    }
    let scale = 720.0 / window.height().max(1.0);
    let point = window.cursor_position().map(|cursor| {
        Vec2::new(
            cursor.x - window.width() * 0.5,
            window.height() * 0.5 - cursor.y,
        ) * scale
    });
    let Some(point) = point.filter(|point| point.is_finite()) else {
        *dragging = None;
        return;
    };
    if let Some(offset) = *dragging {
        let desired = point - offset;
        // setPos floors the absolute top-left position in GUI coordinates.
        let top_left = Vec2::new(window.width() * 0.5 * scale - 280.0, 10.0);
        let desired_top_left = top_left + Vec2::new(desired.x, -desired.y);
        let floored = (desired_top_left / scale).floor() * scale;
        layout.position = Vec2::new(floored.x - top_left.x, top_left.y - floored.y);
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let local = point - layout.position;
    if !Rect::from_corners(Vec2::new(-280.0, -350.0), Vec2::new(280.0, 350.0)).contains(local) {
        return;
    }
    // Native startMouseMoving runs on unused window space when no item owns the click.
    let content = content_point(&layout, point);
    let occupied = items
        .iter()
        .any(|(role, transform, sprite, visible, button)| {
            if *visible == Visibility::Hidden {
                return false;
            }
            let candidate = match role {
                Role::Field(_)
                | Role::Resource(_)
                | Role::Add
                | Role::Save
                | Role::Tab
                | Role::Image(_)
                | Role::TestFrame => content,
                Role::Chrome(_)
                    if button.is_some_and(|button| matches!(button.0, ShipUploadAction::Close)) =>
                {
                    Some(point)
                }
                _ => None,
            };
            candidate.is_some_and(|point| {
                Rect::from_center_size(
                    transform.translation.truncate(),
                    sprite.custom_size.unwrap_or(Vec2::ZERO),
                )
                .contains(point)
            })
        });
    let scrollbar = layout.maximum > 0.0
        && (269.0..=277.0).contains(&local.x)
        && (rect().min.y..=rect().max.y).contains(&local.y);
    if !occupied && !scrollbar {
        *dragging = Some(point - layout.position);
    }
}

#[derive(Component)]
pub(crate) struct Decoration {kind:bool,row:usize}
pub(crate) fn sync_window_decorations(upload:Res<ship_upload::SourceShipUpload>,layout:Res<Layout>,
    mut decorations:Query<(&Decoration,&mut Sprite,&mut Transform,&mut Visibility)>) {
    for (tag,mut sprite,mut transform,mut visibility) in &mut decorations {
        *visibility=Visibility::Hidden;
        let Some(w)=layout.source_window.filter(|_|upload.window_open()) else {continue};
        if tag.kind&&w.collapsed {continue;}
        let unit=w.gui.font18*0.5/7.0;
        let row=tag.row as f32;
        let (p,size)=if tag.kind {
            let side=(w.gui.font18*1.35).max(w.gui.window_rounding+1.0+w.gui.font18*0.2).floor();
            let unit=side/7.0; (Vec2::new(w.size.x-unit*(row+1.0)*0.5,w.size.y-side+unit*(row+0.5)),Vec2::new(unit*(row+1.0),unit))
        }else if w.collapsed {
            (Vec2::new(w.gui.frame_padding.x+unit*(row+0.5),w.title_height()*0.5),Vec2::new(unit,unit*(7.0-row)))
        }else {(Vec2::new(w.gui.frame_padding.x+unit*3.5,w.title_height()*0.5+unit*(row-3.0)),Vec2::new(unit*(7.0-row),unit))};
        let p=w.local(p)+layout.position;
        transform.translation.x=p.x;transform.translation.y=p.y;sprite.custom_size=Some(size*w.scale());
        *visibility=Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configured_source_editor_dimensions_and_blank_content_reflow_match_cpu_probe() {
        let gui=GuiMetrics::new(1.25,[1920,1080],[1920,1080]);
        let viewport=UiViewport::new(Vec2::new(1920.0,1080.0),Vec2::new(1536.0,864.0),UVec2::new(1920,1080)).unwrap();
        let w=SourceWindow {origin:Vec2::new(755.0,90.0),size:Vec2::new(847.0,803.0),collapsed:false,gui,viewport};
        let textures=ShipResourceType::values().into_iter().map(|kind|(kind,None,Vec2::ZERO)).collect::<Vec<_>>();
        let stack=source_stack(w,&textures,false,false,None,false,true);
        assert_eq!(stack.content_height,532.0);
        assert_eq!(stack.description.height(),186.0);
        assert_eq!(stack.name.height(),28.5);
        let mut layout=Layout::default();layout.source_window=Some(w);layout.refresh_source_position();
        let clip=layout.body_rect();
        let min=viewport.world_to_source(clip.min+layout.position);
        let max=viewport.world_to_source(clip.max+layout.position);
        assert!((min.x-760.0).abs()<0.001);
        assert!((max.x-1597.0).abs()<0.001);
        assert!(layout.chrome_point(viewport.source_to_world(Vec2::new(800.0,100.0))).is_some());
        assert!(layout.chrome_point(viewport.source_to_world(Vec2::new(800.0,150.0))).is_none());
    }
    #[test]
    fn source_editor_resize_reflows_previews_and_collapse_preserves_full_size() {
        let gui=GuiMetrics::new(1.25,[1280,720],[1280,720]);
        let viewport=UiViewport::new(Vec2::new(1280.0,720.0),Vec2::new(1280.0,720.0),UVec2::new(1280,720)).unwrap();
        let w=SourceWindow {origin:Vec2::new(60.0,60.0),size:Vec2::new(847.0,600.0),collapsed:false,gui,viewport};
        let mut layout=Layout::default();layout.source_window=Some(w);layout.refresh_source_position();
        let mut window=Window {resolution:(1280,720).into(),..default()};
        let mut mouse=ButtonInput::<MouseButton>::default();let mut state=SourceGesture::default();
        window.set_cursor_position(Some(w.origin+w.size-Vec2::splat(5.0)));mouse.press(MouseButton::Left);
        source_gesture(&mouse,&window,&mut layout,&mut state);mouse.clear();
        window.set_cursor_position(Some(w.origin+w.size+Vec2::new(95.0,45.0)));
        source_gesture(&mouse,&window,&mut layout,&mut state);
        assert_eq!(layout.source_window.unwrap().size,w.size+Vec2::new(100.0,50.0));
        mouse.release(MouseButton::Left);source_gesture(&mouse,&window,&mut layout,&mut state);mouse.clear();
        mouse.press(MouseButton::Left);window.set_cursor_position(Some(w.origin+Vec2::splat(10.0)));
        let before=layout.source_window.unwrap().size;source_gesture(&mouse,&window,&mut layout,&mut state);
        assert!(layout.is_collapsed());assert_eq!(layout.source_window.unwrap().size,before);
        assert!((layout.window_rect().height()-28.5).abs()<0.001);
        assert!(content_point(&layout,layout.window_rect().center()).is_none());
    }

    #[test]
    fn moving_editor_preserves_click_offset_viewport_and_local_hit_coordinates() {
        let mut upload = ship_upload::SourceShipUpload::default();
        upload.create_new();
        let mut app = App::new();
        app.insert_resource(upload)
            .init_resource::<ShipUploadUiState>()
            .init_resource::<Layout>()
            .init_resource::<Assets<Image>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Startup, spawn)
            .add_systems(Update, (move_window, route, sync).chain());
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(640.0, 27.0)));
        let window = app.world_mut().spawn(window).id();
        let dimmer = app
            .world_mut()
            .spawn((
                ShipUploadModalContent,
                Sprite::from_color(Color::BLACK, Vec2::new(1280.0, 720.0)),
                Transform::default(),
                Visibility::Inherited,
            ))
            .id();
        let field = app
            .world_mut()
            .spawn((
                ShipUploadModalContent,
                ShipUploadFieldControl(ShipUploadField::Name),
                Sprite::default(),
                Transform::default(),
                Visibility::Inherited,
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(dimmer),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Layout>().position, Vec2::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(740.8, 77.4)));
        app.update();
        assert_eq!(
            app.world().resource::<Layout>().position,
            Vec2::new(100.0, -50.0)
        );
        let camera = app
            .world_mut()
            .query_filtered::<(&Camera, &Transform), With<EditorCamera>>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            camera.0.viewport.as_ref().unwrap().physical_position,
            UVec2::new(472, 99)
        );
        assert_eq!(
            camera.0.viewport.as_ref().unwrap().physical_size,
            UVec2::new(536, 621)
        );
        // The bottom extends off screen; the camera center compensates for the clipped viewport.
        assert_eq!(camera.1.translation.truncate(), Vec2::new(0.0, 0.5));
        assert_eq!(
            content_point(app.world().resource::<Layout>(), Vec2::new(100.0, 241.0)),
            Some(Vec2::new(0.0, 291.0))
        );
        assert_eq!(
            app.world()
                .get::<Transform>(field)
                .unwrap()
                .translation
                .truncate(),
            Vec2::new(0.0, 291.0)
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(740.0, 120.0)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update(); // Clicking the name field must not start a window drag.
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(800.0, 200.0)));
        app.update();
        assert_eq!(
            app.world().resource::<Layout>().position,
            Vec2::new(100.0, -50.0)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.update();
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .create_new();
        app.update();
        assert_eq!(
            app.world().resource::<Layout>().position,
            Vec2::new(100.0, -50.0)
        );
    }
    #[test]
    fn editor_stack_uses_source_image_caps_and_keeps_selectors_between_images() {
        let textures: Vec<_> = ShipResourceType::values()
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    if kind == ShipResourceType::Materials {
                        None
                    } else {
                        Some(Handle::default())
                    },
                    Vec2::new(1000.0, 1000.0),
                )
            })
            .collect();
        let (slots, add, tabs, tail) = stack(
            &textures,
            true,
            true,
            Some((Handle::default(), Vec2::new(1000.0, 1000.0))),
        );
        assert_eq!(slots.len(), 6);
        assert_eq!(slots[0].frame, Vec2::splat(200.0));
        assert_eq!(slots[0].size, Vec2::splat(194.0));
        assert_eq!(slots[1].frame, Vec2::new(522.0, 30.0));
        assert_eq!(slots[5].frame, Vec2::splat(350.0));
        assert!(add < slots[1].center.y - slots[1].frame.y * 0.5);
        assert!(tabs < add && tabs > slots[2].center.y + slots[2].frame.y * 0.5);
        assert!(tail < -342.0);
        for slots in slots.windows(2) {
            assert!(
                slots[0].center.y - slots[0].frame.y * 0.5
                    > slots[1].center.y + slots[1].frame.y * 0.5
            );
        }
    }
    #[test]
    fn active_editor_routes_all_resources_clips_scrolls_and_resets_for_new_ship() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder =
            std::env::temp_dir().join(format!("ss2-editor-layout-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        let mut upload = ship_upload::SourceShipUpload::default();
        upload.create_new();
        upload.set_ship_name("Fixture");
        let mut paths = vec![];
        for kind in [
            ShipResourceType::Base,
            ShipResourceType::Texture,
            ShipResourceType::InLights,
            ShipResourceType::ExLights,
        ] {
            let path = folder.join(format!("Fixture_{}.png", kind.name().to_lowercase()));
            image::RgbaImage::from_pixel(20, 20, image::Rgba([10, 20, 30, 255]))
                .save(&path)
                .unwrap();
            upload
                .select_file(path.clone(), kind, ShipLayer::default())
                .unwrap();
            paths.push(path);
        }
        // Large real resources exercise both the source cap and scroll extent.
        image::RgbaImage::from_pixel(400, 400, image::Rgba([10, 20, 30, 255]))
            .save(&paths[0])
            .unwrap();
        image::RgbaImage::from_pixel(400, 400, image::Rgba([10, 20, 30, 255]))
            .save(&paths[1])
            .unwrap();
        image::RgbaImage::from_pixel(400, 400, image::Rgba([10, 20, 30, 255]))
            .save(&paths[2])
            .unwrap();
        image::RgbaImage::from_pixel(400, 400, image::Rgba([10, 20, 30, 255]))
            .save(&paths[3])
            .unwrap();
        let mut app = App::new();
        app.insert_resource(upload)
            .init_resource::<ShipUploadUiState>()
            .init_resource::<Layout>()
            .init_resource::<Assets<Image>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>()
            .add_systems(Startup, spawn)
            .add_systems(Update, (scroll, route, sync).chain());
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(640.0, 260.0)));
        let window = app.world_mut().spawn(window).id();
        let button = app
            .world_mut()
            .spawn((
                ShipUploadModalContent,
                ShipUploadButton(ShipUploadAction::SelectResource(ShipResourceType::Base)),
                Sprite::default(),
                Transform::default(),
                Visibility::Hidden,
                RenderLayers::layer(1),
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((
                ShipUploadModalContent,
                Text2d::new("BASE (Required)"),
                Transform::default(),
                Visibility::Hidden,
                RenderLayers::layer(1),
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<RenderLayers>(button),
            Some(&RenderLayers::layer(5))
        );
        assert_eq!(
            app.world().get::<Sprite>(button).unwrap().custom_size,
            Some(Vec2::splat(200.0))
        );
        assert_eq!(
            app.world().get::<Visibility>(label),
            Some(&Visibility::Hidden)
        );
        let layout = app.world().resource::<Layout>();
        assert!(layout.maximum > 0.0);
        assert_eq!(layout.slots.len(), 6);
        let camera = app
            .world_mut()
            .query_filtered::<Entity, With<EditorCamera>>()
            .single(app.world())
            .unwrap();
        let viewport = app
            .world()
            .get::<Camera>(camera)
            .unwrap()
            .viewport
            .as_ref()
            .unwrap();
        assert_eq!(viewport.physical_position, UVec2::new(372, 49));
        assert_eq!(viewport.physical_size, UVec2::new(536, 653));
        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: -1.0,
                window,
                phase: bevy::input::touch::TouchPhase::Moved,
            });
        app.update();
        assert_eq!(app.world().resource::<Layout>().scroll, 90.0);
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation.y,
            -105.5
        );
        let row = &app.world().resource::<Layout>().slots[0];
        let screen_point = row.center + Vec2::new(0.0, 90.0);
        assert_eq!(
            content_point(app.world().resource::<Layout>(), screen_point),
            Some(row.center)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .create_new();
        app.update();
        assert_eq!(app.world().resource::<Layout>().maximum, 0.0);
        assert_eq!(app.world().resource::<Layout>().scroll, 0.0);
        assert_eq!(
            app.world().get::<Visibility>(label),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.update();
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);
        for path in paths {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
}

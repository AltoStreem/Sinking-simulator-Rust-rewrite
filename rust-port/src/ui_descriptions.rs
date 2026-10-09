//! Active GUIKt.description adapter, including source mouse tooltip placement.
//! Source pixel sizing/placement is shared with constructor metrics. Keyboard
//! navigation, constructor-specific atlas rasterization and draw-list AA remain pending.
use crate::gui_kt::{DescriptionBackend, DescriptionState};
use crate::*;
use bevy::sprite::Anchor;
use bevy::text::{TextBounds, TextLayoutInfo, LineHeight};
use crate::source_ui_metrics::{GuiMetrics,SourceUiMetrics};
use std::hash::{Hash, Hasher};

#[derive(Component)]
pub(crate) struct DescriptionText;
#[derive(Component)]
pub(crate) struct DescriptionBackground;
#[derive(Component)]
pub(crate) struct DescriptionCamera;
#[derive(Component)]
pub(crate) struct DescriptionBorder(usize);

fn packed_color(rgba: [f32;4]) -> Color {
    let c=rgba.map(|v|((v*255.0+0.5) as u32).min(255) as f32/255.0);
    Color::srgba(c[0],c[1],c[2],c[3])
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction { Right, Down, Up, Left }
#[derive(Default)]
pub(crate) struct TooltipPlacement { last: Option<Direction> }
impl TooltipPlacement {
    // PopupsModalsTooltips.findBestWindowPosForPopup, default mouse policy.
    // Native coordinates: origin top left, positive Y down.
    fn place(&mut self, reference: Vec2, size: Vec2, display: Vec2) -> Vec2 {
        self.place_with_style(reference,size,display,Vec2::splat(3.0),1.0)
    }
    fn place_with_style(&mut self,reference:Vec2,size:Vec2,display:Vec2,safe:Vec2,cursor_scale:f32)->Vec2 {
        let padding=Vec2::new(if display.x>safe.x*2.0 {safe.x}else {0.0}, if display.y>safe.y*2.0 {safe.y}else {0.0});
        let outer=Rect::from_corners(padding,display-padding);
        let avoid=Rect::from_corners(reference-Vec2::new(16.0,8.0),reference+Vec2::splat(24.0*cursor_scale));
        let base=(reference.min(outer.max-size)).max(outer.min);
        let mut order=Vec::with_capacity(5);
        if let Some(last)=self.last {order.push(last);}
        for direction in [Direction::Right,Direction::Down,Direction::Up,Direction::Left] {
            if Some(direction)!=self.last {order.push(direction);}
        }
        for direction in order {
            let width=(if direction==Direction::Left {avoid.min.x}else {outer.max.x})
                -(if direction==Direction::Right {avoid.max.x}else {outer.min.x});
            let height=(if direction==Direction::Up {avoid.min.y}else {outer.max.y})
                -(if direction==Direction::Down {avoid.max.y}else {outer.min.y});
            if width<size.x || height<size.y {continue;}
            self.last=Some(direction);
            return Vec2::new(match direction {Direction::Left=>avoid.min.x-size.x,Direction::Right=>avoid.max.x,_=>base.x},
                match direction {Direction::Up=>avoid.min.y-size.y,Direction::Down=>avoid.max.y,_=>base.y});
        }
        self.last=None;
        // Source tooltip fallback intentionally follows the mouse, even if clipped.
        reference+Vec2::splat(2.0)
    }
}

/// calcTextSize rounds X with +0.95; Window.calcContentSize then floors both
/// dimensions. Tooltip calcAutoFitSize adds padding without minimum/display clamp.
/// Existing F18 advances preserve source measurement rules; constructor-specific
/// atlas advances/rasterization at fractional font scales remain a separate gate.
fn source_tooltip_size(text:&str,gui:GuiMetrics)->Option<Vec2> {
    let measured=crate::source_font_text::Font {font_size:18.0,advances:crate::source_font_text::F18}
        .measure(gui.font18,f32::MAX,gui.font18*35.0,&text.encode_utf16().collect::<Vec<_>>(),-1).ok()?;
    Some(Vec2::new(ui_numeric::floor(measured.size.x+0.95),ui_numeric::floor(measured.size.y))
        +gui.window_padding*2.0)
}
fn source_placement(placement:&mut TooltipPlacement,reference:Vec2,size:Vec2,
    display:Vec2,gui:GuiMetrics)->Vec2 {
    let safe=(Vec2::splat(3.0)*gui.gui_scale).floor();
    let cursor_scale=(1.0*gui.gui_scale).floor();
    placement.place_with_style(reference.floor(),size,display,safe,cursor_scale).floor()
}

pub(crate) fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 11,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        DescriptionCamera,
        RenderLayers::layer(6),
    ));
    commands.spawn((
        DescriptionText,
        Text2d::new(""),
        TextFont::from_font_size(18.0),
        TextColor(packed_color([0.9,0.9,0.9,1.0])),
        TextBounds::new_horizontal(18.0 * 35.0),
        Anchor::TOP_LEFT,
        Transform::from_xyz(0.0, 0.0, 91.0),
        RenderLayers::layer(6),
        Visibility::Hidden,
    ));
    commands.spawn((
        DescriptionBackground,
        Sprite::from_color(packed_color([0.11,0.11,0.14,0.92]), Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 90.0),
        RenderLayers::layer(6),
        Visibility::Hidden,
    ));
    for edge in 0..4 {
        commands.spawn((DescriptionBorder(edge),
            Sprite::from_color(packed_color([0.5,0.5,0.5,0.5]),Vec2::ONE),
            Transform::from_xyz(0.0,0.0,90.5),RenderLayers::layer(6),Visibility::Hidden));
    }
}

struct Backend {
    id: i32,
    hovered: bool,
    font_size: f32,
    text: Option<String>,
    wrap: f32,
    now: Option<i64>,
}
#[cfg(test)]
#[derive(Resource)]
pub(crate) struct TestClock(i64);
impl DescriptionBackend for Backend {
    fn item_id(&mut self) -> i32 {
        self.id
    }
    fn current_time_millis(&mut self) -> i64 {
        self.now.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        })
    }
    fn is_item_hovered(&mut self, flags: i32) -> bool {
        debug_assert_eq!(flags, 0);
        self.hovered
    }
    fn begin_tooltip(&mut self) {}
    fn font_size(&mut self) -> f32 {
        self.font_size
    }
    fn push_text_wrap_pos(&mut self, position: f32) {
        self.wrap = position;
    }
    fn text_ex(&mut self, text: &str, flags: i32, end: Option<usize>) {
        debug_assert_eq!(flags, 0);
        debug_assert!(end.is_none());
        self.text = Some(text.into());
    }
    fn pop_text_wrap_pos(&mut self) {}
    fn end_tooltip(&mut self) {}
}

fn control_description(
    row: Option<usize>,
    action: Option<SettingAction>,
    tool: Option<Tool>,
    player: &music_player::MusicPlayer,
) -> Option<&'static str> {
    if let Some(row) = row {
        return Some(match row {
            0 => "The width of the waves (in meters)",
            1 => "The height of the waves (in meters)",
            2 => "The depth of the sea (in meters)",
            3 => "Multiplies how much the materials float",
            4 => "Multiplies how thick the air and water is",
            5 => "Multiplies how fast the water flows within a ship",
            6 => "Multiplies how fast the water flows into a ship",
            7 => "Makes the water go crazy (more bernoulli's velocity)",
            8 => "The gravity",
            9 => "Multiplies the force necessary to break a link",
            10 => "Multiplies the force keeping a link at the same length",
            11 => "Reduces how bouncy links are",
            12 => "Multiplies how heavy the water is inside the ship",
            13 => "How thick the hull of the ship is (in % of the cross-section) (1 is 100%)",
            14 => "How long the day-night cycle is (in seconds)",
            15 => "Multiplies how dark the sea gets in the depth",
            16 => {
                "How many times the physics are calculated per frame, it affects: strength, rigidity, bounciness"
            }
            17 => {
                "How many times the water inside the ship is calculated per frame, the higher the less buggy the water is."
            }
            18 => "The time of day",
            _ => return None,
        });
    }
    if let Some(tool) = tool {
        return Some(match tool {
            Tool::Break => "Break",
            Tool::Flood => "Flood",
            Tool::Dry => "Dry",
            Tool::Move => "Move",
            Tool::None => return None,
        });
    }
    Some(match action? {
        SettingAction::ToggleCycle => "If the day-night cycle is active",
        SettingAction::ToggleShuffle => {
            if player.shuffle {
                "Disable shuffling"
            } else {
                "Enable shuffling"
            }
        }
        SettingAction::ToggleRepeat => {
            if player.repeat {
                "Disable loop"
            } else {
                "Enable loop"
            }
        }
        SettingAction::ToggleMusic => {
            if player.paused {
                "Unpause music"
            } else {
                "Pause music"
            }
        }
        SettingAction::NextTrack => "Skip track",
        _ => return None,
    })
}

fn editor_item_id(index: usize, image: &Handle<Image>, layer: &ShipLayer) -> i32 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    "Edit Ship".hash(&mut hash);
    // beginTabItem pushes the selected tab ID; the combined preview is outside that scope.
    if (2..5).contains(&index) {
        layer.hash(&mut hash);
    }
    image.id().hash(&mut hash);
    hash.finish() as i32
}

pub(crate) fn update(
    mut commands:Commands,
    metrics:Option<Res<SourceUiMetrics>>,
    simulation: Res<Simulation>,
    catalog: Res<ShipCatalog>,
    layout: Res<ship_browser_ui::BrowserLayout>,
    upload: Res<ship_upload::SourceShipUpload>,
    editor_layout: Option<Res<ship_upload_layout::Layout>>,
    editor_ui: Option<Res<ShipUploadUiState>>,
    windows: Query<&Window>,
    images: Query<(&ShipThumbnail, &Sprite)>,
    player: Res<music_player::MusicPlayer>,
    controls: Query<
        (
            Entity,
            &Transform,
            &Sprite,
            Option<&TabPage>,
            Option<&SettingRow>,
            Option<&SettingButton>,
            Option<&ToolGlyph>,
        ),
        Or<(With<SettingRow>, With<SettingButton>, With<ToolGlyph>)>,
    >,
    mut text: Query<
        (Entity,&mut Text2d, &mut TextFont, Option<&mut crate::ui_font::SourceSize>,
            Option<&mut LineHeight>,&mut TextBounds, &mut Visibility, &mut TextLayout),
        With<DescriptionText>,
    >,
    mut state: Local<DescriptionState>,
    #[cfg(test)] clock: Option<Res<TestClock>>,
) {
    #[cfg(not(test))]
    let now = None;
    #[cfg(test)]
    let now = clock.map(|clock| clock.0);
    let Ok((text_entity,mut text, mut font, mut source_size, mut line_height, mut bounds, mut visibility, mut text_layout)) = text.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let Ok(window) = windows.single() else {
        return;
    };
    let point = window.cursor_position().map(|cursor| {
        Vec2::new(
            cursor.x - window.width() * 0.5,
            window.height() * 0.5 - cursor.y,
        ) * (720.0 / window.height().max(1.0))
    });
    let clip = ship_browser_ui::rect(&simulation);
    // Source Edit Ship is an ordinary window (flags None), not a modal.
    // It blocks only items under its actual chrome, rather than every tooltip.
    let editor_blocks_point=upload.window_open() && point.is_some_and(|point| {
        editor_layout.as_ref().map_or(Rect::from_center_size(Vec2::ZERO,Vec2::new(560.0,700.0)),|layout|layout.window_rect()).contains(point)
    });
    let source_metrics=metrics.as_ref().and_then(|m|m.gui.zip(m.viewport));
    let font_size=if let Some((gui,viewport))=source_metrics {
        let height=viewport.source_length_to_world(gui.font18);
        let wanted=FontSize::Px(crate::ui_font::em_size(height));
        if font.font_size!=wanted {font.font_size=wanted;text_layout.set_changed();}
        if let Some(source)=source_size.as_mut() {if source.0!=FontSize::Px(height) {source.0=FontSize::Px(height);}}
        else {commands.entity(text_entity).insert(crate::ui_font::SourceSize(FontSize::Px(height)));}
        if let Some(line)=line_height.as_mut() {if **line!=LineHeight::Px(height) {**line=LineHeight::Px(height);}}
        else {commands.entity(text_entity).insert(LineHeight::Px(height));}
        height
    } else {
        source_size.as_deref().map_or(font.font_size,|size|size.0)
            .eval(Vec2::new(window.width(), window.height()),16.0)
    };
    for (entity, transform, sprite, page, row, button, tool) in &controls {
        let rendered = if tool.is_some() {
            simulation.show_tools
        } else {
            !simulation.toolbox_collapsed
                && page.is_some_and(|page| page.0 == simulation.active_tab)
        };
        if !rendered {
            continue;
        }
        let Some(description) = control_description(
            row.map(|row| row.0),
            button.map(|button| button.0),
            tool.map(|tool| tool.tool),
            &player,
        ) else {
            continue;
        };
        let hover_point = if tool.is_some() {
            point
        } else {
            point.and_then(|point| toolbox_viewport::content_point(&simulation, point))
        };
        let mut control_bounds = Rect::from_center_size(
            transform.translation.truncate(),
            sprite.custom_size.unwrap_or(Vec2::ONE),
        );
        if let Some(tool)=tool {
            if let Some(layout)=simulation.source_tools {
                if let Some(index)=source_tools_layout::Layout::index(tool.tool) {control_bounds=layout.buttons[index];}
            }
        }
        if row.is_some() {
            control_bounds.max.x = -305.0;
        }
        if matches!(
            button.map(|button| button.0),
            Some(SettingAction::ToggleCycle)
        ) {
            control_bounds.max.x = -500.0;
        }
        let hovered = !editor_blocks_point
            && !simulation.layer_dropdown_open
            && hover_point.is_some_and(|point| control_bounds.contains(point));
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        if tool.is_some() {
            sprite.image.id().hash(&mut hash);
        } else {
            entity.hash(&mut hash);
        }
        let mut backend = Backend {
            id: hash.finish() as i32,
            hovered,
            font_size,
            text: None,
            wrap: 0.0,
            now,
        };
        state.description(description, &mut backend);
        if let Some(description) = backend.text {
            if text.0 != description {
                text.0 = description;
                // A hidden tooltip leaves its camera without visible sprites.
                // Bevy can skip layout on the first frame of the next hover,
                // consuming Text2d's change tick without refreshing its buffer.
                // TextLayout changes also set ComputedTextBlock.needs_rerender,
                // which remains pending until that buffer is actually updated.
                text_layout.set_changed();
            }
            bounds.width = Some(backend.wrap);
            *visibility = Visibility::Inherited;
        }
    }
    if let (Some(editor), Some(ui)) = (editor_layout, editor_ui) {
        let hover_point = point.and_then(|point| ship_upload_layout::content_point(&editor, point));
        for (index, kind, image, frame) in editor.description_targets() {
            let description = if index == 5 {
                format!("Test Ship: {}", upload.ship_name())
            } else {
                format!("Click to change the {} {}", kind.name(), kind.extension())
            };
            let mut backend = Backend {
                id: editor_item_id(index, image, &ui.active_layer),
                hovered: upload.window_open()
                    && hover_point.is_some_and(|point| frame.contains(point)),
                font_size,
                text: None,
                wrap: 0.0,
                now,
            };
            state.description(&description, &mut backend);
            if let Some(description) = backend.text {
                if text.0 != description {
                    text.0 = description;
                    text_layout.set_changed();
                }
                bounds.width = Some(backend.wrap);
                *visibility = Visibility::Inherited;
            }
        }
    }
    if simulation.active_tab != ToolboxTab::Ships || simulation.toolbox_collapsed {
        return;
    }
    for (index, row, _) in &layout.rows {
        let Some((_, sprite)) = images.iter().find(|(image, _)| image.0 == *index) else {
            continue;
        };
        // imageButton's texture identity supplies the item ID. Duplicate textures share hover state.
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        sprite.image.id().hash(&mut hash);
        let hovered = !editor_blocks_point
            && !simulation.layer_dropdown_open
            && point.is_some_and(|point| {
                clip.contains(point)
                    && row.contains(point - Vec2::new(0.0, layout.scroll))
                    && !(layout.maximum > 0.0 && point.x >= clip.max.x - 14.0)
            });
        let mut backend = Backend {
            id: hash.finish() as i32,
            hovered,
            font_size,
            text: None,
            wrap: 0.0,
            now,
        };
        if let Some(choice) = catalog.0.get(*index) {
            state.description(&choice.name, &mut backend);
        }
        if let Some(description) = backend.text {
            if text.0 != description {
                text.0 = description;
                // A hidden tooltip leaves its camera without visible sprites.
                // Bevy can skip layout on the first frame of the next hover,
                // consuming Text2d's change tick without refreshing its buffer.
                // TextLayout changes also set ComputedTextBlock.needs_rerender,
                // which remains pending until that buffer is actually updated.
                text_layout.set_changed();
            }
            bounds.width = Some(backend.wrap);
            *visibility = Visibility::Inherited;
        }
    }
}

pub(crate) fn position(
    metrics:Option<Res<SourceUiMetrics>>,
    windows: Query<&Window>,
    mut texts: Query<
        (&Text2d,&TextLayoutInfo, &Visibility, &mut Transform),
        (With<DescriptionText>, Without<DescriptionBackground>, Without<DescriptionBorder>),
    >,
    mut backgrounds: Query<
        (&mut Sprite, &mut Visibility, &mut Transform),
        (With<DescriptionBackground>, Without<DescriptionText>, Without<DescriptionBorder>),
    >,
    mut borders: Query<(&DescriptionBorder,&mut Sprite,&mut Visibility,&mut Transform),
        (Without<DescriptionText>,Without<DescriptionBackground>)>,
    mut placement: Local<TooltipPlacement>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((text,info, shown, mut text_transform)) = texts.single_mut() else {
        return;
    };
    let Ok((mut sprite, mut visibility, mut background_transform)) = backgrounds.single_mut()
    else {
        return;
    };
    *visibility = *shown;
    for (_,_,mut visible,_) in &mut borders { *visible=*shown; }
    if *shown == Visibility::Hidden {
        placement.last=None;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        *visibility = Visibility::Hidden;
        for (_,_,mut visible,_) in &mut borders { *visible=Visibility::Hidden; }
        placement.last=None;
        return;
    };
    let (size,top_left,padding,border_width)=if let Some((gui,viewport))=metrics.as_ref().and_then(|m|m.gui.zip(m.viewport)) {
        let reference=viewport.logical_to_source(cursor);
        // Non-ASCII source wrapping is still unsupported in the translated Font;
        // retain a usable measured native fallback rather than invent advances.
        let source_size=source_tooltip_size(&text.0,gui).unwrap_or_else(||
            info.size*(viewport.source_size.y/720.0)+gui.window_padding*2.0);
        let native=source_placement(&mut placement,reference,source_size,viewport.source_size,gui);
        (source_size*(720.0/viewport.source_size.y),viewport.source_to_world(native),
            gui.window_padding*(720.0/viewport.source_size.y),viewport.source_length_to_world(1.0))
    } else {
        let scale=720.0/window.height().max(1.0);
        let width=window.width()*scale;
        let reference=Vec2::new(ui_numeric::floor(cursor.x*scale),ui_numeric::floor(cursor.y*scale));
        // Bevy has already divided layout size by target scale factor.
        let size=info.size+Vec2::splat(16.0);
        let native=placement.place(reference,size,Vec2::new(width,720.0));
        (size,Vec2::new(native.x-width*0.5,360.0-native.y),Vec2::splat(8.0),1.0)
    };
    sprite.custom_size = Some(size);
    background_transform.translation.x = top_left.x + size.x * 0.5;
    background_transform.translation.y = top_left.y - size.y * 0.5;
    text_transform.translation.x = top_left.x + padding.x;
    text_transform.translation.y = top_left.y - padding.y;
    for (edge,mut border,_,mut transform) in &mut borders {
        let (offset,extent)=match edge.0 {
            0=>(Vec2::new(size.x*0.5,-border_width*0.5),Vec2::new(size.x,border_width)),
            1=>(Vec2::new(size.x*0.5,-size.y+border_width*0.5),Vec2::new(size.x,border_width)),
            2=>(Vec2::new(border_width*0.5,-size.y*0.5),Vec2::new(border_width,(size.y-border_width*2.0).max(0.0))),
            _=>(Vec2::new(size.x-border_width*0.5,-size.y*0.5),Vec2::new(border_width,(size.y-border_width*2.0).max(0.0))),
        };
        border.custom_size=Some(extent);
        transform.translation.x=top_left.x+offset.x;
        transform.translation.y=top_left.y+offset.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_tooltip_measurement_uses_constructor_font_padding_and_source_wrap() {
        let gui=GuiMetrics::new(1.25,[1280,720],[1280,720]);
        let text="Break";
        let measured=crate::source_font_text::Font {font_size:18.0,advances:crate::source_font_text::F18}
            .measure(22.5,f32::MAX,787.5,&text.encode_utf16().collect::<Vec<_>>(),-1).unwrap();
        let size=source_tooltip_size(text,gui).unwrap();
        assert_eq!(size.x,ui_numeric::floor(measured.size.x+0.95)+20.0);
        assert_eq!(size.y,42.0); // floor(font22.5) + padding20
        let long="The source tooltip wraps this sentence. ".repeat(20);
        assert!(source_tooltip_size(&long,gui).unwrap().y>size.y);
    }
    #[test]
    fn source_tooltip_pixel_placement_and_size_survive_window_height_resize() {
        use crate::source_ui_metrics::UiViewport;
        let gui=GuiMetrics::new(1.25,[1280,720],[1280,720]);
        let size=source_tooltip_size("Break",gui).unwrap();
        for (width,height) in [(1280.0,720.0),(2560.0,1440.0)] {
            let viewport=UiViewport::new(Vec2::new(width,height),Vec2::new(width/1.25,height/1.25),UVec2::new(width as u32,height as u32)).unwrap();
            let reference=viewport.logical_to_source(Vec2::new(80.7,80.7));
            let placed=source_placement(&mut TooltipPlacement::default(),reference,size,viewport.source_size,gui);
            // MouseCursorScale is floor(1.25)=1; left/top avoidance is unscaled.
            assert_eq!(placed,Vec2::new(124.0,100.0));
            let world=viewport.source_to_world(placed);
            assert_eq!(viewport.world_to_source(world),placed);
            let world_width=viewport.source_length_to_world(size.x);
            assert!((world_width*height/720.0-size.x).abs()<0.0001);
        }
    }
    #[test]
    fn source_tooltip_safe_area_and_cursor_scale_use_constructor_style_floors() {
        let gui=GuiMetrics::new(2.0,[1280,720],[1280,720]);
        let position=source_placement(&mut TooltipPlacement::default(),Vec2::new(100.9,100.9),Vec2::new(100.0,40.0),Vec2::new(1280.0,720.0),gui);
        assert_eq!(position,Vec2::new(148.0,100.0));
        let edge=source_placement(&mut TooltipPlacement::default(),Vec2::new(1270.0,100.0),Vec2::new(100.0,40.0),Vec2::new(1280.0,720.0),gui);
        assert_eq!(edge.x,1174.0); // right safe padding floor(3*2)=6
    }
    #[test]
    #[ignore = "Requires GPU and game assets; run explicitly with --ignored"]
    fn live_game_tooltips_switch_without_retaining_first_description() {
        for (scale,threaded) in [(1.0,false),(1.25,false),(1.25,true)] {live_game_tooltips_at_scale(scale,threaded);}
    }
    fn live_game_tooltips_at_scale(scale:f32,threaded:bool) {
        use bevy::{app::PluginsState,camera::RenderTarget,render::{gpu_readback::{Readback,ReadbackComplete},render_resource::{TextureFormat,TextureUsages}},window::{ExitCondition,PrimaryWindow}};
        let plugins=crate::active_default_plugins().disable::<bevy::log::LogPlugin>()
            .set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
            .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
            .disable::<bevy::winit::WinitPlugin>();
        let plugins=if threaded {plugins}else {plugins.disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()};
        let mut app=crate::game_app(plugins);
        app.insert_resource(TestClock(1000));
        app.world_mut().resource_mut::<music_player::MusicPlayer>().paused=true;
        while app.plugins_state()!=PluginsState::Ready {bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));}
        app.finish();app.cleanup();
        let width=(1280.0*scale) as u32;let height=(720.0*scale) as u32;
        let mut window=Window {resolution:(width,height).into(),..default()};
        window.resolution.set_scale_factor(scale);
        let window=app.world_mut().spawn((window,PrimaryWindow)).id();
        let mut target=Image::new_target_texture(width,height,TextureFormat::Rgba8UnormSrgb,None);
        target.texture_descriptor.usage|=TextureUsages::COPY_SRC;
        let handle=app.world_mut().resource_mut::<Assets<Image>>().add(target);
        #[derive(Resource,Default)] struct Pixels(Vec<u8>);
        app.init_resource::<Pixels>();
        app.world_mut().spawn(Readback::texture(handle.clone())).observe(|event:On<ReadbackComplete>,mut pixels:ResMut<Pixels>|{pixels.0=event.data.clone();});
        app.update();
        let world=app.world_mut();let mut cameras=world.query::<(Entity,&RenderTarget)>();
        let cameras:Vec<_>=cameras.iter(world).filter(|(_,target)|matches!(target,RenderTarget::Window(_))).map(|(entity,_)|entity).collect();
        for camera in cameras {app.world_mut().entity_mut(camera).insert((RenderTarget::Image(bevy::camera::ImageRenderTarget {handle:handle.clone(),scale_factor:scale}),Msaa::Off,bevy::camera::CompositingSpace::Srgb));}
        // The source publishes from the previous scan and skips pending textures.
        // Wait for that actual startup path rather than forcing synchronous loading.
        let ready=std::time::Instant::now();
        loop {
            app.update();std::thread::sleep(std::time::Duration::from_millis(10));
            let layout=app.world().resource::<ship_browser_ui::BrowserLayout>();
            let clip=ship_browser_ui::rect(app.world().resource::<Simulation>());
            if !layout.rows.is_empty() && layout.rows.len()==app.world().resource::<ShipCatalog>().0.len() && layout.rows.iter().filter(|(_,rect,_)|clip.contains(rect.center()+Vec2::new(0.0,layout.scroll))).count()>=3 {break;}
            assert!(ready.elapsed()<std::time::Duration::from_secs(30),"Source catalog did not publish multiple ready browser textures");
        }
        let text=app.world_mut().query_filtered::<Entity,With<DescriptionText>>().single(app.world()).unwrap();
        let mut cases=Vec::new();
        for (tool,name) in [(Tool::Dry,"Dry"),(Tool::Move,"Move"),(Tool::Break,"Break"),(Tool::Flood,"Flood")] {
            let world=app.world_mut();let mut q=world.query::<(&ToolGlyph,&Transform)>();
            let point=q.iter(world).find(|(glyph,_)|glyph.tool==tool).unwrap().1.translation.truncate();
            cases.push((point,name.to_owned(),ToolboxTab::Ships));
        }
        let layout=app.world().resource::<ship_browser_ui::BrowserLayout>();
        let clip=ship_browser_ui::rect(app.world().resource::<Simulation>());
        for (index,rect,_) in layout.rows.iter().filter(|(_,rect,_)|clip.contains(rect.center()+Vec2::new(0.0,layout.scroll))) {
            cases.push((rect.center()+Vec2::new(0.0,layout.scroll),app.world().resource::<ShipCatalog>().0[*index].name.clone(),ToolboxTab::Ships));
        }
        assert!(cases.len()>6,"Check multiple ship thumbnails after the first hovered tool");
        cases.push(cases[0].clone());
        // The recording also crosses from tools to music controls. Their text
        // depends on the actual player's current flags, not the selected tool.
        for action in [SettingAction::ToggleShuffle,SettingAction::ToggleRepeat,
            SettingAction::ToggleMusic,SettingAction::NextTrack] {
            let world=app.world_mut();
            let mut query=world.query::<(&SettingButton,&Transform,&TabPage)>();
            let point=query.iter(world).find(|(button,_,page)|
                std::mem::discriminant(&button.0)==std::mem::discriminant(&action) && page.0==ToolboxTab::Music).unwrap().1.translation.truncate();
            let description=control_description(None,Some(action),None,
                world.resource::<music_player::MusicPlayer>()).unwrap();
            cases.push((point,description.to_owned(),ToolboxTab::Music));
        }
        for (index,(point,expected,tab)) in cases.iter().enumerate() {
            app.world_mut().resource_mut::<Simulation>().active_tab=*tab;
            // A real cursor crosses empty space between controls. Once the tooltip
            // sprites leave the camera's prior VisibleEntities, text layout may
            // be skipped on the first frame of the next hover.
            app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(None);
            for _ in 0..8 {app.update();}
            app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(640.0+point.x,360.0-point.y)));
            app.world_mut().resource_mut::<TestClock>().0=2000+index as i64*1000;
            app.update();
            app.world_mut().resource_mut::<TestClock>().0+=301;
            // Native Latin glyphs use independent advances. Keep the fresh-layout
            // identity/position check so other controls cannot retain old glyphs.
            // Compare with a fresh, hidden layout of the requested text using the
            // actual tooltip font/bounds and camera scale. This also checks glyph
            // identities/positions, not merely a possibly stale character count.
            let font=app.world().get::<TextFont>(text).unwrap().clone();
            let bounds=app.world().get::<TextBounds>(text).unwrap().clone();
            let layout=app.world().get::<TextLayout>(text).unwrap().clone();
            let source_size=*app.world().get::<crate::ui_font::SourceSize>(text).unwrap();
            let reference=app.world_mut().spawn((Text2d::new(expected),font,source_size,bounds,layout,RenderLayers::layer(6),Visibility::Hidden)).id();
            for _ in 0..20 {app.update();std::thread::sleep(std::time::Duration::from_millis(10));}
            let value=app.world().get::<Text2d>(text).unwrap();
            let info=app.world().get::<TextLayoutInfo>(text).unwrap();
            println!("Game tooltip scale={scale}, threaded={threaded}, expected={expected}, stored={}, glyphs={}, visibility={:?}, transform={:?}",value.0,info.glyphs.len(),app.world().get::<Visibility>(text).unwrap(),app.world().get::<Transform>(text).unwrap().translation);
            let pixels=&app.world().resource::<Pixels>().0;
            if pixels.len()==(width*height*4) as usize {if let Ok(folder)=std::env::var("SS2_HOVER_DIAGNOSTIC_OUTPUT") {image::RgbaImage::from_raw(width,height,pixels.clone()).unwrap().save(format!("{folder}/game-tooltip-{scale}-{threaded}-{index}.png")).unwrap();}}
            assert_eq!(value.0,*expected);
            assert_eq!(app.world().get::<Visibility>(text).unwrap(),&Visibility::Inherited);
            let reference_info=app.world().get::<TextLayoutInfo>(reference).unwrap();
            let glyphs=|info:&TextLayoutInfo|info.glyphs.iter().map(|glyph|(glyph.position,glyph.atlas_info.texture,glyph.atlas_info.rect,glyph.atlas_info.offset,glyph.atlas_info.is_alpha_mask,glyph.section_index,glyph.line_index)).collect::<Vec<_>>();
            assert!(!reference_info.glyphs.is_empty());
            assert_eq!(glyphs(info),glyphs(reference_info),"Rendered tooltip glyphs differ from a fresh layout of {expected}");
            assert_eq!(info.size,reference_info.size);
            app.world_mut().despawn(reference);
        }
    }
    #[test]
    #[ignore = "Requires GPU text layout; run explicitly with --ignored"]
    fn live_tooltip_glyphs_change_between_tools_and_ship_thumbnails() {
        use bevy::{app::PluginsState, window::ExitCondition, camera::RenderTarget,
            render::render_resource::TextureFormat};
        let mut app=App::new();
        app.add_plugins(DefaultPlugins
            .set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
            .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
            .init_resource::<Simulation>()
            .init_resource::<ship_browser_ui::BrowserLayout>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .insert_resource(ShipCatalog(vec![ShipChoice {name:"Thumbnail Fixture".into(),asset:String::new(),physics_asset:String::new(),material_map:false,scale:1.0,source_key:None}],vec![]))
            .insert_resource(music_player::MusicPlayer::new(vec![]))
            .insert_resource(TestClock(1000))
            .add_systems(Startup,spawn)
            .add_systems(PostUpdate,update.before(bevy::sprite::update_text2d_layout))
            .add_systems(PostUpdate,position.after(bevy::sprite::update_text2d_layout).before(bevy::transform::TransformSystems::Propagate))
            .add_plugins(crate::gui_gpu_blend::SourceGuiBlendPlugin)
            .add_systems(PostUpdate,crate::ui_font::apply.before(bevy::sprite::update_text2d_layout));
        while app.plugins_state()!=PluginsState::Ready {
            bevy::tasks::tick_global_task_pools_on_main_thread();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        app.finish();app.cleanup();
        let mut window=Window {resolution:(1280,720).into(),..default()};
        window.set_cursor_position(Some(Vec2::new(200.0,600.0)));
        let window=app.world_mut().spawn(window).id();
        let mut target=Image::new_target_texture(1280,720,TextureFormat::Rgba8UnormSrgb,None);
        target.texture_descriptor.usage|=bevy::render::render_resource::TextureUsages::COPY_SRC;
        let handle=app.world_mut().resource_mut::<Assets<Image>>().add(target);
        #[derive(Resource,Default)] struct Pixels(Vec<u8>);
        app.init_resource::<Pixels>();
        app.world_mut().spawn(bevy::render::gpu_readback::Readback::texture(handle.clone())).observe(|event:On<bevy::render::gpu_readback::ReadbackComplete>,mut pixels:ResMut<Pixels>|{pixels.0=event.data.clone();});
        app.world_mut().spawn((Camera2d,Camera {order:0,clear_color:ClearColorConfig::Custom(Color::BLACK),..default()},RenderTarget::Image(handle.clone().into()),RenderLayers::layer(0),Msaa::Off,bevy::camera::CompositingSpace::Srgb));
        for (tool,x) in [(Tool::Dry,-440.0),(Tool::Move,-340.0)] {
            let icon=app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
            app.world_mut().spawn((ToolGlyph {tool,normal:icon.clone(),active:icon.clone()},
                Sprite {image:icon,custom_size:Some(Vec2::splat(56.0)),..default()},Transform::from_xyz(x,-240.0,25.0)));
        }
        app.world_mut().spawn((ShipThumbnail(0),Sprite::from_color(Color::WHITE,Vec2::new(200.0,60.0))));
        app.world_mut().resource_mut::<ship_browser_ui::BrowserLayout>().rows.push((0,Rect::from_center_size(Vec2::new(-500.0,100.0),Vec2::new(200.0,60.0)),Vec2::ZERO));
        app.update();
        let camera=app.world_mut().query_filtered::<Entity,With<DescriptionCamera>>().single(app.world()).unwrap();
        app.world_mut().entity_mut(camera).insert((RenderTarget::Image(handle.into()),Msaa::Off,bevy::camera::CompositingSpace::Srgb));
        let text=app.world_mut().query_filtered::<Entity,With<DescriptionText>>().single(app.world()).unwrap();
        for (cursor,description,time) in [(Vec2::new(200.0,600.0),"Dry",1400),(Vec2::new(300.0,600.0),"Move",1800),(Vec2::new(140.0,260.0),"Thumbnail Fixture",2200),(Vec2::new(200.0,600.0),"Dry",2600)] {
            app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(Some(cursor));
            app.world_mut().resource_mut::<TestClock>().0=time;
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
            loop {
                app.update();
                let value=app.world().get::<Text2d>(text).unwrap();
                let info=app.world().get::<TextLayoutInfo>(text).unwrap();
                println!("Tooltip expected={description}, stored={}, glyphs={}, size={:?}",value.0,info.glyphs.len(),info.size);
                if !info.glyphs.is_empty() {break;}
                assert!(std::time::Instant::now()<deadline,"Font layout timeout");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert_eq!(app.world().get::<Text2d>(text).unwrap().0,description);
            // Include multiple frames to detect permanently stale glyphs, not just one-frame scheduling lag.
            // The source publishes from the previous scan and skips pending textures.
        // Wait for that actual startup path rather than forcing synchronous loading.
        let ready=std::time::Instant::now();
        loop {
            app.update();std::thread::sleep(std::time::Duration::from_millis(10));
            let layout=app.world().resource::<ship_browser_ui::BrowserLayout>();
            let clip=ship_browser_ui::rect(app.world().resource::<Simulation>());
            if !layout.rows.is_empty() && layout.rows.len()==app.world().resource::<ShipCatalog>().0.len() && layout.rows.iter().filter(|(_,rect,_)|clip.contains(rect.center()+Vec2::new(0.0,layout.scroll))).count()>=3 {break;}
            assert!(ready.elapsed()<std::time::Duration::from_secs(30),"Source catalog did not publish multiple ready browser textures");
        }
            let pixels=&app.world().resource::<Pixels>().0;
            if pixels.len()==1280*720*4 {
                if let Ok(folder)=std::env::var("SS2_HOVER_DIAGNOSTIC_OUTPUT") {
                    image::RgbaImage::from_raw(1280,720,pixels.clone()).unwrap().save(format!("{folder}/tooltip-{time}.png")).unwrap();
                }
            }
            println!("Tooltip transform {:?}, global {:?}",app.world().get::<Transform>(text).unwrap().translation,app.world().get::<GlobalTransform>(text).unwrap().translation());
            assert_eq!(app.world().get::<TextLayoutInfo>(text).unwrap().glyphs.len(),description.chars().count(),"Rendered tooltip retained glyphs from another control");
        }
    }
    #[test]
    fn tooltip_mouse_placement_preserves_source_direction_order_and_fallback() {
        let mut placement=TooltipPlacement::default();
        let display=Vec2::new(1280.0,720.0);let size=Vec2::new(200.0,60.0);
        assert_eq!(placement.place(Vec2::new(100.0,100.0),size,display),Vec2::new(124.0,100.0));
        assert_eq!(placement.place(Vec2::new(1200.0,100.0),size,display),Vec2::new(1077.0,124.0));
        // Once Down is selected, source tries it before Right while it still fits.
        assert_eq!(placement.place(Vec2::new(100.0,100.0),size,display),Vec2::new(100.0,124.0));
        assert_eq!(placement.place(Vec2::new(1200.0,700.0),size,display),Vec2::new(1077.0,632.0));
        let mut narrow=TooltipPlacement::default();
        assert_eq!(narrow.place(Vec2::new(200.0,50.0),Vec2::new(100.0,70.0),Vec2::new(300.0,100.0)),Vec2::new(84.0,27.0));
        let mut oversized=TooltipPlacement::default();
        assert_eq!(oversized.place(Vec2::new(30.0,40.0),Vec2::splat(500.0),Vec2::splat(100.0)),Vec2::new(32.0,42.0));
        assert!(oversized.last.is_none());
    }
    #[test]
    fn tooltip_uses_source_classic_colors_padding_border_and_hides_all_parts() {
        let mut app=App::new();app.add_systems(Startup,spawn).add_systems(Update,position);
        let mut window=Window {resolution:(1280,720).into(),..default()};
        window.set_cursor_position(Some(Vec2::new(640.0,360.0)));app.world_mut().spawn(window);
        app.update();
        let text=app.world_mut().query_filtered::<Entity,With<DescriptionText>>().single(app.world()).unwrap();
        let mut info=TextLayoutInfo::default();info.size=Vec2::new(100.0,20.0);info.scale_factor=1.0;
        app.world_mut().entity_mut(text).insert((info,Visibility::Inherited));app.update();
        assert_eq!(app.world().get::<Transform>(text).unwrap().translation.truncate(),Vec2::new(32.0,-8.0));
        assert_eq!(app.world().get::<TextColor>(text).unwrap().0,packed_color([0.9,0.9,0.9,1.0]));
        let (sprite,visibility)=app.world_mut().query_filtered::<(&Sprite,&Visibility),With<DescriptionBackground>>().single(app.world()).unwrap();
        assert_eq!(sprite.color,packed_color([0.11,0.11,0.14,0.92]));assert_eq!(sprite.custom_size,Some(Vec2::new(116.0,36.0)));assert_eq!(*visibility,Visibility::Inherited);
        assert_eq!(app.world_mut().query::<&DescriptionBorder>().iter(app.world()).count(),4);
        for (sprite,visibility) in app.world_mut().query_filtered::<(&Sprite,&Visibility),With<DescriptionBorder>>().iter(app.world()) {
            assert_eq!(sprite.color,packed_color([0.5,0.5,0.5,0.5]));assert_eq!(*visibility,Visibility::Inherited);
        }
        *app.world_mut().get_mut::<Visibility>(text).unwrap()=Visibility::Hidden;app.update();
        for visibility in app.world_mut().query_filtered::<&Visibility,Or<(With<DescriptionBackground>,With<DescriptionBorder>)>>().iter(app.world()) {assert_eq!(*visibility,Visibility::Hidden);}
    }
    #[test]
    fn editor_image_descriptions_follow_source_scopes_scroll_and_overlay_order() {
        let image = Handle::<Image>::default();
        assert_ne!(
            editor_item_id(2, &image, &ShipLayer::default()),
            editor_item_id(5, &image, &ShipLayer::default())
        );
        let folder = std::env::temp_dir().join(format!(
            "ss2-tooltip-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("preview.png");
        image::RgbaImage::from_pixel(400, 400, image::Rgba([10, 20, 30, 255]))
            .save(&path)
            .unwrap();
        let mut upload = ship_upload::SourceShipUpload::default();
        upload.create_new();
        upload.set_ship_name("Fixture");
        for kind in [
            ShipResourceType::Base,
            ShipResourceType::Texture,
            ShipResourceType::InLights,
            ShipResourceType::ExLights,
        ] {
            upload
                .select_file(path.clone(), kind, ShipLayer::default())
                .unwrap();
        }
        let mut app = App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ship_browser_ui::BrowserLayout>()
            .init_resource::<ship_upload_layout::Layout>()
            .init_resource::<ShipUploadUiState>()
            .init_resource::<Assets<Image>>()
            .insert_resource(upload)
            .insert_resource(ShipCatalog(vec![], vec![]))
            .insert_resource(music_player::MusicPlayer::new(vec![]))
            .insert_resource(TestClock(1000))
            .add_systems(Startup, (spawn, ship_upload_layout::spawn))
            .add_systems(Update, (ship_upload_layout::sync, update).chain());
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(640.0, 333.0)));
        let window = app.world_mut().spawn(window).id();
        app.update();
        let text = app
            .world_mut()
            .query_filtered::<Entity, With<DescriptionText>>()
            .single(app.world())
            .unwrap();
        let camera = app
            .world_mut()
            .query_filtered::<&Camera, With<DescriptionCamera>>()
            .single(app.world())
            .unwrap();
        assert_eq!(camera.order, 11); // Tooltips remain above the three clipped field cameras.
        assert_eq!(
            app.world().get::<RenderLayers>(text),
            Some(&RenderLayers::layer(6))
        );
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut().resource_mut::<TestClock>().0 = 1300;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut().resource_mut::<TestClock>().0 = 1301;
        app.update();
        assert_eq!(
            app.world().get::<Text2d>(text).unwrap().0,
            "Click to change the BASE png"
        );
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Inherited)
        );
        assert_eq!(
            app.world()
                .resource::<ship_upload_layout::Layout>()
                .description_targets()
                .count(),
            5
        );
        // MATERIALS has no image and therefore contributes no description target.
        assert!(
            app.world()
                .resource::<ship_upload_layout::Layout>()
                .description_targets()
                .all(|(_, kind, _, _)| kind != ShipResourceType::Materials)
        );
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .scroll = 10000.0;
        app.world_mut().resource_mut::<TestClock>().0 = 1400;
        app.update();
        let layout = app.world().resource::<ship_upload_layout::Layout>();
        let point = layout.test_bounds().unwrap().center() + Vec2::new(0.0, layout.scroll);
        assert!(ship_upload_layout::rect().contains(point));
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(640.0 + point.x, 360.0 - point.y)));
        app.world_mut().resource_mut::<TestClock>().0 = 1701;
        app.update();
        assert_eq!(
            app.world().get::<Text2d>(text).unwrap().0,
            "Test Ship: Fixture"
        );
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Inherited)
        );
        // Clipping refreshes the timer, even when the content-space frame remains under the pointer.
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(640.0, 719.0)));
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn descriptions_follow_scrolled_controls_and_source_nonmodal_editor_occlusion() {
        let mut app = App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ship_browser_ui::BrowserLayout>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .insert_resource(ShipCatalog(vec![], vec![]))
            .insert_resource(music_player::MusicPlayer::new(vec![]))
            .insert_resource(TestClock(1000))
            .init_resource::<ship_upload_layout::Layout>()
            .add_systems(Startup, spawn)
            .add_systems(Update, update);
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(126.0, 511.0)));
        let window = app.world_mut().spawn(window).id();
        {
            let mut simulation = app.world_mut().resource_mut::<Simulation>();
            simulation.active_tab = ToolboxTab::Physics;
            simulation.settings_scroll[1] = 173.0;
        }
        app.world_mut().spawn((
            SettingRow(13),
            TabPage(ToolboxTab::Physics),
            Sprite::from_color(Color::WHITE, Vec2::new(225.0, 34.0)),
            Transform::from_xyz(-514.0, -324.0, 24.5),
        ));
        app.world_mut().spawn((
            ToolGlyph {
                tool: Tool::Break,
                normal: Handle::default(),
                active: Handle::default(),
            },
            Sprite::from_color(Color::WHITE, Vec2::splat(56.0)),
            Transform::from_xyz(-600.0, -220.0, 25.0),
        ));
        app.update();
        let text = app
            .world_mut()
            .query_filtered::<Entity, With<DescriptionText>>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut().resource_mut::<TestClock>().0 = 1301;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Inherited)
        );
        assert!(
            app.world()
                .get::<Text2d>(text)
                .unwrap()
                .0
                .starts_with("How thick the hull")
        );
        // Removing scroll puts this row outside the clipped settings window.
        app.world_mut().resource_mut::<Simulation>().settings_scroll[1] = 0.0;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .resource_mut::<Simulation>()
            .toolbox_collapsed = true;
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(40.0, 580.0)));
        app.world_mut().resource_mut::<TestClock>().0 = 1602;
        app.update();
        assert_eq!(app.world().get::<Text2d>(text).unwrap().0, "Break");
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .create_new();
        app.world_mut().resource_mut::<TestClock>().0 = 1700;
        app.update();
        // Editor at the center does not occlude the bottom-left tool.
        assert_eq!(app.world().get::<Visibility>(text),Some(&Visibility::Inherited));
        // Moving its ordinary window over that same tool blocks hover and resets delay.
        app.world_mut().resource_mut::<ship_upload_layout::Layout>().position=Vec2::new(-600.0,-220.0);
        app.update();
        assert_eq!(app.world().get::<Visibility>(text),Some(&Visibility::Hidden));
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.world_mut().resource_mut::<TestClock>().0 = 2000;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
        app.world_mut().resource_mut::<TestClock>().0 = 2001;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .resource_mut::<Simulation>()
            .layer_dropdown_open = true;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(text),
            Some(&Visibility::Hidden)
        );
    }
    #[test]
    fn source_control_description_coverage_and_dynamic_music_state() {
        let mut player = music_player::MusicPlayer::new(vec![]);
        let original=include_str!("../../SS2/decompiled/com/wicpar/sinkingsimulator/gui/Toolbox.java");
        for row in 0..19 {
            let description=control_description(Some(row),None,None,&player).unwrap();
            assert!(original.contains(&format!("GUIKt.description({description:?})")),"row {row} wording must match source");
        }
        assert!(control_description(Some(20), None, None, &player).is_none());
        assert!(
            control_description(None, Some(SettingAction::ToggleTools), None, &player).is_none()
        );
        assert!(
            control_description(None, Some(SettingAction::GeneratePalette), None, &player)
                .is_none()
        );
        assert_eq!(
            control_description(None, None, Some(Tool::Break), &player),
            Some("Break")
        );
        assert_eq!(
            control_description(None, None, Some(Tool::Flood), &player),
            Some("Flood")
        );
        assert_eq!(
            control_description(None, None, Some(Tool::Dry), &player),
            Some("Dry")
        );
        assert_eq!(
            control_description(None, None, Some(Tool::Move), &player),
            Some("Move")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleShuffle), None, &player),
            Some("Enable shuffling")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleRepeat), None, &player),
            Some("Disable loop")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleMusic), None, &player),
            Some("Pause music")
        );
        player.shuffle = true;
        player.repeat = false;
        player.paused = true;
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleShuffle), None, &player),
            Some("Disable shuffling")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleRepeat), None, &player),
            Some("Enable loop")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::ToggleMusic), None, &player),
            Some("Unpause music")
        );
        assert_eq!(
            control_description(None, Some(SettingAction::NextTrack), None, &player),
            Some("Skip track")
        );
    }
    #[test]
    fn active_tooltip_systems_initialize_and_size_the_background_from_text_layout() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ship_browser_ui::BrowserLayout>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .insert_resource(ShipCatalog(vec![], vec![]))
            .insert_resource(music_player::MusicPlayer::new(vec![]))
            .add_systems(Startup, spawn)
            .add_systems(Update, (update, position).chain());
        let mut window = Window {
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(1270.0, 710.0)));
        app.world_mut().spawn(window);
        app.update();
        let text = app
            .world_mut()
            .query_filtered::<Entity, With<DescriptionText>>()
            .single(app.world())
            .unwrap();
        let background = app
            .world_mut()
            .query_filtered::<Entity, With<DescriptionBackground>>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            app.world().get::<Visibility>(background),
            Some(&Visibility::Hidden)
        );
        *app.world_mut().get_mut::<Visibility>(text).unwrap() = Visibility::Inherited;
        app.world_mut().entity_mut(text).insert(TextLayoutInfo {
            size: Vec2::new(200.0, 40.0),
            scale_factor: 2.0,
            ..default()
        });
        app.world_mut().run_system_once(position).unwrap();
        assert_eq!(
            app.world().get::<Sprite>(background).unwrap().custom_size,
            Some(Vec2::new(216.0, 56.0))
        );
        assert_eq!(
            app.world().get::<Visibility>(background),
            Some(&Visibility::Inherited)
        );
        let center = app
            .world()
            .get::<Transform>(background)
            .unwrap()
            .translation;
        assert_eq!(center.x, 529.0);
        assert_eq!(center.y, -314.0);
    }
    #[test]
    fn active_description_backend_uses_strict_source_delay_reset_and_wrap() {
        let mut state = DescriptionState::default();
        let mut backend = Backend {
            id: 1,
            hovered: true,
            font_size: 18.0,
            text: None,
            wrap: 0.0,
            now: Some(1000),
        };
        state.description("Ship", &mut backend);
        assert!(backend.text.is_none());
        backend.now = Some(1300);
        state.description("Ship", &mut backend);
        assert!(backend.text.is_none());
        backend.now = Some(1301);
        state.description("Ship", &mut backend);
        assert_eq!(backend.text.as_deref(), Some("Ship"));
        assert_eq!(backend.wrap, 630.0);
        backend.text = None;
        backend.hovered = false;
        backend.now = Some(1400);
        state.description("Ship", &mut backend);
        backend.hovered = true;
        backend.now = Some(1700);
        state.description("Ship", &mut backend);
        assert!(backend.text.is_none());
        backend.now = Some(1701);
        state.description("Ship", &mut backend);
        assert_eq!(backend.text.as_deref(), Some("Ship"));
    }
}



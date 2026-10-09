//! Active settings-page clipping and scrolling. Coordinates retain the current
//! virtual UI scale; source DPI sizing and ShipScroll thumbnail layout are pending.
use crate::*;
use bevy::camera::Viewport;

#[derive(Component)]
pub(crate) struct ContentCamera;
#[derive(Component)]
pub(crate) struct Backdrop;
#[derive(Component)]
pub(crate) struct PageBackdrop;
#[derive(Component)]
pub(crate) struct ScrollTrack;
#[derive(Component)]
pub(crate) struct ScrollThumb;

pub(crate) fn tab_index(tab: ToolboxTab) -> usize {
    match tab {
        ToolboxTab::Ships => 0,
        ToolboxTab::Physics => 1,
        ToolboxTab::Graphics => 2,
        ToolboxTab::Music => 3,
        ToolboxTab::Performance => 4,
        ToolboxTab::Advanced => 5,
    }
}
fn available_bottom(simulation: &Simulation) -> f32 {
    if simulation.show_tools {
        simulation.source_tools.map_or(simulation.tool_panel_offset.y+334.0,|layout|layout.window.max.y+layout.viewport.source_length_to_world(layout.gui.layout_padding)).clamp(-350.0,300.0)
    } else {
        -350.0
    }
}
pub(crate) fn bottom(simulation: &Simulation) -> f32 {
    available_bottom(simulation)
        .max(simulation.toolbox_page_bottom[tab_index(simulation.active_tab)])
}
fn clip_rect(simulation: &Simulation) -> Rect {
    Rect::from_corners(
        Vec2::new(-630.0, bottom(simulation) + 8.0),
        Vec2::new(-305.0, 277.0),
    )
}
pub(crate) fn content_point(simulation: &Simulation, point: Vec2) -> Option<Vec2> {
    (!simulation.toolbox_collapsed && clip_rect(simulation).contains(point))
        .then(|| unscroll_point(simulation, point))
}
pub(crate) fn unscroll_point(simulation: &Simulation, point: Vec2) -> Vec2 {
    point
        - Vec2::new(
            0.0,
            simulation.settings_scroll[tab_index(simulation.active_tab)],
        )
}

pub(crate) fn route_layers(
    mut entities: Query<
        (
            &mut RenderLayers,
            Option<&TabPage>,
            Option<&Backdrop>,
            Option<&ShipCard>,
            Option<&ShipThumbnail>,
            Option<&ShipNameLabel>,
        ),
        Without<Camera>,
    >,
) {
    for (mut layers, page, backdrop, card, thumbnail, name) in &mut entities {
        if card.is_some() || thumbnail.is_some() || name.is_some() {
            *layers = RenderLayers::layer(4);
        } else if page.is_some() {
            *layers = RenderLayers::layer(2);
        } else if backdrop.is_none() && *layers == RenderLayers::layer(1) {
            *layers = RenderLayers::layer(3);
        }
    }
}

pub(crate) fn scroll_settings(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    capture: Option<Res<ui_input_capture::Capture>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<Simulation>,
) {
    let events: Vec<_> = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        })
        .collect();
    if simulation.active_tab == ToolboxTab::Ships
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );
    if capture.is_some_and(|capture|capture.editor_point(point)) || content_point(&simulation, point).is_none() {
        return;
    }
    let index = tab_index(simulation.active_tab);
    let step = ui_scrollbar::wheel_step(18.0, 311.0 - (bottom(&simulation) + 4.0));
    for delta in events {
        simulation.settings_scroll[index] = (simulation.settings_scroll[index] - delta * step)
            .clamp(0.0, simulation.settings_scroll_max[index]);
    }
}

pub(crate) fn sync(
    windows: Query<&Window>,
    mut simulation: ResMut<Simulation>,
    pages: Query<
        (&TabPage, &Transform, Option<&Sprite>),
        (Without<PageBackdrop>, Without<ContentCamera>),
    >,
    mut cameras: Query<(&mut Camera, &mut Projection, &mut Transform), With<ContentCamera>>,
    mut backdrops: Query<
        (&mut Sprite, &mut Transform),
        (With<Backdrop>, Without<ContentCamera>, Without<TabPage>),
    >,
    mut scrollbars: Query<
        (
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
            Option<&ScrollThumb>,
        ),
        (
            Or<(With<ScrollTrack>, With<ScrollThumb>)>,
            Without<ContentCamera>,
            Without<Backdrop>,
            Without<TabPage>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let mut lowest = [277.0f32; 6];
    for (page, transform, sprite) in &pages {
        let half_height = sprite
            .and_then(|s| s.custom_size)
            .map_or(10.0, |s| s.y * 0.5);
        let index = tab_index(page.0);
        lowest[index] = lowest[index].min(transform.translation.y - half_height);
    }
    for index in 1..6 {
        simulation.toolbox_page_bottom[index] = lowest[index] - 8.0;
    }
    let rect = clip_rect(&simulation);
    for index in 1..6 {
        let page_bottom = available_bottom(&simulation).max(simulation.toolbox_page_bottom[index]);
        simulation.settings_scroll_max[index] = (page_bottom + 8.0 - lowest[index]).max(0.0);
        simulation.settings_scroll[index] =
            simulation.settings_scroll[index].min(simulation.settings_scroll_max[index]);
    }
    let index = tab_index(simulation.active_tab);
    let scroll = simulation.settings_scroll[index];
    let pixels_per_unit = window.physical_height() as f32 / 720.0;
    let virtual_width = window.physical_width() as f32 / pixels_per_unit.max(0.001);
    let pixel_min = Vec2::new(
        (rect.min.x + virtual_width * 0.5) * pixels_per_unit,
        (360.0 - rect.max.y) * pixels_per_unit,
    );
    let pixel_max = Vec2::new(
        (rect.max.x + virtual_width * 0.5) * pixels_per_unit,
        (360.0 - rect.min.y) * pixels_per_unit,
    );
    let start = pixel_min.round().max(Vec2::ZERO).as_uvec2();
    let end = pixel_max.round().max(Vec2::ZERO).as_uvec2().min(UVec2::new(
        window.physical_width(),
        window.physical_height(),
    ));
    for (mut camera, mut projection, mut transform) in &mut cameras {
        camera.is_active = !simulation.toolbox_collapsed && end.cmpgt(start).all();
        if !camera.is_active {
            continue;
        }
        camera.viewport = Some(Viewport {
            physical_position: start,
            physical_size: end - start,
            ..default()
        });
        let size = (end - start).as_vec2() / pixels_per_unit;
        let center = Vec2::new(
            (start.x + end.x) as f32 * 0.5 / pixels_per_unit - virtual_width * 0.5,
            360.0 - (start.y + end.y) as f32 * 0.5 / pixels_per_unit,
        );
        transform.translation.x = center.x;
        transform.translation.y = center.y - scroll;
        if let Projection::Orthographic(p) = &mut *projection {
            p.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: size.y,
            };
            p.scale = 1.0;
        }
    }
    for (mut sprite, mut transform) in &mut backdrops {
        let width = sprite.custom_size.unwrap_or(Vec2::ZERO).x;
        let border = if width > 350.0 { 3.0 } else { 0.0 };
        let height = 350.0 - bottom(&simulation);
        sprite.custom_size = Some(Vec2::new(width, height + 2.0 * border));
        transform.translation.y = 350.0 - height * 0.5;
    }
    let max = simulation.settings_scroll_max[index];
    let track_top = 311.0;
    let track_bottom = bottom(&simulation) + 4.0;
    let track_height = track_top - track_bottom;
    let geometry = ui_scrollbar::active_geometry(&simulation);
    for (mut sprite, mut transform, mut visibility, thumb) in &mut scrollbars {
        *visibility = if !simulation.toolbox_collapsed && max > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let height = if thumb.is_some() {
            geometry.grab
        } else {
            track_height
        };
        sprite.custom_size = Some(Vec2::new(if thumb.is_some() { 8.0 } else { 14.0 }, height));
        transform.translation.y = if thumb.is_some() {
            geometry.center(scroll)
        } else {
            (track_top + track_bottom) * 0.5
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_page_height_fits_controls_and_changes_on_tab_selection() {
        let mut app = App::new();
        let mut s = Simulation::default();
        s.active_tab = ToolboxTab::Advanced;
        app.insert_resource(s).add_systems(Update, sync);
        app.world_mut().spawn(Window {
            resolution: (1280, 720).into(),
            ..default()
        });
        app.world_mut().spawn((
            TabPage(ToolboxTab::Advanced),
            Transform::from_xyz(-465.0, 252.0, 25.0),
            Sprite::from_color(Color::WHITE, Vec2::new(326.0, 34.0)),
        ));
        app.world_mut().spawn((
            TabPage(ToolboxTab::Music),
            Transform::from_xyz(-461.0, 218.0, 25.0),
            Sprite::from_color(Color::WHITE, Vec2::new(214.0, 26.0)),
        ));
        let backdrop = app
            .world_mut()
            .spawn((
                Backdrop,
                Transform::default(),
                Sprite::from_color(Color::WHITE, Vec2::new(350.0, 700.0)),
            ))
            .id();
        app.update();
        assert_eq!(bottom(app.world().resource::<Simulation>()), 227.0);
        assert_eq!(
            app.world().get::<Sprite>(backdrop).unwrap().custom_size,
            Some(Vec2::new(350.0, 123.0))
        );
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll_max[5],
            0.0
        );
        app.world_mut().resource_mut::<Simulation>().active_tab = ToolboxTab::Music;
        app.update();
        assert_eq!(bottom(app.world().resource::<Simulation>()), 197.0);
        assert_eq!(
            app.world().get::<Sprite>(backdrop).unwrap().custom_size,
            Some(Vec2::new(350.0, 153.0))
        );
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll_max[3],
            0.0
        );
    }
    #[test]
    fn active_viewport_layers_bounds_scroll_and_resize_initialize_together() {
        let mut app = App::new();
        let mut s = Simulation::default();
        s.active_tab = ToolboxTab::Physics;
        app.insert_resource(s)
            .add_systems(Update, (route_layers, sync).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        let camera = app
            .world_mut()
            .spawn((
                Camera2d,
                ContentCamera,
                Projection::Orthographic(OrthographicProjection::default_2d()),
                Transform::default(),
            ))
            .id();
        let field = app
            .world_mut()
            .spawn((
                TabPage(ToolboxTab::Physics),
                Transform::from_xyz(-514.0, -324.0, 24.0),
                Sprite::from_color(Color::WHITE, Vec2::new(225.0, 34.0)),
                RenderLayers::layer(1),
            ))
            .id();
        let backdrop = app
            .world_mut()
            .spawn((
                Backdrop,
                Transform::default(),
                Sprite::from_color(Color::WHITE, Vec2::new(350.0, 700.0)),
                RenderLayers::layer(1),
            ))
            .id();
        let overlay = app.world_mut().spawn(RenderLayers::layer(1)).id();
        app.update();
        assert_eq!(
            app.world().get::<RenderLayers>(field).unwrap(),
            &RenderLayers::layer(2)
        );
        assert_eq!(
            app.world().get::<RenderLayers>(backdrop).unwrap(),
            &RenderLayers::layer(1)
        );
        assert_eq!(
            app.world().get::<RenderLayers>(overlay).unwrap(),
            &RenderLayers::layer(3)
        );
        let viewport = app
            .world()
            .get::<Camera>(camera)
            .unwrap()
            .viewport
            .as_ref()
            .unwrap();
        assert_eq!(viewport.physical_position, UVec2::new(10, 83));
        assert_eq!(viewport.physical_size, UVec2::new(325, 445));
        assert_eq!(
            app.world().get::<Sprite>(backdrop).unwrap().custom_size,
            Some(Vec2::new(350.0, 526.0))
        );
        assert_eq!(
            app.world().resource::<Simulation>().settings_scroll_max[1],
            173.0
        );
        app.world_mut().resource_mut::<Simulation>().settings_scroll[1] = 181.0;
        app.update();
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation.y,
            -118.5
        );
        // Exercise the real settings handler at the field's scrolled screen position.
        app.insert_resource(ButtonInput::<MouseButton>::default())
            .insert_resource(music_player::MusicPlayer::new(vec![]))
            .add_systems(Update, select_toolbox_tab_and_settings.before(sync));
        app.world_mut().entity_mut(field).insert(SettingRow(13));
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(126.0, 503.0)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        let thickness = app.world().resource::<Simulation>().thickness;
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(136.0, 503.0)));
        app.update();
        assert!(
            (app.world().resource::<Simulation>().thickness - thickness - 0.05).abs() < 0.00001
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1920.0, 720.0);
        app.update();
        assert_eq!(
            app.world()
                .get::<Camera>(camera)
                .unwrap()
                .viewport
                .as_ref()
                .unwrap()
                .physical_position,
            UVec2::new(330, 83)
        );
        app.world_mut().resource_mut::<Simulation>().show_tools = false;
        app.update();
        assert_eq!(app.world().resource::<Simulation>().settings_scroll[1], 0.0);
        app.world_mut()
            .resource_mut::<Simulation>()
            .toolbox_collapsed = true;
        app.update();
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);
    }
    #[test]
    fn clipping_scroll_coordinates_and_hidden_tool_reservation_agree() {
        let mut s = Simulation::default();
        s.active_tab = ToolboxTab::Physics;
        s.settings_scroll[1] = 120.0;
        assert_eq!(bottom(&s), -176.0);
        assert_eq!(
            content_point(&s, Vec2::new(-514.0, -160.0)),
            Some(Vec2::new(-514.0, -280.0))
        );
        assert!(content_point(&s, Vec2::new(-514.0, -180.0)).is_none());
        s.show_tools = false;
        assert_eq!(bottom(&s), -350.0);
        assert!(content_point(&s, Vec2::new(-514.0, -180.0)).is_some());
        s.toolbox_collapsed = true;
        assert!(content_point(&s, Vec2::new(-514.0, 100.0)).is_none());
    }
}

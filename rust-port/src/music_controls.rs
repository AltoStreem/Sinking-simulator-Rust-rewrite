//! Active renderer adapter for MusicPlayer.drawUI's two rows and power-2 volume slider.
//! Exact ImGui DPI/style dimensions and per-widget text clipping remain pending.
use crate::*;
pub(crate) const LEFT: f32 = -568.0;
pub(crate) const RIGHT: f32 = -354.0;
pub(crate) const VOLUME_Y: f32 = 252.0;
pub(crate) const PLAYING_Y: f32 = 218.0;
const GRAB: f32 = 10.0;
#[derive(Component)]
pub(crate) struct SliderThumb(bool);
#[derive(Component)]
pub(crate) struct VolumeValue;
fn slider_fraction(x: f32) -> f32 {
    let start = LEFT + 2.0 + GRAB * 0.5;
    let end = RIGHT - 2.0 - GRAB * 0.5;
    ((x - start) / (end - start)).clamp(0.0, 1.0)
}
fn thumb_x(fraction: f32) -> f32 {
    let start = LEFT + 2.0 + GRAB * 0.5;
    start + fraction.clamp(0.0, 1.0) * (RIGHT - LEFT - 4.0 - GRAB)
}
pub(crate) fn spawn(commands: &mut Commands, assets: &AssetServer) {
    for (action, name, x, y) in [
        (SettingAction::ToggleShuffle, "shuffle", -615.0, VOLUME_Y),
        (SettingAction::ToggleRepeat, "repeat-off", -585.0, VOLUME_Y),
        (SettingAction::ToggleMusic, "pause", -615.0, PLAYING_Y),
        (SettingAction::NextTrack, "skip-next", -585.0, PLAYING_Y),
    ] {
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            SettingButton(action),
            Sprite::from_color(Color::srgb(0.23, 0.25, 0.40), Vec2::splat(24.0)),
            Transform::from_xyz(x, y, 24.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        let mut sprite = Sprite::from_image(assets.load(format!("icons/music/{name}.png")));
        sprite.custom_size = Some(Vec2::splat(24.0));
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            music_player::MusicIcon(action),
            sprite,
            Transform::from_xyz(x, y, 25.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    for (volume, y, label) in [(true, VOLUME_Y, "Volume"), (false, PLAYING_Y, "Playing")] {
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(RIGHT - LEFT, 26.0)),
            Transform::from_xyz((LEFT + RIGHT) * 0.5, y, 24.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            SliderThumb(volume),
            Sprite::from_color(Color::srgb(0.45, 0.46, 0.47), Vec2::new(GRAB, 22.0)),
            Transform::from_xyz(LEFT, y, 25.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        spawn_page_label(
            commands,
            ToolboxTab::Music,
            label,
            Vec3::new(-327.0, y, 25.5),
            13.0,
        );
        let text = commands
            .spawn((
                ToolboxContent,
                TabPage(ToolboxTab::Music),
                Text2d::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::no_wrap(),
                Transform::from_xyz((LEFT + RIGHT) * 0.5, y, 25.5),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ))
            .id();
        if volume {
            commands.entity(text).insert(VolumeValue);
        } else {
            commands.entity(text).insert(MusicStatus);
        }
    }
}
pub(crate) fn sync(
    simulation: Res<Simulation>,
    player: Res<music_player::MusicPlayer>,
    mut thumbs: Query<(&SliderThumb, &mut Transform)>,
    mut volumes: Query<&mut Text2d, With<VolumeValue>>,
) {
    for (thumb, mut transform) in &mut thumbs {
        let fraction = if thumb.0 {
            simulation.music_volume.clamp(0.0, 1.0).sqrt()
        } else if player.max_progress > 0.0 {
            player.progress / player.max_progress
        } else {
            0.0
        };
        transform.translation.x = thumb_x(fraction);
    }
    for mut text in &mut volumes {
        text.0 = format!("{:.3}", simulation.music_volume);
    }
}
pub(crate) fn handle_sliders(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    upload: Res<ship_upload::SourceShipUpload>,
    mut simulation: ResMut<Simulation>,
    mut player: ResMut<music_player::MusicPlayer>,
    mut dragging: Local<Option<bool>>,
) {
    if !mouse.pressed(MouseButton::Left)
        || simulation.toolbox_collapsed
        || simulation.active_tab != ToolboxTab::Music
        || upload.window_open()
    {
        *dragging = None;
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
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = toolbox_viewport::content_point(&simulation, point).and_then(|point| {
            if !(LEFT..=RIGHT).contains(&point.x) {
                return None;
            }
            if (point.y - VOLUME_Y).abs() <= 13.0 {
                Some(true)
            } else if (point.y - PLAYING_Y).abs() <= 13.0 {
                Some(false)
            } else {
                None
            }
        });
    }
    let Some(volume) = *dragging else {
        return;
    };
    let point = toolbox_viewport::unscroll_point(&simulation, point);
    let fraction = slider_fraction(point.x);
    if volume {
        simulation.music_volume = fraction * fraction;
    } else {
        player.pending_seek = Some(std::time::Duration::from_secs_f32(
            player.max_progress.max(0.0) * fraction,
        ));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_power_two_volume_and_linear_progress_share_grab_bounds() {
        for fraction in [0.0, 0.25, 0.5, 1.0] {
            assert!((slider_fraction(thumb_x(fraction)) - fraction).abs() < 0.00001);
        }
        assert_eq!(slider_fraction((LEFT + RIGHT) * 0.5).powi(2), 0.25);
        assert_eq!(slider_fraction(LEFT - 100.0), 0.0);
        assert_eq!(slider_fraction(RIGHT + 100.0), 1.0);
    }
    #[test]
    fn real_slider_input_maps_volume_seek_and_rejects_unrelated_held_clicks() {
        let mut app = App::new();
        let mut simulation = Simulation::default();
        simulation.active_tab = ToolboxTab::Music;
        let mut player = music_player::MusicPlayer::new(vec![]);
        player.max_progress = 200.0;
        app.insert_resource(simulation)
            .insert_resource(player)
            .insert_resource(ship_upload::SourceShipUpload::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .add_systems(Update, handle_sliders);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        let cursor = Vec2::new(640.0 + (LEFT + RIGHT) * 0.5, 360.0 - VOLUME_Y);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(cursor));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Simulation>().music_volume, 0.25);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(1000.0, cursor.y)));
        app.update();
        assert_eq!(app.world().resource::<Simulation>().music_volume, 1.0);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(cursor.x, 360.0 - PLAYING_Y)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world()
                .resource::<music_player::MusicPlayer>()
                .pending_seek,
            Some(std::time::Duration::from_secs(100))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(640.0, 350.0)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut().resource_mut::<Simulation>().music_volume = 0.81;
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(cursor));
        app.update();
        assert_eq!(app.world().resource::<Simulation>().music_volume, 0.81);
    }
}

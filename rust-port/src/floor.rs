//! Bevy translation of Floor.java's fullscreen half-plane renderer.
//! The source shader paints opaque RGB (.5,.5,.5) at world y <= -seaFloor,
//! after the ship pass and before the sea pass. Bevy can draw that same
//! region as a viewport-clipped rectangle without a separate fragment pass.
use crate::{SEA_LEVEL, Simulation, WorldCamera};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Floor;

pub(crate) fn setup(mut commands: Commands) {
    commands.spawn((
        Floor,
        Sprite::from_color(Color::srgb(0.5, 0.5, 0.5), Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 0.75),
        Visibility::Hidden,
    ));
}

/// Intersect the source half-plane with the visible world bounds.
fn visible_rect(bounds: Rect, floor_height: f32) -> Option<Rect> {
    let top = bounds.max.y.min(floor_height);
    (top > bounds.min.y).then(|| Rect::from_corners(bounds.min, Vec2::new(bounds.max.x, top)))
}

pub(crate) fn update(
    simulation: Res<Simulation>,
    cameras: Query<(&Projection, &Transform), (With<WorldCamera>, Without<Floor>)>,
    mut floors: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<Floor>>,
) {
    let Ok((Projection::Orthographic(camera), transform)) = cameras.single() else {
        return;
    };
    let offset = transform.translation.truncate();
    let bounds = Rect::from_corners(camera.area.min + offset, camera.area.max + offset);
    // The port translates source world y=0 to SEA_LEVEL for its ocean.
    let region = visible_rect(bounds, SEA_LEVEL - simulation.sea_depth);
    for (mut sprite, mut transform, mut visibility) in &mut floors {
        if let Some(region) = region {
            sprite.custom_size = Some(region.size());
            transform.translation = region.center().extend(0.75);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_floor_clips_to_camera_and_depth() {
        let view = Rect::from_corners(Vec2::new(-100.0, -600.0), Vec2::new(100.0, 100.0));
        let floor = visible_rect(view, -400.0).unwrap();
        assert_eq!(floor.min, view.min);
        assert_eq!(floor.max, Vec2::new(100.0, -400.0));
        assert!(visible_rect(view, -700.0).is_none());
        assert_eq!(visible_rect(view, 200.0), Some(view));
    }
}

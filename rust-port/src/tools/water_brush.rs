//! FloodTool/DryTool execute against live GPU water, not delayed readback.
use crate::*;

pub(super) fn apply(
    flood: bool,
    cursor: Vec2,
    radius: f32,
    _snapshot: &GpuShipPhysicsSnapshot,
    physics: &mut GpuShipPhysicsAssets,
    _buffers: &mut Assets<ShaderBuffer>,
) {
    physics.water_brush = Some([cursor.x, cursor.y, radius, if flood { 1.0 } else { -1.0 }]);
}

/// Reference arithmetic from FloodTool.floodPass / DryTool.floodPass.
#[cfg(test)]
fn apply_amount(mut water: Vec4, distance: f32, radius: f32, flood: bool) -> Vec4 {
    if distance < radius {
        let amount = radius - distance;
        water.x = if flood {
            water.x + amount
        } else {
            (water.x - amount).max(0.0)
        };
    }
    water
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_brush_preserves_other_water_channels_and_has_open_boundary() {
        let water = Vec4::new(0.25, 2.0, 3.0, 4.0);
        assert_eq!(apply_amount(water, 1.0, 1.0, true), water);
        assert_eq!(
            apply_amount(water, 0.5, 1.0, true),
            Vec4::new(0.75, 2.0, 3.0, 4.0)
        );
        assert_eq!(
            apply_amount(water, 0.5, 1.0, false),
            Vec4::new(0.0, 2.0, 3.0, 4.0)
        );
    }
}

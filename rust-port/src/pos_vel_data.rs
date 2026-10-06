//! Rust port of SS2 PosVelDataHolder.createPositions.
use bevy::prelude::Vec2;

/// Per-source-pixel rest coordinates consumed by ShipPhysics.
pub struct PosVelDataHolder {
    pub positions: Vec<Vec2>,
}

impl PosVelDataHolder {
    pub fn new(width: usize, height: usize) -> Self {
        let positions = if width == 0 || height == 0 {
            Vec::new()
        } else {
            (0..width * height)
                .map(|index| {
                    let x = index % width;
                    let y = index / width;
                    Vec2::new(
                        x as f32 - width as f32 * 0.5,
                        height as f32 - y as f32 - height as f32 * 0.25,
                    )
                })
                .collect()
        };
        Self { positions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_positions_preserve_pixel_spacing_and_quarter_height_draft() {
        let p = PosVelDataHolder::new(4, 4).positions;
        assert_eq!(p[0], Vec2::new(-2.0, 3.0));
        assert_eq!(p[15], Vec2::new(1.0, 0.0));
        assert_eq!(p[1] - p[0], Vec2::X);
        assert_eq!(p[4] - p[0], Vec2::NEG_Y);
        let tall = PosVelDataHolder::new(2, 8).positions;
        assert_eq!(tall.last().unwrap().y, -1.0);
    }
}

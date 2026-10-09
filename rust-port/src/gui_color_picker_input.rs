//! Bundled colorUtilities and rectangular colorPicker4 InputRGB behavior.
use bevy::prelude::*;

/// Source HSV uses hue in [0,1], not degrees. Preserve its float operation order.
pub(crate) fn rgb_to_hsv(rgb: Vec3) -> Vec3 {
    let (mut r, mut g, mut b) = (rgb.x, rgb.y, rgb.z);
    let mut k = 0.0;
    if g < b {
        std::mem::swap(&mut g, &mut b);
        k = -1.0;
    }
    if r < g {
        std::mem::swap(&mut r, &mut g);
        k = -0.33333334 - k;
    }
    let chroma = r - if g < b { g } else { b };
    Vec3::new(
        (k + (g - b) / (6.0 * chroma + 1.0e-20)).abs(),
        chroma / (r + 1.0e-20),
        r,
    )
}
pub(crate) fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    // glm.mod(x,1) = x - floor(x), then the source divides by 1/6.
    let h = (hsv.x - hsv.x.floor()) / 0.16666667;
    let sector = h as i32;
    let fraction = h - sector as f32;
    let p = hsv.z * (1.0 - hsv.y);
    let q = hsv.z * (1.0 - hsv.y * fraction);
    let t = hsv.z * (1.0 - hsv.y * (1.0 - fraction));
    match sector {
        0 => Vec3::new(hsv.z, t, p),
        1 => Vec3::new(q, hsv.z, p),
        2 => Vec3::new(p, hsv.z, t),
        3 => Vec3::new(p, q, hsv.z),
        4 => Vec3::new(t, p, hsv.z),
        _ => Vec3::new(hsv.z, p, q),
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ColorMemory {
    last_color: Vec3,
    last_hue: f32,
    last_saturation: f32,
}
impl ColorMemory {
    pub(crate) fn read(&self, rgb: Vec3) -> Vec3 {
        let mut hsv = rgb_to_hsv(rgb);
        if rgb == self.last_color {
            if hsv.y == 0.0 {
                hsv.x = self.last_hue;
            }
            if hsv.z == 0.0 {
                hsv.y = self.last_saturation;
            }
        }
        hsv
    }
    pub(crate) fn edit(&mut self, hsv: Vec3) -> Vec3 {
        let adjusted = Vec3::new(
            if hsv.x >= 1.0 { hsv.x - 1.0e-5 } else { hsv.x },
            if hsv.y > 0.0 { hsv.y } else { 1.0e-5 },
            if hsv.z > 0.0 { hsv.z } else { 1.0e-6 },
        );
        let rgb = hsv_to_rgb(adjusted);
        self.last_color = rgb;
        self.last_hue = hsv.x;
        self.last_saturation = hsv.y;
        rgb
    }
}
/// Positions are in the current virtual layout; source native DPI layout is
/// still pending. Convert upward Y to the source's downward pointer distance.
pub(crate) fn pointer_sv(point: Vec2) -> Vec2 {
    Vec2::new(
        ((point.x + 627.0) / 187.0).clamp(0.0, 1.0),
        1.0 - ((50.0 - point.y) / 359.0).clamp(0.0, 1.0),
    )
}
pub(crate) fn pointer_hue(y: f32) -> f32 {
    ((50.0 - y) / 359.0).clamp(0.0, 1.0)
}
pub(crate) fn pointer_alpha(y: f32) -> f32 {
    1.0 - pointer_hue(y)
}
pub(crate) fn sv_marker(saturation: f32, value: f32) -> Vec2 {
    Vec2::new(
        -627.0
            + (saturation.clamp(0.0, 1.0) * 188.0 + 0.5)
                .floor()
                .clamp(2.0, 186.0),
        50.0 - ((1.0 - value.clamp(0.0, 1.0)) * 360.0 + 0.5)
            .floor()
            .clamp(2.0, 358.0),
    )
}
pub(crate) fn alpha_marker(alpha: f32) -> f32 {
    50.0 - ((1.0 - alpha.clamp(0.0, 1.0)) * 360.0 + 0.5).floor()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_mapping_reaches_end_on_last_pixel_and_clamps_drags() {
        assert_eq!(pointer_sv(Vec2::new(-627.0, 50.0)), Vec2::new(0.0, 1.0));
        assert_eq!(pointer_sv(Vec2::new(-440.0, -309.0)), Vec2::new(1.0, 0.0));
        assert_eq!(pointer_sv(Vec2::new(-900.0, 900.0)), Vec2::new(0.0, 1.0));
        assert_eq!(pointer_hue(-309.0), 1.0);
        assert_eq!(pointer_alpha(-309.0), 0.0);
        assert_eq!(pointer_hue(50.0), 0.0);
        assert_eq!(pointer_alpha(50.0), 1.0);
        assert_eq!(pointer_alpha(-129.5), 0.5);
    }
    #[test]
    fn source_cursor_rounding_and_two_pixel_inset_are_distinct_from_pointer_map() {
        assert_eq!(sv_marker(0.0, 1.0), Vec2::new(-625.0, 48.0));
        assert_eq!(sv_marker(1.0, 0.0), Vec2::new(-441.0, -308.0));
        assert_eq!(sv_marker(0.5, 0.5), Vec2::new(-533.0, -130.0));
        assert_eq!(alpha_marker(0.5), -130.0);
        assert_eq!(alpha_marker(191.0 / 255.0), -40.0);
    }
    #[test]
    fn source_conversion_preserves_tiny_chroma_and_picker_epsilon_values() {
        let rgb = Vec3::new(1.0e-8, 0.0, 0.0);
        assert_eq!(rgb_to_hsv(rgb), Vec3::new(0.0, 1.0, 1.0e-8));
        let mut memory = ColorMemory::default();
        let rgb = memory.edit(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(rgb, hsv_to_rgb(Vec3::new(1.0 - 1.0e-5, 1.0e-5, 1.0e-6)));
        assert!(rgb.min_element() > 0.0 && rgb.max_element() == 1.0e-6);
    }
    #[test]
    fn memory_restores_only_matching_gray_or_black_source_color() {
        let memory = ColorMemory {
            last_color: Vec3::ZERO,
            last_hue: 0.75,
            last_saturation: 0.6,
        };
        assert_eq!(memory.read(Vec3::ZERO), Vec3::new(0.75, 0.6, 0.0));
        assert_eq!(memory.read(Vec3::splat(0.5)), Vec3::new(0.0, 0.0, 0.5));
        let memory = ColorMemory {
            last_color: Vec3::splat(0.5),
            ..memory
        };
        assert_eq!(memory.read(Vec3::splat(0.5)), Vec3::new(0.75, 0.0, 0.5));
    }
}

//! Shared callback arithmetic from BreakTool/FloodTool/DryTool.java.
use super::tool::BrushActivation;
use crate::window::SourceWindow;
use bevy::math::Mat4;
use std::rc::Rc;

pub(crate) trait BrushUniforms {
    fn vec2(&mut self, preview: bool, name: &str, value: [f32; 2]);
    fn matrix(&mut self, preview: bool, name: &str, transpose: bool, value: &Mat4);
}
pub(crate) struct BrushCallbacks<B> {
    pub activation: BrushActivation,
    pub uniforms: B,
    pub camera_window: Rc<SourceWindow>,
}
impl<B: BrushUniforms> BrushCallbacks<B> {
    pub fn new(uniforms: B, camera_window: Rc<SourceWindow>) -> Self {
        Self {
            activation: BrushActivation::default(),
            uniforms,
            camera_window,
        }
    }
    pub fn on_camera_change(&mut self, matrix: Mat4) {
        let inverse = matrix.inverse();
        self.uniforms.matrix(true, "u_inv", false, &inverse);
        self.uniforms.matrix(false, "u_inv", false, &inverse);
    }
    pub fn on_cursor(&mut self, blocked: bool, x: f64, y: f64) -> bool {
        let mouse = [x as f32, y as f32];
        self.uniforms.vec2(true, "u_mouse", mouse);
        self.uniforms.vec2(false, "u_mouse", mouse);
        blocked
    }
    pub fn on_size(&mut self, blocked: bool, height: i32, width: i32, is_break: bool) -> bool {
        // Break reads screen size before framebuffer size; Flood/Dry use event dimensions.
        let screen = if is_break {
            self.camera_window.screen_size()
        } else {
            [width, height]
        };
        let framebuffer = self.camera_window.framebuffer_size();
        size_uniforms(&mut self.uniforms, screen, framebuffer, [width, height]);
        blocked
    }
}
fn size_uniforms(
    uniforms: &mut impl BrushUniforms,
    screen: [i32; 2],
    framebuffer: [i32; 2],
    event: [i32; 2],
) {
    let scale = [
        framebuffer[0] as f32 / screen[0] as f32,
        framebuffer[1] as f32 / screen[1] as f32,
    ];
    uniforms.vec2(true, "u_scale", [1.0 / scale[0], 1.0 / scale[1]]);
    let window = [1.0 / event[0] as f32, 1.0 / event[1] as f32];
    uniforms.vec2(true, "u_window", window);
    uniforms.vec2(false, "u_window", window);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct UniformLog(Vec<(bool, String, [f32; 2])>);
    impl BrushUniforms for UniformLog {
        fn vec2(&mut self, preview: bool, name: &str, value: [f32; 2]) {
            self.0.push((preview, name.into(), value));
        }
        fn matrix(&mut self, _: bool, _: &str, _: bool, _: &Mat4) {
            panic!("not a size operation")
        }
    }
    #[test]
    fn source_brush_size_keeps_break_screen_distinction_and_ieee_values() {
        let mut log = UniformLog::default();
        size_uniforms(&mut log, [100, 50], [400, 200], [200, 100]);
        assert_eq!(
            log.0,
            [
                (true, "u_scale".into(), [0.25, 0.25]),
                (true, "u_window".into(), [0.005, 0.01]),
                (false, "u_window".into(), [0.005, 0.01])
            ]
        );
        log.0.clear();
        size_uniforms(&mut log, [200, 100], [400, 200], [200, 100]);
        assert_eq!(log.0[0].2, [0.5, 0.5]);
        log.0.clear();
        size_uniforms(&mut log, [0, 0], [0, 100], [0, 0]);
        assert!(log.0[0].2[0].is_nan());
        assert_eq!(log.0[0].2[1], 0.0);
        assert!(log.0[1].2[0].is_infinite());
    }
    #[test]
    fn source_brush_blocked_or_modified_press_does_not_cancel_held_click() {
        let mut activation = BrushActivation::default();
        assert!(activation.on_mouse_button(false, 0, 1, 0));
        assert!(activation.active);
        assert!(activation.on_mouse_button(true, 0, 1, 0));
        assert!(activation.active);
        assert!(!activation.on_mouse_button(false, 0, 1, 1));
        assert!(activation.active);
        assert!(!activation.on_mouse_button(false, 0, 2, 0));
        assert!(activation.active);
        assert!(!activation.on_mouse_button(false, 1, 0, 0));
        assert!(activation.active);
        assert!(activation.on_mouse_button(true, 0, 0, 0));
        assert!(!activation.active);
    }
}

//! Toolbox$render$3.class: retained size-constraint callback from the original Toolbox window.
use bevy::math::Vec2;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub trait SizeCallbackData {
    fn position(&mut self) -> Vec2;
    fn desired_size(&mut self) -> Rc<RefCell<Vec2>>;
}
pub trait SizeCallbackBackend {
    fn display_size(&mut self) -> Vec2;
    fn gui_scale(&mut self) -> f32;
}
pub struct SizeConstraintCallback {
    pub padding: f32,
    pub tools_height: Rc<Cell<f32>>,
}
// glm's component max calls Java Math.max: either NaN propagates and +0 wins over -0.
fn java_max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() && b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else if a >= b {
        a
    } else {
        b
    }
}
impl SizeConstraintCallback {
    pub fn invoke(&self, data: &mut impl SizeCallbackData, backend: &mut impl SizeCallbackBackend) {
        let display = backend.display_size();
        let position = data.position();
        let available = (display - position) - Vec2::splat(self.padding);
        let available = available - Vec2::new(0.0, self.tools_height.get());
        // Vec2's synthetic default mask 2 copies x into y; both minimum components are 50*scale.
        let minimum = 50.0 * backend.gui_scale();
        let maximum = Vec2::new(
            java_max(available.x, minimum),
            java_max(available.y, minimum),
        );
        let x = data.desired_size().borrow().x;
        if x > maximum.x {
            data.desired_size().borrow_mut().x = maximum.x;
        }
        let y = data.desired_size().borrow().y;
        if y > maximum.y {
            data.desired_size().borrow_mut().y = maximum.y;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        display: Vec2,
        position: Vec2,
        scale: f32,
        desired: Rc<RefCell<Vec2>>,
        reads: usize,
    }
    impl SizeCallbackData for Fixture {
        fn position(&mut self) -> Vec2 {
            self.position
        }
        fn desired_size(&mut self) -> Rc<RefCell<Vec2>> {
            self.reads += 1;
            self.desired.clone()
        }
    }
    struct Backend {
        display: Vec2,
        scale: f32,
    }
    impl SizeCallbackBackend for Backend {
        fn display_size(&mut self) -> Vec2 {
            self.display
        }
        fn gui_scale(&mut self) -> f32 {
            self.scale
        }
    }
    #[test]
    fn toolbox_constraints_reserve_live_tool_height_and_clamp_only_oversize_components() {
        let height = Rc::new(Cell::new(90.0));
        let callback = SizeConstraintCallback {
            padding: 10.0,
            tools_height: height.clone(),
        };
        let mut f = Fixture {
            display: Vec2::new(800.0, 600.0),
            position: Vec2::new(20.0, 30.0),
            scale: 2.0,
            desired: Rc::new(RefCell::new(Vec2::new(900.0, 700.0))),
            reads: 0,
        };
        let mut b = Backend {
            display: f.display,
            scale: f.scale,
        };
        callback.invoke(&mut f, &mut b);
        assert_eq!(*f.desired.borrow(), Vec2::new(770.0, 470.0));
        assert_eq!(f.reads, 4);
        height.set(580.0);
        *f.desired.borrow_mut() = Vec2::new(25.0, 900.0);
        callback.invoke(&mut f, &mut b);
        assert_eq!(*f.desired.borrow(), Vec2::new(25.0, 100.0));
        assert_eq!(f.reads, 7);
        // NaN max bounds leave desired components untouched: Java comparisons with NaN are false.
        b.scale = f32::NAN;
        callback.invoke(&mut f, &mut b);
        assert_eq!(*f.desired.borrow(), Vec2::new(25.0, 100.0));
        assert_eq!(f.reads, 9);
    }
    #[test]
    fn toolbox_max_retains_java_nan_and_signed_zero_semantics() {
        assert!(java_max(1.0, f32::NAN).is_nan());
        assert!(java_max(f32::NAN, 1.0).is_nan());
        assert_eq!(java_max(-0.0, 0.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(java_max(0.0, -0.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(java_max(-0.0, -0.0).to_bits(), (-0.0f32).to_bits());
    }
}

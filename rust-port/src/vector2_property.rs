//! Translation of Vector2Property.java. Borrowed component views are read-only
//! views into the shared backing array, starting at source positions 0 and 1.
#![allow(dead_code)]
use crate::backed_property::{BackedProperty, java_float_equal};
use bevy::prelude::Vec2;
use std::ops::{Deref, DerefMut};
pub struct Vector2Property(BackedProperty<Vec2, [f32; 2]>);
impl Vector2Property {
    pub fn new(default: Vec2) -> Self {
        Self(BackedProperty::new(
            default.to_array(),
            Vec2::ZERO,
            |b| Vec2::from_array(*b),
            |a, b| java_float_equal(&a.x, &b.x) && java_float_equal(&a.y, &b.y),
        ))
    }
    pub fn x_buffer(&self) -> &[f32] {
        &self.buffer()[..]
    }
    pub fn y_buffer(&self) -> &[f32] {
        &self.buffer()[1..]
    }
}
impl Default for Vector2Property {
    fn default() -> Self {
        Self::new(Vec2::ZERO)
    }
}
impl Deref for Vector2Property {
    type Target = BackedProperty<Vec2, [f32; 2]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vector2Property {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_views_share_backing_storage() {
        let mut p = Vector2Property::new(Vec2::new(2.0, 3.0));
        assert_eq!(*p.old(), Vec2::ZERO);
        assert_eq!(p.x_buffer().as_ptr(), p.buffer().as_ptr());
        assert_eq!(p.y_buffer().as_ptr(), p.buffer()[1..].as_ptr());
        p.buffer_mut()[1] = 4.0;
        assert_eq!(p.y_buffer(), &[4.0]);
        p.before_events();
        assert_eq!(*p.old(), Vec2::new(2.0, 4.0));
    }
}

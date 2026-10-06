//! Translation of FloatProperty.java; defaults retain old=0 until before_events.
#![allow(dead_code)]
use crate::backed_property::{BackedProperty, java_float_equal};
use std::ops::{Deref, DerefMut};
pub struct FloatProperty(BackedProperty<f32, [f32; 1]>);
impl FloatProperty {
    pub fn new(default: f32) -> Self {
        Self(BackedProperty::new(
            [default],
            0.0,
            |b| b[0],
            java_float_equal,
        ))
    }
}
impl Default for FloatProperty {
    fn default() -> Self {
        Self::new(0.0)
    }
}
impl Deref for FloatProperty {
    type Target = BackedProperty<f32, [f32; 1]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FloatProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_observed_on_event_update() {
        let mut p = FloatProperty::new(2.5);
        assert_eq!(*p.old(), 0.0);
        assert_eq!(p.get(), 2.5);
        p.before_events();
        assert_eq!(*p.old(), 2.5);
        p.buffer_mut()[0] = -0.0;
        p.before_events();
        assert_eq!(p.old().to_bits(), (-0.0f32).to_bits());
    }
}

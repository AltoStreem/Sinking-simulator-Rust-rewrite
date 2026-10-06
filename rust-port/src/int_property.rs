//! Translation of IntProperty.java.
#![allow(dead_code)]
use crate::backed_property::BackedProperty;
use std::ops::{Deref, DerefMut};
pub struct IntProperty(BackedProperty<i32, [i32; 1]>);
impl IntProperty {
    pub fn new(default: i32) -> Self {
        Self(BackedProperty::new([default], 0, |b| b[0], |a, b| a == b))
    }
}
impl Default for IntProperty {
    fn default() -> Self {
        Self::new(0)
    }
}
impl Deref for IntProperty {
    type Target = BackedProperty<i32, [i32; 1]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for IntProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_old_is_zero() {
        let mut p = IntProperty::new(50);
        assert_eq!(*p.old(), 0);
        assert_eq!(p.get(), 50);
        p.before_events();
        assert_eq!(*p.old(), 50);
    }
}

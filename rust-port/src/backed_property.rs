//! Translation of engine/util/properties/BackedProperty.java.
//! Call `before_events` at the source window's before-events boundary. Automatic
//! window registration is pending the Window/Resource lifecycle conversion.
#![allow(dead_code)]

type Listener<T> = (u64, Box<dyn FnMut(&T, &T) + Send + Sync>);

pub struct BackedProperty<T, B> {
    buffer: B,
    old: T,
    read: fn(&B) -> T,
    equal: fn(&T, &T) -> bool,
    listeners: Vec<Listener<T>>,
    next_id: u64,
}

impl<T, B> BackedProperty<T, B> {
    pub fn new(buffer: B, old: T, read: fn(&B) -> T, equal: fn(&T, &T) -> bool) -> Self {
        Self {
            buffer,
            old,
            read,
            equal,
            listeners: Vec::new(),
            next_id: 0,
        }
    }
    pub fn get(&self) -> T {
        (self.read)(&self.buffer)
    }
    pub fn old(&self) -> &T {
        &self.old
    }
    pub fn buffer(&self) -> &B {
        &self.buffer
    }
    pub fn buffer_mut(&mut self) -> &mut B {
        &mut self.buffer
    }
    pub fn add_change_listener(
        &mut self,
        mut listener: impl FnMut(&T, &T) + Send + Sync + 'static,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        // Source invokes registration even when old and current are equal.
        listener(&self.old, &self.get());
        self.listeners.push((id, Box::new(listener)));
        id
    }
    pub fn remove_change_listener(&mut self, id: u64) {
        if let Some(index) = self.listeners.iter().position(|entry| entry.0 == id) {
            let _ = self.listeners.remove(index);
        }
    }
    pub fn before_events(&mut self) {
        let current = self.get();
        if !(self.equal)(&current, &self.old) {
            for (_, listener) in &mut self.listeners {
                listener(&self.old, &current);
            }
            self.old = current;
        }
    }
}

/// Java Float.equals/JOML component equality canonicalizes NaNs and distinguishes
/// signed zero; Rust floating-point == has neither behavior.
pub fn java_float_equal(a: &f32, b: &f32) -> bool {
    let bits = |v: f32| if v.is_nan() { 0x7fc00000 } else { v.to_bits() };
    bits(*a) == bits(*b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn registration_and_deferred_coalesced_changes() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut property = BackedProperty::new([5i32], 0, |b| b[0], |a, b| a == b);
        let output = events.clone();
        let id = property.add_change_listener(move |a, b| output.lock().unwrap().push((*a, *b)));
        property.buffer_mut()[0] = 7;
        property.buffer_mut()[0] = 9;
        assert_eq!(*events.lock().unwrap(), vec![(0, 5)]);
        property.before_events();
        property.before_events();
        assert_eq!(*events.lock().unwrap(), vec![(0, 5), (0, 9)]);
        property.remove_change_listener(id);
        property.buffer_mut()[0] = 10;
        property.before_events();
        assert_eq!(*property.old(), 10);
        assert_eq!(events.lock().unwrap().len(), 2);
    }
    #[test]
    fn listeners_follow_registration_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut property = BackedProperty::new([0], 0, |b| b[0], |a, b| a == b);
        for index in 0..3 {
            let output = events.clone();
            property.add_change_listener(move |_, _| output.lock().unwrap().push(index));
        }
        events.lock().unwrap().clear();
        property.buffer_mut()[0] = 1;
        property.before_events();
        assert_eq!(*events.lock().unwrap(), vec![0, 1, 2]);
    }
    #[test]
    fn boxed_float_comparison() {
        assert!(java_float_equal(&f32::NAN, &f32::from_bits(0xffc00001)));
        assert!(!java_float_equal(&0.0, &-0.0));
    }
}

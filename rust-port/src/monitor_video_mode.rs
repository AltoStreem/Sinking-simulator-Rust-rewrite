//! Monitor.VideoMode: copied channel values and shared mutable Vector2i size.
#![allow(dead_code)]
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug)]
pub(crate) struct VideoMode {
    pub size: Arc<Mutex<[i32; 2]>>,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub refresh: i32,
}
impl VideoMode {
    pub fn new(size: Arc<Mutex<[i32; 2]>>, r: i32, g: i32, b: i32, refresh: i32) -> Self {
        Self {
            size,
            r,
            g,
            b,
            refresh,
        }
    }
    pub fn copy_with(
        &self,
        size: Option<Arc<Mutex<[i32; 2]>>>,
        r: Option<i32>,
        g: Option<i32>,
        b: Option<i32>,
        refresh: Option<i32>,
    ) -> Self {
        Self::new(
            size.unwrap_or_else(|| self.size.clone()),
            r.unwrap_or(self.r),
            g.unwrap_or(self.g),
            b.unwrap_or(self.b),
            refresh.unwrap_or(self.refresh),
        )
    }
    /// Kotlin `Monitor.VideoMode.toString()` with the original JOML Vector2i spelling.
    pub fn source_to_string(&self) -> String {
        let [width, height] = *self.size.lock().unwrap();
        format!(
            "VideoMode(size=({width} {height}), r={}, g={}, b={}, refresh={})",
            self.r, self.g, self.b, self.refresh,
        )
    }
    pub fn java_hash(&self) -> i32 {
        let [x, y] = *self.size.lock().unwrap();
        let mut hash = 31_i32.wrapping_add(x).wrapping_mul(31).wrapping_add(y);
        for value in [self.r, self.g, self.b, self.refresh] {
            hash = hash.wrapping_mul(31).wrapping_add(value);
        }
        hash
    }
}
impl PartialEq for VideoMode {
    fn eq(&self, other: &Self) -> bool {
        let a = *self.size.lock().unwrap();
        let b = *other.size.lock().unwrap();
        a == b
            && self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && self.refresh == other.refresh
    }
}
impl Eq for VideoMode {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_copy_shares_size_and_equality_hash_read_live_values() {
        let mode = VideoMode::new(Arc::new(Mutex::new([800, 600])), 8, 8, 8, 60);
        let copy = mode.copy_with(None, None, None, None, None);
        assert_eq!(mode, copy);
        assert!(Arc::ptr_eq(&mode.size, &copy.size));
        let hash = mode.java_hash();
        *copy.size.lock().unwrap() = [1920, 1080];
        assert_ne!(mode.java_hash(), hash);
        assert_eq!(mode.java_hash(), copy.java_hash());
        assert_ne!(mode, mode.copy_with(None, Some(10), None, None, None));
    }
}

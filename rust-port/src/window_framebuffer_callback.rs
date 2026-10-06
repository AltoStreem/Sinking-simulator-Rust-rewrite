//! Window$2$16.class: constructor-installed framebuffer resize callback.
#![allow(dead_code)]
pub(crate) trait ViewportBackend {
    fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32);
}
pub(crate) fn invoke(backend: &mut dyn ViewportBackend, width: i32, height: i32) {
    backend.viewport(0, 0, width, height);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Backend(Vec<[i32; 4]>);
    impl ViewportBackend for Backend {
        fn viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
            self.0.push([x, y, w, h]);
        }
    }
    #[test]
    fn forwards_zero_origin_and_dimensions_without_clamping() {
        let mut backend = Backend::default();
        invoke(&mut backend, 800, 600);
        invoke(&mut backend, 0, 0);
        invoke(&mut backend, -1, 3);
        assert_eq!(backend.0, [[0, 0, 800, 600], [0, 0, 0, 0], [0, 0, -1, 3]]);
    }
}

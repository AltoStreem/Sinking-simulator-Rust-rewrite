//! Camera2D.java's matrix operations, sixteen-pixel initial scale and callbacks.
use bevy::prelude::*;
type Callback = Box<dyn Fn(&Camera2D) + Send + Sync>;
pub(crate) struct Camera2D {
    pub matrix: Mat4,
    pub size: UVec2,
    pub scale: f32,
    callbacks: Vec<(u64, Callback)>,
    next_callback: u64,
}
impl Camera2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self::from_view(UVec2::new(width, height), Vec2::ZERO, 16.0)
    }
    /// Adapter constructor for the port's existing normalized world coordinates.
    /// The source-native constructor above always starts at sixteen pixels/unit.
    pub fn from_view(size: UVec2, center: Vec2, pixels_per_world_unit: f32) -> Self {
        assert!(size.x > 0 && size.y > 0 && pixels_per_world_unit > 0.0);
        let half = size.as_vec2() * 0.5;
        let matrix = Mat4::orthographic_rh_gl(-half.x, half.x, -half.y, half.y, -1.0, 1.0)
            * Mat4::from_scale(Vec3::new(pixels_per_world_unit, pixels_per_world_unit, 1.0))
            * Mat4::from_translation((-center).extend(0.0));
        Self {
            matrix,
            size,
            scale: pixels_per_world_unit,
            callbacks: Vec::new(),
            next_callback: 0,
        }
    }
    fn on_update(&self) {
        for (_, callback) in &self.callbacks {
            callback(self);
        }
    }
    pub fn resize(&mut self, current: UVec2) {
        assert!(current.x > 0 && current.y > 0);
        let relative = self.matrix.inverse() * Vec4::new(-1.0, 1.0, 0.0, 1.0);
        let ratio = self.size.as_vec2() / current.as_vec2();
        self.around(Vec3::new(ratio.x, ratio.y, 1.0), relative.truncate());
        self.size = current;
        self.on_update();
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.matrix *= Mat4::from_translation(Vec3::new(x / self.scale, y / self.scale, 0.0));
        self.on_update();
    }
    fn around(&mut self, multiplier: Vec3, relative: Vec3) {
        self.matrix = self.matrix
            * Mat4::from_translation(relative)
            * Mat4::from_scale(multiplier)
            * Mat4::from_translation(-relative);
    }
    pub fn zoom(&mut self, multiplier: f32, relative: Vec4) {
        let relative = self.matrix.inverse() * relative;
        self.around(Vec3::new(multiplier, multiplier, 1.0), relative.truncate());
        self.scale *= multiplier;
        self.on_update();
    }
    pub fn world_at(&self, clip: Vec2) -> Vec2 {
        (self.matrix.inverse() * clip.extend(0.0).extend(1.0))
            .truncate()
            .truncate()
    }
    pub fn add_camera_callback(
        &mut self,
        callback: impl Fn(&Camera2D) + Send + Sync + 'static,
    ) -> u64 {
        let id = self.next_callback;
        self.next_callback += 1;
        self.callbacks.push((id, Box::new(callback)));
        (self.callbacks.last().unwrap().1)(self);
        id
    }
    pub fn remove_camera_callback(&mut self, id: u64) {
        self.callbacks.retain(|(current, _)| *current != id);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: Vec2, b: Vec2) {
        assert!(a.distance(b) < 0.0002, "{a:?} != {b:?}");
    }
    #[test]
    fn source_camera_starts_at_sixteen_pixels_per_world_unit() {
        let camera = Camera2D::new(1600, 800);
        near(camera.world_at(Vec2::new(1.0, 1.0)), Vec2::new(50.0, 25.0));
        assert_eq!(camera.scale, 16.0);
    }
    #[test]
    fn source_camera_zoom_keeps_relative_clip_point_fixed() {
        let mut camera = Camera2D::new(1600, 800);
        let pointer = Vec2::new(0.6, -0.4);
        let before = camera.world_at(pointer);
        camera.zoom(2.0, pointer.extend(0.0).extend(1.0));
        near(camera.world_at(pointer), before);
        assert_eq!(camera.scale, 32.0);
    }
    #[test]
    fn source_camera_resize_keeps_top_left_and_pixel_scale() {
        let mut camera = Camera2D::new(1600, 800);
        camera.translate(30.0, -40.0);
        let anchor = camera.world_at(Vec2::new(-1.0, 1.0));
        camera.resize(UVec2::new(2000, 1000));
        near(camera.world_at(Vec2::new(-1.0, 1.0)), anchor);
        assert_eq!(camera.scale, 16.0);
        near(
            camera.world_at(Vec2::new(1.0, 1.0)) - anchor,
            Vec2::new(125.0, 0.0),
        );
    }
    #[test]
    fn source_camera_callbacks_fire_on_registration_and_mutation_then_remove() {
        use std::sync::{Arc, Mutex};
        let count = Arc::new(Mutex::new(0));
        let observed = count.clone();
        let mut camera = Camera2D::new(800, 600);
        let id = camera.add_camera_callback(move |_| *observed.lock().unwrap() += 1);
        camera.translate(16.0, 0.0);
        camera.zoom(2.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
        camera.resize(UVec2::new(1000, 700));
        assert_eq!(*count.lock().unwrap(), 4);
        camera.remove_camera_callback(id);
        camera.translate(0.0, 16.0);
        assert_eq!(*count.lock().unwrap(), 4);
    }
}

/// Source object path, separate from the Send/Sync Bevy camera adapter above.
/// Returned matrix/size references retain identity and may be mutated by callers.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CameraCallbackKey {
    pub receiver: u64,
    pub owner: &'static str,
    pub name: &'static str,
    pub signature: &'static str,
}
#[derive(Clone)]
pub(crate) struct SourceCameraCallback {
    pub key: CameraCallbackKey,
    pub invoke: std::rc::Rc<dyn Fn(&SourceCamera2D)>,
}
pub(crate) struct SourceCamera2D {
    pub matrix: std::rc::Rc<std::cell::RefCell<Mat4>>,
    pub size: std::rc::Rc<std::cell::RefCell<[i32; 2]>>,
    scale: std::cell::Cell<f32>,
    rel: std::cell::RefCell<Vec4>,
    callbacks: std::cell::RefCell<Vec<SourceCameraCallback>>,
    callback_generation: std::cell::Cell<u64>,
}
impl SourceCamera2D {
    pub fn new(width: i32, height: i32) -> Self {
        // Matrix4f.ortho2D uses z=-1 and no zero-size guard.
        let xmid = width as f32 * 0.5;
        let ymid = height as f32 * 0.5;
        let mut matrix = Mat4::from_cols_array(&[
            2.0 / (xmid - -xmid),
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / (ymid - -ymid),
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0,
            0.0,
            (-xmid + xmid) / (-xmid - xmid),
            (-ymid + ymid) / (-ymid - ymid),
            0.0,
            1.0,
        ]);
        matrix.x_axis *= 16.0;
        matrix.y_axis *= 16.0;
        Self {
            matrix: std::rc::Rc::new(std::cell::RefCell::new(matrix)),
            size: std::rc::Rc::new(std::cell::RefCell::new([width, height])),
            scale: std::cell::Cell::new(16.0),
            rel: std::cell::RefCell::new(Vec4::ZERO),
            callbacks: std::cell::RefCell::new(vec![]),
            callback_generation: std::cell::Cell::new(0),
        }
    }
    pub fn scale_value(&self) -> f32 {
        self.scale.get()
    }
    pub fn add_camera_callback(&self, callback: SourceCameraCallback) {
        self.callbacks.borrow_mut().push(callback.clone());
        self.callback_generation
            .set(self.callback_generation.get().wrapping_add(1));
        (callback.invoke)(self);
    }
    pub fn remove_camera_callback(&self, key: &CameraCallbackKey) {
        let mut callbacks = self.callbacks.borrow_mut();
        if let Some(index) = callbacks.iter().position(|callback| &callback.key == key) {
            callbacks.remove(index);
            self.callback_generation
                .set(self.callback_generation.get().wrapping_add(1));
        }
    }
    fn on_update(&self) {
        let generation = self.callback_generation.get();
        let mut index = 0;
        loop {
            // LinkedList.hasNext checks current size; next checks modification count.
            if index >= self.callbacks.borrow().len() {
                break;
            }
            assert_eq!(
                self.callback_generation.get(),
                generation,
                "ConcurrentModificationException"
            );
            let callback = self.callbacks.borrow()[index].clone();
            index += 1;
            (callback.invoke)(self);
        }
    }
    fn scale_around(&self, multiplier: Vec3, relative: Vec4) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix = *matrix
            * Mat4::from_translation(relative.truncate())
            * Mat4::from_scale(multiplier)
            * Mat4::from_translation(-relative.truncate());
    }
    pub fn resize(&self, current: [i32; 2]) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix = matrix.inverse();
        let relative = *matrix * Vec4::new(-1.0, 1.0, 0.0, 1.0);
        *self.rel.borrow_mut() = relative;
        *matrix = matrix.inverse();
        drop(matrix);
        let size = *self.size.borrow();
        self.scale_around(
            Vec3::new(
                size[0] as f32 / current[0] as f32,
                size[1] as f32 / current[1] as f32,
                1.0,
            ),
            relative,
        );
        *self.size.borrow_mut() = current;
        self.on_update();
    }
    pub fn translate(&self, x: f32, y: f32) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix *=
            Mat4::from_translation(Vec3::new(x / self.scale.get(), y / self.scale.get(), 0.0));
        drop(matrix);
        self.on_update();
    }
    /// JOML transform(relative) mutates the supplied vector, visible to its caller.
    pub fn scale(&self, multiplier: f32, relative: &mut Vec4) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix = matrix.inverse();
        *relative = *matrix * *relative;
        *matrix = matrix.inverse();
        drop(matrix);
        self.scale_around(Vec3::new(multiplier, multiplier, 1.0), *relative);
        self.scale.set(self.scale.get() * multiplier);
        self.on_update();
    }
}
#[cfg(test)]
mod source_object_tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    fn key(receiver: u64) -> CameraCallbackKey {
        CameraCallbackKey {
            receiver,
            owner: "Tool",
            name: "onCamChange",
            signature: "onCamChange(Camera2D)V",
        }
    }
    #[test]
    fn source_camera_references_callback_equality_and_mutated_scale_argument() {
        let camera = SourceCamera2D::new(1600, 800);
        let matrix = camera.matrix.clone();
        let size = camera.size.clone();
        let events = Rc::new(RefCell::new(vec![]));
        for id in [1, 1, 2] {
            let events = events.clone();
            camera.add_camera_callback(SourceCameraCallback {
                key: key(id),
                invoke: Rc::new(move |_| events.borrow_mut().push(id)),
            });
        }
        assert_eq!(*events.borrow(), [1, 1, 2]);
        camera.remove_camera_callback(&key(1)); // remove first equal callback only.
        events.borrow_mut().clear();
        camera.translate(16.0, 0.0);
        assert_eq!(*events.borrow(), [1, 2]);
        let mut relative = Vec4::new(0.6, -0.4, 0.0, 1.0);
        let expected = camera.matrix.borrow().inverse() * relative;
        camera.scale(2.0, &mut relative);
        assert!((relative - expected).length() < 0.0002);
        assert_eq!(camera.scale_value(), 32.0);
        camera.resize([800, 400]);
        assert!(Rc::ptr_eq(&matrix, &camera.matrix));
        assert!(Rc::ptr_eq(&size, &camera.size));
        assert_eq!(*size.borrow(), [800, 400]);
        matrix.borrow_mut().w_axis.x = 12.0;
        assert_eq!(camera.matrix.borrow().w_axis.x, 12.0);
    }
    #[test]
    fn source_camera_callback_added_before_initial_invoke_and_iterator_modification() {
        let camera = SourceCamera2D::new(800, 400);
        let count = Rc::new(Cell::new(0));
        let count2 = count.clone();
        let callback = SourceCameraCallback {
            key: key(1),
            invoke: Rc::new(move |camera| {
                count2.set(count2.get() + 1);
                camera.remove_camera_callback(&key(1));
            }),
        };
        camera.add_camera_callback(callback);
        assert_eq!(count.get(), 1);
        camera.translate(1.0, 2.0);
        assert_eq!(count.get(), 1);
        let updating = Rc::new(Cell::new(false));
        let updating2 = updating.clone();
        camera.add_camera_callback(SourceCameraCallback {
            key: key(2),
            invoke: Rc::new(move |camera| {
                if updating2.get() {
                    camera.add_camera_callback(SourceCameraCallback {
                        key: key(3),
                        invoke: Rc::new(|_| {}),
                    });
                }
            }),
        });
        updating.set(true);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| camera.translate(0.0, 0.0)))
                .is_err()
        );
    }
}

#[cfg(test)]
mod source_lifecycle_tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    #[test]
    fn source_tool_method_references_match_and_deferred_free_releases_camera_receiver() {
        use crate::tools::{
            break_tool_camera_reference as registered, break_tool_free_camera_reference as removed,
        };
        let key = registered::key(17);
        assert!(key == removed::key(17));
        assert!(key != removed::key(18));
        assert!(key != crate::tools::flood_tool_camera_reference::key(17));
        assert!(
            crate::tools::flood_tool_camera_reference::key(17)
                == crate::tools::flood_tool_free_camera_reference::key(17)
        );
        assert!(
            crate::tools::dry_tool_camera_reference::key(17)
                == crate::tools::dry_tool_free_camera_reference::key(17)
        );
        let runtime = crate::resource::ResourceRuntime::default();
        let camera = Rc::new(SourceCamera2D::new(800, 400));
        let receiver = Rc::new(Cell::new(0));
        let weak = Rc::downgrade(&receiver);
        let callback_receiver = receiver.clone();
        let cleanup_camera = camera.clone();
        let resource = runtime.allocate_local(&[], move || {
            cleanup_camera.remove_camera_callback(&removed::key(17))
        });
        camera.add_camera_callback(SourceCameraCallback {
            key,
            invoke: Rc::new(move |_| callback_receiver.set(callback_receiver.get() + 1)),
        });
        assert_eq!(receiver.get(), 1);
        drop(receiver);
        camera.translate(0.0, 0.0);
        assert_eq!(weak.upgrade().unwrap().get(), 2);
        resource.close();
        camera.translate(0.0, 0.0);
        assert_eq!(weak.upgrade().unwrap().get(), 3);
        runtime.run_main();
        assert!(weak.upgrade().is_none());
        camera.translate(0.0, 0.0);
    }
    #[test]
    fn source_camera_keeps_signed_dimensions_and_zero_constructor_ieee_results() {
        let zero = SourceCamera2D::new(0, 0);
        assert_eq!(*zero.size.borrow(), [0, 0]);
        assert!(zero.matrix.borrow().x_axis.x.is_infinite());
        assert_eq!(zero.matrix.borrow().x_axis.y, 0.0);
        let negative = SourceCamera2D::new(-800, 400);
        assert_eq!(*negative.size.borrow(), [-800, 400]);
        assert_eq!(negative.matrix.borrow().x_axis.x, -0.04);
    }
}

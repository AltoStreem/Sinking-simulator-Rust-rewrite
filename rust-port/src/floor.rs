//! Bevy translation of Floor.java's fullscreen half-plane renderer.
//! The source shader paints opaque RGB (.5,.5,.5) at world y <= -seaFloor,
//! after the ship pass and before the sea pass. Bevy can draw that same
//! region as a viewport-clipped rectangle without a separate fragment pass.
use crate::{SEA_LEVEL, Simulation};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Floor;

pub(crate) use crate::floor_companion::FloorShader;

pub(crate) fn setup(mut commands: Commands) {
    commands.spawn((
        Floor,
        // Numeric source RGB is written to the UNORM scene target. Sprite
        // extraction must not decode .5 to linear .214 before that write.
        Sprite::from_color(Color::linear_rgb(0.5, 0.5, 0.5), Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 0.75),
        Visibility::Hidden,
    ));
}

/// Intersect the source half-plane with the visible world bounds.
fn visible_rect(bounds: Rect, floor_height: f32) -> Option<Rect> {
    let top = bounds.max.y.min(floor_height);
    (top > bounds.min.y).then(|| Rect::from_corners(bounds.min, Vec2::new(bounds.max.x, top)))
}

pub(crate) fn update(
    simulation: Res<Simulation>,
    control: Res<crate::camera_control::CameraControlState>,
    mut floors: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<Floor>>,
) {
    let Some((minimum, maximum)) = control.world_bounds() else {
        return;
    };
    let bounds = Rect::from_corners(minimum, maximum);
    // The port translates source world y=0 to SEA_LEVEL for its ocean.
    let region = visible_rect(bounds, SEA_LEVEL - simulation.sea_depth);
    for (mut sprite, mut transform, mut visibility) in &mut floors {
        if let Some(region) = region {
            sprite.custom_size = Some(region.size());
            transform.translation = region.center().extend(0.75);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_floor_writes_half_rgb_into_unorm_scene_capture() {
        let mut app = App::new();
        app.add_systems(Update, setup);
        app.update();
        let world = app.world_mut();
        let sprite = world.query_filtered::<&Sprite, With<Floor>>().single(world).unwrap();
        assert_eq!(sprite.color.to_linear(), LinearRgba::new(0.5, 0.5, 0.5, 1.0));
    }
    #[test]
    fn source_floor_clips_to_camera_and_depth() {
        let view = Rect::from_corners(Vec2::new(-100.0, -600.0), Vec2::new(100.0, 100.0));
        let floor = visible_rect(view, -400.0).unwrap();
        assert_eq!(floor.min, view.min);
        assert_eq!(floor.max, Vec2::new(100.0, -400.0));
        assert!(visible_rect(view, -700.0).is_none());
        assert_eq!(visible_rect(view, 200.0), Some(view));
    }
}

/// Native operations of Floor's ShaderProgram and inherited Fullscreen render.
/// A backend owns shader/model allocation before this constructor registers its
/// camera callback, preserving the source superclass construction boundary.
#[allow(dead_code)]
pub(crate) trait FloorBackend {
    fn inverse_camera(&self, matrix: Mat4);
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        self.inverse_camera(camera.matrix.borrow().inverse());
    }
    fn floor_height(&self, value: f32);
    fn blending(&self, enabled: bool);
    fn render_fullscreen(&self);
}

/// Concrete bridge through the converted ShaderProgram and ShadedModel, rather
/// than reproducing their uniform lookup/bind/draw calls in the Floor adapter.
#[allow(dead_code)]
pub(crate) struct ProgramFloorBackend {
    shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
    fullscreen: Box<dyn crate::i_drawable::IDrawable>,
    blend: Box<dyn Fn(bool)>,
}
#[allow(dead_code)]
impl ProgramFloorBackend {
    pub fn new(
        model: crate::model::SourceModel,
        shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
        blend: impl Fn(bool) + 'static,
    ) -> Self {
        let fullscreen = crate::shaded_model::ShadedModel::new(model, shader.clone());
        Self {
            shader,
            fullscreen: Box::new(fullscreen),
            blend: Box::new(blend),
        }
    }
    pub fn new_uv(
        fullscreen: crate::uv_model::SourceUVModel,
        shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
        blend: impl Fn(bool) + 'static,
    ) -> Self {
        Self {
            shader,
            fullscreen: Box::new(fullscreen),
            blend: Box::new(blend),
        }
    }
}
impl FloorBackend for ProgramFloorBackend {
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        crate::floor_camera_callback::upload(camera, || self.shader.clone());
    }
    fn inverse_camera(&self, matrix: Mat4) {
        self.shader.set_matrix(
            self.shader.uniform_location("inv"),
            false,
            &matrix.to_cols_array(),
        );
    }
    fn floor_height(&self, value: f32) {
        self.shader
            .set_floats(self.shader.uniform_location("floorHeight"), &[value])
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fn blending(&self, enabled: bool) {
        (self.blend)(enabled);
    }
    fn render_fullscreen(&self) {
        self.fullscreen.render();
    }
}

/// Floor.java's retained CameraControl, anonymous callback, render and free hook.
/// Bevy's active rectangle adapter remains above; this source object can drive
/// the converted ShaderProgram/Fullscreen backend when that adapter is provided.
#[allow(dead_code)]
pub(crate) struct SourceFloor {
    camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    backend: std::rc::Rc<dyn FloorBackend>,
    sea_floor: Box<dyn Fn() -> f32>,
    callback: crate::camera_2d::SourceCameraCallback,
    // Retain the identity allocation for the anonymous source callback.
    identity: std::rc::Rc<()>,
}
#[allow(dead_code)]
impl SourceFloor {
    /// Shared shader objects come from the source static FloorShader and
    /// VertexShaders.none owners. Each Floor links its own program, constructs
    /// its superclass geometry, then registers the immediate camera callback.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        vertex_shader: std::sync::Arc<crate::shader::Shader>,
        floor_shader: std::sync::Arc<crate::shader::Shader>,
        program_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::shader_program::ProgramBackend>,
        >,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        model_backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        sea_floor: impl Fn() -> f32 + 'static,
        blend: impl Fn(bool) + 'static,
    ) -> Self {
        let shader = std::sync::Arc::new(crate::shader_program::ShaderProgram::new(
            vec![vertex_shader, floor_shader],
            program_backend,
            context.clone(),
            runtime,
        ));
        let fullscreen = crate::fullscreen::Fullscreen::source(
            shader.clone(),
            buffer_backend,
            vao_backend,
            model_backend,
            context,
            runtime,
        );
        let lifetime = fullscreen.shaded.model.resource_handle();
        let floor = Self::from_parts(
            camera_control,
            std::rc::Rc::new(ProgramFloorBackend::new_uv(fullscreen, shader, blend)),
            sea_floor,
        );
        let control = floor.camera_control.clone();
        let key = floor.callback.key.clone();
        lifetime.before_free_local(move || {
            control.borrow().camera().remove_camera_callback(&key);
        });
        floor
    }
    pub fn from_parts(
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        backend: std::rc::Rc<dyn FloorBackend>,
        sea_floor: impl Fn() -> f32 + 'static,
    ) -> Self {
        use std::rc::Rc;
        let identity = Rc::new(());
        let callback = crate::floor_camera_callback::create(identity.clone(), backend.clone());
        camera_control
            .borrow()
            .camera()
            .add_camera_callback(callback.clone());
        Self {
            camera_control,
            backend,
            sea_floor: Box::new(sea_floor),
            callback,
            identity,
        }
    }
    pub fn camera_control(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>> {
        self.camera_control.clone()
    }
    pub fn render(&self) {
        self.backend.floor_height(-(self.sea_floor)());
        self.backend.blending(true);
        self.backend.render_fullscreen();
        self.backend.blending(false);
    }
    pub fn free(&self) {
        self.camera_control
            .borrow()
            .camera()
            .remove_camera_callback(&self.callback.key);
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    #[derive(Debug, PartialEq)]
    enum Event {
        Inverse(Mat4),
        Height(f32),
        Blend(bool),
        Draw,
    }
    struct Backend {
        events: RefCell<Vec<Event>>,
        fail: Cell<bool>,
    }
    impl FloorBackend for Backend {
        fn inverse_camera(&self, matrix: Mat4) {
            self.events.borrow_mut().push(Event::Inverse(matrix));
        }
        fn floor_height(&self, value: f32) {
            self.events.borrow_mut().push(Event::Height(value));
        }
        fn blending(&self, enabled: bool) {
            self.events.borrow_mut().push(Event::Blend(enabled));
        }
        fn render_fullscreen(&self) {
            self.events.borrow_mut().push(Event::Draw);
            assert!(!self.fail.get(), "backend draw failure");
        }
    }
    #[test]
    fn inherited_model_cleanup_removes_sky_sea_and_floor_callbacks_on_engine_thread() {
        use std::sync::{Arc, Mutex};
        struct Texture;
        impl crate::sky::SkyTexture for Texture {
            fn bind(&self, _: i32) {}
            fn unbind(&self, _: i32) {}
        }
        impl crate::sea::SeaTexture for Texture {
            fn bind(&self, _: i32) {}
            fn unbind(&self, _: i32) {}
        }
        struct Framebuffer;
        impl crate::sea::SeaFramebuffer for Framebuffer {
            fn texture(&self) -> Rc<dyn crate::sea::SeaTexture> {
                Rc::new(Texture)
            }
        }
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(vec![]));
        let shaders = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            events.clone(),
        )));
        let geometry = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(
            events.clone(),
        )));
        let fragment = FloorShader::new(shaders.clone(), context.clone(), &runtime).unwrap();
        let sea_fragment =
            crate::sea::SeaShader::new(shaders.clone(), context.clone(), &runtime).unwrap();
        let vertices = crate::vertex_shaders::SourceVertexShaders::new(
            shaders.clone(),
            context.clone(),
            &runtime,
        );
        let vertex = vertices.none().unwrap();
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        let original_callbacks = window.framebuffer_size_callbacks.borrow().len();
        let floor = SourceFloor::new(
            control.clone(),
            vertex.clone(),
            fragment.shader(),
            shaders.clone(),
            geometry.clone(),
            geometry.clone(),
            geometry.clone(),
            context.clone(),
            &runtime,
            || 400.0,
            |_| {},
        );
        let identity = Rc::downgrade(&floor.identity);
        let sea = crate::sea::SourceSea::new(
            Rc::new(Framebuffer),
            control.clone(),
            vertex.clone(),
            sea_fragment.shader(),
            shaders.clone(),
            geometry.clone(),
            geometry.clone(),
            geometry.clone(),
            context.clone(),
            &runtime,
        );
        let sky = crate::sky::SourceSky::new(
            Rc::new(Texture),
            control,
            vertex,
            fragment.shader(),
            shaders,
            geometry.clone(),
            geometry.clone(),
            geometry,
            context.clone(),
            &runtime,
            |_| Rc::new(Texture),
        );
        assert_eq!(
            window.framebuffer_size_callbacks.borrow().len(),
            original_callbacks + 2
        );
        drop((floor, sea, sky));
        // Callback receivers remain valid even after the caller drops the objects.
        assert!(identity.upgrade().is_some());
        events.lock().unwrap().clear();
        camera.translate(1.0, 2.0);
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.starts_with("matrix:"))
                .count(),
            4
        );
        context.close();
        // Resource closure queues the source free hooks; it does not execute them here.
        assert_eq!(
            window.framebuffer_size_callbacks.borrow().len(),
            original_callbacks + 2
        );
        assert!(identity.upgrade().is_some());
        runtime.run_main();
        assert_eq!(
            window.framebuffer_size_callbacks.borrow().len(),
            original_callbacks
        );
        assert!(identity.upgrade().is_none());
        events.lock().unwrap().clear();
        camera.translate(1.0, 2.0);
        window.emit_framebuffer_size(900, 500);
        assert!(events.lock().unwrap().is_empty());
        context.close();
        runtime.run_main();
        assert!(events.lock().unwrap().is_empty());
    }
    #[test]
    fn source_constructor_reuses_fragment_links_before_geometry_and_registers_after_uv() {
        use std::sync::{Arc, Mutex};
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(vec![]));
        let program_backend = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            events.clone(),
        )));
        let geometry_backend = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(
            events.clone(),
        )));
        let fragment =
            FloorShader::new(program_backend.clone(), context.clone(), &runtime).unwrap();
        assert_eq!(fragment.shader().shader_type, 35632);
        assert!(fragment.shader().bindings.is_empty());
        assert!(Arc::ptr_eq(&fragment.shader(), &fragment.shader()));
        let vertex_shaders = crate::vertex_shaders::SourceVertexShaders::new(
            program_backend.clone(),
            context.clone(),
            &runtime,
        );
        let vertex = vertex_shaders.none().unwrap();
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window, camera.clone()),
        ));
        let construct = || {
            let blend_events = events.clone();
            SourceFloor::new(
                control.clone(),
                vertex.clone(),
                fragment.shader(),
                program_backend.clone(),
                geometry_backend.clone(),
                geometry_backend.clone(),
                geometry_backend.clone(),
                context.clone(),
                &runtime,
                || 123.0,
                move |enabled| {
                    blend_events
                        .lock()
                        .unwrap()
                        .push(format!("blend:{enabled}"))
                },
            )
        };
        let first = construct();
        let setup = events.lock().unwrap().clone();
        let link = setup.iter().position(|e| e == "link").unwrap();
        let geometry = setup.iter().position(|e| e == "create_buffer").unwrap();
        let validation = setup.iter().position(|e| e == "validate").unwrap();
        let uv = setup
            .iter()
            .position(|e| e == "pointer:1:2:5126:false:0:0")
            .unwrap();
        let inverse = setup.iter().position(|e| e.starts_with("matrix:")).unwrap();
        assert!(link < geometry && geometry < validation && validation < uv && uv < inverse);
        let second = construct();
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| *e == "create program")
                .count(),
            2
        );
        events.lock().unwrap().clear();
        first.render();
        assert_eq!(
            *events.lock().unwrap(),
            [
                "use:9",
                "float:-1:[-123.0]",
                "use:0",
                "blend:true",
                "use:9",
                "vao:21",
                "enable:0",
                "enable:1",
                "draw:4:6:5125:0",
                "disable:1",
                "disable:0",
                "vao:0",
                "use:0",
                "blend:false"
            ]
        );
        first.free();
        events.lock().unwrap().clear();
        camera.translate(3.0, 4.0);
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.starts_with("matrix:"))
                .count(),
            1
        );
        second.free();
        events.lock().unwrap().clear();
        camera.translate(3.0, 4.0);
        assert!(events.lock().unwrap().is_empty());
    }
    #[test]
    fn floor_source_callback_render_getter_and_explicit_cleanup_order() {
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window, camera.clone()),
        ));
        let backend = Rc::new(Backend {
            events: RefCell::new(vec![]),
            fail: Cell::new(false),
        });
        let depth = Rc::new(Cell::new(400.0));
        let live = depth.clone();
        let floor = SourceFloor::from_parts(control.clone(), backend.clone(), move || live.get());
        assert!(Rc::ptr_eq(&control, &floor.camera_control()));
        assert_eq!(
            *backend.events.borrow(),
            [Event::Inverse(camera.matrix.borrow().inverse())]
        );
        backend.events.borrow_mut().clear();
        depth.set(123.0);
        floor.render();
        assert_eq!(
            *backend.events.borrow(),
            [
                Event::Height(-123.0),
                Event::Blend(true),
                Event::Draw,
                Event::Blend(false)
            ]
        );
        backend.events.borrow_mut().clear();
        camera.translate(16.0, -32.0);
        assert_eq!(
            *backend.events.borrow(),
            [Event::Inverse(camera.matrix.borrow().inverse())]
        );
        floor.free();
        floor.free();
        backend.events.borrow_mut().clear();
        camera.translate(0.0, 16.0);
        assert!(backend.events.borrow().is_empty());
    }
    #[test]
    fn source_floor_draw_failure_does_not_invent_finally_blend_reset() {
        let (window, _, _) = crate::window::tests::fixture();
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::new(window),
        ));
        let backend = Rc::new(Backend {
            events: RefCell::new(vec![]),
            fail: Cell::new(true),
        });
        let floor = SourceFloor::from_parts(control, backend.clone(), || 400.0);
        backend.events.borrow_mut().clear();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| floor.render())).is_err());
        assert_eq!(
            *backend.events.borrow(),
            [Event::Height(-400.0), Event::Blend(true), Event::Draw]
        );
        floor.free();
    }
}

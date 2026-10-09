//! Sea.java wave and fullscreen framebuffer composition. Driver filter parity
//! and source stencil behavior still require verification/conversion.

pub(crate) use crate::sea_companion::SeaShader;

pub(crate) fn wave_height(x: f32, time: f32, amplitude: f32, width: f32) -> f32 {
    let inv_wave = 3.141592 / width;
    (((x * inv_wave + time * 0.3).sin() * 0.7 + (inv_wave * 3.0 * x - time).sin() * 0.3) + 1.0)
        * 0.5
        * amplitude
}

/// Source texture capture: Sea retains the texture fetched before the draw and
/// unbinds that same object even if its framebuffer targets change during draw.
#[allow(dead_code)]
pub(crate) trait SeaTexture {
    fn bind(&self, unit: i32);
    fn unbind(&self, unit: i32);
}
impl SeaTexture for crate::texture::Texture {
    fn bind(&self, unit: i32) {
        self.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.unbind_unit(unit);
    }
}
#[allow(dead_code)]
pub(crate) trait SeaFramebuffer {
    fn texture(&self) -> std::rc::Rc<dyn SeaTexture>;
}
#[allow(dead_code)]
pub(crate) trait SeaBackend {
    fn floats(&self, name: &str, values: &[f32]);
    fn resolution_changed(&self, width: i32, height: i32) {
        self.floats("resolution", &[width as f32, height as f32]);
    }
    fn matrix(&self, name: &str, matrix: bevy::prelude::Mat4);
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        self.matrix("transform", *camera.matrix.borrow());
        self.matrix("inv", camera.matrix.borrow().inverse());
    }
    fn render_fullscreen(&self);
}
#[allow(dead_code)]
pub(crate) struct ProgramSeaBackend {
    pub shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
    pub fullscreen: crate::uv_model::SourceUVModel,
}
impl SeaBackend for ProgramSeaBackend {
    fn resolution_changed(&self, width: i32, height: i32) {
        crate::sea_resolution_callback::upload(width, height, || self.shader.clone());
    }
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        crate::sea_camera_callback::upload(camera, || self.shader.clone());
    }
    fn floats(&self, name: &str, values: &[f32]) {
        self.shader
            .set_floats(self.shader.uniform_location(name), values)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fn matrix(&self, name: &str, matrix: bevy::prelude::Mat4) {
        self.shader.set_matrix(
            self.shader.uniform_location(name),
            false,
            &matrix.to_cols_array(),
        );
    }
    fn render_fullscreen(&self) {
        crate::i_drawable::IDrawable::render(&self.fullscreen);
    }
}

/// Sea.java's retained objects and callback/render/free lifecycle. The active
/// Bevy compositor below remains its separate graphics backend adaptation.
#[allow(dead_code)]
pub(crate) struct SourceSea {
    screen_fbo: std::rc::Rc<dyn SeaFramebuffer>,
    camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    backend: std::rc::Rc<dyn SeaBackend>,
    resolution_callback: std::rc::Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)>,
    camera_callback: crate::camera_2d::SourceCameraCallback,
    identity: std::rc::Rc<()>,
}
#[allow(dead_code)]
impl SourceSea {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        screen_fbo: std::rc::Rc<dyn SeaFramebuffer>,
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        vertex_shader: std::sync::Arc<crate::shader::Shader>,
        sea_shader: std::sync::Arc<crate::shader::Shader>,
        program_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::shader_program::ProgramBackend>,
        >,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        model_backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let shader = std::sync::Arc::new(crate::shader_program::ShaderProgram::new(
            vec![vertex_shader, sea_shader],
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
        let sea = Self::from_parts(
            screen_fbo,
            camera_control,
            std::rc::Rc::new(ProgramSeaBackend { shader, fullscreen }),
        );
        let control = sea.camera_control.clone();
        let resolution_callback = sea.resolution_callback.clone();
        let key = sea.camera_callback.key.clone();
        lifetime.before_free_local(move || {
            let window = control.borrow().window();
            let mut callbacks = window.framebuffer_size_callbacks.borrow_mut();
            if let Some(index) = callbacks
                .iter()
                .position(|callback| std::rc::Rc::ptr_eq(callback, &resolution_callback))
            {
                callbacks.remove(index);
            }
            drop(callbacks);
            control.borrow().camera().remove_camera_callback(&key);
        });
        sea
    }
    pub fn from_parts(
        screen_fbo: std::rc::Rc<dyn SeaFramebuffer>,
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        backend: std::rc::Rc<dyn SeaBackend>,
    ) -> Self {
        use std::rc::Rc;
        let identity = Rc::new(());
        let resolution_callback = crate::sea_resolution_callback::create(backend.clone());
        let camera_callback = crate::sea_camera_callback::create(identity.clone(), backend.clone());
        let window = camera_control.borrow().window();
        let size = window.framebuffer_size();
        backend.floats("resolution", &[size[0] as f32, size[1] as f32]);
        window
            .framebuffer_size_callbacks
            .borrow_mut()
            .push(resolution_callback.clone());
        camera_control
            .borrow()
            .camera()
            .add_camera_callback(camera_callback.clone());
        Self {
            screen_fbo,
            camera_control,
            backend,
            resolution_callback,
            camera_callback,
            identity,
        }
    }
    pub fn screen_fbo(&self) -> std::rc::Rc<dyn SeaFramebuffer> {
        self.screen_fbo.clone()
    }
    pub fn camera_control(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>> {
        self.camera_control.clone()
    }
    pub fn set_time(&self, time: f32) {
        self.backend.floats("time", &[time]);
    }
    pub fn render(&self) {
        use crate::game_parameter_provider_kt::get_game_parameter_provider;
        // Read the global provider afresh at each original getter site, rather
        // than retaining an earlier provider or snapshotting the entire frame.
        let x = get_game_parameter_provider().borrow().waves().borrow().x;
        let y = get_game_parameter_provider().borrow().waves().borrow().y;
        self.backend.floats("waveSize", &[x, y]);
        let darkness = get_game_parameter_provider().borrow().water_darkness();
        self.backend.floats("waterDarkness", &[darkness]);
        let color = get_game_parameter_provider()
            .borrow()
            .water_color()
            .borrow()
            .to_array();
        self.backend.floats("col", &color);
        let texture = self.screen_fbo.texture();
        texture.bind(0);
        self.backend.render_fullscreen();
        texture.unbind(0);
    }
    pub fn free(&self) {
        let window = self.camera_control.borrow().window();
        let mut callbacks = window.framebuffer_size_callbacks.borrow_mut();
        if let Some(index) = callbacks
            .iter()
            .position(|callback| std::rc::Rc::ptr_eq(callback, &self.resolution_callback))
        {
            callbacks.remove(index);
        }
        drop(callbacks);
        self.camera_control
            .borrow()
            .camera()
            .remove_camera_callback(&self.camera_callback.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    #[derive(Debug, PartialEq)]
    enum Event {
        Floats(String, Vec<f32>),
        Matrix(String, bevy::prelude::Mat4),
        Texture(i32, bool),
        Draw,
    }
    struct Backend {
        events: Rc<RefCell<Vec<Event>>>,
        fail: Cell<bool>,
        on_floats: RefCell<Option<Box<dyn Fn(&str)>>>,
    }
    impl SeaBackend for Backend {
        fn floats(&self, name: &str, values: &[f32]) {
            self.events
                .borrow_mut()
                .push(Event::Floats(name.into(), values.to_vec()));
            if let Some(hook) = &*self.on_floats.borrow() {
                hook(name);
            }
        }
        fn matrix(&self, name: &str, matrix: bevy::prelude::Mat4) {
            self.events
                .borrow_mut()
                .push(Event::Matrix(name.into(), matrix));
        }
        fn render_fullscreen(&self) {
            self.events.borrow_mut().push(Event::Draw);
            assert!(!self.fail.get(), "draw failed");
        }
    }
    struct Texture(Rc<RefCell<Vec<Event>>>);
    impl SeaTexture for Texture {
        fn bind(&self, unit: i32) {
            self.0.borrow_mut().push(Event::Texture(unit, true));
        }
        fn unbind(&self, unit: i32) {
            self.0.borrow_mut().push(Event::Texture(unit, false));
        }
    }
    struct Framebuffer(Rc<Texture>);
    impl SeaFramebuffer for Framebuffer {
        fn texture(&self) -> Rc<dyn SeaTexture> {
            self.0.clone()
        }
    }
    #[test]
    fn sea_constructor_links_shared_shader_then_uv_geometry_and_live_callbacks() {
        use std::sync::{Arc, Mutex};
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let native_events = Arc::new(Mutex::new(vec![]));
        let shaders = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            native_events.clone(),
        )));
        let geometry = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(
            native_events.clone(),
        )));
        let fragment = SeaShader::new(shaders.clone(), context.clone(), &runtime).unwrap();
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
        let texture_events = Rc::new(RefCell::new(vec![]));
        let framebuffer: Rc<dyn SeaFramebuffer> =
            Rc::new(Framebuffer(Rc::new(Texture(texture_events.clone()))));
        let sea = SourceSea::new(
            framebuffer.clone(),
            control.clone(),
            vertex,
            fragment.shader(),
            shaders,
            geometry.clone(),
            geometry.clone(),
            geometry,
            context.clone(),
            &runtime,
        );
        assert!(Rc::ptr_eq(&sea.screen_fbo(), &framebuffer));
        assert!(Rc::ptr_eq(&sea.camera_control(), &control));
        let setup = native_events.lock().unwrap().clone();
        let link = setup.iter().position(|event| event == "link").unwrap();
        let geometry = setup
            .iter()
            .position(|event| event == "create_buffer")
            .unwrap();
        let validation = setup.iter().position(|event| event == "validate").unwrap();
        let uv = setup
            .iter()
            .position(|event| event == "pointer:1:2:5126:false:0:0")
            .unwrap();
        let resolution = setup
            .iter()
            .position(|event| event == "float:-1:[800.0, 400.0]")
            .unwrap();
        let matrix = setup
            .iter()
            .position(|event| event.starts_with("matrix:"))
            .unwrap();
        assert!(
            link < geometry
                && geometry < validation
                && validation < uv
                && uv < resolution
                && resolution < matrix
        );
        // GLSL's default sampler unit zero is retained; the source never sets tex.
        assert!(!setup.iter().any(|event| event.starts_with("int:")));
        native_events.lock().unwrap().clear();
        window.emit_framebuffer_size(0, -10);
        assert_eq!(
            *native_events.lock().unwrap(),
            ["use:9", "float:-1:[0.0, -10.0]", "use:0"]
        );
        native_events.lock().unwrap().clear();
        sea.set_time(2.5);
        assert_eq!(
            *native_events.lock().unwrap(),
            ["use:9", "float:-1:[2.5]", "use:0"]
        );
        sea.render();
        assert_eq!(
            *texture_events.borrow(),
            [Event::Texture(0, true), Event::Texture(0, false)]
        );
        assert!(
            native_events
                .lock()
                .unwrap()
                .iter()
                .any(|event| event == "draw:4:6:5125:0")
        );
        context.close();
        runtime.run_main();
        native_events.lock().unwrap().clear();
        window.emit_framebuffer_size(900, 500);
        camera.translate(1.0, 2.0);
        assert!(native_events.lock().unwrap().is_empty());
    }
    #[test]
    fn sea_companion_retains_the_exact_original_fragment_shader() {
        use std::sync::{Arc, Mutex};
        let original =
            std::fs::read_to_string("../SS2/decompiled/com/wicpar/sinkingsimulator/Sea.java")
                .unwrap();
        assert!(original.contains(&format!("new Shader({:?}, 35632", SeaShader::SOURCE)));
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            events.clone(),
        )));
        let companion = SeaShader::new(backend, context.clone(), &runtime).unwrap();
        let first = companion.shader();
        assert!(Arc::ptr_eq(&first, &companion.shader()));
        assert_eq!(first.shader_type, 35632);
        assert!(first.bindings.is_empty());
        let constructed = events.lock().unwrap().clone();
        first.close();
        assert!(Arc::ptr_eq(&first, &companion.shader()));
        // The source getter returns its retained shader even after closure.
        assert_eq!(*events.lock().unwrap(), constructed);
        context.close();
        runtime.run_main();
    }
    #[test]
    fn source_sea_callbacks_uniform_draw_order_live_settings_and_explicit_free() {
        use crate::game_parameter_provider_kt::{
            get_game_parameter_provider, set_game_parameter_provider,
        };
        let previous = get_game_parameter_provider();
        let provider = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        set_game_parameter_provider(provider.clone());
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        let events = Rc::new(RefCell::new(vec![]));
        let backend = Rc::new(Backend {
            events: events.clone(),
            fail: Cell::new(false),
            on_floats: RefCell::new(None),
        });
        let framebuffer: Rc<dyn SeaFramebuffer> =
            Rc::new(Framebuffer(Rc::new(Texture(events.clone()))));
        let sea = SourceSea::from_parts(framebuffer.clone(), control.clone(), backend.clone());
        assert!(Rc::ptr_eq(&sea.screen_fbo(), &framebuffer));
        assert!(Rc::ptr_eq(&sea.camera_control(), &control));
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("resolution".into(), vec![800., 400.]),
                Event::Matrix("transform".into(), *camera.matrix.borrow()),
                Event::Matrix("inv".into(), camera.matrix.borrow().inverse())
            ]
        );
        events.borrow_mut().clear();
        window.emit_framebuffer_size(0, 1200);
        assert_eq!(
            *events.borrow(),
            [Event::Floats("resolution".into(), vec![0., 1200.])]
        );
        events.borrow_mut().clear();
        camera.translate(7., -9.);
        assert_eq!(
            *events.borrow(),
            [
                Event::Matrix("transform".into(), *camera.matrix.borrow()),
                Event::Matrix("inv".into(), camera.matrix.borrow().inverse())
            ]
        );
        events.borrow_mut().clear();
        *provider.borrow().waves().borrow_mut() = bevy::prelude::Vec2::new(60., 3.);
        provider.borrow_mut().set_water_darkness(0.3);
        *provider.borrow().water_color().borrow_mut() = bevy::prelude::Vec4::new(0., 0.2, 0.6, 0.7);
        sea.set_time(15.);
        sea.render();
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("time".into(), vec![15.]),
                Event::Floats("waveSize".into(), vec![60., 3.]),
                Event::Floats("waterDarkness".into(), vec![0.3]),
                Event::Floats("col".into(), vec![0., 0.2, 0.6, 0.7]),
                Event::Texture(0, true),
                Event::Draw,
                Event::Texture(0, false)
            ]
        );
        let replacement = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        replacement.borrow_mut().set_water_darkness(2.0);
        *replacement.borrow().water_color().borrow_mut() = bevy::prelude::Vec4::ONE;
        let next = replacement.clone();
        *backend.on_floats.borrow_mut() = Some(Box::new(move |name| {
            if name == "waveSize" {
                set_game_parameter_provider(next.clone());
            }
        }));
        events.borrow_mut().clear();
        sea.render();
        assert_eq!(
            &events.borrow()[..3],
            [
                Event::Floats("waveSize".into(), vec![60., 3.]),
                Event::Floats("waterDarkness".into(), vec![2.]),
                Event::Floats("col".into(), vec![1., 1., 1., 1.])
            ]
        );
        *backend.on_floats.borrow_mut() = None;
        events.borrow_mut().clear();
        backend.fail.set(true);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sea.render())).is_err());
        assert_eq!(
            &events.borrow()[3..],
            [Event::Texture(0, true), Event::Draw]
        );
        sea.free();
        sea.free();
        events.borrow_mut().clear();
        window.emit_framebuffer_size(123, 456);
        camera.translate(1., 2.);
        assert!(events.borrow().is_empty());
        set_game_parameter_provider(previous);
    }
    #[test]
    fn source_wave_origin_and_zero_amplitude() {
        assert_eq!(wave_height(0.0, 0.0, 2.0, 40.0), 1.0);
        assert_eq!(wave_height(123.0, 9.0, 0.0, 40.0), 0.0);
    }
}

use crate::{SEA_LEVEL, Simulation, WorldCamera, screen_fbo::ScreenFbo};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::Material2d,
};
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct SeaMaterial {
    #[uniform(0)]
    pub waves_time_darkness: Vec4,
    #[uniform(1)]
    pub view: Vec4,
    #[uniform(2)]
    pub color: Vec4,
    #[texture(3)]
    #[sampler(4)]
    pub texture: Handle<Image>,
    /// Native GL-compatible sampler available on the initialized device.
    #[uniform(5)]
    pub sampling: Vec4,
}
impl Material2d for SeaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sea.wgsl".into()
    }
}
#[derive(Component)]
pub(crate) struct Sea(pub Handle<Mesh>, pub Handle<SeaMaterial>);
#[derive(Component)]
pub(crate) struct SeaCamera;
#[derive(Resource)]
pub(crate) struct SeaFrame {
    pub(crate) texture: Handle<Image>,
    size: UVec2,
}

// Main.java draws Sea after the scene framebuffer and before GUI.render().
// Toolbox's clipped settings camera owns layer 2; sharing it causes both
// cameras to render each other's meshes with different projections/viewports.
const COMPOSITOR_LAYER: usize = 7;

pub(crate) fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut world: Query<(Entity, &mut Camera), With<WorldCamera>>,
    legacy: Query<
        Entity,
        Or<(
            With<crate::OceanSurface>,
            With<crate::OceanDepth>,
            With<crate::UnderwaterEffect>,
            With<crate::ReflectionMesh>,
            With<crate::InternalWaterMesh>,
        )>,
    >,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SeaMaterial>>,
    simulation: Res<Simulation>,
    device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    let native_border = device.is_some_and(|device| {
        device
            .features()
            .contains(bevy::render::settings::WgpuFeatures::ADDRESS_MODE_CLAMP_TO_BORDER)
    });
    let target = ScreenFbo::with_native_border(size, &mut images, native_border);
    for (entity, mut camera) in &mut world {
        camera.order = -3;
        commands
            .entity(entity)
            .insert(RenderTarget::Image(target.texture.clone().into()));
    }
    for entity in &legacy {
        commands.entity(entity).despawn();
    }
    let mesh = meshes.add(crate::fullscreen::Fullscreen::mesh());
    let material = materials.add(SeaMaterial {
        waves_time_darkness: Vec4::new(
            simulation.wave_width,
            simulation.wave_amplitude,
            simulation.elapsed,
            simulation.water_darkness,
        ),
        view: Vec4::new(-360.0, 360.0, SEA_LEVEL, size.y as f32),
        color: simulation.water_color(),
        texture: target.filtered.clone(),
        sampling: Vec4::new(if native_border { 1.0 } else { 0.0 }, 0.0, 0.0, 1.0),
    });
    commands.spawn((
        Sea(mesh.clone(), material.clone()),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        RenderLayers::layer(COMPOSITOR_LAYER),
    ));
    let frame_texture=images.add(Image::new_target_texture(size.x.max(1),size.y.max(1),
        bevy::render::render_resource::TextureFormat::Rgba8Unorm,None));
    commands.spawn((Camera2d, SeaCamera, Camera {order:-2,..default()}, Msaa::Off,
        RenderTarget::Image(frame_texture.clone().into()),RenderLayers::layer(COMPOSITOR_LAYER)));
    commands.insert_resource(SeaFrame {texture:frame_texture,size});
    commands.insert_resource(target);
}

pub(crate) fn sync(
    simulation: Res<Simulation>,
    control: Res<crate::camera_control::CameraControlState>,
    windows: Query<&Window>,
    world: Query<(&Transform, &Projection), (With<WorldCamera>, Without<SeaCamera>, Without<Sea>)>,
    mut cameras: Query<
        (&mut Transform, &mut Projection),
        (With<SeaCamera>, Without<WorldCamera>, Without<Sea>),
    >,
    mut seas: Query<(&Sea, &mut Transform), (Without<WorldCamera>, Without<SeaCamera>)>,
    mut target: ResMut<ScreenFbo>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SeaMaterial>>,
    mut frame: ResMut<SeaFrame>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((transform, projection)) = world.single() else {
        return;
    };
    let Projection::Orthographic(_) = projection else {
        return;
    };
    let Some((minimum, maximum)) = control.world_bounds() else {
        return;
    };
    let area = Rect::from_corners(
        minimum - transform.translation.truncate(),
        maximum - transform.translation.truncate(),
    );
    target.resize(
        UVec2::new(window.physical_width(), window.physical_height()),
        &mut images,
    );
    let frame_size=UVec2::new(window.physical_width().max(1),window.physical_height().max(1));
    if frame.size!=frame_size {
        if let Some(mut image)=images.get_mut(&frame.texture) {
            image.resize(bevy::render::render_resource::Extent3d {width:frame_size.x,height:frame_size.y,depth_or_array_layers:1});
        }
        frame.size=frame_size;
    }
    for (mut current, mut current_projection) in &mut cameras {
        *current = *transform;
        *current_projection = projection.clone();
    }
    for (sea, mut current) in &mut seas {
        current.translation.x = transform.translation.x;
        current.translation.y = transform.translation.y;
        if let Some(mut mesh) = meshes.get_mut(&sea.0) {
            crate::fullscreen::Fullscreen::fit(&mut mesh, area);
        }
        if let Some(mut material) = materials.get_mut(&sea.1) {
            material.waves_time_darkness = Vec4::new(
                simulation.wave_width,
                simulation.wave_amplitude,
                simulation.elapsed,
                simulation.water_darkness,
            );
            material.view = Vec4::new(minimum.y, maximum.y, SEA_LEVEL, target.size.y as f32);
            material.color = simulation.water_color();
        }
    }
}

#[cfg(test)]
mod shader_tests {
    use super::*;

    #[test]
    fn compositor_setup_keeps_scene_sea_and_gui_passes_separate() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<SeaMaterial>>()
            .init_resource::<Simulation>()
            .add_systems(Update, setup);
        app.world_mut().spawn(Window::default());
        let legacy_water = app.world_mut().spawn(crate::InternalWaterMesh {
            mesh: Handle::default(), material: Handle::default(),
            cells: vec![], width: 1, height: 1, interior: vec![false],
        }).id();
        let world_camera = app.world_mut().spawn((
            Camera::default(), WorldCamera, RenderLayers::layer(0),
        )).id();
        let gui_camera = app.world_mut().spawn((
            Camera::default(), crate::toolbox_viewport::ContentCamera,
            RenderLayers::layer(2),
        )).id();
        app.update();
        let world = app.world_mut();
        assert!(world.get_entity(legacy_water).is_err());
        let target = world.resource::<ScreenFbo>().texture.clone();
        assert_eq!(world.get::<Camera>(world_camera).unwrap().order, -3);
        let frame=world.resource::<SeaFrame>().texture.clone();
        assert_eq!(world.resource::<Assets<Image>>().get(&frame).unwrap().texture_descriptor.format,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm);
        assert!(matches!(world.get::<RenderTarget>(world_camera),
            Some(RenderTarget::Image(image)) if image.handle == target));
        let gui_layers = world.get::<RenderLayers>(gui_camera).unwrap().clone();
        let sea_layers = world.query_filtered::<&RenderLayers, With<Sea>>()
            .single(world).unwrap().clone();
        let camera_layers = world.query_filtered::<&RenderLayers, With<SeaCamera>>()
            .single(world).unwrap().clone();
        assert!(matches!(world.query_filtered::<&RenderTarget,With<SeaCamera>>().single(world).unwrap(),
            RenderTarget::Image(image) if image.handle==frame));
        assert!(sea_layers.intersects(&camera_layers));
        assert!(!sea_layers.intersects(&gui_layers));
        assert!(!camera_layers.intersects(&RenderLayers::layer(0)));
        for layer in 1..=6 {
            assert!(!camera_layers.intersects(&RenderLayers::layer(layer)));
        }
    }

    #[test]
    fn source_sea_shader_validates() {
        let source = include_str!("../assets/shaders/sea.wgsl")
            .replace("#import bevy_sprite::mesh2d_vertex_output::VertexOutput",
                "struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) world_position: vec4<f32>, @location(1) uv: vec2<f32>, }")
            .replace("#{MATERIAL_BIND_GROUP}", "0");
        let module = naga::front::wgsl::parse_str(&source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}

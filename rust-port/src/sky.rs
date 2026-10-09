//! Translation of Sky.java's camera-relative horizon and source star field.
use crate::{SEA_LEVEL, Simulation, WorldCamera};
use bevy::{
    camera::visibility::RenderLayers, prelude::*, reflect::TypePath,
    render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::Material2d,
};

#[allow(dead_code)]
pub(crate) trait SkyTexture {
    fn bind(&self, unit: i32);
    fn unbind(&self, unit: i32);
}
pub(crate) use crate::sky_companion::SourceSkyShaders;
impl SkyTexture for crate::texture::Texture {
    fn bind(&self, unit: i32) {
        self.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.unbind_unit(unit);
    }
}
impl SkyTexture for crate::texture_2d::SourceTexture2D {
    fn bind(&self, unit: i32) {
        self.texture.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.texture.unbind_unit(unit);
    }
}
#[allow(dead_code)]
pub(crate) trait SkyBackend {
    fn floats(&self, name: &str, values: &[f32]);
    fn integer(&self, name: &str, value: i32);
    fn inverse_camera(&self, matrix: Mat4);
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        self.inverse_camera(camera.matrix.borrow().inverse());
    }
    fn make_stars(&self, size: [i32; 2]) -> std::rc::Rc<dyn SkyTexture>;
    fn render_fullscreen(&self);
}
#[allow(dead_code)]
pub(crate) struct ProgramSkyBackend {
    shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
    fullscreen: crate::uv_model::SourceUVModel,
    bake: Box<dyn Fn([i32; 2]) -> std::rc::Rc<dyn SkyTexture>>,
}
impl SkyBackend for ProgramSkyBackend {
    fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        crate::sky_camera_reference::upload(camera, || self.shader.clone());
    }
    fn floats(&self, name: &str, values: &[f32]) {
        self.shader
            .set_floats(self.shader.uniform_location(name), values)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fn integer(&self, name: &str, value: i32) {
        self.shader
            .set_ints(self.shader.uniform_location(name), &[value])
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fn inverse_camera(&self, matrix: Mat4) {
        self.shader.set_matrix(
            self.shader.uniform_location("inv"),
            false,
            &matrix.to_cols_array(),
        );
    }
    fn make_stars(&self, size: [i32; 2]) -> std::rc::Rc<dyn SkyTexture> {
        (self.bake)(size)
    }
    fn render_fullscreen(&self) {
        crate::i_drawable::IDrawable::render(&self.fullscreen);
    }
}
#[allow(dead_code)]
pub(crate) struct SourceSkyState {
    sky_map: std::rc::Rc<dyn SkyTexture>,
    stars: std::cell::RefCell<std::rc::Rc<dyn SkyTexture>>,
    control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    backend: std::rc::Rc<dyn SkyBackend>,
}
impl SourceSkyState {
    pub(crate) fn camera_changed(&self, camera: &crate::camera_2d::SourceCamera2D) {
        self.backend.camera_changed(camera);
    }
    pub(crate) fn resolution(&self, width: i32, height: i32) {
        self.backend
            .floats("resolution", &[width as f32, height as f32]);
        // Every positive callback rebakes, including a repeat of the same size.
        // Assign only after baking completes, leaving the previous stars on failure.
        if width > 0 && height > 0 {
            let stars = self.backend.make_stars([width, height]);
            *self.stars.borrow_mut() = stars;
        }
    }
}
/// Sky.java's retained objects, source constructor and explicit callback cleanup.
/// Native texture/program construction is supplied by the backend; active Bevy
/// star baking and camera fitting remain below.
#[allow(dead_code)]
pub(crate) struct SourceSky {
    state: std::rc::Rc<SourceSkyState>,
    resize_callback: std::rc::Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)>,
    camera_callback: crate::camera_2d::SourceCameraCallback,
}
#[allow(dead_code)]
impl SourceSky {
    /// Sky(String, CameraControl) loads four channels and constructs its map
    /// before delegating to the texture constructor. The FileReader owner and
    /// native backends are supplied by the engine registry.
    #[allow(clippy::too_many_arguments)]
    pub fn from_path(
        path: &std::path::Path,
        reader: &crate::file_reader::FileReader,
        control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        vertex_shader: std::sync::Arc<crate::shader::Shader>,
        sky_shader: std::sync::Arc<crate::shader::Shader>,
        program_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::shader_program::ProgramBackend>,
        >,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        model_backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        texture_backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        bake: impl Fn([i32; 2]) -> std::rc::Rc<dyn SkyTexture> + 'static,
    ) -> Result<Self, String> {
        let image = reader.read_image(path, 4)?;
        let sky_map = std::rc::Rc::new(crate::texture_2d::SourceTexture2D::from_image(
            &image,
            32856,
            true,
            std::sync::Arc::new(crate::sky_texture::configure),
            texture_backend,
            context.clone(),
            runtime,
        ));
        Ok(Self::new(
            sky_map,
            control,
            vertex_shader,
            sky_shader,
            program_backend,
            buffer_backend,
            vao_backend,
            model_backend,
            context,
            runtime,
            bake,
        ))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sky_map: std::rc::Rc<dyn SkyTexture>,
        control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        vertex_shader: std::sync::Arc<crate::shader::Shader>,
        sky_shader: std::sync::Arc<crate::shader::Shader>,
        program_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::shader_program::ProgramBackend>,
        >,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        model_backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        bake: impl Fn([i32; 2]) -> std::rc::Rc<dyn SkyTexture> + 'static,
    ) -> Self {
        let shader = std::sync::Arc::new(crate::shader_program::ShaderProgram::new(
            vec![vertex_shader, sky_shader],
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
        let sky = Self::from_parts(
            sky_map,
            control,
            std::rc::Rc::new(ProgramSkyBackend {
                shader,
                fullscreen,
                bake: Box::new(bake),
            }),
        );
        let state = sky.state.clone();
        lifetime.before_free_local(move || {
            let window = state.control.borrow().window();
            crate::sky_free_resolution_reference::remove(&window, state.clone());
            let key = crate::sky_free_camera_reference::create(state.clone()).key;
            state.control.borrow().camera().remove_camera_callback(&key);
        });
        sky
    }
    pub fn from_parts(
        sky_map: std::rc::Rc<dyn SkyTexture>,
        control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        backend: std::rc::Rc<dyn SkyBackend>,
    ) -> Self {
        use std::{cell::RefCell, rc::Rc};
        let window = control.borrow().window();
        let stars = backend.make_stars(window.framebuffer_size());
        let state = Rc::new(SourceSkyState {
            sky_map,
            stars: RefCell::new(stars),
            control,
            backend,
        });
        let resize_callback = crate::sky_resolution_reference::create(state.clone());
        let camera_callback = crate::sky_camera_reference::create(state.clone());
        let size = window.framebuffer_size();
        state
            .backend
            .floats("resolution", &[size[0] as f32, size[1] as f32]);
        window
            .framebuffer_size_callbacks
            .borrow_mut()
            .push(resize_callback.clone());
        state
            .control
            .borrow()
            .camera()
            .add_camera_callback(camera_callback.clone());
        state.backend.integer("sky", 0);
        state.backend.integer("stars", 1);
        Self {
            state,
            resize_callback,
            camera_callback,
        }
    }
    pub fn sky_map(&self) -> std::rc::Rc<dyn SkyTexture> {
        self.state.sky_map.clone()
    }
    pub fn camera_control(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>> {
        self.state.control.clone()
    }
    pub fn render(&self) {
        let day = crate::game_parameter_provider_kt::get_game_parameter_provider()
            .borrow()
            .day();
        self.state.backend.floats("day", &[day]);
        self.state.sky_map.bind(0);
        let stars = self.state.stars.borrow().clone();
        stars.bind(1);
        self.state.backend.render_fullscreen();
        // Unlike Sea's captured texture, the source reads this.stars again here.
        let current_stars = self.state.stars.borrow().clone();
        current_stars.unbind(1);
        self.state.sky_map.unbind(0);
    }
    pub fn free(&self) {
        let window = self.state.control.borrow().window();
        crate::sky_free_resolution_reference::remove(&window, self.state.clone());
        let key = crate::sky_free_camera_reference::create(self.state.clone()).key;
        self.state
            .control
            .borrow()
            .camera()
            .remove_camera_callback(&key);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct SkyMaterial {
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(1)]
    pub resolution: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub texture: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub stars: Handle<Image>,
}
impl Material2d for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky.wgsl".into()
    }
}
#[derive(Component)]
pub(crate) struct SkyMesh(pub Handle<Mesh>, pub Handle<SkyMaterial>);

pub(crate) fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    windows: Query<&Window>,
) {
    let size = windows
        .single()
        .map(|window| UVec2::new(window.physical_width(), window.physical_height()))
        .unwrap_or(UVec2::ONE);
    let stars = crate::sky_star_field::StarField::new(size, &mut images);
    let texture = images.add(
        crate::sky_texture::load(
            &crate::file_reader::FileReader::game(),
            std::path::Path::new("config/sky.png"),
        )
        .unwrap_or_else(|error| panic!("Failed to construct source Sky: {error}")),
    );
    let mut mesh = crate::fullscreen::Fullscreen::mesh();
    crate::fullscreen::Fullscreen::fit(
        &mut mesh,
        Rect::from_corners(Vec2::new(-640.0, -360.0), Vec2::new(640.0, 360.0)),
    );
    let mesh = meshes.add(mesh);
    let material = materials.add(SkyMaterial {
        params: Vec4::new(1.0, 0.0, 360.0, SEA_LEVEL),
        resolution: Vec4::ZERO,
        texture,
        stars: stars.texture.clone(),
    });
    commands.spawn((
        SkyMesh(mesh.clone(), material.clone()),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, -10.0),
        RenderLayers::layer(0),
    ));
    commands.insert_resource(stars);
}

pub(crate) fn animate_sky(
    simulation: Res<Simulation>,
    control: Res<crate::camera_control::CameraControlState>,
    windows: Query<&Window>,
    cameras: Query<&Transform, With<WorldCamera>>,
    mut sky_meshes: Query<(&SkyMesh, &mut Transform), Without<WorldCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut stars: ResMut<crate::sky_star_field::StarField>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(camera_transform) = cameras.single() else {
        return;
    };
    let Some((minimum, maximum)) = control.world_bounds() else {
        return;
    };
    // CameraControl has already changed the source matrix this frame. Bevy's
    // projection area is refreshed later, during its camera update systems.
    let offset = camera_transform.translation.truncate();
    let area = Rect::from_corners(minimum - offset, maximum - offset);
    stars.sync_size(
        UVec2::new(window.physical_width(), window.physical_height()),
        &mut images,
    );
    for (sky, mut transform) in &mut sky_meshes {
        if let Some(mut mesh) = meshes.get_mut(&sky.0) {
            crate::fullscreen::Fullscreen::fit(&mut mesh, area);
        }
        transform.translation.x = camera_transform.translation.x;
        transform.translation.y = camera_transform.translation.y;
        if let Some(mut material) = materials.get_mut(&sky.1) {
            material.stars = stars.texture.clone();
            material.params = Vec4::new(
                simulation.day,
                camera_transform.translation.y - SEA_LEVEL,
                area.height() * 0.5,
                SEA_LEVEL,
            );
            material.resolution = Vec4::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
                0.0,
                0.0,
            );
        }
    }
}

/// Reference for SkyShader's inverse-camera mapping. Source y=0 is the sea.
#[cfg(test)]
fn horizon_uv(world_y: f32, center_y: f32, half_height: f32) -> f32 {
    let sign = if world_y > 0.0 {
        1.0
    } else if world_y < 0.0 {
        -1.0
    } else {
        0.0
    };
    let limit = center_y + sign * half_height;
    if world_y > 0.0 {
        0.5 - world_y / limit * 0.5
    } else {
        0.5 + world_y / limit * 0.5
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_constructor_uploads_original_map_before_geometry_and_bakes_native_stars() {
        use std::{
            cell::RefCell,
            rc::Rc,
            sync::{Arc, Mutex},
        };
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let geometry = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(log.clone())));
        let programs = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            log.clone(),
        )));
        let vertices = crate::vertex_shaders::SourceVertexShaders::new(
            programs.clone(),
            context.clone(),
            &runtime,
        );
        let shaders = SourceSkyShaders::new(
            &vertices,
            programs.clone(),
            programs.clone(),
            context.clone(),
            &runtime,
        )
        .unwrap();
        let fullscreen = crate::physics_full_screen::SourcePhysicsFullscreen::new(
            geometry.clone(),
            geometry.clone(),
            geometry.clone(),
            context.clone(),
            &runtime,
        );
        let blend_log = log.clone();
        let baker = crate::sky_star_field::SourceStarBaker::new(
            shaders.star_field,
            fullscreen.model,
            textures.clone(),
            textures.clone(),
            context.clone(),
            &runtime,
            move |enabled| blend_log.lock().unwrap().push(format!("blend:{enabled}")),
        );
        let vertex = vertices.none().unwrap();
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        let reader = crate::file_reader::FileReader {
            ss_home: std::path::PathBuf::from("assets"),
            ss_resources: std::path::PathBuf::from("assets"),
            bundled_resources: std::path::PathBuf::from("assets"),
        };
        let image = reader
            .read_image(std::path::Path::new("config/sky.png"), 4)
            .unwrap();
        log.lock().unwrap().clear();
        let sky = SourceSky::from_path(
            std::path::Path::new("config/sky.png"),
            &reader,
            control,
            vertex,
            shaders.sky,
            programs,
            geometry.clone(),
            geometry.clone(),
            geometry,
            textures,
            context.clone(),
            &runtime,
            move |size| baker.make_sky_stars(size),
        )
        .unwrap();
        let setup = log.lock().unwrap().clone();
        let upload = setup
            .iter()
            .position(|event| {
                event
                    == &format!(
                        "image2:1:3553:32856:[{}, {}]:6408:5121:false",
                        image.width, image.height
                    )
            })
            .unwrap();
        let mipmaps = setup
            .iter()
            .position(|event| event.starts_with("mips:"))
            .unwrap();
        let link = setup.iter().position(|event| event == "link").unwrap();
        let uv = setup
            .iter()
            .position(|event| event == "pointer:1:2:5126:false:0:0")
            .unwrap();
        let stars = setup
            .iter()
            .position(|event| event == "image2:2:3553:32856:[800, 400]:6408:5121:true")
            .unwrap();
        let bake = setup
            .iter()
            .position(|event| event == "blend:false")
            .unwrap();
        let resolution = setup
            .iter()
            .position(|event| event == "float:-1:[800.0, 400.0]")
            .unwrap();
        let inverse = setup
            .iter()
            .position(|event| event.starts_with("matrix:"))
            .unwrap();
        assert!(
            upload < mipmaps
                && mipmaps < link
                && link < uv
                && uv < stars
                && stars < bake
                && bake < resolution
                && resolution < inverse
        );
        for (parameter, value) in [(10240, 9729), (10241, 9987), (10242, 33071), (10243, 33071)] {
            assert!(setup.contains(&format!("parameter:3553:{parameter}:{value}")));
        }
        assert_eq!(
            setup
                .iter()
                .filter(|event| event.starts_with("mips:"))
                .count(),
            1
        );
        assert!(setup.contains(&"int:-1:[0]".into()));
        assert!(setup.contains(&"int:-1:[1]".into()));
        log.lock().unwrap().clear();
        window.emit_framebuffer_size(800, 400);
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|event| event == "image2:3:3553:32856:[800, 400]:6408:5121:true")
        );
        log.lock().unwrap().clear();
        window.emit_framebuffer_size(0, 400);
        assert!(
            !log.lock()
                .unwrap()
                .iter()
                .any(|event| event.starts_with("image2:"))
        );
        log.lock().unwrap().clear();
        sky.render();
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|event| event == "draw:4:6:5125:0")
        );
        context.close();
        runtime.run_main();
        log.lock().unwrap().clear();
        window.emit_framebuffer_size(900, 500);
        camera.translate(1.0, 2.0);
        assert!(log.lock().unwrap().is_empty());
    }
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    #[derive(Debug, PartialEq)]
    enum Event {
        Bake([i32; 2]),
        Floats(String, Vec<f32>),
        Integer(String, i32),
        Inverse(Mat4),
        Bind(u32, i32),
        Unbind(u32, i32),
        Draw,
    }
    struct Texture {
        id: u32,
        events: Rc<RefCell<Vec<Event>>>,
    }
    impl SkyTexture for Texture {
        fn bind(&self, unit: i32) {
            self.events.borrow_mut().push(Event::Bind(self.id, unit));
        }
        fn unbind(&self, unit: i32) {
            self.events.borrow_mut().push(Event::Unbind(self.id, unit));
        }
    }
    struct Backend {
        events: Rc<RefCell<Vec<Event>>>,
        next: Cell<u32>,
        fail_bake: Cell<bool>,
        fail_draw: Cell<bool>,
        on_draw: RefCell<Option<Box<dyn Fn()>>>,
    }
    impl SkyBackend for Backend {
        fn floats(&self, name: &str, values: &[f32]) {
            self.events
                .borrow_mut()
                .push(Event::Floats(name.into(), values.to_vec()));
        }
        fn integer(&self, name: &str, value: i32) {
            self.events
                .borrow_mut()
                .push(Event::Integer(name.into(), value));
        }
        fn inverse_camera(&self, matrix: Mat4) {
            self.events.borrow_mut().push(Event::Inverse(matrix));
        }
        fn make_stars(&self, size: [i32; 2]) -> Rc<dyn SkyTexture> {
            self.events.borrow_mut().push(Event::Bake(size));
            assert!(!self.fail_bake.get(), "bake failed");
            let id = self.next.get();
            self.next.set(id + 1);
            Rc::new(Texture {
                id,
                events: self.events.clone(),
            })
        }
        fn render_fullscreen(&self) {
            self.events.borrow_mut().push(Event::Draw);
            assert!(!self.fail_draw.get(), "draw failed");
            if let Some(hook) = &*self.on_draw.borrow() {
                hook();
            }
        }
    }
    #[test]
    fn source_sky_constructor_resize_render_reentrant_texture_read_and_free_order() {
        use crate::game_parameter_provider_kt::{
            get_game_parameter_provider, set_game_parameter_provider,
        };
        let previous = get_game_parameter_provider();
        let provider = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        provider.borrow_mut().set_day(0.25);
        set_game_parameter_provider(provider);
        let events = Rc::new(RefCell::new(vec![]));
        let backend = Rc::new(Backend {
            events: events.clone(),
            next: Cell::new(1),
            fail_bake: Cell::new(false),
            fail_draw: Cell::new(false),
            on_draw: RefCell::new(None),
        });
        let map: Rc<dyn SkyTexture> = Rc::new(Texture {
            id: 0,
            events: events.clone(),
        });
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera.clone()),
        ));
        let sky = SourceSky::from_parts(map.clone(), control.clone(), backend.clone());
        assert!(Rc::ptr_eq(&map, &sky.sky_map()));
        assert!(Rc::ptr_eq(&control, &sky.camera_control()));
        assert_eq!(
            *events.borrow(),
            [
                Event::Bake([800, 400]),
                Event::Floats("resolution".into(), vec![800., 400.]),
                Event::Inverse(camera.matrix.borrow().inverse()),
                Event::Integer("sky".into(), 0),
                Event::Integer("stars".into(), 1)
            ]
        );
        events.borrow_mut().clear();
        window.emit_framebuffer_size(800, 400);
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("resolution".into(), vec![800., 400.]),
                Event::Bake([800, 400])
            ]
        );
        events.borrow_mut().clear();
        window.emit_framebuffer_size(0, 200);
        window.emit_framebuffer_size(100, -1);
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("resolution".into(), vec![0., 200.]),
                Event::Floats("resolution".into(), vec![100., -1.])
            ]
        );
        events.borrow_mut().clear();
        sky.render();
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("day".into(), vec![0.25]),
                Event::Bind(0, 0),
                Event::Bind(2, 1),
                Event::Draw,
                Event::Unbind(2, 1),
                Event::Unbind(0, 0)
            ]
        );
        events.borrow_mut().clear();
        backend.fail_bake.set(true);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || window.emit_framebuffer_size(900, 500)
            ))
            .is_err()
        );
        backend.fail_bake.set(false);
        events.borrow_mut().clear();
        sky.render();
        assert!(events.borrow().contains(&Event::Bind(2, 1)));
        let resize_window = window.clone();
        *backend.on_draw.borrow_mut() = Some(Box::new(move || {
            resize_window.emit_framebuffer_size(900, 500)
        }));
        events.borrow_mut().clear();
        sky.render();
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("day".into(), vec![0.25]),
                Event::Bind(0, 0),
                Event::Bind(2, 1),
                Event::Draw,
                Event::Floats("resolution".into(), vec![900., 500.]),
                Event::Bake([900, 500]),
                Event::Unbind(3, 1),
                Event::Unbind(0, 0)
            ]
        );
        *backend.on_draw.borrow_mut() = None;
        backend.fail_draw.set(true);
        events.borrow_mut().clear();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sky.render())).is_err());
        assert_eq!(
            *events.borrow(),
            [
                Event::Floats("day".into(), vec![0.25]),
                Event::Bind(0, 0),
                Event::Bind(3, 1),
                Event::Draw
            ]
        );
        sky.free();
        sky.free();
        events.borrow_mut().clear();
        window.emit_framebuffer_size(100, 200);
        camera.translate(1., 2.);
        assert!(events.borrow().is_empty());
        set_game_parameter_provider(previous);
    }
    #[test]
    fn source_sky_horizon_tracks_panning_and_zoom() {
        assert!(horizon_uv(0.0, 0.0, 360.0).is_nan());
        assert_eq!(horizon_uv(0.0, 10.0, 360.0), 0.5);
        assert_eq!(horizon_uv(360.0, 0.0, 360.0), 0.0);
        assert_eq!(horizon_uv(-360.0, 0.0, 360.0), 1.0);
        assert_eq!(horizon_uv(500.0, 140.0, 360.0), 0.0);
        assert_eq!(horizon_uv(-220.0, 140.0, 360.0), 1.0);
        assert_eq!(horizon_uv(720.0, 0.0, 720.0), 0.0);
    }
    #[test]
    fn fresh_cleanup_references_remove_first_equal_receiver_reference() {
        let events = Rc::new(RefCell::new(vec![]));
        let backend = Rc::new(Backend {
            events: events.clone(),
            next: Cell::new(1),
            fail_bake: Cell::new(false),
            fail_draw: Cell::new(false),
            on_draw: RefCell::new(None),
        });
        let map: Rc<dyn SkyTexture> = Rc::new(Texture { id: 0, events });
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window.clone(), camera),
        ));
        let sky = SourceSky::from_parts(map, control, backend);
        let first = crate::sky_free_resolution_reference::create(sky.state.clone());
        assert!(!Rc::ptr_eq(&first, &sky.resize_callback));
        assert!(crate::sky_resolution_reference::equal(
            &first,
            &sky.resize_callback
        ));
        let camera_reference = crate::sky_free_camera_reference::create(sky.state.clone());
        assert!(camera_reference.key == sky.camera_callback.key);
        window
            .framebuffer_size_callbacks
            .borrow_mut()
            .insert(0, first);
        sky.free();
        let remaining = window.framebuffer_size_callbacks.borrow();
        assert_eq!(remaining.len(), 1);
        assert!(Rc::ptr_eq(&remaining[0], &sky.resize_callback));
        drop(remaining);
        sky.free();
        assert!(window.framebuffer_size_callbacks.borrow().is_empty());
    }
    #[test]
    fn source_sky_wgsl_parses_and_validates() {
        let body = include_str!("../assets/shaders/sky.wgsl")
            .lines()
            .filter(|line| !line.starts_with("#import"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("#{MATERIAL_BIND_GROUP}", "2");
        let interface = "struct VertexOutput { @builtin(position) position:vec4<f32>, @location(0) world_position:vec4<f32>, @location(1) world_normal:vec3<f32>, @location(2) uv:vec2<f32>, };\n";
        let source = format!("{interface}{body}");
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}

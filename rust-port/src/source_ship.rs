//! Ship.java object construction and native runtime composition.
//! GPU operations and resource texture loading are explicit backend inputs.
#![allow(dead_code)]
use crate::{
    camera_2d::SourceCameraCallback,
    camera_control::SourceCameraControl,
    gl_state::StateBackend,
    i_drawable::IDrawable,
    input_handler::InputHandler,
    materials::SourceMaterials,
    passes::native_pass_factory::{NativePassEnvironment, NativePassFactory},
    shaded_model::ShadedModel,
    shader_program::ShaderProgram,
    ship::SourceShipEvents,
    ship_data::SourceShipData,
    ship_render::{ShipRenderBackend, ShipRenderTexture, SourceShipRender},
    ship_resources::{ShipLayer, ShipResourceType},
    ship_shaders::SourceShipShaders,
    ship_struts::{SourceShipStrutShader, SourceShipStruts},
    ship_thumbnail::{ShipThumbnail, ThumbnailResource},
    source_ship_physics::SourceShipPhysics,
    texture::TextureBackend,
    texture_2d::SourceTexture2D,
    window::SourceWindow,
};
use bevy::prelude::Mat4;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub(crate) trait ShipSceneStateBackend: StateBackend {
    fn clear(&mut self, mask: i32);
    fn enable(&mut self, capability: i32);
    fn disable(&mut self, capability: i32);
    fn blend_equation(&mut self, equation: i32);
    fn blend_func(&mut self, source: i32, destination: i32);
}
pub(crate) type TextureResolver =
    Rc<dyn Fn(&ThumbnailResource) -> Result<Option<Arc<SourceTexture2D>>, String>>;
pub(crate) struct SourceShip {
    pub dat: Rc<SourceShipData<SourceMaterials>>,
    pub thumbnail: Rc<ShipThumbnail>,
    pub camera_control: Rc<RefCell<SourceCameraControl>>,
    pub model: ShadedModel,
    pub shader: Arc<ShaderProgram>,
    pub struts: SourceShipStruts,
    pub physics: RefCell<SourceShipPhysics>,
    pub rendering_samplers: Vec<String>,
    pub layers: Vec<ShipLayer>,
    pub layer_names: Vec<String>,
    pub current_layer: i32,
    pub events: RefCell<SourceShipEvents>,
    pub camera_callback: SourceCameraCallback,
    inverse: Rc<RefCell<Mat4>>,
    render_sequence: SourceShipRender,
    static_resources: Arc<SourceShipShaders>,
    texture_resolver: TextureResolver,
    state: Rc<RefCell<dyn ShipSceneStateBackend>>,
}
impl SourceShip {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dat: Rc<SourceShipData<SourceMaterials>>,
        thumbnail: Rc<ShipThumbnail>,
        camera_control: Rc<RefCell<SourceCameraControl>>,
        environment: NativePassEnvironment,
        textures: Arc<Mutex<dyn TextureBackend>>,
        static_resources: Arc<SourceShipShaders>,
        strut_shader: Arc<SourceShipStrutShader>,
        texture_resolver: TextureResolver,
        state: Rc<RefCell<dyn ShipSceneStateBackend>>,
        clock: Box<dyn FnMut() -> i64>,
    ) -> Result<Self, String> {
        // Source superclass arguments evaluate left to right before entering ShadedModel.
        let indices = crate::ship_companion::create_indices(dat.as_ref())?;
        let vertices = crate::ship_companion::create_vertices(dat.as_ref())?;
        let vertex = environment.vertices.none()?;
        let shader = Arc::new(ShaderProgram::new(
            vec![
                vertex,
                static_resources.geometry.clone(),
                static_resources.fragment.clone(),
            ],
            environment.programs.clone(),
            environment.context.clone(),
            &environment.runtime,
        ));
        let model = ShadedModel::from_arrays(
            &indices,
            &vertices,
            2,
            shader.clone(),
            0,
            environment.buffers.clone(),
            environment.arrays.clone(),
            environment.draws.clone(),
            environment.context.clone(),
            &environment.runtime,
        );
        let inverse = Rc::new(RefCell::new(
            camera_control.borrow().camera().matrix.borrow().inverse(),
        ));
        let camera_callback =
            crate::ship_camera_callback::create(Rc::new(()), inverse.clone(), shader.clone());
        let rendering_samplers = crate::ship::ShipLayers::RENDERING_SAMPLERS
            .map(str::to_owned)
            .to_vec();
        let struts = SourceShipStruts::new(
            dat.as_ref(),
            camera_control.clone(),
            &rendering_samplers,
            environment.vertices.none()?,
            strut_shader.shader(),
            static_resources.fragment.clone(),
            environment.programs.clone(),
            environment.buffers.clone(),
            environment.arrays.clone(),
            environment.draws.clone(),
            environment.context.clone(),
            &environment.runtime,
        );
        let mut factory = NativePassFactory(environment.clone());
        let physics = SourceShipPhysics::new(
            dat.clone(),
            textures,
            environment.renderbuffers.clone(),
            &mut factory,
            environment.context.clone(),
            &environment.runtime,
            clock,
        )?;
        environment.state.borrow_mut().stencil_test(true);
        shader.set_floats(
            shader.uniform_location("resolution"),
            &[dat.width as f32, dat.height as f32],
        )?;
        camera_control
            .borrow()
            .camera()
            .add_camera_callback(camera_callback.clone());
        for (unit, name) in rendering_samplers.iter().enumerate() {
            shader.set_ints(shader.uniform_location(name), &[unit as i32])?;
        }
        let layers = thumbnail.ordered_layers().to_vec();
        let layer_names = layers
            .iter()
            .map(|layer| layer.display_name().to_owned())
            .collect();
        let render_sequence = SourceShipRender::new();
        let screen_size = camera_control.borrow().window().screen_size();
        let mut events = SourceShipEvents::new(Mat4::IDENTITY, screen_size);
        events.inverse_matrix = *inverse.borrow();
        let cleanup_control = camera_control.clone();
        let key = camera_callback.key.clone();
        model.model.resource_handle().before_free_local(move || {
            cleanup_control
                .borrow()
                .camera()
                .remove_camera_callback(&key);
        });
        Ok(Self {
            dat,
            thumbnail,
            camera_control,
            model,
            shader,
            struts,
            physics: RefCell::new(physics),
            rendering_samplers,
            layers,
            layer_names,
            current_layer: 0,
            events: RefCell::new(events),
            camera_callback,
            inverse,
            render_sequence,
            static_resources,
            texture_resolver,
            state,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_thumbnail(
        thumbnail: Rc<ShipThumbnail>,
        global: Arc<SourceMaterials>,
        camera_control: Rc<RefCell<SourceCameraControl>>,
        environment: NativePassEnvironment,
        textures: Arc<Mutex<dyn TextureBackend>>,
        static_resources: Arc<SourceShipShaders>,
        strut_shader: Arc<SourceShipStrutShader>,
        texture_resolver: TextureResolver,
        state: Rc<RefCell<dyn ShipSceneStateBackend>>,
        clock: Box<dyn FnMut() -> i64>,
    ) -> Result<Self, String> {
        Self::from_thumbnail_current(
            thumbnail,
            move || global.clone(),
            camera_control,
            environment,
            textures,
            static_resources,
            strut_shader,
            texture_resolver,
            state,
            clock,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_thumbnail_current(
        thumbnail: Rc<ShipThumbnail>,
        global: impl Fn() -> Arc<SourceMaterials>,
        camera_control: Rc<RefCell<SourceCameraControl>>,
        environment: NativePassEnvironment,
        textures: Arc<Mutex<dyn TextureBackend>>,
        static_resources: Arc<SourceShipShaders>,
        strut_shader: Arc<SourceShipStrutShader>,
        texture_resolver: TextureResolver,
        state: Rc<RefCell<dyn ShipSceneStateBackend>>,
        clock: Box<dyn FnMut() -> i64>,
    ) -> Result<Self, String> {
        let dat = Rc::new(SourceShipData::from_thumbnail(
            &crate::ship_thumbnail::SourceShipDataThumbnailCurrent {
                thumbnail: &thumbnail,
                global: &global,
            },
        )?);
        Self::new(
            dat,
            thumbnail,
            camera_control,
            environment,
            textures,
            static_resources,
            strut_shader,
            texture_resolver,
            state,
            clock,
        )
    }
    fn layer(&self) -> Result<&ShipLayer, String> {
        usize::try_from(self.current_layer)
            .ok()
            .and_then(|i| self.layers.get(i))
            .ok_or_else(|| "source layer array index out of bounds".into())
    }
    pub fn display_texture(&self) -> Result<Arc<SourceTexture2D>, String> {
        let resource = self
            .thumbnail
            .get_resource(ShipResourceType::Texture, self.layer()?)
            .ok_or_else(|| "source display resource is null".to_owned())?;
        (self.texture_resolver)(resource)?.ok_or_else(|| "source display texture is null".into())
    }
    pub fn lights_texture(&self, external: bool) -> Result<Arc<SourceTexture2D>, String> {
        let kind = if external {
            ShipResourceType::ExLights
        } else {
            ShipResourceType::InLights
        };
        match self.thumbnail.get_resource(kind, self.layer()?) {
            Some(resource) => Ok((self.texture_resolver)(resource)?
                .unwrap_or_else(|| self.static_resources.black_texture.clone())),
            None => Ok(self.static_resources.black_texture.clone()),
        }
    }
    pub fn start_drag(&self) {
        self.events.borrow_mut().start_drag();
    }
    pub fn update(&self) -> Result<(), String> {
        if self.events.borrow().moving {
            self.physics.borrow_mut().update()?;
        }
        Ok(())
    }
    pub fn set_time(&self, time: f32) {
        self.shader
            .set_floats(self.shader.uniform_location("time"), &[time])
            .unwrap_or_else(|e| panic!("{e}"));
        self.struts.set_time(time);
        self.physics.borrow_mut().set_time(time);
    }
    pub fn free(&self) {
        self.camera_control
            .borrow()
            .camera()
            .remove_camera_callback(&self.camera_callback.key);
    }
    pub fn close(&self) {
        self.model.model.close();
    }
    pub fn freed(&self) -> bool {
        self.model.model.freed()
    }
}
impl InputHandler<SourceWindow> for SourceShip {
    fn on_size(&mut self, blocked: bool, _win: &SourceWindow, height: i32, width: i32) -> bool {
        self.events.borrow_mut().on_size(blocked, height, width)
    }
    fn on_cursor_pos(&mut self, blocked: bool, _win: &SourceWindow, x: f64, y: f64) -> bool {
        let mut events = self.events.borrow_mut();
        events.inverse_matrix = *self.inverse.borrow();
        events.on_cursor(
            blocked,
            x,
            y,
            |offset| {
                self.shader
                    .set_floats(self.shader.uniform_location("u_mouse"), &offset.to_array())
                    .unwrap_or_else(|e| panic!("{e}"))
            },
            |offset| self.struts.set_displacement(offset),
        )
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        _win: &SourceWindow,
        button: i32,
        action: i32,
        _mods: i32,
    ) -> bool {
        self.events.borrow_mut().on_mouse_button(
            blocked,
            button,
            action,
            |offset| self.physics.borrow_mut().move_by(offset),
            |offset| {
                self.shader
                    .set_floats(self.shader.uniform_location("u_mouse"), &offset.to_array())
                    .unwrap_or_else(|e| panic!("{e}"))
            },
            |offset| self.struts.set_displacement(offset),
        )
    }
}
struct NativeTexture(Arc<SourceTexture2D>);
impl ShipRenderTexture for NativeTexture {
    fn bind(&self, unit: i32) {
        self.0.texture.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.0.texture.unbind_unit(unit);
    }
}
struct RenderAdapter<'a>(&'a SourceShip);
impl StateBackend for RenderAdapter<'_> {
    fn get_integer(&mut self, p: i32) -> i32 {
        self.0.state.borrow_mut().get_integer(p)
    }
    fn stencil_mask(&mut self, m: i32) {
        self.0.state.borrow_mut().stencil_mask(m);
    }
    fn stencil_func(&mut self, f: i32, r: i32, m: i32) {
        self.0.state.borrow_mut().stencil_func(f, r, m);
    }
    fn stencil_op(&mut self, f: i32, d: i32, s: i32) {
        self.0.state.borrow_mut().stencil_op(f, d, s);
    }
}
impl ShipRenderBackend for RenderAdapter<'_> {
    fn display_arg(&mut self, name: &str, v: &[f32]) {
        self.0
            .shader
            .set_floats(self.0.shader.uniform_location(name), v)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    fn struts_arg(&mut self, name: &str, v: &[f32]) {
        self.0
            .struts
            .shader
            .set_floats(self.0.struts.shader.uniform_location(name), v)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    fn clear(&mut self, m: i32) {
        self.0.state.borrow_mut().clear(m);
    }
    fn enable(&mut self, c: i32) {
        self.0.state.borrow_mut().enable(c);
    }
    fn disable(&mut self, c: i32) {
        self.0.state.borrow_mut().disable(c);
    }
    fn blend_equation(&mut self, e: i32) {
        self.0.state.borrow_mut().blend_equation(e);
    }
    fn blend_func(&mut self, s: i32, d: i32) {
        self.0.state.borrow_mut().blend_func(s, d);
    }
    fn texture(&mut self, slot: i32) -> Rc<dyn ShipRenderTexture> {
        use crate::gl_data_holder::SourceGlDataHolder;
        let texture = match slot {
            0 => self.0.display_texture().unwrap_or_else(|e| panic!("{e}")),
            1 => self.0.physics.borrow().pos_vel.source_texture(),
            2 => self.0.physics.borrow().mask_struts.source_texture(),
            3 => self.0.physics.borrow().water.source_texture(),
            4 => self
                .0
                .lights_texture(false)
                .unwrap_or_else(|e| panic!("{e}")),
            5 => self
                .0
                .lights_texture(true)
                .unwrap_or_else(|e| panic!("{e}")),
            _ => panic!("unknown source ship texture slot"),
        };
        Rc::new(NativeTexture(texture))
    }
    fn render_surface(&mut self) {
        self.0.model.render();
    }
    fn render_struts(&mut self) {
        self.0.struts.render();
    }
    fn display_time(&mut self, t: f32) {
        self.display_arg("time", &[t]);
    }
    fn struts_time(&mut self, t: f32) {
        self.0.struts.set_time(t);
    }
    fn physics_time(&mut self, t: f32) {
        self.0.physics.borrow_mut().set_time(t);
    }
}
impl IDrawable for SourceShip {
    fn render(&self) {
        self.render_sequence.render(&mut RenderAdapter(self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Log = Arc<Mutex<Vec<String>>>;
    struct State(Log);
    impl StateBackend for State {
        fn get_integer(&mut self, p: i32) -> i32 {
            self.0.lock().unwrap().push(format!("get:{p}"));
            p
        }
        fn stencil_mask(&mut self, m: i32) {
            self.0.lock().unwrap().push(format!("mask:{m}"));
        }
        fn stencil_func(&mut self, f: i32, r: i32, m: i32) {
            self.0.lock().unwrap().push(format!("func:{f}:{r}:{m}"));
        }
        fn stencil_op(&mut self, f: i32, d: i32, s: i32) {
            self.0.lock().unwrap().push(format!("op:{f}:{d}:{s}"));
        }
    }
    impl ShipSceneStateBackend for State {
        fn clear(&mut self, m: i32) {
            self.0.lock().unwrap().push(format!("clear:{m}"));
        }
        fn enable(&mut self, c: i32) {
            self.0.lock().unwrap().push(format!("enable-cap:{c}"));
        }
        fn disable(&mut self, c: i32) {
            self.0.lock().unwrap().push(format!("disable-cap:{c}"));
        }
        fn blend_equation(&mut self, e: i32) {
            self.0.lock().unwrap().push(format!("equation:{e}"));
        }
        fn blend_func(&mut self, s: i32, d: i32) {
            self.0.lock().unwrap().push(format!("blend:{s}:{d}"));
        }
    }
    #[test]
    fn native_ship_constructor_render_placement_and_inherited_cleanup_follow_source() {
        let log = Log::default();
        let environment = crate::passes::native_pass_factory::tests::fixture(log.clone()).0;
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let statics = Arc::new(
            SourceShipShaders::new(
                textures.clone(),
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
                |_| {},
            )
            .unwrap(),
        );
        let strut_shader = Arc::new(
            SourceShipStrutShader::new(
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
            )
            .unwrap(),
        );
        let (window, _, window_log) = crate::window::tests::fixture();
        let control = Rc::new(RefCell::new(SourceCameraControl::new(window.clone())));
        let camera = control.borrow().camera();
        let dat = Rc::new(SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(SourceMaterials::default()),
            Rc::new(RefCell::new(vec![
                Some(Arc::new(
                    crate::materials::Material::default()
                ));
                4
            ])),
            2,
            2,
        ));
        let thumbnail = Rc::new(
            ShipThumbnail::new(
                [
                    "Native_base.png",
                    "Native_inlights.png",
                    "Native_exterior_texture.png",
                ]
                .map(|path| {
                    crate::ship_resources::parse_resource_path(std::path::Path::new(path)).unwrap()
                }),
            )
            .unwrap(),
        );
        let resolver_texture = statics.black_texture.clone();
        let resolver_log = log.clone();
        let resolver: TextureResolver = Rc::new(move |resource| {
            if matches!(resource,ThumbnailResource::File(file) if file.resource_type==ShipResourceType::InLights)
            {
                resolver_log
                    .lock()
                    .unwrap()
                    .push("resolve-null-light".into());
                return Ok(None);
            }
            resolver_log.lock().unwrap().push("resolve-display".into());
            Ok(Some(resolver_texture.clone()))
        });
        log.lock().unwrap().clear();
        window_log.lock().unwrap().clear();
        let clock_log = log.clone();
        let mut ship = SourceShip::new(
            dat.clone(),
            thumbnail.clone(),
            control.clone(),
            environment.clone(),
            textures,
            statics.clone(),
            strut_shader,
            resolver,
            Rc::new(RefCell::new(State(log.clone()))),
            Box::new(move || {
                clock_log.lock().unwrap().push("clock".into());
                123
            }),
        )
        .unwrap();
        assert!(Rc::ptr_eq(&ship.dat, &dat));
        assert!(Rc::ptr_eq(&ship.thumbnail, &thumbnail));
        assert!(Rc::ptr_eq(&ship.camera_control, &control));
        assert!(Rc::ptr_eq(&ship.physics.borrow().dat, &dat));
        assert_eq!(ship.layer_names, ["Default", "exterior"]);
        assert_eq!(ship.current_layer, 0);
        assert!(!ship.events.borrow().moving);
        assert_eq!(ship.events.borrow().screen_size, [400, 200]);
        assert_eq!(*window_log.lock().unwrap(), ["screen"]);
        let setup = log.lock().unwrap().clone();
        assert_eq!(
            setup
                .iter()
                .filter(|s| s.as_str() == "create program")
                .count(),
            13
        );
        assert_eq!(
            setup.iter().filter(|s| s.as_str() == "create_vao").count(),
            3
        );
        let validations: Vec<_> = setup
            .iter()
            .enumerate()
            .filter_map(|(i, s)| (s == "validate").then_some(i))
            .collect();
        assert_eq!(validations.len(), 3);
        let clock = setup.iter().position(|s| s == "clock").unwrap();
        assert!(validations[2] < clock);
        assert_eq!(
            &setup[clock + 1..clock + 5],
            ["stencil:true", "use:9", "float:-1:[2.0, 2.0]", "use:0"]
        );
        let matrix = setup[clock..]
            .iter()
            .position(|s| s.starts_with("matrix:"))
            .unwrap()
            + clock;
        let first_sampler = setup[clock..]
            .iter()
            .position(|s| s == "int:-1:[0]")
            .unwrap()
            + clock;
        assert!(matrix < first_sampler);
        assert!(Arc::ptr_eq(
            &ship.lights_texture(false).unwrap(),
            &statics.black_texture
        ));
        assert!(Arc::ptr_eq(
            &ship.lights_texture(true).unwrap(),
            &statics.black_texture
        ));
        log.lock().unwrap().clear();
        ship.update().unwrap();
        assert!(log.lock().unwrap().is_empty());
        ship.render();
        let render = log.lock().unwrap().clone();
        assert_eq!(
            render
                .iter()
                .filter(|s| s.as_str() == "resolve-display")
                .count(),
            2
        );
        assert_eq!(
            render
                .iter()
                .filter(|s| s.as_str() == "resolve-null-light")
                .count(),
            2
        );
        let hull = render.iter().position(|s| s == "draw:0:1:5125:0").unwrap();
        let struts = render.iter().position(|s| s == "draw:1:12:5125:0").unwrap();
        assert!(hull < struts);
        assert_eq!(
            &render[render.len() - 2..],
            ["disable-cap:3042", "disable-cap:2960"]
        );
        ship.current_layer = -1;
        assert!(ship.display_texture().err().unwrap().contains("layer"));
        ship.current_layer = 1;
        assert!(Arc::ptr_eq(
            &ship.display_texture().unwrap(),
            &statics.black_texture
        ));
        log.lock().unwrap().clear();
        assert!(Arc::ptr_eq(
            &ship.lights_texture(false).unwrap(),
            &statics.black_texture
        ));
        assert!(log.lock().unwrap().is_empty()); // Selected layer does not inherit default lights.
        ship.current_layer = 0;
        log.lock().unwrap().clear();
        ship.on_cursor_pos(true, &window, 300., 50.);
        assert!(log.lock().unwrap().is_empty());
        ship.start_drag();
        ship.on_cursor_pos(false, &window, 300., 50.);
        assert_eq!(ship.events.borrow().offset, bevy::prelude::Vec4::ZERO);
        ship.on_mouse_button(true, &window, 0, 0, 0);
        assert!(!ship.events.borrow().moving);
        log.lock().unwrap().clear();
        ship.on_mouse_button(false, &window, 0, 0, 0);
        assert!(ship.events.borrow().moving);
        let commit = log.lock().unwrap().clone();
        let move_draw = commit.iter().position(|s| s == "draw:4:6:5125:0").unwrap();
        assert_eq!(
            commit[move_draw..]
                .iter()
                .filter(|s| s.as_str() == "float:-1:[0.0, 0.0]")
                .count(),
            2
        );
        log.lock().unwrap().clear();
        ship.set_time(3.5);
        assert_eq!(
            &log.lock().unwrap()[..6],
            [
                "use:9",
                "float:-1:[3.5]",
                "use:0",
                "use:9",
                "float:-1:[3.5]",
                "use:0"
            ]
        );
        ship.on_size(true, &window, 0, -7);
        assert_eq!(ship.events.borrow().screen_size, [-7, 0]);
        log.lock().unwrap().clear();
        ship.close();
        assert!(ship.freed());
        assert!(!ship.struts.freed());
        assert!(!ship.physics.borrow().freed());
        camera.translate(1., 2.);
        assert_eq!(*ship.inverse.borrow(), camera.matrix.borrow().inverse());
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|s| s.starts_with("matrix:"))
                .count(),
            2
        );
        environment.runtime.run_main();
        log.lock().unwrap().clear();
        camera.translate(1., 2.);
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|s| s.starts_with("matrix:"))
                .count(),
            1
        );
        environment.context.close();
        environment.runtime.run_main();
    }
    #[test]
    fn native_move_tool_resolves_current_ship_at_update_and_commits_its_drag() {
        use crate::tools::move_tool::SourceMoveTool;
        use crate::tools::tool::SourceTool;
        use std::cell::Cell;
        let log = Log::default();
        let environment = crate::passes::native_pass_factory::tests::fixture(log.clone()).0;
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let statics = Arc::new(
            SourceShipShaders::new(
                textures.clone(),
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
                |_| {},
            )
            .unwrap(),
        );
        let strut_shader = Arc::new(
            SourceShipStrutShader::new(
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
            )
            .unwrap(),
        );
        let (window, _, _) = crate::window::tests::fixture();
        let control = Rc::new(RefCell::new(SourceCameraControl::new(window.clone())));
        let dat = Rc::new(SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(SourceMaterials::default()),
            Rc::new(RefCell::new(vec![
                Some(Arc::new(
                    crate::materials::Material::default()
                ));
                4
            ])),
            2,
            2,
        ));
        let thumbnail = Rc::new(
            ShipThumbnail::new(["Native_base.png"].map(|path| {
                crate::ship_resources::parse_resource_path(std::path::Path::new(path)).unwrap()
            }))
            .unwrap(),
        );
        let black = statics.black_texture.clone();
        let resolver: TextureResolver = Rc::new(move |_| Ok(Some(black.clone())));
        let make_ship = || {
            Rc::new(RefCell::new(
                SourceShip::new(
                    dat.clone(),
                    thumbnail.clone(),
                    control.clone(),
                    environment.clone(),
                    textures.clone(),
                    statics.clone(),
                    strut_shader.clone(),
                    resolver.clone(),
                    Rc::new(RefCell::new(State(log.clone()))),
                    Box::new(|| 123),
                )
                .unwrap(),
            ))
        };
        let old = make_ship();
        let replacement = make_ship();
        let mut globals = crate::main_globals::MainGlobals::default();
        let list = globals.ship_list();
        list.borrow_mut().push(old.clone());
        globals.global_ship = Some(old.clone());
        assert!(Rc::ptr_eq(&globals.global_ship(), &old));
        globals.global_ship = Some(replacement.clone());
        assert!(Rc::ptr_eq(&globals.global_ship(), &replacement));
        assert!(Rc::ptr_eq(&globals.ship_list().borrow()[0], &old)); // Setter does not change the list.
        let brush_current = Rc::new(RefCell::new(old.clone()));
        let brush_retained = brush_current.clone();
        let brush_reads = Rc::new(Cell::new(0));
        let brush_calls = brush_reads.clone();
        let mut provider = crate::tools::source_brush_ship::current_ship_provider(
            crate::tools::source_brush_ship::BrushTarget::Water,
            move || {
                brush_calls.set(brush_calls.get() + 1);
                brush_retained.borrow().clone()
            },
        );
        assert_eq!(brush_reads.get(), 0);
        let first = provider();
        assert!(Arc::ptr_eq(&first, &provider()));
        *brush_current.borrow_mut() = replacement.clone();
        let second = provider();
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&second, &provider()));
        assert_eq!(brush_reads.get(), 4);
        use crate::gl_data_holder::SourceGlDataHolder;
        let replacement_water: Arc<dyn crate::framebuffer_target::FramebufferTarget> =
            replacement.borrow().physics.borrow().water.source_texture();
        assert!(Arc::ptr_eq(&second.target(), &replacement_water));
        // Finish each newly loaded ship's initial placement before using Move.
        for ship in [&old, &replacement] {
            ship.borrow_mut().on_mouse_button(false, &window, 0, 0, 0);
        }
        assert!(old.borrow().events.borrow().moving);
        assert!(replacement.borrow().events.borrow().moving);
        replacement
            .borrow_mut()
            .on_cursor_pos(false, &window, 200., 100.);
        let cursor = replacement.borrow().events.borrow().cursor;
        let current = Rc::new(RefCell::new(old.clone()));
        let retained = current.clone();
        let reads = Rc::new(Cell::new(0));
        let calls = reads.clone();
        let reader = crate::file_reader::FileReader {
            ss_home: "assets".into(),
            ss_resources: "assets".into(),
            bundled_resources: "assets".into(),
        };
        let mut tool = SourceMoveTool::with_current_ship(
            &reader,
            textures,
            environment.context.clone(),
            &environment.runtime,
            move || {
                calls.set(calls.get() + 1);
                retained.borrow().clone()
            },
        )
        .unwrap();
        assert_eq!(reads.get(), 0);
        assert!(tool.texture().width > 0);
        assert!(tool.active_texture().width > 0);
        assert_eq!(tool.name(), "Move");
        assert!(tool.on_mouse_button(false, &window, 0, 1, 0));
        assert_eq!(reads.get(), 0);
        *current.borrow_mut() = replacement.clone();
        log.lock().unwrap().clear();
        tool.update();
        tool.update();
        assert_eq!(reads.get(), 1);
        assert!(log.lock().unwrap().is_empty()); // startDrag only changes Ship event state.
        assert!(old.borrow().events.borrow().moving);
        assert!(!replacement.borrow().events.borrow().moving);
        assert_eq!(replacement.borrow().events.borrow().base, -cursor);
        replacement
            .borrow_mut()
            .on_cursor_pos(false, &window, 250., 125.);
        let offset = replacement.borrow().events.borrow().offset;
        assert_ne!(offset, bevy::prelude::Vec4::ZERO);
        log.lock().unwrap().clear();
        replacement.borrow().update().unwrap();
        assert!(log.lock().unwrap().is_empty()); // Physics remains frozen during drag.
        replacement
            .borrow_mut()
            .on_mouse_button(true, &window, 0, 0, 0);
        assert!(!replacement.borrow().events.borrow().moving);
        assert!(log.lock().unwrap().is_empty());
        replacement
            .borrow_mut()
            .on_mouse_button(false, &window, 0, 0, 0);
        assert!(replacement.borrow().events.borrow().moving);
        let commit = log.lock().unwrap().clone();
        assert_eq!(
            commit
                .iter()
                .filter(|s| s.as_str() == "draw:4:6:5125:0")
                .count(),
            1
        );
        assert_eq!(
            commit
                .iter()
                .filter(|s| s.as_str() == "float:-1:[0.0, 0.0]")
                .count(),
            2
        );
        log.lock().unwrap().clear();
        replacement
            .borrow_mut()
            .on_mouse_button(false, &window, 0, 0, 0);
        assert!(log.lock().unwrap().is_empty());
        assert!(!tool.on_mouse_button(false, &window, 0, 1, 1));
        tool.update();
        assert_eq!(reads.get(), 1);
        assert!(tool.on_mouse_button(true, &window, 0, 1, 0));
        tool.update();
        assert_eq!(reads.get(), 1);
        environment.context.close();
        environment.runtime.run_main();
    }
    #[test]
    fn original_brush_constructors_draw_into_retained_native_ship_holders() {
        use crate::{
            gl_data_holder::SourceGlDataHolder,
            gui::GuiToolFactory,
            tools::{
                brush_runtime::BrushShip,
                source_brush_ship::{BrushTarget, SourceBrushShip},
            },
        };
        let log = Log::default();
        let environment = crate::passes::native_pass_factory::tests::fixture(log.clone()).0;
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let statics = Arc::new(
            SourceShipShaders::new(
                textures.clone(),
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
                |_| {},
            )
            .unwrap(),
        );
        let strut_shader = Arc::new(
            SourceShipStrutShader::new(
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
            )
            .unwrap(),
        );
        let (window, _, _) = crate::window::tests::fixture();
        let control = Rc::new(RefCell::new(SourceCameraControl::new(window.clone())));
        let dat = Rc::new(SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(SourceMaterials::default()),
            Rc::new(RefCell::new(vec![
                Some(Arc::new(
                    crate::materials::Material::default()
                ));
                4
            ])),
            2,
            2,
        ));
        let thumbnail = Rc::new(
            ShipThumbnail::new(["Native_base.png"].map(|path| {
                crate::ship_resources::parse_resource_path(std::path::Path::new(path)).unwrap()
            }))
            .unwrap(),
        );
        let black = statics.black_texture.clone();
        let state: Rc<RefCell<dyn ShipSceneStateBackend>> =
            Rc::new(RefCell::new(State(log.clone())));
        let ship = Rc::new(RefCell::new(
            SourceShip::new(
                dat,
                thumbnail,
                control.clone(),
                environment.clone(),
                textures.clone(),
                statics,
                strut_shader,
                Rc::new(move |_| Ok(Some(black.clone()))),
                state.clone(),
                Box::new(|| 123),
            )
            .unwrap(),
        ));
        let positions = ship.borrow().physics.borrow().pos_vel.source_texture();
        let mask = ship.borrow().physics.borrow().mask_struts.source_texture();
        let water = ship.borrow().physics.borrow().water.source_texture();
        let parameters = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        let retained = ship.clone();
        let reads = Rc::new(std::cell::Cell::new(0));
        let getter_reads = reads.clone();
        let mut factory = crate::gui_tool_factory::SourceGuiToolFactory::native(
            crate::file_reader::FileReader {
                ss_home: "assets".into(),
                ss_resources: "assets".into(),
                bundled_resources: "assets".into(),
            },
            environment.clone(),
            textures.clone(),
            state.clone(),
            Rc::new(move || {
                getter_reads.set(getter_reads.get() + 1);
                retained.clone()
            }),
        );
        assert_eq!(reads.get(), 0);
        for kind in 0..3 {
            let receiver: Arc<dyn BrushShip> = Arc::new(SourceBrushShip {
                ship: ship.clone(),
                target: if kind == 0 {
                    BrushTarget::MaskStruts
                } else {
                    BrushTarget::Water
                },
            });
            let target = if kind == 0 {
                mask.clone()
            } else {
                water.clone()
            };
            let expected: Arc<dyn crate::framebuffer_target::FramebufferTarget> = target.clone();
            assert!(Arc::ptr_eq(&receiver.target(), &expected));
            let expected_positions: Arc<dyn crate::framebuffer_target::FramebufferTarget> =
                positions.clone();
            assert!(Arc::ptr_eq(&receiver.positions(), &expected_positions));
            assert_eq!([receiver.width(), receiver.height()], [2, 2]);
            log.lock().unwrap().clear();
            let tool = match kind {
                0 => factory
                    .break_tool(control.clone(), parameters.clone())
                    .unwrap(),
                1 => factory
                    .flood_tool(control.clone(), parameters.clone())
                    .unwrap(),
                _ => factory
                    .dry_tool(control.clone(), parameters.clone())
                    .unwrap(),
            };
            let construction = log.lock().unwrap().clone();
            assert_eq!(reads.get(), kind * 2 + 1);
            assert!(
                construction.contains(&format!("attach_tex:36160:36064:{}:0", target.texture.id()))
            );
            assert!(construction.contains(&"attach_rb:36160:33306:36161:51".into()));
            log.lock().unwrap().clear();
            tool.borrow_mut().update();
            assert_eq!(reads.get(), kind * 2 + 1);
            assert!(
                !log.lock()
                    .unwrap()
                    .iter()
                    .any(|s| s == "viewport:[0, 0, 2, 2]")
            );
            tool.borrow_mut().on_cursor_pos(false, &window, 200., 100.);
            tool.borrow_mut().on_mouse_button(false, &window, 0, 1, 0);
            log.lock().unwrap().clear();
            tool.borrow_mut().update();
            let draw = log.lock().unwrap().clone();
            assert_eq!(reads.get(), kind * 2 + 2);
            assert!(draw.contains(&"viewport:[0, 0, 2, 2]".into()));
            assert!(!draw.iter().any(|s| s == "create_fbo"));
            let pos = draw
                .iter()
                .position(|s| s == &format!("texture:3553:{}", positions.texture.id()))
                .unwrap();
            let dst = draw
                .iter()
                .position(|s| s == &format!("texture:3553:{}", target.texture.id()))
                .unwrap();
            assert!(pos < dst);
            assert_eq!(
                draw.iter()
                    .filter(|s| s.as_str() == "draw:4:6:5125:0")
                    .count(),
                2
            );
            assert!(draw.contains(&"enable-cap:3042".into()));
            assert!(draw.contains(&"disable-cap:3042".into()));
            tool.borrow_mut().on_mouse_button(false, &window, 0, 0, 0);
        }
        ship.borrow_mut().on_mouse_button(false, &window, 0, 0, 0);
        assert!(ship.borrow().events.borrow().moving);
        let move_tool = factory.move_tool().unwrap();
        assert_eq!(reads.get(), 6);
        assert_eq!(move_tool.borrow().name(), "Move");
        move_tool
            .borrow_mut()
            .on_mouse_button(false, &window, 0, 1, 0);
        move_tool.borrow_mut().update();
        assert_eq!(reads.get(), 7);
        assert!(!ship.borrow().events.borrow().moving);
        environment.context.close();
        environment.runtime.run_main();
    }
    #[test]
    fn thumbnail_overload_loads_real_native_resources_and_retains_texture_cache() {
        let _serial = crate::main_globals::MATERIALS_TEST_LOCK.lock().unwrap();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!("Native{}{}", std::process::id(), stamp);
        let base_path = std::env::temp_dir().join(format!("{name}_base.png"));
        let exterior_path = std::env::temp_dir().join(format!("{name}_exterior_texture.png"));
        image::RgbaImage::from_pixel(2, 2, image::Rgba([0x12, 0x34, 0x56, 123]))
            .save(&base_path)
            .unwrap();
        image::RgbaImage::from_pixel(2, 2, image::Rgba([7, 8, 9, 255]))
            .save(&exterior_path)
            .unwrap();
        let thumbnail = Rc::new(
            ShipThumbnail::new(
                [&base_path, &exterior_path]
                    .map(|path| crate::ship_resources::parse_resource_path(path).unwrap()),
            )
            .unwrap(),
        );
        let global =
            Arc::new(SourceMaterials::from_json(r##"[{"color":"#123456","mass":100}]"##).unwrap());
        let log = Log::default();
        let environment = crate::passes::native_pass_factory::tests::fixture(log.clone()).0;
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let texture_environment = crate::ship_resources::SourceTextureEnvironment {
            backend: textures.clone(),
            context: environment.context.clone(),
            runtime: environment.runtime.clone(),
        };
        let resolver =
            crate::ship_resources::source_texture_resolver_globals(texture_environment.clone());
        crate::main_globals::set_global_materials(global.clone());
        let statics = Arc::new(
            SourceShipShaders::new(
                textures.clone(),
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
                |_| {},
            )
            .unwrap(),
        );
        let strut_shader = Arc::new(
            SourceShipStrutShader::new(
                environment.shaders.clone(),
                environment.context.clone(),
                &environment.runtime,
            )
            .unwrap(),
        );
        let (window, _, _) = crate::window::tests::fixture();
        let control = Rc::new(RefCell::new(SourceCameraControl::new(window.clone())));
        let state: Rc<RefCell<dyn ShipSceneStateBackend>> =
            Rc::new(RefCell::new(State(log.clone())));
        let mut reset = crate::gui_reset_ship::SourceResetShipOperations::native(
            environment.clone(),
            textures.clone(),
            statics.clone(),
            strut_shader.clone(),
            resolver.clone(),
            state.clone(),
            Rc::new(|| 0),
        );
        let mut ship = SourceShip::from_thumbnail(
            thumbnail.clone(),
            global.clone(),
            control.clone(),
            environment.clone(),
            textures,
            statics,
            strut_shader,
            resolver,
            state,
            Box::new(|| 0),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&ship.dat.materials, &global));
        assert_eq!((ship.dat.width, ship.dat.height), (2, 2));
        assert!(
            ship.dat
                .material_buffer
                .borrow()
                .iter()
                .all(Option::is_some)
        );
        let display = ship.display_texture().unwrap();
        assert_eq!(
            *display.img.as_ref().unwrap().lock().unwrap(),
            [0x12, 0x34, 0x56, 123].repeat(4)
        );
        assert!(Arc::ptr_eq(&display, &ship.display_texture().unwrap()));
        ship.render();
        ship.current_layer = ship
            .layers
            .iter()
            .position(|layer| layer.name() == "exterior")
            .unwrap() as i32;
        let exterior = ship.display_texture().unwrap();
        assert_eq!(
            *exterior.img.as_ref().unwrap().lock().unwrap(),
            [7, 8, 9, 255].repeat(4)
        );
        let resource = match thumbnail
            .get_resource(ShipResourceType::Texture, ship.layer().unwrap())
            .unwrap()
        {
            ThumbnailResource::File(file) => file,
            _ => panic!("expected exterior file"),
        };
        assert!(Arc::ptr_eq(
            &exterior,
            &resource
                .clone()
                .source_texture_now(&texture_environment)
                .unwrap()
        ));
        log.lock().unwrap().clear();
        resource.free_resources();
        assert!(exterior.texture.freed());
        let deleted = format!("delete_tex:{}", exterior.texture.id());
        assert!(!log.lock().unwrap().contains(&deleted));
        environment.runtime.run_main();
        assert!(log.lock().unwrap().contains(&deleted));
        let replacement = resource.source_texture_now(&texture_environment).unwrap();
        assert!(!Arc::ptr_eq(&exterior, &replacement));
        assert!(resource.release_if_idle(resource.last_used_ms().wrapping_add(10001)));
        assert!(replacement.texture.freed());
        // Bind Main's retained list before GUI reset mutates it; no handler snapshot.
        use crate::gui::ResetShipOperations;
        crate::main_globals::set_global_materials(global.clone());
        let old = Rc::new(RefCell::new(ship));
        reset.set_global_ship(old.clone());
        reset.clear_ship_list();
        reset.add_ship(old.clone());
        let shared_list = crate::main_globals::get_ship_list();
        struct CapturingGui;
        impl InputHandler<SourceWindow> for CapturingGui {
            fn on_cursor_pos(&mut self, _: bool, _: &SourceWindow, _: f64, _: f64) -> bool {
                true
            }
        }
        crate::main_ship_input::register(
            &window,
            Rc::new(RefCell::new(CapturingGui)),
            control.clone(),
            shared_list.clone(),
        );
        let current = reset.global_ship();
        let thumb = reset.thumbnail(&current);
        assert!(Rc::ptr_eq(&thumb, &thumbnail));
        let current = reset.global_ship();
        reset.close_ship(&current);
        assert!(old.borrow().freed());
        assert!(!old.borrow().physics.borrow().freed());
        let next = reset.construct_ship(thumb, control.clone());
        assert!(!Rc::ptr_eq(&old, &next));
        assert!(Rc::ptr_eq(&next.borrow().thumbnail, &thumbnail));
        assert!(Rc::ptr_eq(&next.borrow().camera_control, &control));
        assert!(Arc::ptr_eq(&next.borrow().dat.materials, &global));
        assert!(!next.borrow().events.borrow().moving);
        reset.set_global_ship(next.clone());
        reset.clear_ship_list();
        let current = reset.global_ship();
        reset.add_ship(current);
        assert!(Rc::ptr_eq(
            &shared_list,
            &crate::main_globals::get_ship_list()
        ));
        assert_eq!(shared_list.borrow().len(), 1);
        assert!(Rc::ptr_eq(&shared_list.borrow()[0], &next));
        let old_cursor = old.borrow().events.borrow().cursor;
        let old_size = old.borrow().events.borrow().screen_size;
        window.emit_size(77, 55);
        assert_eq!(next.borrow().events.borrow().screen_size, [77, 55]);
        assert_eq!(old.borrow().events.borrow().screen_size, old_size);
        window.emit_cursor_pos(30., 20.);
        assert_eq!(old.borrow().events.borrow().cursor, old_cursor);
        assert_ne!(
            next.borrow().events.borrow().offset,
            bevy::prelude::Vec4::ZERO
        );
        log.lock().unwrap().clear();
        window.emit_mouse_button(0, 0, 0);
        assert!(next.borrow().events.borrow().moving);
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|s| s.as_str() == "draw:4:6:5125:0")
                .count(),
            1
        );
        // Source's second input stack begins unblocked even when GUI captures in the first.
        reset.clear_ship_list();
        log.lock().unwrap().clear();
        window.emit_cursor_pos(32., 21.);
        assert!(log.lock().unwrap().is_empty());
        // Main's initial load builds a fresh thumbnail/ship, publishes it, and
        // appends without closing or clearing a ship already in the shared list.
        struct StartupFiles(Vec<std::path::PathBuf>);
        impl crate::main_flat_files::Files for StartupFiles {
            fn is_directory(&self, path: &std::path::Path) -> bool {
                path == std::path::Path::new("startup-root")
            }
            fn list_files(&self, _: &std::path::Path) -> Option<Vec<std::path::PathBuf>> {
                Some(self.0.clone())
            }
        }
        impl crate::main_initial_ship::Files for StartupFiles {
            fn is_file(&self, path: &std::path::Path) -> bool {
                crate::main_ship_file_predicate::invoke(path)
            }
        }
        shared_list.borrow_mut().push(next.clone());
        let mut startup = crate::main_initial_ship::NativeOperations {
            factory: reset,
            control: control.clone(),
        };
        let initial = crate::main_initial_ship::load(
            std::path::Path::new("startup-root"),
            &StartupFiles(vec![base_path.clone(), exterior_path.clone()]),
            &mut startup,
        )
        .unwrap();
        assert!(Rc::ptr_eq(
            &crate::main_globals::get_global_ship(),
            &initial
        ));
        assert_eq!(shared_list.borrow().len(), 2);
        assert!(Rc::ptr_eq(&shared_list.borrow()[0], &next));
        assert!(Rc::ptr_eq(&shared_list.borrow()[1], &initial));
        assert!(!next.borrow().freed());
        assert!(!Rc::ptr_eq(&initial.borrow().thumbnail, &thumbnail));
        assert!(Rc::ptr_eq(&initial.borrow().camera_control, &control));
        assert!(Arc::ptr_eq(&initial.borrow().dat.materials, &global));
        assert_eq!(
            (initial.borrow().dat.width, initial.borrow().dat.height),
            (2, 2)
        );
        assert!(!initial.borrow().events.borrow().moving);
        environment.context.close();
        environment.runtime.run_main();
        std::fs::remove_file(base_path).unwrap();
        std::fs::remove_file(exterior_path).unwrap();
    }
}

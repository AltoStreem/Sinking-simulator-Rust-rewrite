//! Connect translated pass construction to Shader/ShaderProgram, FullScreen,
//! RenderBuffer and TexturedFBO. Platform GPU operations remain backend inputs.
#![allow(dead_code)]
use super::{
    pass_builder::{NamedTexture, PassFactory},
    shader_pass_backend::ShaderPassBackend,
    target_pass::{StencilTarget, TargetBinding},
};
use crate::{
    fbo::{Fbo, FramebufferBackend},
    gl_state::{GlState, StateBackend},
    i_drawable::IDrawable,
    model::ModelBackend,
    physics_full_screen::SourcePhysicsFullscreen,
    render_buffer::{RenderBuffer, RenderBufferBackend, SharedStencilTarget},
    resource::{ResourceHandle, ResourceRuntime},
    shader::{Shader, ShaderBackend},
    shader_program::{ProgramBackend, ShaderProgram},
    textured_fbo::TexturedFbo,
    vao::VertexArrayBackend,
    vbo::BufferBackend,
    vertex_shaders::SourceVertexShaders,
};
use std::{
    any::Any,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub(crate) trait PassStateBackend: StateBackend {
    fn stencil_test(&mut self, enabled: bool);
    fn color_mask(&mut self, mask: [bool; 4]);
}

#[derive(Clone)]
pub(crate) struct NativePassEnvironment {
    pub shaders: Arc<Mutex<dyn ShaderBackend>>,
    pub programs: Arc<Mutex<dyn ProgramBackend>>,
    pub vertices: Arc<SourceVertexShaders>,
    pub buffers: Arc<Mutex<dyn BufferBackend>>,
    pub arrays: Arc<Mutex<dyn VertexArrayBackend>>,
    pub draws: Arc<Mutex<dyn ModelBackend>>,
    pub renderbuffers: Arc<Mutex<dyn RenderBufferBackend>>,
    pub framebuffers: Arc<Mutex<dyn FramebufferBackend>>,
    pub state: Rc<RefCell<dyn PassStateBackend>>,
    pub fullscreen: Rc<RefCell<Option<Arc<SourcePhysicsFullscreen>>>>,
    pub context: ResourceHandle,
    pub runtime: ResourceRuntime,
}

pub(crate) struct NativePassFactory(pub NativePassEnvironment);
impl PassFactory for NativePassFactory {
    fn shader_backend(&mut self, state: Rc<dyn GlState>) -> Box<dyn ShaderPassBackend> {
        Box::new(NativeShaderPass {
            environment: self.0.clone(),
            state,
            program: None,
        })
    }
    fn stencil(&mut self, size: [i32; 2], internal_format: i32) -> Rc<dyn StencilTarget> {
        Rc::new(SharedStencilTarget(Arc::new(RenderBuffer::new(
            size[0],
            size[1],
            internal_format,
            self.0.renderbuffers.clone(),
            self.0.context.clone(),
            &self.0.runtime,
        ))))
    }
    fn target(
        &mut self,
        dst: &[NamedTexture],
        stencil: Rc<dyn StencilTarget>,
    ) -> Box<dyn TargetBinding> {
        let targets: Vec<Arc<dyn crate::framebuffer_target::FramebufferTarget>> = dst
            .iter()
            .map(|target| {
                let texture = target
                    .texture
                    .borrow()
                    .native_texture()
                    .expect("native target factory requires a retained Texture2D binding");
                texture as Arc<dyn crate::framebuffer_target::FramebufferTarget>
            })
            .collect();
        let depth = stencil
            .native_target()
            .expect("native target factory requires a retained renderbuffer");
        let size = stencil.size();
        let fbo = Fbo::new(
            self.0.framebuffers.clone(),
            self.0.context.clone(),
            &self.0.runtime,
        );
        Box::new(
            TexturedFbo::new(
                size[0],
                size[1],
                Arc::new(Mutex::new(targets.into_boxed_slice())),
                Some(depth),
                fbo,
            )
            .unwrap_or_else(|error| panic!("{error}")),
        )
    }
}

struct NativeShaderPass {
    environment: NativePassEnvironment,
    state: Rc<dyn GlState>,
    program: Option<Arc<ShaderProgram>>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::passes::{
        pass::Pass, standard_pass::StandardPass, stateful_pass::StatefulPass,
        stencil_pass::StencilPass,
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct State(Log);
    impl StateBackend for State {
        fn get_integer(&mut self, parameter: i32) -> i32 {
            self.0.lock().unwrap().push(format!("get:{parameter}"));
            23
        }
        fn stencil_mask(&mut self, mask: i32) {
            self.0.lock().unwrap().push(format!("mask:{mask}"));
        }
        fn stencil_func(&mut self, func: i32, reference: i32, mask: i32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("func:{func}:{reference}:{mask}"));
        }
        fn stencil_op(&mut self, failure: i32, depth_failure: i32, success: i32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("op:{failure}:{depth_failure}:{success}"));
        }
    }
    impl PassStateBackend for State {
        fn stencil_test(&mut self, enabled: bool) {
            self.0.lock().unwrap().push(format!("stencil:{enabled}"));
        }
        fn color_mask(&mut self, mask: [bool; 4]) {
            self.0.lock().unwrap().push(format!("colors:{mask:?}"));
        }
    }
    pub(crate) fn fixture(log: Log) -> NativePassFactory {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let programs = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            log.clone(),
        )));
        let geometry = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(log.clone())));
        let framebuffer = crate::render_fbo::source_tests::backend(log.clone());
        NativePassFactory(NativePassEnvironment {
            shaders: programs.clone(),
            programs: programs.clone(),
            vertices: Arc::new(SourceVertexShaders::new(
                programs,
                context.clone(),
                &runtime,
            )),
            buffers: geometry.clone(),
            arrays: geometry.clone(),
            draws: geometry,
            renderbuffers: framebuffer.clone(),
            framebuffers: framebuffer,
            state: Rc::new(RefCell::new(State(log))),
            fullscreen: Rc::new(RefCell::new(None)),
            context,
            runtime,
        })
    }
    #[test]
    fn native_shader_passes_preserve_uniform_state_order_and_share_lazy_quad() {
        let log = Arc::new(Mutex::new(vec![]));
        let mut factory = fixture(log.clone());
        let state = Rc::new(crate::gl_state::stencil_write_mask::StencilWriteMask::new(
            128,
        ));
        let mut standard = StandardPass::new(
            factory.shader_backend(state.clone()),
            &["in_pos".into(), "in_mass".into()],
            &["out_force".into()],
            "fragment",
        );
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"attribute:0:Position".into()));
        assert!(events.contains(&"output:0:out_force".into()));
        assert_eq!(
            &events[events.len() - 6..],
            [
                "use:9",
                "int:-1:[0]",
                "use:0",
                "use:9",
                "int:-1:[1]",
                "use:0"
            ]
        );
        assert!(factory.0.fullscreen.borrow().is_none());
        log.lock().unwrap().clear();
        standard.set_float_arg("deltaT", &[0.25]);
        assert_eq!(*log.lock().unwrap(), ["use:9", "float:-1:[0.25]", "use:0"]);
        log.lock().unwrap().clear();
        standard.render();
        let retained = factory.0.fullscreen.borrow().clone().unwrap();
        let events = log.lock().unwrap().clone();
        assert_eq!(
            &events[..4],
            ["stencil:true", "get:2968", "mask:128", "use:9"]
        );
        assert_eq!(
            &events[events.len() - 3..],
            ["use:0", "mask:23", "stencil:false"]
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.as_str() == "draw:4:6:5125:0")
                .count(),
            1
        );
        let mut stencil = StencilPass::new(factory.shader_backend(state), &[], "filter");
        log.lock().unwrap().clear();
        stencil.render();
        let events = log.lock().unwrap().clone();
        assert_eq!(
            &events[..4],
            [
                "stencil:true",
                "colors:[false, false, false, false]",
                "mask:128",
                "use:9"
            ]
        );
        assert_eq!(
            &events[events.len() - 4..],
            [
                "use:0",
                "mask:23",
                "colors:[true, true, true, true]",
                "stencil:false"
            ]
        );
        assert!(Arc::ptr_eq(
            &retained,
            factory.0.fullscreen.borrow().as_ref().unwrap()
        ));
        assert!(
            !events
                .iter()
                .any(|event| event == "create_buffer" || event == "create_vao")
        );
    }
    #[test]
    fn native_target_attaches_real_texture_and_stencil_then_restores_framebuffer_scope() {
        use crate::{
            gl_data_holder::SourceGlDataHolder,
            passes::{
                custom_pass::CustomPass, provider_pass::TextureBinding, target_pass::TargetPass,
            },
        };
        struct Binding(Arc<crate::texture_2d::SourceTexture2D>);
        impl TextureBinding for Binding {
            fn bind(&mut self, unit: usize) {
                self.0.texture.bind_unit(unit as i32);
            }
            fn unbind(&mut self, unit: usize) {
                self.0.texture.unbind_unit(unit as i32);
            }
            fn native_texture(&self) -> Option<Arc<crate::texture_2d::SourceTexture2D>> {
                Some(self.0.clone())
            }
        }
        let log = Arc::new(Mutex::new(vec![]));
        let mut factory = fixture(log.clone());
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let holder = crate::float_data_holder::SourceFloatDataHolder::<4>::new(
            3,
            4,
            textures,
            factory.0.context.clone(),
            &factory.0.runtime,
        )
        .unwrap();
        let target = NamedTexture {
            name: "out_water".into(),
            size: [3, 4],
            texture: Rc::new(RefCell::new(Binding(holder.source_texture()))),
        };
        let stencil = factory.stencil([3, 4], 35056);
        let binding = factory.target(&[target], stencil);
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"storage:36161:35056:3:4".into()));
        assert!(events.contains(&"attach_tex:36160:36064:1:0".into()));
        assert!(events.contains(&"attach_rb:36160:33306:36161:51".into()));
        let inner = log.clone();
        let mut pass = TargetPass::new(
            binding,
            1,
            [3, 4],
            vec![Box::new(CustomPass::new(move || {
                inner.lock().unwrap().push("runtime draw".into());
            }))],
            vec![],
        );
        log.lock().unwrap().clear();
        pass.render();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "get_viewport",
                "fbo:36160:41",
                "check:FBO Bind",
                "draw_buffers:[36064]",
                "viewport:[0, 0, 3, 4]",
                "runtime draw",
                "check:After FB Draw",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "check:FB Unbind",
                "viewport:[0, 0, 800, 600]",
                "check:FB reset wiewport"
            ]
        );
    }
    #[test]
    fn source_physics_constructor_and_movement_execute_through_native_factory() {
        let log = Arc::new(Mutex::new(vec![]));
        let mut factory = fixture(log.clone());
        let backend = crate::render_fbo::source_tests::backend(log.clone());
        let dat = Rc::new(crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(crate::materials::SourceMaterials::default()),
            Rc::new(RefCell::new(vec![Some(Arc::new(
                crate::materials::Material::default(),
            ))])),
            1,
            1,
        ));
        let context = factory.0.context.clone();
        let runtime = factory.0.runtime.clone();
        let mut physics = crate::source_ship_physics::SourceShipPhysics::new(
            dat,
            backend.clone(),
            backend,
            &mut factory,
            context.clone(),
            &runtime,
            Box::new(|| 123),
        )
        .unwrap();
        let events = log.lock().unwrap().clone();
        assert_eq!(
            events
                .iter()
                .filter(|s| s.as_str() == "create program")
                .count(),
            11
        );
        assert_eq!(
            events.iter().filter(|s| s.as_str() == "create_fbo").count(),
            8
        );
        assert_eq!(
            events.iter().filter(|s| s.starts_with("image2:")).count(),
            9
        );
        assert_eq!(
            events
                .iter()
                .filter(|s| s.as_str() == "draw:4:6:5125:0")
                .count(),
            2
        );
        assert_eq!(
            events.iter().filter(|s| s.as_str() == "create_vao").count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|s| s.starts_with("attach_tex:"))
                .count(),
            12
        );
        assert_eq!(
            events
                .iter()
                .filter(|s| s.starts_with("attach_rb:"))
                .count(),
            8
        );
        log.lock().unwrap().clear();
        physics.move_by(bevy::prelude::Vec2::new(3., -4.));
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"float:-1:[3.0, -4.0]".into()));
        assert_eq!(
            events
                .iter()
                .filter(|s| s.as_str() == "draw:4:6:5125:0")
                .count(),
            1
        );
        assert!(events.contains(&"check:FB reset wiewport".into()));
        assert!(!events.iter().any(|s| s.starts_with("delete_")));
        context.close();
        runtime.run_main();
        let events = log.lock().unwrap().clone();
        assert_eq!(
            events
                .iter()
                .filter(|s| s.starts_with("delete_tex:"))
                .count(),
            9
        );
        assert_eq!(
            events
                .iter()
                .filter(|s| s.starts_with("delete_fbo:"))
                .count(),
            8
        );
    }
}
impl NativeShaderPass {
    fn program(&self) -> &ShaderProgram {
        self.program
            .as_deref()
            .expect("shader pass not initialized")
    }
}
impl ShaderPassBackend for NativeShaderPass {
    fn create_shader(&mut self, fragment: &str, output_bindings: &[String]) {
        // Source evaluates VertexShaders.nothing before allocating the fragment.
        let vertex = self
            .environment
            .vertices
            .nothing()
            .unwrap_or_else(|error| panic!("{error}"));
        let fragment = Arc::new(
            Shader::new(
                fragment,
                35632,
                output_bindings.to_vec(),
                self.environment.shaders.clone(),
                self.environment.context.clone(),
                &self.environment.runtime,
            )
            .unwrap_or_else(|error| panic!("{error}")),
        );
        self.program = Some(Arc::new(ShaderProgram::new(
            vec![vertex, fragment],
            self.environment.programs.clone(),
            self.environment.context.clone(),
            &self.environment.runtime,
        )));
    }
    fn uniform_location(&mut self, name: &str) -> i32 {
        self.program().uniform_location(name)
    }
    fn start_shader(&mut self) {
        self.program().start();
    }
    fn stop_shader(&mut self) {
        self.program().stop();
    }
    fn write_float_uniform(&mut self, location: i32, values: &[f32]) {
        self.environment
            .programs
            .lock()
            .unwrap()
            .float_uniform(location, values);
    }
    fn write_int_uniform(&mut self, location: i32, values: &[i32]) {
        self.environment
            .programs
            .lock()
            .unwrap()
            .int_uniform(location, values);
    }
    fn write_matrix_uniform(&mut self, location: i32, transposed: bool, matrix: &[f32; 16]) {
        self.environment
            .programs
            .lock()
            .unwrap()
            .matrix_uniform(location, transposed, matrix);
    }
    fn stencil_test(&mut self, enabled: bool) {
        self.environment.state.borrow_mut().stencil_test(enabled);
    }
    fn color_mask(&mut self, mask: [bool; 4]) {
        self.environment.state.borrow_mut().color_mask(mask);
    }
    fn current_state(&mut self) -> Box<dyn Any> {
        Box::new(
            self.state
                .current_state(&mut *self.environment.state.borrow_mut()),
        )
    }
    fn apply_state(&mut self) {
        self.state.apply(&mut *self.environment.state.borrow_mut());
    }
    fn restore_state(&mut self, state: Box<dyn Any>) {
        let state = state
            .downcast::<Rc<dyn GlState>>()
            .expect("native pass state type mismatch");
        state.apply(&mut *self.environment.state.borrow_mut());
    }
    fn draw_fullscreen(&mut self) {
        let retained = self.environment.fullscreen.borrow().clone();
        let fullscreen = retained.unwrap_or_else(|| {
            let fullscreen = Arc::new(SourcePhysicsFullscreen::new(
                self.environment.buffers.clone(),
                self.environment.arrays.clone(),
                self.environment.draws.clone(),
                self.environment.context.clone(),
                &self.environment.runtime,
            ));
            *self.environment.fullscreen.borrow_mut() = Some(fullscreen.clone());
            fullscreen
        });
        fullscreen.render();
    }
}

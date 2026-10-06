//! Environment for original brush constructors, using the converted classes.
use super::brush_runtime::{BrushBlendBackend, BrushPasses, BrushShip};
use crate::{
    camera_2d::SourceCamera2D,
    fbo::FramebufferBackend,
    file_reader::FileReader,
    gl_state::NoState,
    passes::{
        pass_builder::{PassBuilder, PassFactory},
        standard_pass::StandardPass,
    },
    resource::{ResourceHandle, ResourceRuntime},
    texture::{Configure, TextureBackend},
    texture_2d::SourceTexture2D,
    window::SourceWindow,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
pub(crate) struct BrushEnvironment {
    pub reader: FileReader,
    pub camera_control: Rc<RefCell<crate::camera_control::SourceCameraControl>>,
    pub textures: Arc<Mutex<dyn TextureBackend>>,
    pub context: ResourceHandle,
    pub resources: ResourceRuntime,
    pub factory: Box<dyn PassFactory>,
    pub current_ship: Box<dyn FnMut() -> Arc<dyn BrushShip>>,
    pub game_parameter_provider:
        std::rc::Rc<std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>>,
    pub blend: Box<dyn BrushBlendBackend>,
    pub framebuffer: Arc<Mutex<dyn FramebufferBackend>>,
}
impl BrushEnvironment {
    pub fn icon(&self, path: &str, configure: Configure) -> Result<SourceTexture2D, String> {
        let image = self.reader.read_image(std::path::Path::new(path), 0)?;
        Ok(SourceTexture2D::from_image(
            &image,
            32856,
            true,
            configure,
            self.textures.clone(),
            self.context.clone(),
            &self.resources,
        ))
    }
}
pub(crate) fn passes(
    factory: &mut dyn PassFactory,
    preview: &str,
    action: &str,
    is_break: bool,
) -> Rc<RefCell<BrushPasses>> {
    let preview = PassBuilder::with(vec![], factory, |step| {
        step.direct_pass(|direct| {
            direct.pass(preview, None);
        })
    });
    let (src, dst) = if is_break {
        (
            vec!["in_pos_vel".into(), "in_mask_struts".into()],
            vec!["out_mask_struts".into()],
        )
    } else {
        (
            vec!["in_pos_vel".into(), "in_water".into()],
            vec!["out_water".into()],
        )
    };
    let action = StandardPass::new(factory.shader_backend(Rc::new(NoState)), &src, &dst, action);
    Rc::new(RefCell::new(BrushPasses {
        preview: Box::new(preview),
        action: Box::new(action),
    }))
}
pub(crate) fn initial_uniforms(
    passes: &Rc<RefCell<BrushPasses>>,
    window: &SourceWindow,
    is_break: bool,
) {
    let size = window.screen_size();
    let framebuffer = window.framebuffer_size();
    let scale = [
        framebuffer[0] as f32 / size[0] as f32,
        framebuffer[1] as f32 / size[1] as f32,
    ];
    let reciprocal = [1.0 / size[0] as f32, 1.0 / size[1] as f32];
    let mut passes = passes.borrow_mut();
    if is_break {
        passes.action.set_float_arg("u_window", &reciprocal);
        passes.preview.set_float_arg("u_window", &reciprocal);
        passes
            .preview
            .set_float_arg("u_scale", &[1.0 / scale[0], 1.0 / scale[1]]);
    } else {
        passes
            .preview
            .set_float_arg("u_scale", &[1.0 / scale[0], 1.0 / scale[1]]);
        passes.preview.set_float_arg("u_window", &reciprocal);
        passes.action.set_float_arg("u_window", &reciprocal);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        framebuffer_target::FramebufferTarget,
        passes::{
            pass_builder::NamedTexture,
            shader_pass_backend::ShaderPassBackend,
            target_pass::{StencilTarget, TargetBinding},
        },
        texture::Texture,
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct Shader {
        label: &'static str,
        log: Log,
        names: Vec<String>,
    }
    impl ShaderPassBackend for Shader {
        fn create_shader(&mut self, fragment: &str, outputs: &[String]) {
            assert!(fragment.contains("#version 150 core"));
            self.log
                .lock()
                .unwrap()
                .push(format!("shader:{}:{outputs:?}", self.label));
        }
        fn uniform_location(&mut self, name: &str) -> i32 {
            self.names.push(name.into());
            (self.names.len() - 1) as i32
        }
        fn start_shader(&mut self) {}
        fn stop_shader(&mut self) {}
        fn write_float_uniform(&mut self, location: i32, values: &[f32]) {
            self.log.lock().unwrap().push(format!(
                "float:{}:{}:{values:?}",
                self.label, self.names[location as usize]
            ));
        }
        fn write_int_uniform(&mut self, location: i32, values: &[i32]) {
            self.log.lock().unwrap().push(format!(
                "int:{}:{}:{values:?}",
                self.label, self.names[location as usize]
            ));
        }
        fn write_matrix_uniform(&mut self, location: i32, transpose: bool, _: &[f32; 16]) {
            self.log.lock().unwrap().push(format!(
                "matrix:{}:{}:{transpose}",
                self.label, self.names[location as usize]
            ));
        }
        fn stencil_test(&mut self, _: bool) {}
        fn color_mask(&mut self, _: [bool; 4]) {}
        fn current_state(&mut self) -> Box<dyn std::any::Any> {
            Box::new(())
        }
        fn apply_state(&mut self) {}
        fn restore_state(&mut self, _: Box<dyn std::any::Any>) {}
        fn draw_fullscreen(&mut self) {
            self.log
                .lock()
                .unwrap()
                .push(format!("draw:{}", self.label));
        }
    }
    struct Factory {
        log: Log,
        count: usize,
    }
    impl PassFactory for Factory {
        fn shader_backend(
            &mut self,
            _: Rc<dyn crate::gl_state::GlState>,
        ) -> Box<dyn ShaderPassBackend> {
            let label = if self.count == 0 { "preview" } else { "action" };
            self.count += 1;
            Box::new(Shader {
                label,
                log: self.log.clone(),
                names: vec![],
            })
        }
        fn stencil(&mut self, _: [i32; 2], _: i32) -> Rc<dyn StencilTarget> {
            panic!("brush preview has no target/stencil")
        }
        fn target(
            &mut self,
            _: &[NamedTexture],
            _: Rc<dyn StencilTarget>,
        ) -> Box<dyn TargetBinding> {
            panic!("brush preview is direct")
        }
    }
    struct Ship {
        texture: Arc<Texture>,
        log: Log,
    }
    impl BrushShip for Ship {
        fn target(&self) -> Arc<Texture> {
            self.log.lock().unwrap().push("ship:target".into());
            self.texture.clone()
        }
        fn physics_filter(&self) -> Arc<dyn FramebufferTarget> {
            self.log.lock().unwrap().push("ship:filter".into());
            self.texture.clone()
        }
        fn width(&self) -> i32 {
            self.log.lock().unwrap().push("ship:width".into());
            4
        }
        fn height(&self) -> i32 {
            self.log.lock().unwrap().push("ship:height".into());
            2
        }
        fn positions(&self) -> Arc<Texture> {
            self.texture.clone()
        }
    }
    struct Blend(Log);
    impl BrushBlendBackend for Blend {
        fn blending(&mut self, on: bool) {
            self.0.lock().unwrap().push(format!("blend:{on}"));
        }
    }
    pub(crate) fn fixture() -> (BrushEnvironment, Log) {
        let (window, resources, _) = crate::window::tests::fixture();
        let context = resources.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let textures = crate::texture::tests::backend(log.clone());
        let texture = Arc::new(Texture::new(
            3553,
            Texture::empty_configuration(),
            textures.clone(),
            context.clone(),
            &resources,
        ));
        let ship: Arc<dyn BrushShip> = Arc::new(Ship {
            texture,
            log: log.clone(),
        });
        let reads = log.clone();
        let reader = FileReader {
            ss_home: "assets".into(),
            ss_resources: "assets".into(),
            bundled_resources: "assets".into(),
        };
        log.lock().unwrap().clear();
        (
            BrushEnvironment {
                reader,
                camera_control: Rc::new(RefCell::new(
                    crate::camera_control::SourceCameraControl::with_camera(
                        window,
                        Rc::new(SourceCamera2D::new(400, 200)),
                    ),
                )),
                textures,
                context,
                resources,
                factory: Box::new(Factory {
                    log: log.clone(),
                    count: 0,
                }),
                current_ship: Box::new(move || {
                    reads.lock().unwrap().push("ship:current".into());
                    ship.clone()
                }),
                game_parameter_provider: Rc::new(RefCell::new(
                    crate::game_parameters::SourceGameParameterProvider::default(),
                )),
                blend: Box::new(Blend(log.clone())),
                framebuffer: crate::textured_fbo::tests::backend(log.clone()),
            },
            log,
        )
    }
    type AnyTool =
        Rc<RefCell<dyn super::super::tool::SourceTool<SourceWindow, Texture = SourceTexture2D>>>;
    fn construct(kind: usize, env: BrushEnvironment) -> (AnyTool, ResourceHandle) {
        let control = env.camera_control.clone();
        let parameters = env.game_parameter_provider.clone();
        match kind {
            0 => {
                let tool = super::super::break_tool::SourceBreakTool::new(env).unwrap();
                assert!(Rc::ptr_eq(&tool.borrow().camera_control(), &control));
                assert!(Rc::ptr_eq(
                    &tool.borrow().game_parameter_provider(),
                    &parameters
                ));
                let resource = tool.borrow().resource.clone().unwrap();
                (tool, resource)
            }
            1 => {
                let tool = super::super::flood_tool::SourceFloodTool::new(env).unwrap();
                assert!(Rc::ptr_eq(&tool.borrow().camera_control(), &control));
                assert!(Rc::ptr_eq(
                    &tool.borrow().game_parameter_provider(),
                    &parameters
                ));
                let resource = tool.borrow().resource.clone().unwrap();
                (tool, resource)
            }
            _ => {
                let tool = super::super::dry_tool::SourceDryTool::new(env).unwrap();
                assert!(Rc::ptr_eq(&tool.borrow().camera_control(), &control));
                assert!(Rc::ptr_eq(
                    &tool.borrow().game_parameter_provider(),
                    &parameters
                ));
                let resource = tool.borrow().resource.clone().unwrap();
                (tool, resource)
            }
        }
    }
    #[test]
    fn source_brush_constructors_load_icons_build_passes_register_camera_then_create_fbo() {
        for kind in 0..3 {
            let (env, log) = fixture();
            let camera = env.camera_control.borrow().camera();
            let resources = env.resources.clone();
            let (tool, resource) = construct(kind, env);
            let events = log.lock().unwrap().clone();
            let preview = events
                .iter()
                .position(|s| s == "shader:preview:[]")
                .unwrap();
            assert_eq!(
                events[..preview]
                    .iter()
                    .filter(|s| s.as_str() == "create")
                    .count(),
                2
            );
            assert_eq!(
                events[..preview]
                    .iter()
                    .filter(|s| s.as_str() == "mips:3553")
                    .count(),
                2
            );
            let writes: Vec<_> = events
                .iter()
                .filter(|s| s.starts_with("float:") || s.starts_with("matrix:"))
                .cloned()
                .collect();
            let uniforms = if kind == 0 {
                vec![
                    "float:action:u_window:[0.0025, 0.005]",
                    "float:preview:u_window:[0.0025, 0.005]",
                    "float:preview:u_scale:[0.5, 0.5]",
                ]
            } else {
                vec![
                    "float:preview:u_scale:[0.5, 0.5]",
                    "float:preview:u_window:[0.0025, 0.005]",
                    "float:action:u_window:[0.0025, 0.005]",
                ]
            };
            assert_eq!(&writes[..3], uniforms);
            assert_eq!(
                &writes[3..],
                ["matrix:preview:u_inv:false", "matrix:action:u_inv:false"]
            );
            let matrix = events
                .iter()
                .position(|s| s == "matrix:action:u_inv:false")
                .unwrap();
            let ship = events.iter().position(|s| s == "ship:current").unwrap();
            assert!(matrix < ship);
            assert_eq!(
                &events[ship..ship + 5],
                [
                    "ship:current",
                    "ship:target",
                    "ship:filter",
                    "ship:width",
                    "ship:height"
                ]
            );
            assert_eq!(tool.borrow().name(), ["Break", "Flood", "Dry"][kind]);
            log.lock().unwrap().clear();
            camera.translate(0.0, 0.0);
            assert_eq!(
                *log.lock().unwrap(),
                ["matrix:preview:u_inv:false", "matrix:action:u_inv:false"]
            );
            let weak = Rc::downgrade(&tool);
            drop(tool);
            assert!(weak.upgrade().is_some());
            resource.close();
            resources.run_main();
            assert!(weak.upgrade().is_none());
            log.lock().unwrap().clear();
            camera.translate(0.0, 0.0);
            assert!(log.lock().unwrap().is_empty());
        }
    }
    #[test]
    fn tools_read_retained_provider_even_after_global_replacement() {
        for kind in 0..3 {
            let (env, log) = fixture();
            let provider = env.game_parameter_provider.clone();
            let resources = env.resources.clone();
            let saved_global = crate::game_parameter_provider_kt::get_game_parameter_provider();
            crate::game_parameter_provider_kt::set_game_parameter_provider(provider.clone());
            let (tool, resource) = construct(kind, env);
            let replacement = Rc::new(RefCell::new(
                crate::game_parameters::SourceGameParameterProvider::default(),
            ));
            replacement.borrow_mut().set_tool(9.0);
            crate::game_parameter_provider_kt::set_game_parameter_provider(replacement);
            provider.borrow_mut().set_tool(2.75);
            log.lock().unwrap().clear();
            tool.borrow_mut().update();
            let sizes: Vec<_> = log
                .lock()
                .unwrap()
                .iter()
                .filter(|event| event.contains(":u_size:"))
                .cloned()
                .collect();
            assert_eq!(
                sizes,
                ["float:preview:u_size:[2.75]", "float:action:u_size:[2.75]"]
            );
            crate::game_parameter_provider_kt::set_game_parameter_provider(saved_global);
            resource.close();
            resources.run_main();
        }
    }
    #[test]
    fn source_icon_failure_stops_before_pass_creation_or_camera_registration() {
        let (mut env, log) = fixture();
        let camera = env.camera_control.borrow().camera();
        let resources = env.resources.clone();
        env.reader = FileReader {
            ss_home: "missing-icon-fixture".into(),
            ss_resources: "missing-icon-fixture".into(),
            bundled_resources: "missing-icon-fixture".into(),
        };
        assert!(super::super::break_tool::SourceBreakTool::new(env).is_err());
        assert!(log.lock().unwrap().is_empty());
        camera.translate(1.0, 1.0);
        assert!(log.lock().unwrap().is_empty());
        resources.run_main();
    }
}

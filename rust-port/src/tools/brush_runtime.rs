//! Break/Flood/Dry update() translated through the source pass, texture and FBO classes.
use crate::{
    fbo::{Fbo, FramebufferBackend},
    framebuffer_target::FramebufferTarget,
    passes::stateful_pass::StatefulPass,
    resource::{ResourceHandle, ResourceRuntime},
    texture::Texture,
    textured_fbo::TexturedFbo,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub(crate) trait BrushBlendBackend {
    fn blending(&mut self, enabled: bool);
}
/// The adapter must retain one object identity per source Ship and resolve current texture getters.
pub(crate) trait BrushShip {
    fn target(&self) -> Arc<Texture>;
    fn physics_filter(&self) -> Arc<dyn FramebufferTarget>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn positions(&self) -> Arc<Texture>;
}
pub(crate) struct BrushPasses {
    pub preview: Box<dyn StatefulPass>,
    pub action: Box<dyn StatefulPass>,
}
#[derive(Clone)]
pub(crate) struct BrushPassUniforms(pub Rc<RefCell<BrushPasses>>);
impl super::brush_callbacks::BrushUniforms for BrushPassUniforms {
    fn vec2(&mut self, preview: bool, name: &str, values: [f32; 2]) {
        let mut passes = self.0.borrow_mut();
        if preview {
            passes.preview.set_float_arg(name, &values);
        } else {
            passes.action.set_float_arg(name, &values);
        }
    }
    fn matrix(&mut self, preview: bool, name: &str, transpose: bool, value: &bevy::math::Mat4) {
        let mut passes = self.0.borrow_mut();
        if preview {
            passes
                .preview
                .set_matrix_arg(name, transpose, &value.to_cols_array());
        } else {
            passes
                .action
                .set_matrix_arg(name, transpose, &value.to_cols_array());
        }
    }
}
pub(crate) struct BrushRuntime {
    pub passes: Rc<RefCell<BrushPasses>>,
    pub last_ship: Arc<dyn BrushShip>,
    pub fbo: TexturedFbo,
    current_ship: Box<dyn FnMut() -> Arc<dyn BrushShip>>,
    tool_size: Box<dyn FnMut() -> f32>,
    blend: Box<dyn BrushBlendBackend>,
    framebuffer: Arc<Mutex<dyn FramebufferBackend>>,
    context: ResourceHandle,
    resources: ResourceRuntime,
}
impl BrushRuntime {
    pub fn new(
        passes: BrushPasses,
        current_ship: impl FnMut() -> Arc<dyn BrushShip> + 'static,
        tool_size: impl FnMut() -> f32 + 'static,
        blend: Box<dyn BrushBlendBackend>,
        framebuffer: Arc<Mutex<dyn FramebufferBackend>>,
        context: ResourceHandle,
        resources: ResourceRuntime,
    ) -> Result<Self, String> {
        Self::from_shared(
            Rc::new(RefCell::new(passes)),
            current_ship,
            tool_size,
            blend,
            framebuffer,
            context,
            resources,
        )
    }
    pub fn from_shared(
        passes: Rc<RefCell<BrushPasses>>,
        mut current_ship: impl FnMut() -> Arc<dyn BrushShip> + 'static,
        tool_size: impl FnMut() -> f32 + 'static,
        blend: Box<dyn BrushBlendBackend>,
        framebuffer: Arc<Mutex<dyn FramebufferBackend>>,
        context: ResourceHandle,
        resources: ResourceRuntime,
    ) -> Result<Self, String> {
        let last_ship = current_ship();
        let fbo = Self::make_fbo(
            last_ship.as_ref(),
            framebuffer.clone(),
            context.clone(),
            &resources,
        )?;
        Ok(Self {
            passes,
            last_ship,
            fbo,
            current_ship: Box::new(current_ship),
            tool_size: Box::new(tool_size),
            blend,
            framebuffer,
            context,
            resources,
        })
    }
    fn make_fbo(
        ship: &dyn BrushShip,
        backend: Arc<Mutex<dyn FramebufferBackend>>,
        context: ResourceHandle,
        resources: &ResourceRuntime,
    ) -> Result<TexturedFbo, String> {
        let target: Arc<dyn FramebufferTarget> = ship.target();
        let filter = ship.physics_filter();
        let width = ship.width();
        let height = ship.height();
        TexturedFbo::new(
            width,
            height,
            Arc::new(Mutex::new(vec![target].into_boxed_slice())),
            Some(filter),
            Fbo::new(backend, context, resources),
        )
    }
    pub fn use_source_parameters(
        &mut self,
        parameters: std::rc::Rc<
            std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>,
        >,
    ) {
        self.tool_size = Box::new(move || parameters.borrow().tool());
    }
    pub fn uniforms(&self) -> BrushPassUniforms {
        BrushPassUniforms(self.passes.clone())
    }
    pub fn update(&mut self, active: bool) -> Result<(), String> {
        // Source invokes getTool separately for each pass, not once per frame.
        self.passes
            .borrow_mut()
            .preview
            .set_float_arg("u_size", &[(self.tool_size)()]);
        self.passes
            .borrow_mut()
            .action
            .set_float_arg("u_size", &[(self.tool_size)()]);
        self.blend.blending(true);
        self.passes.borrow_mut().preview.render();
        self.blend.blending(false);
        if active {
            let ship = (self.current_ship)();
            if !Arc::ptr_eq(&ship, &self.last_ship) {
                // Preserve assignment-before-construction, including construction failure.
                self.last_ship = ship.clone();
                self.fbo = Self::make_fbo(
                    ship.as_ref(),
                    self.framebuffer.clone(),
                    self.context.clone(),
                    &self.resources,
                )?;
            }
            let passes = self.passes.clone();
            self.fbo.draw(|| {
                ship.positions().bind_unit(0);
                ship.target().bind_unit(1);
                passes.borrow_mut().action.render();
                ship.target().unbind_unit(1);
                ship.positions().unbind_unit(0);
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::pass::Pass;
    type Log = Arc<Mutex<Vec<String>>>;
    struct PassLog {
        label: &'static str,
        log: Log,
        fail: bool,
    }
    impl Pass for PassLog {
        fn render(&mut self) {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}:render", self.label));
            if self.fail {
                panic!("draw");
            }
        }
    }
    impl StatefulPass for PassLog {
        fn set_float_arg(&mut self, name: &str, values: &[f32]) {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}:{name}:{values:?}", self.label));
        }
    }
    struct Blend(Log);
    impl BrushBlendBackend for Blend {
        fn blending(&mut self, enabled: bool) {
            self.0.lock().unwrap().push(format!("blend:{enabled}"));
        }
    }
    struct Ship {
        texture: Arc<Texture>,
        size: [i32; 2],
        log: Log,
    }
    impl BrushShip for Ship {
        fn target(&self) -> Arc<Texture> {
            self.log.lock().unwrap().push("target".into());
            self.texture.clone()
        }
        fn physics_filter(&self) -> Arc<dyn FramebufferTarget> {
            self.log.lock().unwrap().push("filter".into());
            self.texture.clone()
        }
        fn width(&self) -> i32 {
            self.log.lock().unwrap().push("width".into());
            self.size[0]
        }
        fn height(&self) -> i32 {
            self.log.lock().unwrap().push("height".into());
            self.size[1]
        }
        fn positions(&self) -> Arc<Texture> {
            self.log.lock().unwrap().push("positions".into());
            self.texture.clone()
        }
    }
    fn fixture(
        fail_preview: bool,
        fail_action: bool,
    ) -> (BrushRuntime, Log, Rc<RefCell<Arc<dyn BrushShip>>>) {
        let resources = ResourceRuntime::default();
        let context = resources.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let texture = Arc::new(Texture::new(
            3553,
            Texture::empty_configuration(),
            crate::texture::tests::backend(log.clone()),
            context.clone(),
            &resources,
        ));
        let ship: Arc<dyn BrushShip> = Arc::new(Ship {
            texture,
            size: [4, 2],
            log: log.clone(),
        });
        let current = Rc::new(RefCell::new(ship));
        let source = current.clone();
        let reads = log.clone();
        let sizes = log.clone();
        let mut size = 0;
        let runtime = BrushRuntime::new(
            BrushPasses {
                preview: Box::new(PassLog {
                    label: "preview",
                    log: log.clone(),
                    fail: fail_preview,
                }),
                action: Box::new(PassLog {
                    label: "action",
                    log: log.clone(),
                    fail: fail_action,
                }),
            },
            move || {
                reads.lock().unwrap().push("current".into());
                source.borrow().clone()
            },
            move || {
                size += 1;
                sizes.lock().unwrap().push(format!("size:{size}"));
                size as f32
            },
            Box::new(Blend(log.clone())),
            crate::textured_fbo::tests::backend(log.clone()),
            context,
            resources,
        )
        .unwrap();
        (runtime, log, current)
    }
    #[test]
    fn source_update_preserves_preview_and_framebuffer_draw_order() {
        let (mut runtime, log, _) = fixture(false, false);
        let construction = log.lock().unwrap().clone();
        assert_eq!(
            &construction[1..6],
            ["current", "target", "filter", "width", "height"]
        );
        log.lock().unwrap().clear();
        runtime.update(false).unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "size:1",
                "preview:u_size:[1.0]",
                "size:2",
                "action:u_size:[2.0]",
                "blend:true",
                "preview:render",
                "blend:false"
            ]
        );
        log.lock().unwrap().clear();
        runtime.update(true).unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "size:3",
                "preview:u_size:[3.0]",
                "size:4",
                "action:u_size:[4.0]",
                "blend:true",
                "preview:render",
                "blend:false",
                "current",
                "get_viewport",
                "fbo:36160:41",
                "check:FBO Bind",
                "draw_buffers:[36064]",
                "viewport:[0, 0, 4, 2]",
                "positions",
                "active:33984",
                "bind:3553:61",
                "active:33984",
                "target",
                "active:33985",
                "bind:3553:61",
                "active:33984",
                "action:render",
                "target",
                "active:33985",
                "bind:3553:0",
                "active:33984",
                "positions",
                "active:33984",
                "bind:3553:0",
                "active:33984",
                "check:After FB Draw",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "check:FB Unbind",
                "viewport:[2, 3, 800, 600]",
                "check:FB reset wiewport",
            ]
        );
    }
    #[test]
    fn source_ship_change_replaces_fbo_only_during_active_update() {
        let (mut runtime, log, current) = fixture(false, false);
        let old_id = runtime.last_ship.clone();
        let replacement: Arc<dyn BrushShip> = Arc::new(Ship {
            texture: old_id.target(),
            size: [8, 4],
            log: log.clone(),
        });
        *current.borrow_mut() = replacement.clone();
        log.lock().unwrap().clear();
        runtime.update(false).unwrap();
        assert!(Arc::ptr_eq(&runtime.last_ship, &old_id));
        runtime.update(true).unwrap();
        assert!(Arc::ptr_eq(&runtime.last_ship, &replacement));
        assert_eq!([runtime.fbo.width, runtime.fbo.height], [8, 4]);
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|s| s.as_str() == "create_fbo")
                .count(),
            1
        );
        log.lock().unwrap().clear();
        runtime.update(true).unwrap();
        assert!(!log.lock().unwrap().iter().any(|s| s == "create_fbo"));
    }
    #[test]
    fn source_draw_failure_does_not_add_finally_cleanup() {
        let (mut runtime, log, _) = fixture(true, false);
        log.lock().unwrap().clear();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.update(false)))
                .is_err()
        );
        assert_eq!(log.lock().unwrap().last().unwrap(), "preview:render");
        assert!(!log.lock().unwrap().iter().any(|s| s == "blend:false"));
        let (mut runtime, log, _) = fixture(false, true);
        log.lock().unwrap().clear();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.update(true)))
                .is_err()
        );
        assert_eq!(log.lock().unwrap().last().unwrap(), "action:render");
        assert!(!log.lock().unwrap().iter().any(|s| s == "fbo:36160:0"));
    }
}

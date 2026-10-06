//! PassBuilder.java nested construction steps and immediate setup contract.
//! Concrete GPU allocation is supplied by a factory, never substituted by no-op passes.
use super::{
    custom_pass::CustomPass,
    direct_pass::DirectPass,
    initializable_pass::InitializablePass,
    pass::Pass,
    provider_pass::{ProviderPass, TextureBinding},
    shader_pass_backend::ShaderPassBackend,
    standard_pass::StandardPass,
    stateful_pass::StatefulPass,
    stencil_pass::StencilPass,
    target_pass::{StencilTarget, TargetBinding, TargetPass},
};
use crate::gl_state::{GlState, NoState};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
pub(crate) struct NamedTexture {
    pub name: String,
    pub size: [i32; 2],
    pub texture: Rc<RefCell<dyn TextureBinding>>,
}
pub(crate) trait PassFactory {
    fn shader_backend(&mut self, state: Rc<dyn GlState>) -> Box<dyn ShaderPassBackend>;
    fn stencil(&mut self, size: [i32; 2], internal_format: i32) -> Rc<dyn StencilTarget>;
    fn target(
        &mut self,
        dst: &[NamedTexture],
        stencil: Rc<dyn StencilTarget>,
    ) -> Box<dyn TargetBinding>;
}
struct SharedTexture(Rc<RefCell<dyn TextureBinding>>);
impl TextureBinding for SharedTexture {
    fn bind(&mut self, unit: usize) {
        self.0.borrow_mut().bind(unit);
    }
    fn unbind(&mut self, unit: usize) {
        self.0.borrow_mut().unbind(unit);
    }
}
/// Source builder returns shader passes so callers can set uniforms after insertion.
struct SharedStateful<T: StatefulPass>(Rc<RefCell<T>>);
impl<T: StatefulPass> Pass for SharedStateful<T> {
    fn render(&mut self) {
        self.0.borrow_mut().render();
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
}
impl<T: StatefulPass> StatefulPass for SharedStateful<T> {
    fn set_float_arg(&mut self, name: &str, values: &[f32]) {
        self.0.borrow_mut().set_float_arg(name, values);
    }
    fn set_int_arg(&mut self, name: &str, values: &[i32]) {
        self.0.borrow_mut().set_int_arg(name, values);
    }
    fn set_matrix_arg(&mut self, name: &str, transpose: bool, matrix: &[f32; 16]) {
        self.0.borrow_mut().set_matrix_arg(name, transpose, matrix);
    }
}
pub(crate) struct PassBuilder;
impl PassBuilder {
    pub fn with(
        src: Vec<NamedTexture>,
        factory: &mut dyn PassFactory,
        configure: impl FnOnce(&mut TextureStep<'_>),
    ) -> ProviderPass {
        let mut step = TextureStep {
            src,
            passes: Vec::new(),
            stencil: None,
            factory,
        };
        configure(&mut step);
        let mut result = step.build();
        result.setup();
        result
    }
}
pub(crate) struct TextureStep<'a> {
    pub src: Vec<NamedTexture>,
    pub passes: Vec<Box<dyn Pass>>,
    pub stencil: Option<Rc<dyn StencilTarget>>,
    factory: &'a mut dyn PassFactory,
}
impl TextureStep<'_> {
    pub fn set_stencil(&mut self) {
        let size = self
            .src
            .first()
            .expect("Pass builder texture step needs at least one target to create stencil")
            .size;
        self.stencil = Some(self.factory.stencil(size, 35056));
    }
    pub fn reset_stencil(&mut self) {
        self.stencil = None;
    }
    pub fn add_pass(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }
    pub fn direct_pass(&mut self, configure: impl FnOnce(&mut DirectStep<'_>)) {
        let mut step = DirectStep {
            src: self.src.iter().map(|t| t.name.clone()).collect(),
            runtime: Vec::new(),
            factory: self.factory,
        };
        configure(&mut step);
        let result = step.build();
        self.add_pass(Box::new(result));
    }
    pub fn target_pass(
        &mut self,
        dst: Vec<NamedTexture>,
        configure: impl FnOnce(&mut TargetStep<'_>),
    ) {
        let mut step = TargetStep {
            src: self.src.iter().map(|t| t.name.clone()).collect(),
            dst,
            stencil: self.stencil.clone(),
            setup: Vec::new(),
            runtime: Vec::new(),
            factory: self.factory,
        };
        configure(&mut step);
        let result = step.build();
        self.add_pass(Box::new(result));
    }
    pub fn build(self) -> ProviderPass {
        let src = self
            .src
            .into_iter()
            .map(|t| Box::new(SharedTexture(t.texture)) as Box<dyn TextureBinding>)
            .collect();
        ProviderPass::new(src, self.passes)
    }
}
pub(crate) struct DirectStep<'a> {
    src: Vec<String>,
    runtime: Vec<Box<dyn Pass>>,
    factory: &'a mut dyn PassFactory,
}
impl DirectStep<'_> {
    pub fn pass(
        &mut self,
        shader: &str,
        state: Option<Rc<dyn GlState>>,
    ) -> Rc<RefCell<StandardPass>> {
        let pass = Rc::new(RefCell::new(StandardPass::new(
            self.factory
                .shader_backend(state.unwrap_or_else(|| Rc::new(NoState))),
            &self.src,
            &[],
            shader,
        )));
        self.runtime.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn stencil_pass(
        &mut self,
        shader: &str,
        state: Option<Rc<dyn GlState>>,
    ) -> Rc<RefCell<StencilPass>> {
        let pass = Rc::new(RefCell::new(StencilPass::new(
            self.factory
                .shader_backend(state.unwrap_or_else(|| Rc::new(NoState))),
            &self.src,
            shader,
        )));
        self.runtime.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn custom_pass(&mut self, callback: impl FnMut() + 'static) {
        self.runtime.push(Box::new(CustomPass::new(callback)));
    }
    pub fn build(self) -> DirectPass {
        DirectPass::new(self.runtime)
    }
}
pub(crate) struct TargetStep<'a> {
    src: Vec<String>,
    pub dst: Vec<NamedTexture>,
    pub stencil: Option<Rc<dyn StencilTarget>>,
    setup: Vec<Box<dyn Pass>>,
    runtime: Vec<Box<dyn Pass>>,
    factory: &'a mut dyn PassFactory,
}
impl TargetStep<'_> {
    fn make_standard(&mut self, state: Rc<dyn GlState>, shader: &str) -> Rc<RefCell<StandardPass>> {
        let dst = self.dst.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        Rc::new(RefCell::new(StandardPass::new(
            self.factory.shader_backend(state),
            &self.src,
            &dst,
            shader,
        )))
    }
    fn make_stencil(&mut self, state: Rc<dyn GlState>, shader: &str) -> Rc<RefCell<StencilPass>> {
        Rc::new(RefCell::new(StencilPass::new(
            self.factory.shader_backend(state),
            &self.src,
            shader,
        )))
    }
    pub fn setup_pass(
        &mut self,
        state: Rc<dyn GlState>,
        shader: &str,
    ) -> Rc<RefCell<StandardPass>> {
        let pass = self.make_standard(state, shader);
        self.setup.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn pass(&mut self, state: Rc<dyn GlState>, shader: &str) -> Rc<RefCell<StandardPass>> {
        let pass = self.make_standard(state, shader);
        self.runtime.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn setup_stencil_pass(
        &mut self,
        state: Rc<dyn GlState>,
        shader: &str,
    ) -> Rc<RefCell<StencilPass>> {
        let pass = self.make_stencil(state, shader);
        self.setup.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn stencil_pass(
        &mut self,
        state: Rc<dyn GlState>,
        shader: &str,
    ) -> Rc<RefCell<StencilPass>> {
        let pass = self.make_stencil(state, shader);
        self.runtime.push(Box::new(SharedStateful(pass.clone())));
        pass
    }
    pub fn setup_custom_pass(&mut self, callback: impl FnMut() + 'static) {
        self.setup.push(Box::new(CustomPass::new(callback)));
    }
    pub fn custom_pass(&mut self, callback: impl FnMut() + 'static) {
        self.runtime.push(Box::new(CustomPass::new(callback)));
    }
    pub fn build(self) -> TargetPass {
        let stencil = self.stencil.unwrap_or_else(|| {
            let size = self
                .dst
                .first()
                .expect("TargetPass needs at least one bound target texture")
                .size;
            self.factory.stencil(size, 35056)
        });
        let size = stencil.size();
        let target = self.factory.target(&self.dst, stencil.clone());
        let mut pass = TargetPass::new(target, self.dst.len(), size, self.runtime, self.setup);
        pass.stencil = Some(stencil);
        pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Events = Rc<RefCell<Vec<String>>>;
    struct Texture(Events);
    impl TextureBinding for Texture {
        fn bind(&mut self, unit: usize) {
            self.0.borrow_mut().push(format!("source bind:{unit}"));
        }
        fn unbind(&mut self, unit: usize) {
            self.0.borrow_mut().push(format!("source unbind:{unit}"));
        }
    }
    struct Stencil([i32; 2]);
    impl StencilTarget for Stencil {
        fn size(&self) -> [i32; 2] {
            self.0
        }
    }
    struct Target(Events);
    impl TargetBinding for Target {
        fn viewport(&mut self) -> [i32; 4] {
            [0, 0, 640, 480]
        }
        fn bind(&mut self) {
            self.0.borrow_mut().push("target bind".into());
        }
        fn draw_buffers(&mut self, buffers: &[u32]) {
            self.0.borrow_mut().push(format!("outputs:{buffers:?}"));
        }
        fn set_viewport(&mut self, viewport: [i32; 4]) {
            self.0.borrow_mut().push(format!("viewport:{viewport:?}"));
        }
        fn unbind(&mut self) {
            self.0.borrow_mut().push("target unbind".into());
        }
    }
    struct Factory(Events);
    impl PassFactory for Factory {
        fn shader_backend(&mut self, _: Rc<dyn GlState>) -> Box<dyn ShaderPassBackend> {
            panic!("Shader construction not expected in custom-pass test")
        }
        fn stencil(&mut self, size: [i32; 2], format: i32) -> Rc<dyn StencilTarget> {
            self.0
                .borrow_mut()
                .push(format!("allocate stencil:{size:?}:{format}"));
            Rc::new(Stencil(size))
        }
        fn target(
            &mut self,
            dst: &[NamedTexture],
            stencil: Rc<dyn StencilTarget>,
        ) -> Box<dyn TargetBinding> {
            self.0.borrow_mut().push(format!(
                "allocate target:{}:{:?}",
                dst.len(),
                stencil.size()
            ));
            Box::new(Target(self.0.clone()))
        }
    }
    fn texture(events: &Events, name: &str, size: [i32; 2]) -> NamedTexture {
        NamedTexture {
            name: name.into(),
            size,
            texture: Rc::new(RefCell::new(Texture(events.clone()))),
        }
    }
    #[test]
    fn builder_runs_setup_before_return_and_preserves_custom_runtime_order() {
        let events = Events::default();
        let mut factory = Factory(events.clone());
        let src = vec![texture(&events, "source", [3, 4])];
        let dst = vec![texture(&events, "output", [5, 6])];
        let mut result = PassBuilder::with(src, &mut factory, |step| {
            let direct_events = events.clone();
            step.direct_pass(|direct| {
                direct.custom_pass(move || direct_events.borrow_mut().push("direct runtime".into()))
            });
            step.target_pass(dst, |target| {
                let setup_events = events.clone();
                let runtime_events = events.clone();
                target.setup_custom_pass(move || {
                    setup_events.borrow_mut().push("target setup".into())
                });
                target
                    .custom_pass(move || runtime_events.borrow_mut().push("target runtime".into()));
            });
        });
        assert!(events.borrow().contains(&"target setup".into()));
        assert!(!events.borrow().contains(&"direct runtime".into()));
        assert!(
            events
                .borrow()
                .contains(&"allocate stencil:[5, 6]:35056".into())
        );
        events.borrow_mut().clear();
        result.render();
        result.render();
        let custom = events
            .borrow()
            .iter()
            .filter(|v| v.contains("runtime") || v.contains("setup"))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            custom,
            [
                "direct runtime",
                "target runtime",
                "direct runtime",
                "target runtime"
            ]
        );
    }
    #[test]
    fn shared_stencil_controls_target_size_until_reset() {
        let events = Events::default();
        let mut factory = Factory(events.clone());
        let src = vec![texture(&events, "source", [3, 4])];
        PassBuilder::with(src, &mut factory, |step| {
            step.set_stencil();
            step.target_pass(vec![texture(&events, "a", [5, 6])], |_| {});
            step.target_pass(vec![texture(&events, "b", [7, 8])], |_| {});
            step.reset_stencil();
            step.target_pass(vec![texture(&events, "c", [9, 10])], |_| {});
        });
        let allocations = events
            .borrow()
            .iter()
            .filter(|v| v.starts_with("allocate"))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            allocations,
            [
                "allocate stencil:[3, 4]:35056",
                "allocate target:1:[3, 4]",
                "allocate target:1:[3, 4]",
                "allocate stencil:[9, 10]:35056",
                "allocate target:1:[9, 10]"
            ]
        );
    }
    #[test]
    fn empty_sources_can_build_but_cannot_create_shared_stencil() {
        let events = Events::default();
        let mut factory = Factory(events);
        PassBuilder::with(vec![], &mut factory, |_| {});
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| PassBuilder::with(
                vec![],
                &mut factory,
                |step| step.set_stencil()
            )))
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| PassBuilder::with(
                vec![],
                &mut factory,
                |step| step.target_pass(vec![], |_| {})
            )))
            .is_err()
        );
    }
}

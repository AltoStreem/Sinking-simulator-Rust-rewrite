//! ProviderPass.java binds sources in list order around setup or render.
//! Texture bindings are backend adapters; OpenGL texture units are not Bevy bind groups.
use super::{
    direct_pass::DirectPass, initializable_pass::InitializablePass,
    initializable_stateful_pass::InitializableStatefulPass, pass::Pass,
    stateful_pass::StatefulPass,
};
pub(crate) trait TextureBinding {
    fn bind(&mut self, unit: usize);
    fn unbind(&mut self, unit: usize);
}
pub(crate) struct ProviderPass {
    pub src: Vec<Box<dyn TextureBinding>>,
    pub passes: DirectPass,
}
impl ProviderPass {
    pub fn new(src: Vec<Box<dyn TextureBinding>>, passes: Vec<Box<dyn Pass>>) -> Self {
        Self {
            src,
            passes: DirectPass::new(passes),
        }
    }
    fn bind_sources(&mut self) {
        for (unit, source) in self.src.iter_mut().enumerate() {
            source.bind(unit);
        }
    }
    fn unbind_sources(&mut self) {
        for (unit, source) in self.src.iter_mut().enumerate() {
            source.unbind(unit);
        }
    }
}
impl Pass for ProviderPass {
    fn render(&mut self) {
        self.bind_sources();
        self.passes.render();
        self.unbind_sources();
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
    fn initializable(&mut self) -> Option<&mut dyn InitializablePass> {
        Some(self)
    }
}
impl InitializablePass for ProviderPass {
    fn setup(&mut self) {
        self.bind_sources();
        for pass in &mut self.passes.passes {
            if let Some(initializable) = pass.initializable() {
                initializable.setup();
            }
        }
        self.unbind_sources();
    }
}
impl StatefulPass for ProviderPass {
    fn set_float_arg(&mut self, name: &str, values: &[f32]) {
        self.passes.set_float_arg(name, values);
    }
    fn set_int_arg(&mut self, name: &str, values: &[i32]) {
        self.passes.set_int_arg(name, values);
    }
    fn set_matrix_arg(&mut self, name: &str, transposed: bool, matrix: &[f32; 16]) {
        self.passes.set_matrix_arg(name, transposed, matrix);
    }
}
impl InitializableStatefulPass for ProviderPass {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::custom_pass::CustomPass;
    use std::{cell::RefCell, rc::Rc};
    type Events = Rc<RefCell<Vec<String>>>;
    struct Texture(Events, usize);
    impl TextureBinding for Texture {
        fn bind(&mut self, unit: usize) {
            self.0.borrow_mut().push(format!("bind:{}:{unit}", self.1));
        }
        fn unbind(&mut self, unit: usize) {
            self.0
                .borrow_mut()
                .push(format!("unbind:{}:{unit}", self.1));
        }
    }
    struct Stateful(Events);
    impl Pass for Stateful {
        fn render(&mut self) {
            self.0.borrow_mut().push("render".into());
        }
        fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
            Some(self)
        }
        fn initializable(&mut self) -> Option<&mut dyn InitializablePass> {
            Some(self)
        }
    }
    impl InitializablePass for Stateful {
        fn setup(&mut self) {
            self.0.borrow_mut().push("setup".into());
        }
    }
    impl StatefulPass for Stateful {
        fn set_float_arg(&mut self, name: &str, values: &[f32]) {
            self.0.borrow_mut().push(format!("float:{name}:{values:?}"));
        }
        fn set_int_arg(&mut self, name: &str, values: &[i32]) {
            self.0.borrow_mut().push(format!("int:{name}:{values:?}"));
        }
        fn set_matrix_arg(&mut self, name: &str, transpose: bool, matrix: &[f32; 16]) {
            self.0
                .borrow_mut()
                .push(format!("matrix:{name}:{transpose}:{}", matrix[15]));
        }
    }
    #[test]
    fn provider_preserves_bind_setup_render_unbind_and_uniform_filtering_order() {
        let events = Events::default();
        let custom_events = events.clone();
        let mut provider = ProviderPass::new(
            vec![
                Box::new(Texture(events.clone(), 7)),
                Box::new(Texture(events.clone(), 9)),
            ],
            vec![
                Box::new(CustomPass::new(move || {
                    custom_events.borrow_mut().push("custom".into())
                })),
                Box::new(Stateful(events.clone())),
            ],
        );
        provider.setup();
        assert_eq!(
            &*events.borrow(),
            &["bind:7:0", "bind:9:1", "setup", "unbind:7:0", "unbind:9:1"]
        );
        events.borrow_mut().clear();
        provider.render();
        provider.render();
        let once = [
            "bind:7:0",
            "bind:9:1",
            "custom",
            "render",
            "unbind:7:0",
            "unbind:9:1",
        ];
        assert_eq!(&events.borrow()[..6], once);
        assert_eq!(&events.borrow()[6..], once);
        events.borrow_mut().clear();
        provider.set_float_arg("deltaT", &[0.25, 1.0]);
        provider.set_int_arg("iter", &[50]);
        provider.set_matrix_arg("transform", true, &[2.0; 16]);
        assert_eq!(
            &*events.borrow(),
            &[
                "float:deltaT:[0.25, 1.0]",
                "int:iter:[50]",
                "matrix:transform:true:2"
            ]
        );
    }
    #[test]
    fn empty_composite_and_default_uniform_methods_are_no_ops() {
        struct DefaultState;
        impl Pass for DefaultState {
            fn render(&mut self) {}
        }
        impl StatefulPass for DefaultState {}
        let mut state = DefaultState;
        state.set_float_arg("ignored", &[]);
        state.set_int_arg("ignored", &[]);
        state.set_matrix_arg("ignored", false, &[0.0; 16]);
        DirectPass::default().render();
        ProviderPass::new(vec![], vec![]).setup();
    }
}

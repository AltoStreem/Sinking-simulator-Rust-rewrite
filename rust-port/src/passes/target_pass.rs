//! TargetPass.java framebuffer draw scope and separate setup/runtime lists.
//! The backend owns framebuffer/stencil creation; this contract preserves draw order.
use super::{
    direct_pass::DirectPass, initializable_pass::InitializablePass,
    initializable_stateful_pass::InitializableStatefulPass, pass::Pass,
    stateful_pass::StatefulPass,
};
use std::rc::Rc;
pub(crate) trait StencilTarget {
    fn size(&self) -> [i32; 2];
}
pub(crate) trait TargetBinding {
    fn viewport(&mut self) -> [i32; 4];
    fn bind(&mut self);
    fn draw_buffers(&mut self, attachments: &[u32]);
    fn set_viewport(&mut self, viewport: [i32; 4]);
    fn unbind(&mut self);
}
pub(crate) struct TargetPass {
    pub target: Box<dyn TargetBinding>,
    pub stencil: Option<Rc<dyn StencilTarget>>,
    pub passes: DirectPass,
    pub setup_passes: DirectPass,
    pub size: [i32; 2],
    attachments: Vec<u32>,
}
impl TargetPass {
    pub fn new(
        target: Box<dyn TargetBinding>,
        target_count: usize,
        size: [i32; 2],
        passes: Vec<Box<dyn Pass>>,
        setup_passes: Vec<Box<dyn Pass>>,
    ) -> Self {
        Self {
            target,
            stencil: None,
            passes: DirectPass::new(passes),
            setup_passes: DirectPass::new(setup_passes),
            size,
            attachments: (0..target_count).map(|i| 36064 + i as u32).collect(),
        }
    }
    fn begin_draw(&mut self) -> [i32; 4] {
        let viewport = self.target.viewport();
        self.target.bind();
        self.target.draw_buffers(&self.attachments);
        self.target.set_viewport([0, 0, self.size[0], self.size[1]]);
        viewport
    }
    fn end_draw(&mut self, viewport: [i32; 4]) {
        self.target.unbind();
        self.target.set_viewport(viewport);
    }
}
impl Pass for TargetPass {
    fn render(&mut self) {
        let viewport = self.begin_draw();
        self.passes.render();
        self.end_draw(viewport);
    }
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        Some(self)
    }
    fn initializable(&mut self) -> Option<&mut dyn InitializablePass> {
        Some(self)
    }
}
impl InitializablePass for TargetPass {
    fn setup(&mut self) {
        let viewport = self.begin_draw();
        self.setup_passes.render();
        self.end_draw(viewport);
    }
}
impl StatefulPass for TargetPass {
    fn set_float_arg(&mut self, name: &str, values: &[f32]) {
        self.setup_passes.set_float_arg(name, values);
        self.passes.set_float_arg(name, values);
    }
    fn set_int_arg(&mut self, name: &str, values: &[i32]) {
        self.setup_passes.set_int_arg(name, values);
        self.passes.set_int_arg(name, values);
    }
    fn set_matrix_arg(&mut self, name: &str, transposed: bool, matrix: &[f32; 16]) {
        self.setup_passes.set_matrix_arg(name, transposed, matrix);
        self.passes.set_matrix_arg(name, transposed, matrix);
    }
}
impl InitializableStatefulPass for TargetPass {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::custom_pass::CustomPass;
    use std::{cell::RefCell, rc::Rc};
    struct Backend(Rc<RefCell<Vec<String>>>);
    impl TargetBinding for Backend {
        fn viewport(&mut self) -> [i32; 4] {
            self.0.borrow_mut().push("get viewport".into());
            [10, 20, 640, 480]
        }
        fn bind(&mut self) {
            self.0.borrow_mut().push("bind".into());
        }
        fn draw_buffers(&mut self, attachments: &[u32]) {
            self.0
                .borrow_mut()
                .push(format!("attachments:{attachments:?}"));
        }
        fn set_viewport(&mut self, viewport: [i32; 4]) {
            self.0.borrow_mut().push(format!("viewport:{viewport:?}"));
        }
        fn unbind(&mut self) {
            self.0.borrow_mut().push("unbind".into());
        }
    }
    #[test]
    fn target_setup_renders_setup_list_and_restores_viewport_after_unbind() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let setup_events = events.clone();
        let render_events = events.clone();
        let mut pass = TargetPass::new(
            Box::new(Backend(events.clone())),
            2,
            [3, 4],
            vec![Box::new(CustomPass::new(move || {
                render_events.borrow_mut().push("runtime render".into())
            }))],
            vec![Box::new(CustomPass::new(move || {
                setup_events.borrow_mut().push("setup render".into())
            }))],
        );
        pass.setup();
        assert_eq!(
            &*events.borrow(),
            &[
                "get viewport",
                "bind",
                "attachments:[36064, 36065]",
                "viewport:[0, 0, 3, 4]",
                "setup render",
                "unbind",
                "viewport:[10, 20, 640, 480]"
            ]
        );
        events.borrow_mut().clear();
        pass.render();
        assert_eq!(events.borrow()[4], "runtime render");
        assert_eq!(events.borrow()[6], "viewport:[10, 20, 640, 480]");
    }
}

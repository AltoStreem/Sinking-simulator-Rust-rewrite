//! GLState.java and source lazy restoration contract; Concat checked against bytecode.
#![allow(dead_code)]
pub mod concat;
pub mod stencil_config;
pub mod stencil_func;
pub mod stencil_op;
pub mod stencil_write_mask;
pub mod util_kt;
use std::{cell::RefCell, rc::Rc};
pub(crate) trait StateBackend {
    fn get_integer(&mut self, parameter: i32) -> i32;
    fn stencil_mask(&mut self, mask: i32);
    fn stencil_func(&mut self, func: i32, reference: i32, value_mask: i32);
    fn stencil_op(&mut self, failure: i32, depth_failure: i32, success: i32);
}
pub(crate) trait GlState {
    fn apply(&self, backend: &mut dyn StateBackend);
    fn current_state(&self, backend: &mut dyn StateBackend) -> Rc<dyn GlState>;
}
thread_local! { static NO_STATE: Rc<dyn GlState> = Rc::new(NoState); }
#[derive(Default)]
pub(crate) struct NoState;
impl GlState for NoState {
    fn apply(&self, _: &mut dyn StateBackend) {}
    fn current_state(&self, _: &mut dyn StateBackend) -> Rc<dyn GlState> {
        NO_STATE.with(|state| state.clone())
    }
}
#[derive(Default)]
pub(super) struct LazyState(RefCell<Option<Rc<dyn GlState>>>);
impl LazyState {
    pub fn get(&self, initialize: impl FnOnce() -> Rc<dyn GlState>) -> Rc<dyn GlState> {
        if let Some(value) = self.0.borrow().as_ref() {
            return value.clone();
        }
        let value = initialize();
        *self.0.borrow_mut() = Some(value.clone());
        value
    }
}

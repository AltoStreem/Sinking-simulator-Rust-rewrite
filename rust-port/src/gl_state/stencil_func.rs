//! StencilFunc.java source comparison/reference/value-mask state.
use super::{GlState, LazyState, StateBackend};
use std::rc::Rc;
pub(crate) struct StencilFunc {
    pub func: i32,
    pub reference: i32,
    pub value_mask: i32,
    current: LazyState,
}
impl StencilFunc {
    pub fn new(func: i32, reference: i32, value_mask: i32) -> Self {
        Self {
            func,
            reference,
            value_mask,
            current: LazyState::default(),
        }
    }
    pub fn from_backend(backend: &mut dyn StateBackend) -> Self {
        Self::with_defaults(backend, None, None, None)
    }
    pub fn with_defaults(
        backend: &mut dyn StateBackend,
        func: Option<i32>,
        reference: Option<i32>,
        value_mask: Option<i32>,
    ) -> Self {
        let func = func.unwrap_or_else(|| backend.get_integer(2962));
        let reference = reference.unwrap_or_else(|| backend.get_integer(2967));
        let value_mask = value_mask.unwrap_or_else(|| backend.get_integer(2963));
        Self::new(func, reference, value_mask)
    }
    pub fn java_hash(&self) -> i32 {
        self.func
            .wrapping_mul(31)
            .wrapping_add(self.reference)
            .wrapping_mul(31)
            .wrapping_add(self.value_mask)
    }
}
impl Clone for StencilFunc {
    fn clone(&self) -> Self {
        Self::new(self.func, self.reference, self.value_mask)
    }
}
impl PartialEq for StencilFunc {
    fn eq(&self, other: &Self) -> bool {
        (self.func, self.reference, self.value_mask)
            == (other.func, other.reference, other.value_mask)
    }
}
impl Eq for StencilFunc {}
impl GlState for StencilFunc {
    fn apply(&self, backend: &mut dyn StateBackend) {
        backend.stencil_func(self.func, self.reference, self.value_mask);
    }
    fn current_state(&self, backend: &mut dyn StateBackend) -> Rc<dyn GlState> {
        self.current.get(|| Rc::new(Self::from_backend(backend)))
    }
}

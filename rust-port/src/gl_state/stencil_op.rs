//! StencilOp.java operation state and source default queries.
use super::{GlState, LazyState, StateBackend};
use std::rc::Rc;
pub(crate) struct StencilOp {
    pub on_failure: i32,
    pub on_depth_failure: i32,
    pub on_success: i32,
    current: LazyState,
}
impl StencilOp {
    pub fn new(on_failure: i32, on_depth_failure: i32, on_success: i32) -> Self {
        Self {
            on_failure,
            on_depth_failure,
            on_success,
            current: LazyState::default(),
        }
    }
    pub fn from_backend(backend: &mut dyn StateBackend) -> Self {
        Self::with_defaults(backend, None, None, None)
    }
    pub fn with_defaults(
        backend: &mut dyn StateBackend,
        failure: Option<i32>,
        depth_failure: Option<i32>,
        success: Option<i32>,
    ) -> Self {
        let failure = failure.unwrap_or_else(|| backend.get_integer(2964));
        let depth_failure = depth_failure.unwrap_or_else(|| backend.get_integer(2965));
        let success = success.unwrap_or_else(|| backend.get_integer(2966));
        Self::new(failure, depth_failure, success)
    }
    pub fn java_hash(&self) -> i32 {
        self.on_failure
            .wrapping_mul(31)
            .wrapping_add(self.on_depth_failure)
            .wrapping_mul(31)
            .wrapping_add(self.on_success)
    }
}
impl Clone for StencilOp {
    fn clone(&self) -> Self {
        Self::new(self.on_failure, self.on_depth_failure, self.on_success)
    }
}
impl PartialEq for StencilOp {
    fn eq(&self, other: &Self) -> bool {
        (self.on_failure, self.on_depth_failure, self.on_success)
            == (other.on_failure, other.on_depth_failure, other.on_success)
    }
}
impl Eq for StencilOp {}
impl GlState for StencilOp {
    fn apply(&self, backend: &mut dyn StateBackend) {
        backend.stencil_op(self.on_failure, self.on_depth_failure, self.on_success);
    }
    fn current_state(&self, backend: &mut dyn StateBackend) -> Rc<dyn GlState> {
        self.current.get(|| Rc::new(Self::from_backend(backend)))
    }
}

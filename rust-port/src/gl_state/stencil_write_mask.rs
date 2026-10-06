//! StencilWriteMask.java; default constructor queries GL_STENCIL_WRITEMASK.
use super::{GlState, LazyState, StateBackend};
use std::rc::Rc;
pub(crate) struct StencilWriteMask {
    pub mask: i32,
    current: LazyState,
}
impl StencilWriteMask {
    pub fn new(mask: i32) -> Self {
        Self {
            mask,
            current: LazyState::default(),
        }
    }
    pub fn from_backend(backend: &mut dyn StateBackend) -> Self {
        Self::new(backend.get_integer(2968))
    }
    pub fn java_hash(&self) -> i32 {
        self.mask
    }
}
impl Clone for StencilWriteMask {
    fn clone(&self) -> Self {
        Self::new(self.mask)
    }
}
impl PartialEq for StencilWriteMask {
    fn eq(&self, other: &Self) -> bool {
        self.mask == other.mask
    }
}
impl Eq for StencilWriteMask {}
impl GlState for StencilWriteMask {
    fn apply(&self, backend: &mut dyn StateBackend) {
        backend.stencil_mask(self.mask);
    }
    fn current_state(&self, backend: &mut dyn StateBackend) -> Rc<dyn GlState> {
        self.current.get(|| Rc::new(Self::from_backend(backend)))
    }
}

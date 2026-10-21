//! StencilWriteMask.java; default constructor queries GL_STENCIL_WRITEMASK.
use super::{GlState, LazyState, StateBackend};
use std::fmt;
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
    /// Mirrors the Kotlin constructor's synthetic default parameter: an
    /// omitted mask is read from GL_STENCIL_WRITEMASK, while an explicit mask
    /// does not query the backend.
    pub fn with_defaults(backend: &mut dyn StateBackend, mask: Option<i32>) -> Self {
        Self::new(mask.unwrap_or_else(|| backend.get_integer(2968)))
    }
    pub fn component1(&self) -> i32 {
        self.mask
    }
    pub fn copy(&self, mask: i32) -> Self {
        Self::new(mask)
    }
    /// Kotlin's generated `copy$default` behavior: `None` retains the mask.
    pub fn copy_with_defaults(&self, mask: Option<i32>) -> Self {
        self.copy(mask.unwrap_or(self.mask))
    }
    pub fn java_to_string(&self) -> String {
        format!("StencilWriteMask(mask={})", self.mask)
    }
    pub fn java_hash(&self) -> i32 {
        self.mask
    }
}
impl fmt::Display for StencilWriteMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.java_to_string())
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

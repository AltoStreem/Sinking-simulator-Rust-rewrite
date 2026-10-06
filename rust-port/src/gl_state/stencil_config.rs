//! StencilConfig.java nullable component application and full lazy restoration.
use super::{
    GlState, LazyState, StateBackend, stencil_func::StencilFunc, stencil_op::StencilOp,
    stencil_write_mask::StencilWriteMask,
};
use std::rc::Rc;
pub(crate) struct StencilConfig {
    pub write_mask: Option<Rc<StencilWriteMask>>,
    pub func: Option<Rc<StencilFunc>>,
    pub op: Option<Rc<StencilOp>>,
    current: LazyState,
}
impl StencilConfig {
    pub fn new(
        write_mask: Option<StencilWriteMask>,
        func: Option<StencilFunc>,
        op: Option<StencilOp>,
    ) -> Self {
        Self {
            write_mask: write_mask.map(Rc::new),
            func: func.map(Rc::new),
            op: op.map(Rc::new),
            current: LazyState::default(),
        }
    }
    pub fn from_backend(backend: &mut dyn StateBackend) -> Self {
        Self::new(
            Some(StencilWriteMask::from_backend(backend)),
            Some(StencilFunc::from_backend(backend)),
            Some(StencilOp::from_backend(backend)),
        )
    }
    pub fn java_hash(&self) -> i32 {
        self.write_mask
            .as_ref()
            .map_or(0, |v| v.java_hash())
            .wrapping_mul(31)
            .wrapping_add(self.func.as_ref().map_or(0, |v| v.java_hash()))
            .wrapping_mul(31)
            .wrapping_add(self.op.as_ref().map_or(0, |v| v.java_hash()))
    }
}
impl Clone for StencilConfig {
    fn clone(&self) -> Self {
        Self {
            write_mask: self.write_mask.clone(),
            func: self.func.clone(),
            op: self.op.clone(),
            current: LazyState::default(),
        }
    }
}
impl PartialEq for StencilConfig {
    fn eq(&self, other: &Self) -> bool {
        self.write_mask == other.write_mask && self.func == other.func && self.op == other.op
    }
}
impl Eq for StencilConfig {}
impl GlState for StencilConfig {
    fn apply(&self, backend: &mut dyn StateBackend) {
        if let Some(value) = &self.write_mask {
            value.apply(backend);
        }
        if let Some(value) = &self.func {
            value.apply(backend);
        }
        if let Some(value) = &self.op {
            value.apply(backend);
        }
    }
    fn current_state(&self, backend: &mut dyn StateBackend) -> Rc<dyn GlState> {
        self.current.get(|| Rc::new(Self::from_backend(backend)))
    }
}

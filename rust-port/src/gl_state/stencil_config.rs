//! StencilConfig.java nullable component application and full lazy restoration.
use super::{
    GlState, LazyState, StateBackend, stencil_func::StencilFunc, stencil_op::StencilOp,
    stencil_write_mask::StencilWriteMask,
};
use std::fmt;
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
        Self::with_defaults(backend, None, None, None)
    }
    /// Mirrors Kotlin's per-parameter constructor defaults. `None` here means
    /// that the parameter was omitted and must be captured from GL; use `new`
    /// when an explicit null component is intended.
    pub fn with_defaults(
        backend: &mut dyn StateBackend,
        write_mask: Option<StencilWriteMask>,
        func: Option<StencilFunc>,
        op: Option<StencilOp>,
    ) -> Self {
        let write_mask = match write_mask {
            Some(value) => value,
            None => StencilWriteMask::with_defaults(backend, None),
        };
        let func = match func {
            Some(value) => value,
            None => StencilFunc::with_defaults(backend, None, None, None),
        };
        let op = match op {
            Some(value) => value,
            None => StencilOp::with_defaults(backend, None, None, None),
        };
        Self::new(Some(write_mask), Some(func), Some(op))
    }
    /// Kotlin data-class `copy`: supplied nullable components are retained by
    /// reference, and the copied config gets its own lazy current-state cache.
    pub fn copy(
        &self,
        write_mask: Option<Rc<StencilWriteMask>>,
        func: Option<Rc<StencilFunc>>,
        op: Option<Rc<StencilOp>>,
    ) -> Self {
        Self {
            write_mask,
            func,
            op,
            current: LazyState::default(),
        }
    }
    /// Rust representation of Kotlin's `copy$default` masks. An outer `None`
    /// keeps that component; `Some(None)` explicitly clears it.
    pub fn copy_with_defaults(
        &self,
        write_mask: Option<Option<Rc<StencilWriteMask>>>,
        func: Option<Option<Rc<StencilFunc>>>,
        op: Option<Option<Rc<StencilOp>>>,
    ) -> Self {
        self.copy(
            write_mask.unwrap_or_else(|| self.write_mask.clone()),
            func.unwrap_or_else(|| self.func.clone()),
            op.unwrap_or_else(|| self.op.clone()),
        )
    }
    pub fn component1(&self) -> Option<Rc<StencilWriteMask>> {
        self.write_mask.clone()
    }
    pub fn component2(&self) -> Option<Rc<StencilFunc>> {
        self.func.clone()
    }
    pub fn component3(&self) -> Option<Rc<StencilOp>> {
        self.op.clone()
    }
    pub fn java_to_string(&self) -> String {
        let write_mask = self
            .write_mask
            .as_deref()
            .map(StencilWriteMask::java_to_string)
            .unwrap_or_else(|| "null".to_owned());
        let func = self
            .func
            .as_deref()
            .map(StencilFunc::java_to_string)
            .unwrap_or_else(|| "null".to_owned());
        let op = self
            .op
            .as_deref()
            .map(StencilOp::java_to_string)
            .unwrap_or_else(|| "null".to_owned());
        format!("StencilConfig(writeMask={write_mask}, func={func}, op={op})")
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
impl fmt::Display for StencilConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.java_to_string())
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

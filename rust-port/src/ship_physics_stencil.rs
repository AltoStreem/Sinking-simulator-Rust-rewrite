//! ShipPhysics.Companion's shared stencil configurations.
use crate::gl_state::{
    stencil_config::StencilConfig, stencil_func::StencilFunc, stencil_op::StencilOp,
    stencil_write_mask::StencilWriteMask,
};
use std::rc::Rc;
#[allow(non_snake_case)]
pub(crate) struct States {
    pub(crate) set7: Rc<StencilConfig>,
    pub(crate) set0If7: Rc<StencilConfig>,
    pub(crate) set1If7: Rc<StencilConfig>,
    pub(crate) execIf0: Rc<StencilConfig>,
    pub(crate) execIf1: Rc<StencilConfig>,
    pub(crate) execIfnot1: Rc<StencilConfig>,
    pub(crate) execIf7: Rc<StencilConfig>,
    pub(crate) exec: Rc<StencilConfig>,
}
impl States {
    fn new() -> Self {
        Self {
            set7: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(128)),
                Some(StencilFunc::new(519, 128, 128)),
                Some(StencilOp::new(0, 7681, 7681)),
            )),
            set0If7: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(1)),
                Some(StencilFunc::new(513, 1, 129)),
                Some(StencilOp::new(0, 7681, 7681)),
            )),
            set1If7: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(2)),
                Some(StencilFunc::new(513, 2, 130)),
                Some(StencilOp::new(0, 7681, 7681)),
            )),
            execIf0: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(0)),
                Some(StencilFunc::new(514, 1, 1)),
                Some(StencilOp::new(7680, 7680, 7680)),
            )),
            execIf1: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(0)),
                Some(StencilFunc::new(514, 2, 2)),
                Some(StencilOp::new(7680, 7680, 7680)),
            )),
            execIfnot1: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(0)),
                Some(StencilFunc::new(517, 2, 2)),
                Some(StencilOp::new(7680, 7680, 7680)),
            )),
            execIf7: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(0)),
                Some(StencilFunc::new(514, 128, 128)),
                Some(StencilOp::new(7680, 7680, 7680)),
            )),
            exec: Rc::new(StencilConfig::new(
                Some(StencilWriteMask::new(0)),
                Some(StencilFunc::new(519, 0, 0)),
                Some(StencilOp::new(7680, 7680, 7680)),
            )),
        }
    }
}
thread_local! { static STATES: Rc<States> = Rc::new(States::new()); }
pub(crate) fn states() -> Rc<States> {
    STATES.with(Clone::clone)
}

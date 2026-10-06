//! Toolbox$render$1.java: retained receiver, metadata and original property get/set.
use super::{BoolReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<crate::toolbox::SourceToolbox>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "toolsVisible"
    }
    fn signature(&self) -> &'static str {
        "getToolsVisible()Z"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.gui.Toolbox"
    }
}
impl BoolReference for PropertyReference {
    fn get(&self) -> bool {
        self.receiver.borrow().tools_visible()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_tools_visible(Value::boolean(value));
    }
}

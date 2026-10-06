//! Toolbox$render$2$1.java: retained receiver, metadata and original property get/set.
use super::{IntReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<dyn super::LayerReceiver>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "currentLayer"
    }
    fn signature(&self) -> &'static str {
        "getCurrentLayer()I"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.ship.Ship"
    }
}
impl IntReference for PropertyReference {
    fn get(&self) -> i32 {
        self.receiver.borrow().current_layer()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_current_layer(Value::number(value).int_value());
    }
}

//! Toolbox$render$4$1$2$9.java: retained receiver, metadata and original property get/set.
use super::{FloatReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "gravity"
    }
    fn signature(&self) -> &'static str {
        "getGravity()F"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.GameParameterProvider"
    }
}
impl FloatReference for PropertyReference {
    fn get(&self) -> f32 {
        self.receiver.borrow().gravity()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_gravity(Value::number(value).float_value());
    }
}

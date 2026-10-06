//! Toolbox$render$2$3.java: retained receiver, metadata and original property get/set.
use super::{FloatReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "tool"
    }
    fn signature(&self) -> &'static str {
        "getTool()F"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.GameParameterProvider"
    }
}
impl FloatReference for PropertyReference {
    fn get(&self) -> f32 {
        self.receiver.borrow().tool()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_tool(Value::number(value).float_value());
    }
}

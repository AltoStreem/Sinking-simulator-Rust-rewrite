//! Toolbox$render$4$1$3$3.java: retained receiver, metadata and original property get/set.
use super::{BoolReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "daycycle"
    }
    fn signature(&self) -> &'static str {
        "getDaycycle()Z"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.GameParameterProvider"
    }
}
impl BoolReference for PropertyReference {
    fn get(&self) -> bool {
        self.receiver.borrow().daycycle()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_daycycle(Value::boolean(value));
    }
}

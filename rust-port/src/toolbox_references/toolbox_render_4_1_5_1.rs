//! Toolbox$render$4$1$5$1.java: retained receiver, metadata and original property get/set.
use super::{IntReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "physicsSteps"
    }
    fn signature(&self) -> &'static str {
        "getPhysicsSteps()I"
    }
    fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.GameParameterProvider"
    }
}
impl IntReference for PropertyReference {
    fn get(&self) -> i32 {
        self.receiver.borrow().physics_steps()
    }
    fn set(&self, value: Option<Value>) {
        self.receiver
            .borrow_mut()
            .set_physics_steps(Value::number(value).int_value());
    }
}

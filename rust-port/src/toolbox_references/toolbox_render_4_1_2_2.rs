//! Toolbox$render$4$1$2$2.java: retained receiver, metadata and original property get/set.
use super::{FloatReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<bevy::math::Vec2>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "y"
    }
    fn signature(&self) -> &'static str {
        "getY()Ljava/lang/Float;"
    }
    fn owner(&self) -> &'static str {
        "glm_.vec2.Vec2"
    }
}
impl FloatReference for PropertyReference {
    fn get(&self) -> f32 {
        self.receiver.borrow().y
    }
    fn set(&self, value: Option<Value>) {
        self.receiver.borrow_mut().y = Value::number(value).float_value();
    }
}

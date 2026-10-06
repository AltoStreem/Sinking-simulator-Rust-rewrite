//! Toolbox$render$4$1$2$1.java: retained receiver, metadata and original property get/set.
use super::{FloatReference, Metadata, Value};
use std::{cell::RefCell, rc::Rc};
pub struct PropertyReference {
    pub receiver: Rc<RefCell<bevy::math::Vec2>>,
}
impl Metadata for PropertyReference {
    fn name(&self) -> &'static str {
        "x"
    }
    fn signature(&self) -> &'static str {
        "getX()Ljava/lang/Float;"
    }
    fn owner(&self) -> &'static str {
        "glm_.vec2.Vec2"
    }
}
impl FloatReference for PropertyReference {
    fn get(&self) -> f32 {
        self.receiver.borrow().x
    }
    fn set(&self, value: Option<Value>) {
        self.receiver.borrow_mut().x = Value::number(value).float_value();
    }
}

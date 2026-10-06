//! DryTool$free$1.class method reference. Constructor/free references compare equal by metadata/receiver.
use crate::camera_2d::{CameraCallbackKey, SourceCameraCallback};
use std::{cell::RefCell, rc::Rc};
pub(crate) fn key(receiver: u64) -> CameraCallbackKey {
    CameraCallbackKey {
        receiver,
        owner: "com.wicpar.sinkingsimulator.tools.impl.DryTool",
        name: "onCamChange",
        signature: "onCamChange(Lcom/wicpar/engine/world/Camera2D;)V",
    }
}
pub(crate) fn callback(tool: Rc<RefCell<super::dry_tool::SourceDryTool>>) -> SourceCameraCallback {
    SourceCameraCallback {
        key: key(Rc::as_ptr(&tool) as usize as u64),
        invoke: Rc::new(move |camera| tool.borrow_mut().on_camera_change(*camera.matrix.borrow())),
    }
}

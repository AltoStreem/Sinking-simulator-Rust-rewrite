//! Separately recovered Ship$cameraCallback$1.class.
//! Invert the live camera matrix first, then upload a fresh matrix getter.
use bevy::prelude::Mat4;
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub(crate) fn create(
    identity: Rc<()>,
    inverse: Rc<RefCell<Mat4>>,
    shader: Arc<crate::shader_program::ShaderProgram>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::camera_2d::SourceCameraCallback {
        key: crate::camera_2d::CameraCallbackKey {
            receiver: Rc::as_ptr(&identity) as usize as u64,
            owner: "com/wicpar/sinkingsimulator/ship/Ship",
            name: "cameraCallback",
            signature: "(Lcom/wicpar/engine/world/Camera2D;)V",
        },
        invoke: Rc::new(move |camera| {
            let _receiver = &identity;
            *inverse.borrow_mut() = camera.matrix.borrow().inverse();
            shader.set_matrix(
                shader.uniform_location("transform"),
                false,
                &camera.matrix.borrow().to_cols_array(),
            );
        }),
    }
}

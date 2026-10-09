//! Recovered Sky$3.class and its source cameraCallback target.
use std::{rc::Rc, sync::Arc};
pub(crate) fn upload(
    camera: &crate::camera_2d::SourceCamera2D,
    mut shader: impl FnMut() -> Arc<crate::shader_program::ShaderProgram>,
) {
    let receiver = shader();
    let location = shader().uniform_location("inv");
    let inverse = camera.matrix.borrow().inverse();
    receiver.set_matrix(location, false, &inverse.to_cols_array());
}
pub(crate) fn create(
    receiver: Rc<crate::sky::SourceSkyState>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::camera_2d::SourceCameraCallback {
        key: crate::camera_2d::CameraCallbackKey {
            receiver: Rc::as_ptr(&receiver) as usize as u64,
            owner: "com/wicpar/sinkingsimulator/Sky",
            name: "cameraCallback",
            signature: "cameraCallback(Lcom/wicpar/engine/world/Camera2D;)V",
        },
        invoke: Rc::new(move |camera| receiver.camera_changed(camera)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_is_read_after_second_shader_getter_without_mutating_camera() {
        let runtime = crate::resource::ResourceRuntime::default();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let program = Arc::new(crate::shader_program::tests::program(
            &runtime,
            runtime.allocate(&[], || {}),
            &events,
        ));
        events.lock().unwrap().clear();
        let camera = crate::camera_2d::SourceCamera2D::new(400, 200);
        let matrix = bevy::math::Mat4::from_diagonal(bevy::math::Vec4::splat(2.));
        let mut reads = 0;
        upload(&camera, || {
            reads += 1;
            if reads == 2 {
                *camera.matrix.borrow_mut() = matrix;
            }
            program.clone()
        });
        assert_eq!(reads, 2);
        assert_eq!(*camera.matrix.borrow(), matrix);
        assert_eq!(
            *events.lock().unwrap(),
            ["use:9", "matrix:-1:false:0.5", "use:0"]
        );
    }
}

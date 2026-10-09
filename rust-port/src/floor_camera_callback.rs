//! Recovered Floor$cameraCallback$1.class: receiver, location, inverse, upload.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn reads_shader_twice_before_inverse_and_uploads_to_first_receiver() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let first_events = Arc::new(Mutex::new(Vec::new()));
        let second_events = Arc::new(Mutex::new(Vec::new()));
        let first = Arc::new(crate::shader_program::tests::program(
            &runtime,
            context.clone(),
            &first_events,
        ));
        let second = Arc::new(crate::shader_program::tests::program(
            &runtime,
            context,
            &second_events,
        ));
        first_events.lock().unwrap().clear();
        second_events.lock().unwrap().clear();
        let camera = crate::camera_2d::SourceCamera2D::new(400, 200);
        let changed = bevy::math::Mat4::from_diagonal(bevy::math::Vec4::new(2., 3., 4., 5.));
        let mut reads = 0;
        upload(&camera, || {
            reads += 1;
            if reads == 1 {
                first.clone()
            } else {
                *camera.matrix.borrow_mut() = changed;
                second.clone()
            }
        });
        assert_eq!(reads, 2);
        assert_eq!(*camera.matrix.borrow(), changed);
        assert_eq!(
            *first_events.lock().unwrap(),
            [
                "use:9".to_owned(),
                format!("matrix:-1:false:{}", changed.inverse().to_cols_array()[15]),
                "use:0".to_owned(),
            ]
        );
        assert!(second_events.lock().unwrap().is_empty());
    }
}
pub(crate) fn create(
    identity: Rc<()>,
    backend: Rc<dyn crate::floor::FloorBackend>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::camera_2d::SourceCameraCallback {
        key: crate::camera_2d::CameraCallbackKey {
            receiver: Rc::as_ptr(&identity) as usize as u64,
            owner: "com/wicpar/sinkingsimulator/Floor",
            name: "cameraCallback",
            signature: "(Lcom/wicpar/engine/world/Camera2D;)V",
        },
        invoke: Rc::new(move |camera| {
            let _receiver = &identity;
            backend.camera_changed(camera);
        }),
    }
}

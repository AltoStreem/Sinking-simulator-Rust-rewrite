//! Recovered Sea$cameraCallback$1.class.
use std::{rc::Rc, sync::Arc};

pub(crate) fn upload(
    camera: &crate::camera_2d::SourceCamera2D,
    mut shader: impl FnMut() -> Arc<crate::shader_program::ShaderProgram>,
) {
    let transform_receiver = shader();
    let transform_location = shader().uniform_location("transform");
    let transform = *camera.matrix.borrow();
    transform_receiver.set_matrix(transform_location, false, &transform.to_cols_array());
    let inverse_receiver = shader();
    let inverse_location = shader().uniform_location("inv");
    let inverse = camera.matrix.borrow().inverse();
    inverse_receiver.set_matrix(inverse_location, false, &inverse.to_cols_array());
}

pub(crate) fn create(
    identity: Rc<()>,
    backend: Rc<dyn crate::sea::SeaBackend>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::camera_2d::SourceCameraCallback {
        key: crate::camera_2d::CameraCallbackKey {
            receiver: Rc::as_ptr(&identity) as usize as u64,
            owner: "com/wicpar/sinkingsimulator/Sea",
            name: "cameraCallback",
            signature: "(Lcom/wicpar/engine/world/Camera2D;)V",
        },
        invoke: Rc::new(move |camera| {
            let _receiver = &identity;
            backend.camera_changed(camera);
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn source_callback_reads_four_shaders_and_fresh_matrix_for_each_upload() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(Vec::new()));
        let program = Arc::new(crate::shader_program::tests::program(
            &runtime, context, &events,
        ));
        events.lock().unwrap().clear();
        let camera = crate::camera_2d::SourceCamera2D::new(400, 200);
        let transform = bevy::math::Mat4::from_diagonal(bevy::math::Vec4::splat(2.));
        let next = bevy::math::Mat4::from_diagonal(bevy::math::Vec4::splat(4.));
        let mut reads = 0;
        upload(&camera, || {
            reads += 1;
            if reads == 2 {
                *camera.matrix.borrow_mut() = transform;
            }
            if reads == 4 {
                *camera.matrix.borrow_mut() = next;
            }
            program.clone()
        });
        assert_eq!(reads, 4);
        assert_eq!(*camera.matrix.borrow(), next);
        assert_eq!(
            *events.lock().unwrap(),
            [
                "use:9",
                "matrix:-1:false:2",
                "use:0",
                "use:9",
                "matrix:-1:false:0.25",
                "use:0",
            ]
        );
    }
}

//! Recovered ShipStruts$cameraCallback$1.class.
use std::{rc::Rc, sync::Arc};

pub(crate) fn upload(
    camera: &crate::camera_2d::SourceCamera2D,
    mut shader: impl FnMut() -> Arc<crate::shader_program::ShaderProgram>,
) {
    let receiver = shader();
    let location = shader().uniform_location("transform");
    let matrix = *camera.matrix.borrow();
    receiver.set_matrix(location, false, &matrix.to_cols_array());
}

pub(crate) fn create(
    identity: Rc<()>,
    shader: Arc<crate::shader_program::ShaderProgram>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::camera_2d::SourceCameraCallback {
        key: crate::camera_2d::CameraCallbackKey {
            receiver: Rc::as_ptr(&identity) as usize as u64,
            owner: "com/wicpar/sinkingsimulator/ship/ShipStruts",
            name: "cameraCallback",
            signature: "(Lcom/wicpar/engine/world/Camera2D;)V",
        },
        invoke: Rc::new(move |camera| {
            let _receiver = &identity;
            upload(camera, || shader.clone());
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn queries_second_shader_then_reads_camera_and_uploads_first_receiver() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(Vec::new()));
        let other_events = Arc::new(Mutex::new(Vec::new()));
        let first = Arc::new(crate::shader_program::tests::program(
            &runtime,
            context.clone(),
            &events,
        ));
        let second = Arc::new(crate::shader_program::tests::program(
            &runtime,
            context,
            &other_events,
        ));
        events.lock().unwrap().clear();
        other_events.lock().unwrap().clear();
        let camera = crate::camera_2d::SourceCamera2D::new(400, 200);
        let matrix = bevy::math::Mat4::from_diagonal(bevy::math::Vec4::splat(3.));
        let mut reads = 0;
        upload(&camera, || {
            reads += 1;
            if reads == 1 {
                first.clone()
            } else {
                *camera.matrix.borrow_mut() = matrix;
                second.clone()
            }
        });
        assert_eq!(reads, 2);
        assert_eq!(*camera.matrix.borrow(), matrix);
        assert_eq!(
            *events.lock().unwrap(),
            ["use:9", "matrix:-1:false:3", "use:0"]
        );
        assert!(other_events.lock().unwrap().is_empty());
    }
}

//! Recovered Sea$resCallback$1.class: unfiltered framebuffer dimensions.
use std::{rc::Rc, sync::Arc};

pub(crate) fn upload(
    width: i32,
    height: i32,
    mut shader: impl FnMut() -> Arc<crate::shader_program::ShaderProgram>,
) {
    let receiver = shader();
    let location = shader().uniform_location("resolution");
    receiver
        .set_floats(location, &[width as f32, height as f32])
        .unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn create(
    backend: Rc<dyn crate::sea::SeaBackend>,
) -> Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> {
    Rc::new(move |_, width, height| backend.resolution_changed(width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn uses_first_receiver_and_preserves_zero_negative_and_large_dimensions() {
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
        for (width, height) in [(0, -3), (i32::MAX, i32::MIN)] {
            first_events.lock().unwrap().clear();
            second_events.lock().unwrap().clear();
            let mut reads = 0;
            upload(width, height, || {
                reads += 1;
                if reads == 1 {
                    first.clone()
                } else {
                    second.clone()
                }
            });
            assert_eq!(reads, 2);
            assert_eq!(
                *first_events.lock().unwrap(),
                [
                    "use:9".to_owned(),
                    format!("float:-1:{:?}", [width as f32, height as f32]),
                    "use:0".to_owned(),
                ]
            );
            assert!(second_events.lock().unwrap().is_empty());
        }
    }
}

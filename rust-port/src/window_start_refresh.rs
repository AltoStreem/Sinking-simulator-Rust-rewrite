//! Window$start$1.class: refresh the captured window, ignoring the event argument.
use super::SourceWindow;
use std::rc::Rc;

pub(super) fn create(
    window: Rc<SourceWindow>,
    draw: Rc<dyn Fn(&SourceWindow)>,
) -> Rc<dyn Fn(&SourceWindow)> {
    Rc::new(move |_| {
        window.backend.lock().unwrap().clear(17664);
        draw(&window);
        window.backend.lock().unwrap().swap_buffers(window.id);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refresh_retains_captured_window_and_releases_backend_lock_before_draw() {
        let (original, _runtime, log) = crate::window::tests::fixture();
        let (argument, _other_runtime, other_log) = crate::window::tests::fixture();
        let capture = original.clone();
        let draw_log = log.clone();
        let callback = create(
            original.clone(),
            Rc::new(move |window| {
                assert!(std::ptr::eq(window, &*capture));
                window.set_title("during refresh".into());
                draw_log.lock().unwrap().push("draw".into());
            }),
        );
        callback(&argument);
        assert_eq!(
            *log.lock().unwrap(),
            ["clear:17664", "title:during refresh", "draw", "swap:7"]
        );
        assert!(other_log.lock().unwrap().is_empty());
    }
    #[test]
    fn refresh_failure_preserves_clear_and_skips_swap() {
        let (window, _runtime, log) = crate::window::tests::fixture();
        let callback = create(window.clone(), Rc::new(|_| panic!("draw failure")));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(&window))).is_err()
        );
        assert_eq!(*log.lock().unwrap(), ["clear:17664"]);
    }
}

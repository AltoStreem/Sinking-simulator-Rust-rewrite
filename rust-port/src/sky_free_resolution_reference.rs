//! Recovered Sky$free$1.class: fresh reference equal to constructor reference.
use std::rc::Rc;
pub(crate) fn create(
    receiver: Rc<crate::sky::SourceSkyState>,
) -> Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> {
    crate::sky_resolution_reference::create(receiver)
}
pub(crate) fn remove(
    window: &crate::window::SourceWindow,
    receiver: Rc<crate::sky::SourceSkyState>,
) {
    let reference = create(receiver);
    let mut callbacks = window.framebuffer_size_callbacks.borrow_mut();
    if let Some(index) = callbacks
        .iter()
        .position(|callback| crate::sky_resolution_reference::equal(callback, &reference))
    {
        callbacks.remove(index);
    }
}

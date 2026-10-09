//! Recovered Sky$2.class retained function reference.
use std::rc::Rc;
type Callback = dyn Fn(&crate::window::SourceWindow, i32, i32);
thread_local! {
    static REFERENCES: std::cell::RefCell<Vec<(std::rc::Weak<Callback>, crate::camera_2d::CameraCallbackKey)>>
        = const { std::cell::RefCell::new(Vec::new()) };
}
pub(crate) const OWNER: &str = "com/wicpar/sinkingsimulator/Sky";
pub(crate) const NAME: &str = "resCallback";
pub(crate) const SIGNATURE: &str = "resCallback(Lcom/wicpar/engine/glfw/Window;II)V";
pub(crate) fn create(
    receiver: Rc<crate::sky::SourceSkyState>,
) -> Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> {
    let key = crate::camera_2d::CameraCallbackKey {
        receiver: Rc::as_ptr(&receiver) as usize as u64,
        owner: OWNER,
        name: NAME,
        signature: SIGNATURE,
    };
    let callback: Rc<Callback> =
        Rc::new(move |_, width, height| receiver.resolution(width, height));
    REFERENCES.with(|references| {
        let mut references = references.borrow_mut();
        references.retain(|(callback, _)| callback.strong_count() > 0);
        references.push((Rc::downgrade(&callback), key));
    });
    callback
}
pub(crate) fn equal(first: &Rc<Callback>, second: &Rc<Callback>) -> bool {
    if Rc::ptr_eq(first, second) {
        return true;
    }
    REFERENCES.with(|references| {
        let references = references.borrow();
        let key = |callback: &Rc<Callback>| {
            references.iter().find_map(|(weak, key)| {
                weak.upgrade()
                    .filter(|known| Rc::ptr_eq(known, callback))
                    .map(|_| key)
            })
        };
        match (key(first), key(second)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    })
}

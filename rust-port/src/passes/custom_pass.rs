//! CustomPass.java invokes its callback on every render (not once).
use super::pass::Pass;
pub(crate) struct CustomPass<F: FnMut()> {
    callback: F,
}
impl<F: FnMut()> CustomPass<F> {
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}
impl<F: FnMut()> Pass for CustomPass<F> {
    fn render(&mut self) {
        (self.callback)();
    }
}

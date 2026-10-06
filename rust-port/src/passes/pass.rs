//! Pass.java. Capability accessors replace Java instanceof filtering.
use super::{initializable_pass::InitializablePass, stateful_pass::StatefulPass};
pub(crate) trait Pass {
    fn render(&mut self);
    fn stateful(&mut self) -> Option<&mut dyn StatefulPass> {
        None
    }
    fn initializable(&mut self) -> Option<&mut dyn InitializablePass> {
        None
    }
}

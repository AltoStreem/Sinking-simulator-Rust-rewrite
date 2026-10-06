//! InitializablePass.java.
use super::pass::Pass;
pub(crate) trait InitializablePass: Pass {
    fn setup(&mut self);
}

//! GLState.Concat. Bytecode invokes the parent getCurrentState inside its map,
//! so nonempty snapshots recurse indefinitely in Java, not into child states.
use super::{GlState, LazyState, StateBackend};
use std::rc::Rc;
pub(crate) struct Concat {
    pub states: Vec<Rc<dyn GlState>>,
    current: LazyState,
}
impl Concat {
    pub fn new(states: Vec<Rc<dyn GlState>>) -> Self {
        Self {
            states,
            current: LazyState::default(),
        }
    }
}
impl GlState for Concat {
    fn apply(&self, backend: &mut dyn StateBackend) {
        for state in &self.states {
            state.apply(backend);
        }
    }
    fn current_state(&self, _: &mut dyn StateBackend) -> Rc<dyn GlState> {
        self.current.get(|| {
            // Rust stack overflow aborts rather than Java's throwable error.
            // Preserve failure without silently substituting a child snapshot.
            assert!(
                self.states.is_empty(),
                "Source GLState.Concat.currentState recursively calls itself for nonempty states"
            );
            Rc::new(Self::new(Vec::new()))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_state::{NoState, stencil_write_mask::StencilWriteMask};
    #[derive(Default)]
    struct Backend(Vec<i32>);
    impl StateBackend for Backend {
        fn get_integer(&mut self, _: i32) -> i32 {
            panic!("Unexpected child snapshot query")
        }
        fn stencil_mask(&mut self, value: i32) {
            self.0.push(value);
        }
        fn stencil_func(&mut self, _: i32, _: i32, _: i32) {}
        fn stencil_op(&mut self, _: i32, _: i32, _: i32) {}
    }
    #[test]
    fn concatenation_applies_children_in_order_without_capturing_them() {
        let state = Concat::new(vec![
            Rc::new(StencilWriteMask::new(1)),
            Rc::new(StencilWriteMask::new(2)),
        ]);
        let mut backend = Backend::default();
        state.apply(&mut backend);
        assert_eq!(backend.0, [1, 2]);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || state.current_state(&mut backend)
            ))
            .is_err()
        );
        assert_eq!(backend.0, [1, 2]);
    }
    #[test]
    fn empty_concatenation_and_none_return_cached_identity() {
        let state = Concat::new(vec![]);
        let mut backend = Backend::default();
        let first = state.current_state(&mut backend);
        let second = state.current_state(&mut backend);
        assert!(Rc::ptr_eq(&first, &second));
        first.apply(&mut backend);
        assert!(Rc::ptr_eq(
            &NoState.current_state(&mut backend),
            &NoState.current_state(&mut backend)
        ));
    }
}

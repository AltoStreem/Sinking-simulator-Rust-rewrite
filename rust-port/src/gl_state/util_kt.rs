//! UtilKt.use captures cached state, applies, invokes callback, then restores.
//! The source has no finally block: a callback failure does not restore state.
use super::{GlState, StateBackend};
pub(crate) fn use_state(
    state: &dyn GlState,
    backend: &mut dyn StateBackend,
    callback: impl FnOnce(&mut dyn StateBackend),
) {
    let previous = state.current_state(backend);
    state.apply(backend);
    callback(backend);
    previous.apply(backend);
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_state::{stencil_config::StencilConfig, stencil_write_mask::StencilWriteMask};
    #[derive(Default)]
    struct Backend {
        mask: i32,
        queries: Vec<i32>,
        operations: Vec<String>,
    }
    impl StateBackend for Backend {
        fn get_integer(&mut self, parameter: i32) -> i32 {
            self.queries.push(parameter);
            if parameter == 2968 {
                self.mask
            } else {
                parameter
            }
        }
        fn stencil_mask(&mut self, mask: i32) {
            self.mask = mask;
            self.operations.push(format!("mask:{mask}"));
        }
        fn stencil_func(&mut self, func: i32, reference: i32, value_mask: i32) {
            self.operations
                .push(format!("func:{func}:{reference}:{value_mask}"));
        }
        fn stencil_op(&mut self, a: i32, b: i32, c: i32) {
            self.operations.push(format!("op:{a}:{b}:{c}"));
        }
    }
    #[test]
    fn lazy_restoration_uses_first_query_and_copy_captures_again() {
        let state = StencilWriteMask::new(1);
        let mut backend = Backend {
            mask: 255,
            ..Default::default()
        };
        use_state(&state, &mut backend, |_| {});
        backend.mask = 7;
        use_state(&state, &mut backend, |_| {});
        assert_eq!(backend.mask, 255);
        assert_eq!(backend.queries, [2968]);
        backend.mask = 7;
        use_state(&state.clone(), &mut backend, |_| {});
        assert_eq!(backend.mask, 7);
        assert_eq!(backend.queries, [2968, 2968]);
    }
    #[test]
    fn nullable_config_captures_all_components_in_source_order() {
        let state = StencilConfig::new(Some(StencilWriteMask::new(3)), None, None);
        let mut backend = Backend {
            mask: 255,
            ..Default::default()
        };
        use_state(&state, &mut backend, |_| {});
        assert_eq!(backend.queries, [2968, 2962, 2967, 2963, 2964, 2965, 2966]);
        assert_eq!(
            backend.operations,
            [
                "mask:3",
                "mask:255",
                "func:2962:2967:2963",
                "op:2964:2965:2966"
            ]
        );
        assert_eq!(state.java_hash(), 3 * 31 * 31);
    }
    #[test]
    fn config_copy_shares_component_objects_but_recaptures_its_own_lazy_state() {
        let state = StencilConfig::new(Some(StencilWriteMask::new(3)), None, None);
        let mut backend = Backend {
            mask: 255,
            ..Default::default()
        };
        state.current_state(&mut backend);
        let copied = state.clone();
        assert!(std::rc::Rc::ptr_eq(
            state.write_mask.as_ref().unwrap(),
            copied.write_mask.as_ref().unwrap()
        ));
        backend.queries.clear();
        backend.mask = 7;
        use_state(&copied, &mut backend, |_| {});
        assert_eq!(backend.mask, 7);
        assert_eq!(backend.queries.len(), 7);
    }
    #[test]
    fn synthetic_defaults_query_only_omitted_parameters() {
        use crate::gl_state::{stencil_func::StencilFunc, stencil_op::StencilOp};
        let mut backend = Backend::default();
        let func = StencilFunc::with_defaults(&mut backend, Some(514), None, Some(255));
        let op = StencilOp::with_defaults(&mut backend, None, Some(7680), Some(7681));
        assert_eq!(backend.queries, [2967, 2964]);
        func.apply(&mut backend);
        op.apply(&mut backend);
        assert_eq!(
            backend.operations,
            ["func:514:2967:255", "op:2964:7680:7681"]
        );
    }
    #[test]
    fn callback_failure_preserves_source_unrestored_state() {
        let state = StencilWriteMask::new(1);
        let mut backend = Backend {
            mask: 255,
            ..Default::default()
        };
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| use_state(
                &state,
                &mut backend,
                |_| panic!("callback")
            )))
            .is_err()
        );
        assert_eq!(backend.mask, 1);
    }
}

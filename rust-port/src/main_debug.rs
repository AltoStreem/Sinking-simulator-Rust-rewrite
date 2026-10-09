//! Main.java's -debug setup and GL message suppression branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Api {
    Core43,
    Khr,
    Arb,
}
pub(crate) trait Backend {
    fn set_imgui_debug(&mut self, value: bool);
    fn setup_debug_message_callback(&mut self);
    fn message_control(
        &mut self,
        api: Api,
        source: i32,
        kind: i32,
        severity: i32,
        ids: Option<&[i32]>,
        enabled: bool,
    );
}
fn enabled(caps: &crate::gl_context::Capabilities, name: &str) -> bool {
    caps.fields
        .iter()
        .find(|field| field.name == name)
        .is_some_and(|field| field.value == crate::gl_context::CapabilityValue::Boolean(true))
}
pub(crate) fn configure_current<'a>(
    args: impl IntoIterator<Item = &'a str>,
    backend: &mut impl Backend,
) {
    // Main obtains currentContext.capabilities before evaluating -debug.
    let context = crate::gl_context::GlContext::current();
    configure(args, context.capabilities(), backend);
}
pub(crate) fn configure<'a>(
    args: impl IntoIterator<Item = &'a str>,
    caps: &crate::gl_context::Capabilities,
    backend: &mut impl Backend,
) {
    let debug = args.into_iter().any(|arg| arg == "-debug");
    backend.set_imgui_debug(debug);
    if !debug {
        return;
    }
    backend.setup_debug_message_callback();
    let api = if enabled(caps, "OpenGL43") {
        Some(Api::Core43)
    } else if enabled(caps, "GL_KHR_debug") {
        Some(Api::Khr)
    } else if enabled(caps, "GL_ARB_debug_output") {
        Some(Api::Arb)
    } else {
        None
    };
    if let Some(api) = api {
        let severity = if api == Api::Arb { 37192 } else { 33387 };
        backend.message_control(api, 33350, 33361, severity, None, false);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Recorder {
        events: Vec<String>,
    }
    impl Backend for Recorder {
        fn set_imgui_debug(&mut self, value: bool) {
            self.events.push(format!("debug:{value}"));
        }
        fn setup_debug_message_callback(&mut self) {
            self.events.push("callback".into());
        }
        fn message_control(
            &mut self,
            api: Api,
            source: i32,
            kind: i32,
            severity: i32,
            ids: Option<&[i32]>,
            enabled: bool,
        ) {
            assert!(ids.is_none());
            assert!(!enabled);
            self.events
                .push(format!("{api:?}:{source}:{kind}:{severity}"));
        }
    }
    fn caps(core: bool, khr: bool, arb: bool) -> crate::gl_context::Capabilities {
        use crate::gl_context::{Capabilities, CapabilityField, CapabilityValue};
        Capabilities {
            native: std::sync::Arc::new(()),
            fields: [
                ("OpenGL43", core),
                ("GL_KHR_debug", khr),
                ("GL_ARB_debug_output", arb),
            ]
            .into_iter()
            .map(|(name, value)| CapabilityField {
                name: name.into(),
                value: CapabilityValue::Boolean(value),
            })
            .collect(),
        }
    }
    #[test]
    fn core_khr_arb_priority_and_exact_filter_arguments() {
        for (core, khr, arb, last) in [
            (true, true, true, "Core43:33350:33361:33387"),
            (false, true, true, "Khr:33350:33361:33387"),
            (false, false, true, "Arb:33350:33361:37192"),
        ] {
            let mut backend = Recorder::default();
            configure(["other", "-debug"], &caps(core, khr, arb), &mut backend);
            assert_eq!(backend.events, ["debug:true", "callback", last]);
        }
    }
    #[test]
    fn no_supported_api_still_sets_up_callback_and_matching_is_exact() {
        let mut backend = Recorder::default();
        configure(["-debug"], &caps(false, false, false), &mut backend);
        assert_eq!(backend.events, ["debug:true", "callback"]);
        for args in [vec![], vec!["--debug"], vec!["-DEBUG"], vec!["-debug=true"]] {
            backend.events.clear();
            configure(args, &caps(true, true, true), &mut backend);
            assert_eq!(backend.events, ["debug:false"]);
        }
    }
    #[test]
    fn current_context_lookup_precedes_argument_branch_and_debug_mutation() {
        // A fresh thread has no current GLContext, independently of other tests.
        std::thread::spawn(|| {
            for args in [vec![], vec!["-debug"]] {
                let mut backend = Recorder::default();
                assert!(
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        configure_current(args, &mut backend);
                    }))
                    .is_err()
                );
                assert!(backend.events.is_empty());
            }
        })
        .join()
        .unwrap();
    }
}

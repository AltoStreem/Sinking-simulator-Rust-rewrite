//! GLContext.java: thread-local context cache, capability reporting and lifecycle.
//! Backend capability objects retain native identity; concrete Bevy integration is pending.
#![allow(dead_code)]
use crate::resource::{ResourceHandle, ResourceRuntime};
use std::{
    any::Any,
    cell::RefCell,
    sync::{Arc, Mutex},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CapabilityValue {
    Boolean(bool),
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapabilityField {
    pub name: String,
    pub value: CapabilityValue,
}
pub(crate) struct Capabilities {
    pub native: Arc<dyn Any + Send + Sync>,
    /// Must match native public reflection field order; do not sort or deduplicate.
    pub fields: Vec<CapabilityField>,
}
pub(crate) trait GlContextBackend: Send {
    fn create_capabilities(&mut self) -> Capabilities;
    fn get_string(&mut self, name: i32) -> Option<String>;
    fn print_line(&mut self, line: &str);
    fn destroy(&mut self);
}
pub(crate) struct GlContext {
    capabilities: Capabilities,
    window: ResourceHandle,
    lifetime: ResourceHandle,
}
thread_local! {
    static CONTEXT: RefCell<Option<Arc<GlContext>>> = const { RefCell::new(None) };
}
impl GlContext {
    fn new(
        window: ResourceHandle,
        backend: Arc<Mutex<dyn GlContextBackend>>,
        runtime: &ResourceRuntime,
    ) -> Self {
        let cleanup = backend.clone();
        let lifetime =
            runtime.allocate(&[window.clone()], move || cleanup.lock().unwrap().destroy());
        let capabilities = backend.lock().unwrap().create_capabilities();
        let version = backend.lock().unwrap().get_string(7938);
        backend.lock().unwrap().print_line(&format!(
            "GL Version: {}",
            version.as_deref().unwrap_or("null")
        ));
        for field in &capabilities.fields {
            if field.value == CapabilityValue::Boolean(true) {
                backend.lock().unwrap().print_line(&field.name);
            }
        }
        Self {
            capabilities,
            window,
            lifetime,
        }
    }
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
    pub fn resource(&self) -> ResourceHandle {
        self.lifetime.clone()
    }
    pub fn close(&self) {
        self.lifetime.close();
    }
    pub fn freed(&self) -> bool {
        self.lifetime.freed()
    }
    pub fn current() -> Arc<Self> {
        CONTEXT.with(|context| {
            context
                .borrow()
                .clone()
                .expect("GLContext.currentContext is null")
        })
    }
    pub fn get_or_create(
        window: ResourceHandle,
        backend: Arc<Mutex<dyn GlContextBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<Self> {
        if let Some(current) = CONTEXT.with(|context| context.borrow().clone()) {
            return current;
        }
        let created = Arc::new(Self::new(window, backend, runtime));
        CONTEXT.with(|context| *context.borrow_mut() = Some(created.clone()));
        created
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    type Log = Arc<Mutex<Vec<String>>>;
    struct Backend {
        log: Log,
    }
    impl GlContextBackend for Backend {
        fn create_capabilities(&mut self) -> Capabilities {
            self.log.lock().unwrap().push("capabilities".into());
            Capabilities {
                native: Arc::new(73_u32),
                fields: vec![
                    CapabilityField {
                        name: "enabled_b".into(),
                        value: CapabilityValue::Boolean(true),
                    },
                    CapabilityField {
                        name: "disabled".into(),
                        value: CapabilityValue::Boolean(false),
                    },
                    CapabilityField {
                        name: "function_pointer".into(),
                        value: CapabilityValue::Other,
                    },
                    CapabilityField {
                        name: "enabled_a".into(),
                        value: CapabilityValue::Boolean(true),
                    },
                ],
            }
        }
        fn get_string(&mut self, name: i32) -> Option<String> {
            assert_eq!(name, 7938);
            self.log.lock().unwrap().push("version".into());
            None
        }
        fn print_line(&mut self, line: &str) {
            self.log.lock().unwrap().push(line.into());
        }
        fn destroy(&mut self) {
            self.log.lock().unwrap().push("destroy-gl".into());
        }
    }
    #[test]
    fn thread_cache_preserves_identity_reporting_and_closed_context() {
        std::thread::spawn(|| {
            let runtime = ResourceRuntime::default();
            let log = Log::default();
            let parent_log = log.clone();
            let window = runtime.allocate(&[], move || {
                parent_log.lock().unwrap().push("destroy-window".into())
            });
            let backend = Arc::new(Mutex::new(Backend { log: log.clone() }));
            let context = GlContext::get_or_create(window.clone(), backend.clone(), &runtime);
            assert_eq!(
                *context.capabilities().native.downcast_ref::<u32>().unwrap(),
                73
            );
            assert_eq!(
                *log.lock().unwrap(),
                [
                    "capabilities",
                    "version",
                    "GL Version: null",
                    "enabled_b",
                    "enabled_a"
                ]
            );
            let other_window = runtime.allocate(&[], || {});
            assert!(Arc::ptr_eq(
                &context,
                &GlContext::get_or_create(other_window, backend, &runtime)
            ));
            let gpu_log = log.clone();
            let gpu = crate::gl_resource::GlResource::from_current(12, &runtime, move || {
                gpu_log.lock().unwrap().push("destroy-gpu".into())
            });
            window.close();
            assert!(gpu.freed());
            assert!(context.freed());
            assert!(Arc::ptr_eq(&context, &GlContext::current()));
            assert_eq!(log.lock().unwrap().len(), 5);
            runtime.run_main();
            assert_eq!(
                &log.lock().unwrap()[5..],
                ["destroy-gpu", "destroy-gl", "destroy-window"]
            );
        })
        .join()
        .unwrap();
    }
    #[test]
    fn current_context_requires_prior_creation_on_each_thread() {
        std::thread::spawn(|| assert!(std::panic::catch_unwind(GlContext::current).is_err()))
            .join()
            .unwrap();
    }
}

//! GLResource.java's ID and strong context dependency, using the Rust lifecycle.
//! Current-GLContext construction is translated; concrete GPU wrappers remain pending.
#![allow(dead_code)]
use crate::resource::{ResourceHandle, ResourceRuntime};
pub(crate) struct GlResource {
    id: i32,
    context: ResourceHandle,
    native_context: Option<std::sync::Arc<crate::gl_context::GlContext>>,
    lifetime: ResourceHandle,
}
impl GlResource {
    pub fn from_current(
        id: i32,
        runtime: &ResourceRuntime,
        free: impl FnOnce() + Send + 'static,
    ) -> Self {
        let context = crate::gl_context::GlContext::current();
        let mut resource = Self::new(id, context.resource(), runtime, free);
        resource.native_context = Some(context);
        resource
    }
    pub fn new(
        id: i32,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
        free: impl FnOnce() + Send + 'static,
    ) -> Self {
        let lifetime = runtime.allocate(&[context.clone()], free);
        Self {
            id,
            context,
            native_context: None,
            lifetime,
        }
    }
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn register_dependent(&self, dependent: &ResourceHandle) {
        self.lifetime.register_dependent(dependent);
    }
    pub fn close(&self) {
        self.lifetime.close();
    }
    pub fn freed(&self) -> bool {
        self.lifetime.freed()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn context_closure_defers_gpu_cleanup_before_context_cleanup() {
        let runtime = ResourceRuntime::default();
        let log = Arc::new(Mutex::new(Vec::new()));
        let events = log.clone();
        let context = runtime.allocate(&[], move || events.lock().unwrap().push("context"));
        let events = log.clone();
        let gpu = GlResource::new(12, context.clone(), &runtime, move || {
            events.lock().unwrap().push("gpu")
        });
        assert_eq!(gpu.id(), 12);
        context.close();
        assert!(gpu.freed());
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(*log.lock().unwrap(), ["gpu", "context"]);
    }
}

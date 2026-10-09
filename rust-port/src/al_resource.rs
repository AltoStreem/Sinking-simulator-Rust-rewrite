//! ALResource.java's generic ID and dependency forwarding.
//! Used by translated audio classes and the live native Windows audio graph.
#![allow(dead_code)]
use crate::resource::{ResourceHandle, ResourceRuntime};
pub(crate) struct AlResource<T> {
    id: T,
    lifetime: ResourceHandle,
}
impl<T> AlResource<T> {
    pub fn new(
        id: T,
        dependencies: &[ResourceHandle],
        runtime: &ResourceRuntime,
        free: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            id,
            lifetime: runtime.allocate(dependencies, free),
        }
    }
    pub fn id(&self) -> &T {
        &self.id
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
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[test]
    fn generic_id_and_dependency_cleanup_match_source_contract() {
        let runtime = ResourceRuntime::default();
        let count = Arc::new(AtomicUsize::new(0));
        let parent = runtime.allocate(&[], || {});
        let observed = count.clone();
        let audio = AlResource::new(vec![3, 7], &[parent.clone()], &runtime, move || {
            observed.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(audio.id(), &[3, 7]);
        parent.close();
        assert!(audio.freed());
        assert_eq!(count.load(Ordering::SeqCst), 0);
        runtime.run_main();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

//! Resource.java dependency closure and main-thread deferred destruction.
//! Rust last-owner drop replaces JVM finalization; registration and pruning
//! run in Bevy rather than the source coroutine/daemon threads.
use bevy::prelude::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, Weak},
    thread::{self, ThreadId},
    time::{Duration, Instant},
};
type Job = Box<dyn FnOnce() + Send + 'static>;
#[derive(Default)]
struct CloseState {
    freed: bool,
    owner: Option<ThreadId>,
}
struct ResourceState {
    state: Mutex<CloseState>,
    closed: Condvar,
    dependents: Mutex<Vec<Weak<ResourceState>>>,
    free: Mutex<Option<Job>>,
    runtime: Weak<RuntimeInner>,
}
#[derive(Clone)]
pub(crate) struct ResourceHandle(Arc<ResourceState>);
impl ResourceHandle {
    pub fn freed(&self) -> bool {
        self.0.state.lock().unwrap().freed
    }
    pub fn register_dependent(&self, dependent: &Self) {
        let mut dependents = self.0.dependents.lock().unwrap();
        dependents.push(Arc::downgrade(&dependent.0));
        dependents.retain(|resource| resource.strong_count() != 0);
    }
    pub fn close(&self) {
        self.0.close();
    }
}
impl ResourceState {
    fn close(self: &Arc<Self>) {
        let current = thread::current().id();
        let mut state = self.state.lock().unwrap();
        if state.freed {
            // Java's reentrant monitor allows dependency cycles on this thread.
            while state.owner.is_some() && state.owner != Some(current) {
                state = self.closed.wait(state).unwrap();
            }
            return;
        }
        state.freed = true;
        state.owner = Some(current);
        drop(state);
        let dependents: Vec<_> = self
            .dependents
            .lock()
            .unwrap()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for dependent in dependents {
            dependent.close();
        }
        if let Some(runtime) = self.runtime.upgrade() {
            let resource = self.clone();
            runtime
                .queue
                .lock()
                .unwrap()
                .push_back(Box::new(move || resource.invoke_free()));
        }
        let mut state = self.state.lock().unwrap();
        state.owner = None;
        self.closed.notify_all();
    }
    fn invoke_free(&self) {
        let free = self.free.lock().unwrap().take();
        if let Some(free) = free {
            free();
        }
    }
}
impl Drop for ResourceState {
    fn drop(&mut self) {
        if self.state.get_mut().unwrap().freed {
            return;
        }
        self.state.get_mut().unwrap().freed = true;
        let dependents: Vec<_> = self
            .dependents
            .get_mut()
            .unwrap()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for dependent in dependents {
            dependent.close();
        }
        if let Some(free) = self.free.get_mut().unwrap().take()
            && let Some(runtime) = self.runtime.upgrade()
        {
            runtime.queue.lock().unwrap().push_back(free);
        }
    }
}
struct RuntimeInner {
    queue: Mutex<VecDeque<Job>>,
    allocated: Mutex<Vec<Weak<ResourceState>>>,
    main_thread: ThreadId,
    last_prune: Mutex<Instant>,
}
#[derive(Resource, Clone)]
pub(crate) struct ResourceRuntime(Arc<RuntimeInner>);
impl Default for ResourceRuntime {
    fn default() -> Self {
        Self(Arc::new(RuntimeInner {
            queue: Mutex::new(VecDeque::new()),
            allocated: Mutex::new(Vec::new()),
            main_thread: thread::current().id(),
            last_prune: Mutex::new(Instant::now()),
        }))
    }
}
impl ResourceRuntime {
    pub fn allocate(
        &self,
        dependencies: &[ResourceHandle],
        free: impl FnOnce() + Send + 'static,
    ) -> ResourceHandle {
        let resource = ResourceHandle(Arc::new(ResourceState {
            state: Mutex::new(CloseState::default()),
            closed: Condvar::new(),
            dependents: Mutex::new(Vec::new()),
            free: Mutex::new(Some(Box::new(free))),
            runtime: Arc::downgrade(&self.0),
        }));
        self.0
            .allocated
            .lock()
            .unwrap()
            .push(Arc::downgrade(&resource.0));
        for dependency in dependencies {
            dependency.register_dependent(&resource);
        }
        resource
    }
    pub fn enqueue(&self, job: impl FnOnce() + Send + 'static) {
        self.0.queue.lock().unwrap().push_back(Box::new(job));
    }
    pub fn run_main(&self) {
        assert_eq!(
            thread::current().id(),
            self.0.main_thread,
            "Resource.runMain must run on its main thread"
        );
        loop {
            let job = self.0.queue.lock().unwrap().pop_front();
            let Some(job) = job else {
                break;
            };
            job();
        }
    }
    pub fn close_all(&self) {
        // Source holds the allocated registry lock throughout closeAll.
        let mut allocated = self.0.allocated.lock().unwrap();
        let resources: Vec<_> = allocated.iter().filter_map(Weak::upgrade).collect();
        for resource in resources {
            resource.close();
        }
        allocated.clear();
    }
    fn prune(&self) {
        let mut last = self.0.last_prune.lock().unwrap();
        if last.elapsed() >= Duration::from_secs(1) {
            self.0
                .allocated
                .lock()
                .unwrap()
                .retain(|resource| resource.strong_count() != 0);
            *last = Instant::now();
        }
    }
}
pub(crate) struct ResourceMainThread(std::marker::PhantomData<std::rc::Rc<()>>);
pub(crate) struct ResourcePlugin;
impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(ResourceMainThread(std::marker::PhantomData))
            .init_resource::<ResourceRuntime>()
            .add_systems(First, run_main)
            .add_systems(Last, shutdown);
    }
}
fn run_main(runtime: Res<ResourceRuntime>, _thread: NonSend<ResourceMainThread>) {
    runtime.run_main();
    runtime.prune();
}
fn shutdown(
    mut exit: MessageReader<bevy::app::AppExit>,
    runtime: Res<ResourceRuntime>,
    _thread: NonSend<ResourceMainThread>,
) {
    if exit.read().next().is_some() {
        runtime.close_all();
        runtime.run_main();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_cleanup_is_deferred_child_first_and_idempotent() {
        let runtime = ResourceRuntime::default();
        let log = Arc::new(Mutex::new(Vec::new()));
        let a = log.clone();
        let parent = runtime.allocate(&[], move || a.lock().unwrap().push("parent"));
        let a = log.clone();
        let child = runtime.allocate(&[parent.clone()], move || a.lock().unwrap().push("child"));
        parent.close();
        parent.close();
        child.close();
        assert!(parent.freed() && child.freed());
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(*log.lock().unwrap(), ["child", "parent"]);
        runtime.run_main();
        assert_eq!(log.lock().unwrap().len(), 2);
    }
    #[test]
    fn weak_dependents_and_last_owner_drop_do_not_leak_or_double_free() {
        let runtime = ResourceRuntime::default();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let parent = runtime.allocate(&[], || {});
        let a = count.clone();
        let child = runtime.allocate(&[parent.clone()], move || {
            a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        drop(child);
        parent.close();
        runtime.run_main();
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
    #[test]
    fn cycles_and_shutdown_close_each_resource_once() {
        let runtime = ResourceRuntime::default();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let a = count.clone();
        let first = runtime.allocate(&[], move || {
            a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let a = count.clone();
        let second = runtime.allocate(&[first.clone()], move || {
            a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        second.register_dependent(&first);
        runtime.close_all();
        runtime.run_main();
        assert!(first.freed() && second.freed());
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
    #[test]
    fn worker_closes_queue_cleanup_on_main_thread() {
        let runtime = ResourceRuntime::default();
        let expected = thread::current().id();
        let resource = runtime.allocate(&[], move || assert_eq!(thread::current().id(), expected));
        thread::spawn(move || resource.close()).join().unwrap();
        runtime.run_main();
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    #[test]
    fn app_exit_closes_and_frees_on_the_main_thread() {
        let mut app = App::new();
        app.add_plugins(ResourcePlugin);
        let runtime = app.world().resource::<ResourceRuntime>().clone();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = count.clone();
        let resource = runtime.allocate(&[], move || {
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.update();
        assert!(resource.freed());
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
    #[test]
    fn queued_main_thread_work_keeps_fifo_order() {
        let runtime = ResourceRuntime::default();
        let events = Arc::new(Mutex::new(Vec::new()));
        for item in [1, 2, 3] {
            let events = events.clone();
            runtime.enqueue(move || events.lock().unwrap().push(item));
        }
        runtime.run_main();
        assert_eq!(*events.lock().unwrap(), [1, 2, 3]);
    }
}

// Non-Send source objects are retained on the main thread. The deferred queue
// carries only a key; cleanup resolves and invokes its local closure on run_main.
thread_local! {
    static LOCAL_CLEANUPS: std::cell::RefCell<std::collections::HashMap<u64, Box<dyn FnOnce()>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}
static NEXT_LOCAL_CLEANUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
impl ResourceRuntime {
    pub fn allocate_local(
        &self,
        dependencies: &[ResourceHandle],
        free: impl FnOnce() + 'static,
    ) -> ResourceHandle {
        assert_eq!(
            thread::current().id(),
            self.0.main_thread,
            "Local resource construction must use the resource main thread"
        );
        let id = NEXT_LOCAL_CLEANUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        LOCAL_CLEANUPS.with(|cleanups| cleanups.borrow_mut().insert(id, Box::new(free)));
        self.allocate(dependencies, move || {
            let free = LOCAL_CLEANUPS.with(|cleanups| cleanups.borrow_mut().remove(&id));
            if let Some(free) = free {
                free();
            }
        })
    }
}
#[cfg(test)]
mod local_cleanup_tests {
    use super::*;
    #[test]
    fn local_object_cleanup_runs_once_after_cross_thread_close_and_can_allocate_again() {
        let runtime = ResourceRuntime::default();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let receiver = calls.clone();
        let runtime2 = runtime.clone();
        let resource = runtime.allocate_local(&[], move || {
            receiver.set(receiver.get() + 1);
            let receiver = receiver.clone();
            let nested = runtime2.allocate_local(&[], move || receiver.set(receiver.get() + 1));
            nested.close();
        });
        let closer = resource.clone();
        std::thread::spawn(move || closer.close()).join().unwrap();
        assert_eq!(calls.get(), 0);
        runtime.run_main();
        assert_eq!(calls.get(), 2);
        resource.close();
        runtime.run_main();
        assert_eq!(calls.get(), 2);
    }
}

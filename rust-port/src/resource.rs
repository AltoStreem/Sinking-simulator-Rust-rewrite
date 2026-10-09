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
// Source Function0 receivers can be invoked again after a throwing free hook.
type Job = Arc<dyn Fn() + Send + Sync + 'static>;
struct QueueEntry {
    job: Mutex<Option<Job>>,
    next: Mutex<Option<Arc<QueueEntry>>>,
}
impl QueueEntry {
    fn new(job: Job) -> Arc<Self> {
        Arc::new(Self { job: Mutex::new(Some(job)), next: Mutex::new(None) })
    }
}
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
    /// Source engine objects retain Rc callbacks on the engine thread. Schedule
    /// their subclass free hook ahead of the native superclass destructor,
    /// without transferring those callbacks across threads.
    pub(crate) fn before_free_local(&self, hook: impl Fn() + 'static) {
        let runtime = self.0.runtime.upgrade().expect("resource runtime dropped");
        assert_eq!(
            thread::current().id(),
            runtime.main_thread,
            "local free hook must be registered on the engine thread"
        );
        assert!(!self.freed(), "resource already closed");
        let mut free = self.0.free.lock().unwrap();
        let original = free.take().expect("resource cleanup already taken");
        let id = NEXT_LOCAL_CLEANUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        LOCAL_CLEANUPS.with(|hooks| hooks.borrow_mut().insert(id, std::rc::Rc::new(hook)));
        *free = Some(Arc::new(move || {
            let hook = LOCAL_CLEANUPS
                .with(|hooks| hooks.borrow().get(&id).cloned())
                .expect("local cleanup invoked outside its engine thread");
            hook();
            original();
            // A throwing subclass/superclass free keeps the whole hook for retry.
            LOCAL_CLEANUPS.with(|hooks| hooks.borrow_mut().remove(&id));
        }));
    }
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
            runtime.enqueue(Arc::new(move || resource.invoke_free()));
        }
        let mut state = self.state.lock().unwrap();
        state.owner = None;
        self.closed.notify_all();
    }
    fn invoke_free(&self) {
        // Do not hold a mutex during a source callback: it may reenter runMain.
        let free = self.free.lock().unwrap().clone();
        if let Some(free) = free {
            free();
            self.free.lock().unwrap().take();
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
            runtime.enqueue(free);
        }
    }
}
struct RuntimeInner {
    queue: Mutex<VecDeque<Arc<QueueEntry>>>,
    allocated: Mutex<Vec<Weak<ResourceState>>>,
    main_thread: ThreadId,
    last_prune: Mutex<Instant>,
}
impl RuntimeInner {
    fn enqueue(&self, job: Job) {
        let entry = QueueEntry::new(job);
        let mut queue = self.queue.lock().unwrap();
        if let Some(tail) = queue.back() {
            *tail.next.lock().unwrap() = Some(entry.clone());
        }
        queue.push_back(entry);
    }
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
        free: impl Fn() + Send + Sync + 'static,
    ) -> ResourceHandle {
        let resource = ResourceHandle(Arc::new(ResourceState {
            state: Mutex::new(CloseState::default()),
            closed: Condvar::new(),
            dependents: Mutex::new(Vec::new()),
            free: Mutex::new(Some(Arc::new(free))),
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
    pub fn enqueue(&self, job: impl Fn() + Send + Sync + 'static) {
        self.0.enqueue(Arc::new(job));
    }
    pub(crate) fn is_main_thread(&self) -> bool {
        thread::current().id() == self.0.main_thread
    }
    pub fn run_main(&self) {
        assert_eq!(
            thread::current().id(),
            self.0.main_thread,
            "Resource.runMain must run on its main thread"
        );
        // Bundled Java 13 ConcurrentLinkedDeque.removeIf saves next before
        // invoking the predicate. Tail appends from that callback wait until
        // the next runMain; appends before an existing tail can still be seen.
        let mut current = self.0.queue.lock().unwrap().front().cloned();
        while let Some(entry) = current {
            let next = entry.next.lock().unwrap().clone();
            let job = entry.job.lock().unwrap().clone();
            if let Some(job) = job {
                job(); // On panic, this node and the unvisited suffix remain.
                entry.job.lock().unwrap().take();
                let mut queue = self.0.queue.lock().unwrap();
                if let Some(index) = queue.iter().position(|candidate| Arc::ptr_eq(candidate, &entry)) {
                    queue.remove(index);
                }
            }
            current = next;
        }
    }
    #[cfg(test)]
    pub(crate) fn discard_queued_for_probe(&self) {
        // Equivalent to the original probe's explicit deque.clear. Production
        // never discards a throwing callback. Drop jobs outside the queue lock
        // because their retained receiver cleanup may enqueue more work.
        let jobs = std::mem::take(&mut *self.0.queue.lock().unwrap());
        drop(jobs);
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
pub(crate) fn shutdown(
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
    fn local_subclass_hook_runs_on_engine_thread_before_deferred_native_free() {
        use std::{cell::RefCell, rc::Rc};
        let runtime = ResourceRuntime::default();
        let native = Arc::new(Mutex::new(vec![]));
        let local = Rc::new(RefCell::new(vec![]));
        let events = native.clone();
        let resource = runtime.allocate(&[], move || events.lock().unwrap().push("native"));
        let events = native.clone();
        let callbacks = local.clone();
        resource.before_free_local(move || {
            callbacks.borrow_mut().push("removed");
            events.lock().unwrap().push("local");
        });
        let close = resource.clone();
        std::thread::spawn(move || close.close()).join().unwrap();
        assert!(resource.freed());
        assert!(native.lock().unwrap().is_empty());
        assert!(local.borrow().is_empty());
        runtime.run_main();
        assert_eq!(*local.borrow(), ["removed"]);
        assert_eq!(*native.lock().unwrap(), ["local", "native"]);
        runtime.run_main();
        assert_eq!(native.lock().unwrap().len(), 2);
    }
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
    static LOCAL_CLEANUPS: std::cell::RefCell<std::collections::HashMap<u64, std::rc::Rc<dyn Fn()>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}
static NEXT_LOCAL_CLEANUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
impl ResourceRuntime {
    pub fn allocate_local(
        &self,
        dependencies: &[ResourceHandle],
        free: impl Fn() + 'static,
    ) -> ResourceHandle {
        assert_eq!(
            thread::current().id(),
            self.0.main_thread,
            "Local resource construction must use the resource main thread"
        );
        let id = NEXT_LOCAL_CLEANUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        LOCAL_CLEANUPS.with(|cleanups| cleanups.borrow_mut().insert(id, std::rc::Rc::new(free)));
        self.allocate(dependencies, move || {
            let free = LOCAL_CLEANUPS.with(|cleanups| cleanups.borrow().get(&id).cloned());
            if let Some(free) = free {
                free();
                LOCAL_CLEANUPS.with(|cleanups| cleanups.borrow_mut().remove(&id));
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
        assert_eq!(calls.get(), 1);
        runtime.run_main();
        assert_eq!(calls.get(), 2);
        resource.close();
        runtime.run_main();
        assert_eq!(calls.get(), 2);
    }
}

#[cfg(test)]
mod source_queue_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn receiver(runtime: &ResourceRuntime, log: &Arc<Mutex<String>>, id: usize,
                failures: usize, spawn: usize) -> ResourceHandle {
        let runtime2 = runtime.clone();
        let log = log.clone();
        let failures = AtomicUsize::new(failures);
        let spawn = AtomicUsize::new(spawn);
        runtime.allocate(&[], move || {
            log.lock().unwrap().push(char::from_digit(id as u32, 10).unwrap());
            if failures.fetch_update(Ordering::SeqCst, Ordering::SeqCst,
                |n| n.checked_sub(1)).is_ok() {
                panic!("probe failure");
            }
            match spawn.swap(0, Ordering::SeqCst) {
                1 => receiver(&runtime2, &log, 9, 0, 0).close(),
                2 => runtime2.run_main(),
                _ => {},
            }
        })
    }

    #[test]
    fn original_jvm_resource_queue_snapshots_match_live_runtime() {
        let capture: serde_json::Value = serde_json::from_str(include_str!(
            "../tools/source-engine-resource-capture.json")).unwrap();
        let cases = [
            ("fifo", vec![(1, 0, 0), (2, 0, 0), (3, 0, 0)]),
            ("failure-first", vec![(1, 1, 0), (2, 0, 0)]),
            ("failure-middle", vec![(1, 0, 0), (2, 1, 0), (3, 0, 0)]),
            ("nested-only", vec![(1, 0, 1)]),
            ("nested-before-tail", vec![(1, 0, 1), (2, 0, 0)]),
            ("reentrant", vec![(1, 0, 2), (2, 0, 0)]),
            ("dependent-close", vec![(1, 0, 0), (2, 0, 0)]),
        ];
        assert_eq!(capture["cases"].as_array().unwrap().len(), cases.len());
        let mut errors = 0;
        let mut snapshots = 0;
        for (name, items) in cases {
            let runtime = ResourceRuntime::default();
            let log = Arc::new(Mutex::new(String::new()));
            let ships: Vec<_> = items.into_iter().map(|(id, fail, spawn)|
                receiver(&runtime, &log, id, fail, spawn)).collect();
            if name == "dependent-close" {
                ships[0].register_dependent(&ships[1]);
                ships[0].close();
                ships[0].close();
            } else {
                for ship in &ships { ship.close(); }
            }
            assert!(ships.iter().all(ResourceHandle::freed), "{name}");
            for phase in ["closed", "first", "second"] {
                if phase != "closed" {
                    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
                        runtime.run_main())).is_err() { errors += 1; }
                }
                let key = format!("{name}:{phase}");
                let expected = &capture["observations"][&key];
                if expected.is_null() { continue; }
                assert_eq!(&*log.lock().unwrap(), expected["calls"].as_str().unwrap(), "{key}");
                assert_eq!(runtime.0.queue.lock().unwrap().len(),
                    expected["queueSize"].as_u64().unwrap() as usize, "{key}");
                snapshots += 1;
            }
        }
        assert_eq!(snapshots, 20);
        assert_eq!(errors, capture["caughtErrors"].as_array().unwrap().len());
        assert_eq!(capture["freedFlags"], serde_json::json!(["true", "true"]));
    }

    #[test]
    fn actual_bevy_first_stage_resumes_tail_enqueues_on_the_next_frame() {
        let mut app = App::new();
        app.add_plugins(ResourcePlugin);
        let runtime = app.world().resource::<ResourceRuntime>().clone();
        let log = Arc::new(Mutex::new(String::new()));
        let ship = receiver(&runtime, &log, 1, 0, 1);
        ship.close();
        app.update();
        assert_eq!(&*log.lock().unwrap(), "1");
        assert_eq!(runtime.0.queue.lock().unwrap().len(), 1);
        app.update();
        assert_eq!(&*log.lock().unwrap(), "19");
        assert!(runtime.0.queue.lock().unwrap().is_empty());
        app.update();
        assert_eq!(&*log.lock().unwrap(), "19");
    }

    #[test]
    fn local_subclass_free_retries_after_native_free_failure() {
        let runtime = ResourceRuntime::default();
        let native = Arc::new(AtomicUsize::new(0));
        let local = std::rc::Rc::new(std::cell::Cell::new(0));
        let native2 = native.clone();
        let ship = runtime.allocate(&[], move || {
            if native2.fetch_add(1, Ordering::SeqCst) == 0 { panic!("native failure"); }
        });
        let local2 = local.clone();
        ship.before_free_local(move || local2.set(local2.get() + 1));
        ship.close();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.run_main())).is_err());
        assert!(ship.freed());
        assert_eq!(runtime.0.queue.lock().unwrap().len(), 1);
        assert_eq!((local.get(), native.load(Ordering::SeqCst)), (1, 1));
        runtime.run_main();
        assert_eq!((local.get(), native.load(Ordering::SeqCst)), (2, 2));
        ship.close();
        runtime.run_main();
        assert_eq!((local.get(), native.load(Ordering::SeqCst)), (2, 2));
    }

    #[test]
    fn local_throwing_free_is_retained_without_poisoning_the_registry() {
        let runtime = ResourceRuntime::default();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let calls2 = calls.clone();
        let ship = runtime.allocate_local(&[], move || {
            calls2.set(calls2.get() + 1);
            if calls2.get() == 1 { panic!("local failure"); }
        });
        let closer = ship.clone();
        std::thread::spawn(move || closer.close()).join().unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.run_main())).is_err());
        assert_eq!(calls.get(), 1);
        assert!(ship.freed());
        assert_eq!(runtime.0.queue.lock().unwrap().len(), 1);
        runtime.run_main();
        assert_eq!(calls.get(), 2);
        assert!(runtime.0.queue.lock().unwrap().is_empty());
    }
}

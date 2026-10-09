//! Main$main$counter$1.class: enqueue a callback retaining both report values.
//! The injected title sink must target the retained source Window.
//! Native window retention stays on the resource main thread; worker reports queue values.
use std::sync::Arc;
pub(crate) fn invoke(
    runtime: &crate::resource::ResourceRuntime,
    set_title: Arc<dyn Fn(String) + Send + Sync>,
    resources: f64,
    speed: f64,
) {
    runtime.enqueue(move || {
        crate::main_counter_title::invoke(resources, speed, |title| set_title(title))
    });
}

// The reporter transfers only a registry key across threads. The retained Rc
// window stays on ResourceRuntime's main thread, including queued callbacks.
thread_local! {
    static WINDOWS: std::cell::RefCell<std::collections::HashMap<u64, std::rc::Rc<crate::window::SourceWindow>>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}
static NEXT_WINDOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct WindowLease {
    id: u64,
    runtime: crate::resource::ResourceRuntime,
}
impl Drop for WindowLease {
    fn drop(&mut self) {
        let id = self.id;
        self.runtime.enqueue(move || {
            WINDOWS.with(|windows| {
                windows.borrow_mut().remove(&id);
            });
        });
    }
}
pub(crate) fn native(
    runtime: &crate::resource::ResourceRuntime,
    window: std::rc::Rc<crate::window::SourceWindow>,
) -> Arc<dyn Fn(f64, f64) + Send + Sync> {
    assert!(
        runtime.is_main_thread(),
        "Main counter window must be retained on the resource main thread"
    );
    let id = NEXT_WINDOW.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    WINDOWS.with(|windows| {
        windows.borrow_mut().insert(id, window);
    });
    let lease = Arc::new(WindowLease {
        id,
        runtime: runtime.clone(),
    });
    let sink: Arc<dyn Fn(String) + Send + Sync> = Arc::new(move |title| {
        let window = WINDOWS
            .with(|windows| windows.borrow().get(&lease.id).cloned())
            .expect("retained Main counter window missing");
        window.set_title(title);
    });
    let runtime = runtime.clone();
    Arc::new(move |resources, speed| invoke(&runtime, sink.clone(), resources, speed))
}
pub(crate) fn native_counter(
    runtime: &crate::resource::ResourceRuntime,
    window: std::rc::Rc<crate::window::SourceWindow>,
) -> crate::time_sync::SourceTimeSync {
    crate::time_sync::SourceTimeSync::new(native(runtime, window), runtime)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_worker_reports_update_retained_window_and_release_after_queued_jobs() {
        let (window, runtime, log) = crate::window::tests::fixture();
        let initial = window.title();
        let baseline = std::rc::Rc::strong_count(&window);
        let callback = native(&runtime, window.clone());
        assert_eq!(std::rc::Rc::strong_count(&window), baseline + 1);
        std::thread::spawn(move || {
            callback(0.53, 0.98);
            callback(0.54, 0.99);
            // Last callback owner drops on worker; queued reports still retain it.
        })
        .join()
        .unwrap();
        assert_eq!(window.title(), initial);
        assert_eq!(std::rc::Rc::strong_count(&window), baseline + 1);
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(window.title(), crate::main_counter_title::title(0.54, 0.99));
        assert_eq!(
            *log.lock().unwrap(),
            vec![
                format!("title:{}", crate::main_counter_title::title(0.53, 0.98)),
                format!("title:{}", crate::main_counter_title::title(0.54, 0.99))
            ]
        );
        // Dropping the final report's lease appends cleanup at the saved tail.
        assert_eq!(std::rc::Rc::strong_count(&window), baseline + 1);
        runtime.run_main();
        assert_eq!(std::rc::Rc::strong_count(&window), baseline);
    }
    #[test]
    fn native_callbacks_keep_distinct_windows_and_do_not_retarget_to_later_receiver() {
        let (first, runtime, first_log) = crate::window::tests::fixture();
        let (second, _, second_log) = crate::window::tests::fixture();
        let first_callback = native(&runtime, first.clone());
        let second_callback = native(&runtime, second.clone());
        first_callback(0.11, 0.22);
        second_callback(0.33, 0.44);
        runtime.run_main();
        assert_eq!(first.title(), crate::main_counter_title::title(0.11, 0.22));
        assert_eq!(second.title(), crate::main_counter_title::title(0.33, 0.44));
        assert_eq!(first_log.lock().unwrap().len(), 1);
        assert_eq!(second_log.lock().unwrap().len(), 1);
        drop(first_callback);
        drop(second_callback);
        runtime.run_main();
    }
    #[test]
    fn native_nan_failure_keeps_queued_job_and_window_until_explicit_probe_discard() {
        let (window, runtime, log) = crate::window::tests::fixture();
        let baseline = std::rc::Rc::strong_count(&window);
        let callback = native(&runtime, window.clone());
        callback(f64::NAN, 1.0);
        drop(callback);
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../tools/source-counter-queue-capture.json")).unwrap();
        assert_eq!(source["queueSizes"], serde_json::json!([0, 1, 1, 1, 0]));
        for error in source["errors"].as_array().unwrap() {
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.run_main())).unwrap_err();
            let message = failure.downcast_ref::<String>().map(String::as_str)
                .or_else(|| failure.downcast_ref::<&str>().copied()).unwrap();
            assert_eq!(message, error["message"].as_str().unwrap());
            assert!(log.lock().unwrap().is_empty());
            assert_eq!(std::rc::Rc::strong_count(&window), baseline + 1);
        }
        // This is explicit test teardown, matching the original probe's clear;
        // neither source nor live Rust silently removes the failed report.
        runtime.discard_queued_for_probe();
        runtime.run_main();
        assert_eq!(std::rc::Rc::strong_count(&window), baseline);
    }
    #[test]
    fn worker_enqueues_values_and_title_sink_runs_on_main_thread_in_order() {
        let runtime = crate::resource::ResourceRuntime::default();
        let reports = Arc::new(std::sync::Mutex::new(Vec::new()));
        let retained = reports.clone();
        let main = std::thread::current().id();
        let sink: Arc<dyn Fn(String) + Send + Sync> = Arc::new(move |title| {
            assert_eq!(std::thread::current().id(), main);
            retained.lock().unwrap().push(title);
        });
        let worker_runtime = runtime.clone();
        std::thread::spawn(move || {
            invoke(&worker_runtime, sink.clone(), 0.53, 0.98);
            invoke(&worker_runtime, sink, 0.54, 0.99);
        })
        .join()
        .unwrap();
        assert!(reports.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(
            *reports.lock().unwrap(),
            vec![
                crate::main_counter_title::title(0.53, 0.98),
                crate::main_counter_title::title(0.54, 0.99)
            ]
        );
    }
    #[test]
    fn nan_error_is_deferred_until_queue_execution() {
        let runtime = crate::resource::ResourceRuntime::default();
        invoke(
            &runtime,
            Arc::new(|_| panic!("sink must not run")),
            f64::NAN,
            1.0,
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.run_main())).is_err()
        );
    }
}

//! GLFW.java initialization, monitor enumeration and deferred native shutdown.
#![allow(dead_code)]
use crate::{
    monitor::{Monitor, MonitorBackend},
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex};
pub(crate) trait GlfwBackend: Send {
    fn install_print_error_callback(&mut self);
    fn init(&mut self) -> bool;
    /// Full capacity of the source PointerBuffer, indexed from zero regardless of position.
    fn monitors(&mut self) -> Option<Vec<i64>>;
    /// Source GLFW.monitors indexes a PointerBuffer over `0 until capacity`, using absolute gets.
    /// Native adapters can override this to preserve a distinct limit and capacity.
    fn monitor_pointer_buffer(&mut self) -> Option<SourcePointerBuffer> {
        self.monitors().map(SourcePointerBuffer::complete)
    }
    fn primary_monitor(&mut self) -> i64;
    fn monitor_backend(&self) -> Arc<Mutex<dyn MonitorBackend>>;
    fn terminate(&mut self);
    fn clear_error_callback(&mut self);
}
/// NIO-style pointer buffer shape used by GLFW$monitors$1.
/// Absolute get ignores position but still checks the current limit.
#[derive(Clone, Debug)]
pub(crate) struct SourcePointerBuffer {
    addresses: Vec<i64>,
    capacity: usize,
    limit: usize,
}
impl SourcePointerBuffer {
    pub(crate) fn new(addresses: Vec<i64>, capacity: usize, limit: usize) -> Self {
        Self {
            addresses,
            capacity,
            limit,
        }
    }
    fn complete(addresses: Vec<i64>) -> Self {
        let capacity = addresses.len();
        Self {
            addresses,
            capacity,
            limit: capacity,
        }
    }
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
    pub(crate) fn get_absolute(&self, index: usize) -> Result<i64, &'static str> {
        if index >= self.limit {
            return Err("PointerBuffer absolute get exceeds limit");
        }
        self.addresses
            .get(index)
            .copied()
            .ok_or("PointerBuffer absolute get exceeds storage")
    }
}
pub(crate) struct Glfw {
    pub monitors: Vec<Monitor>,
    pub monitor: Monitor,
    lifetime: ResourceHandle,
}
impl Glfw {
    pub fn new(
        backend: Arc<Mutex<dyn GlfwBackend>>,
        runtime: &ResourceRuntime,
    ) -> Result<Self, &'static str> {
        let cleanup = backend.clone();
        let lifetime = runtime.allocate(&[], move || {
            let mut backend = cleanup.lock().unwrap();
            backend.terminate();
            backend.clear_error_callback();
        });
        backend.lock().unwrap().install_print_error_callback();
        if !backend.lock().unwrap().init() {
            return Err("Unable to initialize GLFW");
        }
        let monitors = crate::glfw_monitors::invoke(backend.clone())?;
        let monitor_backend = backend.lock().unwrap().monitor_backend();
        let primary = backend.lock().unwrap().primary_monitor();
        let monitor = Monitor::new(primary, monitor_backend)?;
        Ok(Self {
            monitors,
            monitor,
            lifetime,
        })
    }
    pub fn resource(&self) -> ResourceHandle {
        self.lifetime.clone()
    }
    pub fn close(&self) {
        self.lifetime.close()
    }
    pub fn freed(&self) -> bool {
        self.lifetime.freed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::NativeVideoMode;
    type Log = Arc<Mutex<Vec<String>>>;
    struct MonitorBackendRecorder {
        log: Log,
        physical: i32,
        missing: bool,
    }
    impl MonitorBackend for MonitorBackendRecorder {
        fn video_mode(&mut self, p: i64) -> Option<Arc<Mutex<NativeVideoMode>>> {
            self.log.lock().unwrap().push(format!("mode:{p}"));
            if self.missing {
                None
            } else {
                Some(Arc::new(Mutex::new(NativeVideoMode {
                    width: 800,
                    height: 600,
                    r: 8,
                    g: 8,
                    b: 8,
                    refresh: 60,
                })))
            }
        }
        fn video_modes(&mut self, p: i64) -> Option<Vec<Arc<Mutex<NativeVideoMode>>>> {
            self.log.lock().unwrap().push(format!("modes:{p}"));
            None
        }
        fn content_scale(&mut self, p: i64) -> [f32; 2] {
            self.log.lock().unwrap().push(format!("scale:{p}"));
            [1.25, 1.5]
        }
        fn physical_size(&mut self, p: i64) -> [i32; 2] {
            self.log.lock().unwrap().push(format!("physical:{p}"));
            self.physical += 1;
            [self.physical, 300]
        }
    }
    struct Backend {
        log: Log,
        monitor: Arc<Mutex<dyn MonitorBackend>>,
        init: bool,
        pointers: Option<Vec<i64>>,
    }
    impl GlfwBackend for Backend {
        fn install_print_error_callback(&mut self) {
            self.log.lock().unwrap().push("errors".into())
        }
        fn init(&mut self) -> bool {
            self.log.lock().unwrap().push("init".into());
            self.init
        }
        fn monitors(&mut self) -> Option<Vec<i64>> {
            self.log.lock().unwrap().push("enumerate".into());
            self.pointers.clone()
        }
        fn primary_monitor(&mut self) -> i64 {
            self.log.lock().unwrap().push("primary".into());
            10
        }
        fn monitor_backend(&self) -> Arc<Mutex<dyn MonitorBackend>> {
            self.monitor.clone()
        }
        fn terminate(&mut self) {
            self.log.lock().unwrap().push("terminate".into())
        }
        fn clear_error_callback(&mut self) {
            self.log.lock().unwrap().push("clear-errors".into())
        }
    }
    fn fixture(
        init: bool,
        pointers: Option<Vec<i64>>,
        missing: bool,
    ) -> (Arc<Mutex<dyn GlfwBackend>>, ResourceRuntime, Log) {
        let log = Log::default();
        let monitor = Arc::new(Mutex::new(MonitorBackendRecorder {
            log: log.clone(),
            physical: 500,
            missing,
        }));
        (
            Arc::new(Mutex::new(Backend {
                log: log.clone(),
                monitor,
                init,
                pointers,
            })),
            ResourceRuntime::default(),
            log,
        )
    }
    #[test]
    fn initializes_primary_separately_and_defers_shutdown_in_order() {
        let (backend, runtime, log) = fixture(true, Some(vec![10, 20]), false);
        let glfw = Glfw::new(backend, &runtime).unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "errors",
                "init",
                "enumerate",
                "mode:10",
                "modes:10",
                "scale:10",
                "mode:20",
                "modes:20",
                "scale:20",
                "primary",
                "mode:10",
                "modes:10",
                "scale:10"
            ]
        );
        assert_eq!(glfw.monitors.len(), 2);
        assert!(!Arc::ptr_eq(&glfw.monitors[0].size, &glfw.monitor.size));
        assert_eq!(*glfw.monitor.scale.lock().unwrap(), [1.25, 1.5]);
        glfw.monitor.video_mode.lock().unwrap().width = 1000;
        assert_eq!(*glfw.monitor.size.lock().unwrap(), [800, 600]);
        assert_eq!(glfw.monitor.physical_size(), [501., 300.]);
        assert_eq!(glfw.monitor.physical_size(), [502., 300.]);
        let child_log = log.clone();
        let child = runtime.allocate(&[glfw.resource()], move || {
            child_log.lock().unwrap().push("child".into())
        });
        glfw.close();
        assert!(glfw.freed());
        assert!(child.freed());
        assert!(!log.lock().unwrap().iter().any(|s| s == "terminate"));
        runtime.run_main();
        assert_eq!(
            &log.lock().unwrap()[15..],
            ["child", "terminate", "clear-errors"]
        );
    }
    #[test]
    fn main_shutdown_launches_after_close_before_deferred_native_destruction() {
        let (backend, runtime, log) = fixture(true, Some(vec![10]), false);
        let glfw = Glfw::new(backend, &runtime).unwrap();
        log.lock().unwrap().clear();
        let child_log = log.clone();
        let child = runtime.allocate(&[glfw.resource()], move || {
            child_log.lock().unwrap().push("child".into());
        });
        let other_log = log.clone();
        let other = runtime.allocate(&[], move || {
            other_log.lock().unwrap().push("other".into());
        });
        let launch_log = log.clone();
        let mut operations = crate::main_shutdown::NativeOperations {
            glfw: &glfw,
            runtime: &runtime,
            launch: Box::new(|| {
                assert!(glfw.freed());
                assert!(child.freed());
                assert!(!other.freed());
                assert!(launch_log.lock().unwrap().is_empty());
                launch_log.lock().unwrap().push("launch".into());
                Ok(())
            }),
        };
        crate::main_shutdown::finish(&mut operations).unwrap();
        assert!(other.freed());
        assert_eq!(
            *log.lock().unwrap(),
            ["launch", "child", "terminate", "clear-errors", "other"]
        );
    }
    #[test]
    fn failures_stop_queries_at_the_source_failure_points() {
        for (init, pointers, missing, error, expected) in [
            (
                false,
                Some(vec![10]),
                false,
                "Unable to initialize GLFW",
                vec!["errors", "init"],
            ),
            (
                true,
                None,
                false,
                "No monitors available",
                vec!["errors", "init", "enumerate"],
            ),
            (
                true,
                Some(vec![10]),
                true,
                "No video mode in monitor",
                vec!["errors", "init", "enumerate", "mode:10"],
            ),
        ] {
            let (backend, _, log) = fixture(init, pointers, missing);
            assert_eq!(
                Glfw::new(backend, &ResourceRuntime::default()).err(),
                Some(error)
            );
            assert_eq!(*log.lock().unwrap(), expected);
        }
    }
    #[test]
    fn empty_nonnull_monitor_buffer_still_queries_primary() {
        let (backend, runtime, log) = fixture(true, Some(vec![]), false);
        let glfw = Glfw::new(backend, &runtime).unwrap();
        assert!(glfw.monitors.is_empty());
        assert_eq!(
            *log.lock().unwrap(),
            [
                "errors",
                "init",
                "enumerate",
                "primary",
                "mode:10",
                "modes:10",
                "scale:10"
            ]
        );
    }
}

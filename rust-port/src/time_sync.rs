//! TimeSync.java's frame sample, millisecond pacing and drained averages.
//! Source background reports run on a shared worker; Bevy applies title messages on its main thread.
//! Resource disposal is deferred; exact JVM coroutine and ArrayList race semantics remain pending.
use bevy::prelude::*;
use std::time::{Duration, Instant};
pub(crate) const REFERENCE: f32 = 0.016666668;
#[derive(Resource)]
pub(crate) struct TimeSync {
    pub resources: Vec<f64>,
    pub speed: Vec<f64>,
    last: Instant,
    last_report: Instant,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    lifecycle: Option<crate::resource::ResourceHandle>,
    source: Option<SourceTimeSync>,
    reports: Option<std::sync::Mutex<std::sync::mpsc::Receiver<(f64, f64)>>>,
}
impl Default for TimeSync {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            resources: Vec::new(),
            speed: Vec::new(),
            last: now,
            last_report: now,
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
            lifecycle: None,
            source: None,
            reports: None,
        }
    }
}
impl TimeSync {
    pub fn sample(&mut self, elapsed: Duration) -> Duration {
        let reference = REFERENCE as f64;
        let offset = reference - elapsed.as_secs_f64();
        let millis = ((offset * 1000.0) as i64).max(0) as u64;
        let resources = (1.0 - offset / reference).max(0.0);
        self.resources.push(resources);
        self.speed.push(1.0 / resources.max(1.0));
        Duration::from_millis(millis)
    }
    pub fn update(&mut self) {
        if let Some(source) = &self.source {
            source.update();
            return;
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        // Source exchanges last before sleeping, so the next sample includes sleep.
        self.last = now;
        std::thread::sleep(self.sample(elapsed));
    }
    pub fn take_averages(&mut self) -> (f64, f64) {
        fn average(values: &mut Vec<f64>) -> f64 {
            let result = if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            };
            values.clear();
            result
        }
        (average(&mut self.resources), average(&mut self.speed))
    }
    #[allow(dead_code)]
    pub fn free(&mut self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
pub(crate) fn daylight(time: f32, cycle_length: f32) -> f32 {
    let phase = time * std::f32::consts::PI * 2.0 / cycle_length % (std::f32::consts::PI * 2.0);
    (phase as f64).cos() as f32 * 0.5 + 0.5
}
pub(crate) fn sync_frame(
    mut counter: ResMut<TimeSync>,
    mut simulation: ResMut<crate::Simulation>,
    mut windows: Query<&mut Window>,
    _thread: Option<NonSend<crate::resource::ResourceMainThread>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    if window.physical_width() == 0 || window.physical_height() == 0 {
        return;
    }
    counter.update();
    let reports: Vec<_> = counter
        .reports
        .as_ref()
        .map(|receiver| receiver.lock().unwrap().try_iter().collect())
        .unwrap_or_default();
    for (resources, speed) in reports {
        window.title = format!(
            "Resources: {:>5}% usage, {:>5}% speed",
            (resources * 100.).round() as i32,
            (speed * 100.).round() as i32
        );
    }
    if counter.source.is_none()
        && counter.running.load(std::sync::atomic::Ordering::SeqCst)
        && counter.last_report.elapsed() >= Duration::from_secs(1)
    {
        let (resources, speed) = counter.take_averages();
        window.title = format!(
            "Resources: {:>5}% usage, {:>5}% speed",
            (resources * 100.0).round() as i32,
            (speed * 100.0).round() as i32
        );
        counter.last_report = Instant::now();
    }
    // Main.java advances after the scene and GUI have consumed the current time.
    if !simulation.paused {
        simulation.elapsed += REFERENCE;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn samples_match_source_reference_sleep_and_speed() {
        let mut clock = TimeSync::default();
        assert_eq!(
            clock.sample(Duration::from_millis(5)),
            Duration::from_millis(11)
        );
        assert!((clock.resources[0] - 0.005 / REFERENCE as f64).abs() < 1e-12);
        assert_eq!(clock.speed[0], 1.0);
        assert_eq!(clock.sample(Duration::from_millis(50)), Duration::ZERO);
        assert!((clock.speed[1] - REFERENCE as f64 / 0.05).abs() < 1e-12);
        let averages = clock.take_averages();
        assert!(averages.0 > 1.0);
        assert!(averages.1 < 1.0);
        assert_eq!(clock.take_averages(), (0.0, 0.0));
    }
    #[test]
    fn source_day_cycle_has_no_added_phase_offset() {
        assert_eq!(daylight(0.0, 120.0), 1.0);
        assert_eq!(daylight(60.0, 120.0), 0.0);
        assert!((daylight(30.0, 120.0) - 0.5).abs() < 0.000001);
        assert!(daylight(1.0, 0.0).is_nan());
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    #[test]
    fn active_scene_clock_advances_once_per_frame_and_respects_pause() {
        let mut app = App::new();
        app.init_resource::<TimeSync>()
            .init_resource::<crate::Simulation>();
        app.world_mut().spawn(Window::default());
        app.add_systems(Last, sync_frame);
        app.update();
        assert_eq!(
            app.world().resource::<crate::Simulation>().elapsed,
            REFERENCE
        );
        app.update();
        assert_eq!(
            app.world().resource::<crate::Simulation>().elapsed,
            REFERENCE + REFERENCE
        );
        app.world_mut().resource_mut::<crate::Simulation>().paused = true;
        app.update();
        assert_eq!(
            app.world().resource::<crate::Simulation>().elapsed,
            REFERENCE + REFERENCE
        );
        assert_eq!(app.world().resource::<TimeSync>().resources.len(), 3);
    }
}

pub(crate) fn register_lifecycle(
    runtime: Res<crate::resource::ResourceRuntime>,
    mut clock: ResMut<TimeSync>,
) {
    let (send, receive) = std::sync::mpsc::channel();
    let source = SourceTimeSync::new(
        std::sync::Arc::new(move |resources, speed| {
            let _ = send.send((resources, speed));
        }),
        &runtime,
    );
    clock.running = source.state.running.clone();
    clock.lifecycle = Some(source.resource());
    clock.reports = Some(std::sync::Mutex::new(receive));
    clock.source = Some(source);
}
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    #[test]
    fn time_sync_stops_reporting_only_after_deferred_cleanup_runs() {
        let mut app = App::new();
        app.init_resource::<crate::resource::ResourceRuntime>()
            .init_resource::<TimeSync>();
        app.add_systems(Startup, register_lifecycle);
        app.update();
        let runtime = app
            .world()
            .resource::<crate::resource::ResourceRuntime>()
            .clone();
        runtime.close_all();
        assert!(
            app.world()
                .resource::<TimeSync>()
                .running
                .load(std::sync::atomic::Ordering::SeqCst)
        );
        runtime.run_main();
        assert!(
            !app.world()
                .resource::<TimeSync>()
                .running
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    }
}

/// Source TimeSync's lists are replaced when draining; callers retaining an old
/// list must retain that list rather than observing a clear of the current list.
pub(crate) type SampleList = std::sync::Arc<std::sync::Mutex<Vec<f64>>>;
pub(crate) struct ReportState {
    pub running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    resources: std::sync::Mutex<SampleList>,
    speed: std::sync::Mutex<SampleList>,
    pub callback: std::sync::Arc<dyn Fn(f64, f64) + Send + Sync>,
}
impl ReportState {
    fn take_average(list: &std::sync::Mutex<SampleList>) -> f64 {
        let previous = std::mem::replace(&mut *list.lock().unwrap(), SampleList::default());
        let previous = previous.lock().unwrap();
        if previous.is_empty() {
            0.
        } else {
            previous.iter().sum::<f64>() / previous.len() as f64
        }
    }
    pub fn take_resource_average(&self) -> f64 {
        Self::take_average(&self.resources)
    }
    pub fn take_speed_average(&self) -> f64 {
        Self::take_average(&self.speed)
    }
}
pub(crate) trait TimingBackend: Send {
    fn nano_time(&mut self) -> i64;
    fn sleep_millis(&mut self, millis: i64);
}
struct SystemTiming {
    origin: Instant,
}
impl TimingBackend for SystemTiming {
    fn nano_time(&mut self) -> i64 {
        self.origin.elapsed().as_nanos() as i64
    }
    fn sleep_millis(&mut self, millis: i64) {
        std::thread::sleep(Duration::from_millis(millis as u64));
    }
}
pub(crate) struct SourceTimeSync {
    pub state: std::sync::Arc<ReportState>,
    last: std::sync::atomic::AtomicI64,
    backend: std::sync::Mutex<Box<dyn TimingBackend>>,
    lifecycle: crate::resource::ResourceHandle,
}
static REPORT_DISPATCHER: std::sync::OnceLock<
    std::sync::Arc<crate::executor_kt::SingleThreadDispatcher>,
> = std::sync::OnceLock::new();
impl SourceTimeSync {
    pub fn new(
        callback: std::sync::Arc<dyn Fn(f64, f64) + Send + Sync>,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        Self::with_backend(
            callback,
            runtime,
            Box::new(SystemTiming {
                origin: Instant::now(),
            }),
        )
    }
    pub fn with_backend(
        callback: std::sync::Arc<dyn Fn(f64, f64) + Send + Sync>,
        runtime: &crate::resource::ResourceRuntime,
        mut backend: Box<dyn TimingBackend>,
    ) -> Self {
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let cleanup = running.clone();
        let lifecycle = runtime.allocate(&[], move || {
            cleanup.store(false, std::sync::atomic::Ordering::SeqCst)
        });
        let state = std::sync::Arc::new(ReportState {
            running,
            resources: std::sync::Mutex::new(SampleList::default()),
            speed: std::sync::Mutex::new(SampleList::default()),
            callback,
        });
        let dispatcher = REPORT_DISPATCHER
            .get_or_init(|| {
                std::sync::Arc::new(crate::executor_kt::SingleThreadDispatcher::new("TimeSync"))
            })
            .clone();
        crate::time_sync_reporter::launch(state.clone(), dispatcher);
        // Source launches the reporter before initializing last from nanoTime.
        let last = std::sync::atomic::AtomicI64::new(backend.nano_time());
        Self {
            state,
            last,
            backend: std::sync::Mutex::new(backend),
            lifecycle,
        }
    }
    pub fn resources(&self) -> SampleList {
        self.state.resources.lock().unwrap().clone()
    }
    pub fn set_resources(&self, list: SampleList) {
        *self.state.resources.lock().unwrap() = list;
    }
    pub fn speed(&self) -> SampleList {
        self.state.speed.lock().unwrap().clone()
    }
    pub fn set_speed(&self, list: SampleList) {
        *self.state.speed.lock().unwrap() = list;
    }
    pub fn update(&self) -> (f64, f64) {
        let now = self.backend.lock().unwrap().nano_time();
        let previous = self.last.swap(now, std::sync::atomic::Ordering::SeqCst);
        let offset = REFERENCE as f64 - now.wrapping_sub(previous) as f64 / 1.0e9;
        let sleep = ((offset * 1000.) as i64).max(0);
        let resources = (1. - offset / REFERENCE as f64).max(0.);
        let speed = 1. / resources.max(1.);
        self.resources().lock().unwrap().push(resources);
        self.speed().lock().unwrap().push(speed);
        self.backend.lock().unwrap().sleep_millis(sleep);
        (resources, speed)
    }
    pub fn close(&self) {
        self.lifecycle.close();
    }
    pub fn resource(&self) -> crate::resource::ResourceHandle {
        self.lifecycle.clone()
    }
}
#[cfg(test)]
mod source_tests {
    use super::*;
    use std::sync::{Arc, Mutex, atomic::Ordering, mpsc};
    struct Timing {
        now: i64,
        sleeps: Arc<Mutex<Vec<i64>>>,
    }
    impl TimingBackend for Timing {
        fn nano_time(&mut self) -> i64 {
            self.now += 5_000_000;
            self.now
        }
        fn sleep_millis(&mut self, millis: i64) {
            self.sleeps.lock().unwrap().push(millis);
        }
    }
    #[test]
    fn initial_background_report_update_samples_list_identity_and_deferred_stop() {
        let runtime = crate::resource::ResourceRuntime::default();
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let caller = std::thread::current().id();
        let (send, receive) = mpsc::channel();
        let clock = SourceTimeSync::with_backend(
            Arc::new(move |r, s| send.send((r, s, std::thread::current().id())).unwrap()),
            &runtime,
            Box::new(Timing {
                now: 0,
                sleeps: sleeps.clone(),
            }),
        );
        let report = receive.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((report.0, report.1), (0., 0.));
        assert_ne!(report.2, caller);
        let (resources, speed) = clock.update();
        assert_eq!(speed, 1.);
        assert!((resources - 0.005 / REFERENCE as f64).abs() < 1e-12);
        assert_eq!(*sleeps.lock().unwrap(), [11]);
        let previous = clock.resources();
        assert_eq!(previous.lock().unwrap().len(), 1);
        assert_eq!(clock.state.take_resource_average(), resources);
        assert!(!Arc::ptr_eq(&previous, &clock.resources()));
        assert_eq!(previous.lock().unwrap().len(), 1);
        let replacement = Arc::new(Mutex::new(vec![2., 4.]));
        clock.set_resources(replacement.clone());
        assert!(Arc::ptr_eq(&replacement, &clock.resources()));
        assert_eq!(clock.state.take_resource_average(), 3.);
        clock.close();
        assert!(clock.state.running.load(Ordering::SeqCst));
        runtime.run_main();
        assert!(!clock.state.running.load(Ordering::SeqCst));
    }
}

#[cfg(test)]
mod active_source_tests {
    use super::*;
    #[test]
    fn startup_connects_background_reporter_and_source_frame_pacing() {
        let mut app = App::new();
        app.init_resource::<crate::resource::ResourceRuntime>()
            .init_resource::<TimeSync>()
            .init_resource::<crate::Simulation>();
        app.world_mut().spawn(Window::default());
        app.add_systems(Startup, register_lifecycle)
            .add_systems(Last, sync_frame);
        app.update();
        app.update();
        let clock = app.world().resource::<TimeSync>();
        assert!(clock.source.is_some());
        assert!(clock.reports.is_some());
        assert!(clock.resources.is_empty());
        assert!(clock.speed.is_empty());
        assert_eq!(
            app.world().resource::<crate::Simulation>().elapsed,
            REFERENCE + REFERENCE
        );
        let runtime = app
            .world()
            .resource::<crate::resource::ResourceRuntime>()
            .clone();
        runtime.close_all();
        runtime.run_main();
        assert!(
            !app.world()
                .resource::<TimeSync>()
                .running
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    }
}

//! ShipResource imageData/loader state shared by file and derived resources.
use image::RgbaImage;
use std::sync::{Arc, Condvar, Mutex};

type Outcome = Result<Option<RgbaImage>, String>;
type Generator = Arc<dyn Fn() -> Outcome + Send + Sync>;
pub(crate) type SourceImage = Arc<Mutex<crate::image_data::ImageData>>;
type SourceOutcome = Result<Option<SourceImage>, String>;
pub(crate) type SourceGenerator = Arc<dyn Fn() -> SourceOutcome + Send + Sync>;
fn source_generator(generate: Generator) -> SourceGenerator {
    Arc::new(move || generate().map(|image| image.map(retain_image)))
}

fn retain_image(image: RgbaImage) -> SourceImage {
    Arc::new(Mutex::new(crate::image_data::ImageData::new(
        image.as_raw().clone(),
        image.width() as i32,
        image.height() as i32,
        6408,
    )))
}

fn owned_image(image: &SourceImage) -> Result<RgbaImage, String> {
    image
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
        .into_rgba()
}

#[derive(Debug, Default)]
struct LoadTask {
    result: Mutex<Option<SourceOutcome>>,
    ready: Condvar,
}

impl LoadTask {
    fn start(generate: SourceGenerator) -> Arc<Self> {
        let task = Arc::new(Self::default());
        let worker = task.clone();
        std::thread::spawn(move || {
            let loader =
                super::ship_resource_image_data_loader::ImageLoader::new(Some(()), move || {
                    generate()
                });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                loader.invoke_suspend(Ok(()))
            }))
            .unwrap_or_else(|_| Err("image loader panicked".into()));
            *worker
                .result
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = Some(result);
            worker.ready.notify_all();
        });
        task
    }

    fn completed(&self) -> Option<SourceOutcome> {
        self.result
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn await_result(&self) -> SourceOutcome {
        let mut result = self
            .result
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while result.is_none() {
            result = self
                .ready
                .wait(result)
                .unwrap_or_else(|error| error.into_inner());
        }
        result.as_ref().unwrap().clone()
    }
}

#[derive(Debug)]
struct State {
    image: Option<SourceImage>,
    loader: Option<Arc<LoadTask>>,
    last_used_ms: i64,
}

fn current_time_ms() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => time.as_millis() as i64,
        Err(error) => (error.duration().as_millis() as i64).wrapping_neg(),
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            image: None,
            loader: None,
            last_used_ms: current_time_ms(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ImageCache(Arc<Mutex<State>>);

impl State {
    fn poll(&mut self, generate: Generator, clock: impl FnOnce() -> i64) -> SourceOutcome {
        self.poll_native(source_generator(generate), clock)
    }
    fn poll_native(
        &mut self,
        generate: SourceGenerator,
        clock: impl FnOnce() -> i64,
    ) -> SourceOutcome {
        if self.image.is_none() {
            match self.loader.as_ref() {
                None => self.loader = Some(LoadTask::start(generate)),
                Some(loader) => {
                    if let Some(result) = loader.completed() {
                        // An exceptional Deferred remains installed, as getCompleted
                        // throws before the source clears its loader field.
                        self.image = result?;
                        self.loader = None;
                    }
                }
            }
        }
        // Source assigns lastUsed at the end, including pending/null results,
        // but an exceptional completed Deferred bypasses this assignment.
        self.last_used_ms = clock();
        Ok(self.image.clone())
    }
}

impl ImageCache {
    pub(crate) fn source_poll_native(&self, generate: SourceGenerator) -> SourceOutcome {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .poll_native(generate, current_time_ms)
    }
    pub(crate) fn source_poll(&self, generate: Generator) -> SourceOutcome {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .poll(generate, current_time_ms)
    }
    pub(crate) fn poll(&self, generate: Generator) -> Outcome {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .poll(generate, current_time_ms)?
            .as_ref()
            .map(owned_image)
            .transpose()
    }

    pub(crate) fn now(&self, generate: Generator) -> Result<RgbaImage, String> {
        owned_image(&self.source_now(generate)?)
    }

    pub(crate) fn source_now(&self, generate: Generator) -> Result<SourceImage, String> {
        self.source_now_native(source_generator(generate))
    }
    pub(crate) fn source_now_native(
        &self,
        generate: SourceGenerator,
    ) -> Result<SourceImage, String> {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        // The source captures loader before calling getImageData. On a first
        // call this schedules a task but also invokes genImageData directly.
        let previous_loader = state.loader.clone();
        if let Some(image) = state.poll_native(generate.clone(), current_time_ms)? {
            return Ok(image);
        }
        let image = if let Some(loader) = previous_loader {
            let result = super::ship_resource_image_data_now::blocking(|| loader.await_result())?;
            state.loader = None;
            result
        } else {
            generate()?
        };
        if let Some(image) = image {
            state.image = Some(image.clone());
            Ok(image)
        } else {
            Err("could not get image data".into())
        }
    }

    pub(crate) fn release(&self) {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .image = None;
    }

    pub(crate) fn last_used_ms(&self) -> i64 {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .last_used_ms
    }

    /// Non-PNG getImageData still updates lastUsed without scheduling a load.
    pub(crate) fn touch(&self) {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .last_used_ms = current_time_ms();
    }

    pub(crate) fn release_if_idle(&self, now_ms: i64) -> bool {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if super::ship_resource_cleanup_predicate::idle_expired(now_ms, state.last_used_ms) {
            state.image = None;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn native_generator_retains_exact_image_object_and_metadata() {
        let cache = ImageCache::default();
        let source = Arc::new(Mutex::new(crate::image_data::ImageData {
            buffer: vec![1, 2, 3, 4, 5],
            width: -7,
            height: 0,
            image_format: 33319,
            format: 5126,
        }));
        let receiver = source.clone();
        let generate: SourceGenerator = Arc::new(move || Ok(Some(receiver.clone())));
        let image = cache.source_now_native(generate.clone()).unwrap();
        assert!(Arc::ptr_eq(&image, &source));
        assert_eq!(image.lock().unwrap().width, -7);
        let pending = cache.0.lock().unwrap().loader.clone().unwrap();
        assert!(Arc::ptr_eq(
            &pending.await_result().unwrap().unwrap(),
            &source
        ));
        cache.release();
        assert!(Arc::ptr_eq(
            &cache.source_now_native(generate).unwrap(),
            &source
        ));
    }

    #[test]
    fn source_image_identity_mutations_and_reload_match_retained_resource_state() {
        let cache = ImageCache::default();
        let generate: Generator =
            Arc::new(|| Ok(Some(RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 4])))));
        let original = cache.source_now(generate.clone()).unwrap();
        assert!(Arc::ptr_eq(
            &original,
            &cache.clone().source_now(generate.clone()).unwrap()
        ));
        original.lock().unwrap().buffer[0] = 17;
        assert_eq!(
            cache.now(generate.clone()).unwrap().as_raw(),
            &[17, 2, 3, 4]
        );
        let pending = cache.0.lock().unwrap().loader.clone().unwrap();
        let completed = pending.await_result().unwrap().unwrap();
        cache.release();
        let reloaded = cache.source_now(generate).unwrap();
        assert!(Arc::ptr_eq(&completed, &reloaded));
        assert!(!Arc::ptr_eq(&original, &reloaded));
        assert_eq!(original.lock().unwrap().buffer, vec![17, 2, 3, 4]);
        assert_eq!(reloaded.lock().unwrap().buffer, vec![1, 2, 3, 4]);
    }

    #[test]
    fn access_timestamps_include_pending_and_null_but_skip_completed_failures() {
        let cache = ImageCache::default();
        let empty: Generator = Arc::new(|| Ok(None));
        {
            let mut state = cache.0.lock().unwrap();
            state.poll(empty.clone(), || 100).unwrap();
        }
        assert_eq!(cache.clone().last_used_ms(), 100);
        cache.release();
        assert_eq!(cache.last_used_ms(), 100);
        assert!(!cache.release_if_idle(10_100));
        assert!(cache.release_if_idle(10_101));
        cache
            .0
            .lock()
            .unwrap()
            .loader
            .clone()
            .unwrap()
            .await_result()
            .unwrap();
        cache.0.lock().unwrap().poll(empty, || -12).unwrap();
        assert_eq!(cache.last_used_ms(), -12);
        let failure: Generator = Arc::new(|| Err("failed".into()));
        cache
            .0
            .lock()
            .unwrap()
            .poll(failure.clone(), || 500)
            .unwrap();
        assert!(
            cache
                .0
                .lock()
                .unwrap()
                .loader
                .clone()
                .unwrap()
                .await_result()
                .is_err()
        );
        assert!(
            cache
                .0
                .lock()
                .unwrap()
                .poll(failure, || panic!("clock not reached after throw"))
                .is_err()
        );
        assert_eq!(cache.last_used_ms(), 500);
    }

    #[test]
    fn first_now_schedules_and_generates_directly_then_release_keeps_task() {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let generate: Generator = Arc::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(Some(RgbaImage::new(1, 1)))
        });
        let cache = ImageCache::default();
        cache.now(generate.clone()).unwrap();
        let task = cache.0.lock().unwrap().loader.clone().unwrap();
        task.await_result().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        cache.release();
        assert!(cache.poll(generate).unwrap().is_some());
        assert!(cache.0.lock().unwrap().loader.is_none());
    }

    #[test]
    fn completed_null_can_reload_but_exception_remains_in_deferred() {
        let cache = ImageCache::default();
        let empty: Generator = Arc::new(|| Ok(None));
        assert!(cache.poll(empty.clone()).unwrap().is_none());
        cache
            .0
            .lock()
            .unwrap()
            .loader
            .clone()
            .unwrap()
            .await_result()
            .unwrap();
        assert!(cache.poll(empty.clone()).unwrap().is_none());
        assert!(cache.0.lock().unwrap().loader.is_none());
        let failure: Generator = Arc::new(|| Err("decode failed".into()));
        cache.poll(failure.clone()).unwrap();
        assert!(
            cache
                .0
                .lock()
                .unwrap()
                .loader
                .clone()
                .unwrap()
                .await_result()
                .is_err()
        );
        assert_eq!(cache.poll(failure).unwrap_err(), "decode failed");
        assert!(cache.0.lock().unwrap().loader.is_some());
    }
}

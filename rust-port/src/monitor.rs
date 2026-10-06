//! Monitor.java query order, native mode identity and cached mutable size/scale.
#![allow(dead_code)]
use crate::monitor_video_mode::VideoMode;
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug)]
pub(crate) struct NativeVideoMode {
    pub width: i32,
    pub height: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub refresh: i32,
}
pub(crate) trait MonitorBackend: Send {
    fn video_mode(&mut self, ptr: i64) -> Option<Arc<Mutex<NativeVideoMode>>>;
    fn video_modes(&mut self, ptr: i64) -> Option<Vec<Arc<Mutex<NativeVideoMode>>>>;
    fn content_scale(&mut self, ptr: i64) -> [f32; 2];
    fn physical_size(&mut self, ptr: i64) -> [i32; 2];
}
pub(crate) struct Monitor {
    pub ptr: i64,
    pub video_mode: Arc<Mutex<NativeVideoMode>>,
    pub available_video_modes: Vec<VideoMode>,
    pub size: Arc<Mutex<[i32; 2]>>,
    pub scale: Arc<Mutex<[f32; 2]>>,
    backend: Arc<Mutex<dyn MonitorBackend>>,
}
impl Monitor {
    pub fn new(ptr: i64, backend: Arc<Mutex<dyn MonitorBackend>>) -> Result<Self, &'static str> {
        let video_mode = backend
            .lock()
            .unwrap()
            .video_mode(ptr)
            .ok_or("No video mode in monitor")?;
        let modes = backend.lock().unwrap().video_modes(ptr).unwrap_or_default();
        let available_video_modes = modes
            .iter()
            .map(|mode| {
                let mode = mode.lock().unwrap();
                VideoMode::new(
                    Arc::new(Mutex::new([mode.width, mode.height])),
                    mode.r,
                    mode.g,
                    mode.b,
                    mode.refresh,
                )
            })
            .collect();
        let size = {
            let mode = video_mode.lock().unwrap();
            Arc::new(Mutex::new([mode.width, mode.height]))
        };
        let scale = Arc::new(Mutex::new(crate::monitor_scale::invoke(
            &mut *backend.lock().unwrap(),
            ptr,
        )));
        Ok(Self {
            ptr,
            video_mode,
            available_video_modes,
            size,
            scale,
            backend,
        })
    }
    pub fn physical_size(&self) -> [f32; 2] {
        let size = self.backend.lock().unwrap().physical_size(self.ptr);
        [size[0] as f32, size[1] as f32]
    }
}

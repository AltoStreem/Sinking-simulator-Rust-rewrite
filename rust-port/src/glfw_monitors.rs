//! GLFW$monitors$1.class: capacity-indexed monitor enumeration.
#![allow(dead_code)]
use crate::{glfw::GlfwBackend, monitor::Monitor};
use std::sync::{Arc, Mutex};
pub(crate) fn invoke(backend: Arc<Mutex<dyn GlfwBackend>>) -> Result<Vec<Monitor>, &'static str> {
    let pointers = backend
        .lock()
        .unwrap()
        .monitors()
        .ok_or("No monitors available")?;
    let monitor_backend = backend.lock().unwrap().monitor_backend();
    pointers
        .into_iter()
        .map(|ptr| Monitor::new(ptr, monitor_backend.clone()))
        .collect()
}

//! GLFW$monitors$1.class: absolute capacity-indexed monitor enumeration.
#![allow(dead_code)]
use crate::{
    glfw::{GlfwBackend, SourcePointerBuffer},
    monitor::Monitor,
};
use std::sync::{Arc, Mutex};
pub(crate) fn invoke(backend: Arc<Mutex<dyn GlfwBackend>>) -> Result<Vec<Monitor>, &'static str> {
    let pointers: SourcePointerBuffer = backend
        .lock()
        .unwrap()
        .monitor_pointer_buffer()
        .ok_or("No monitors available")?;
    let monitor_backend = backend.lock().unwrap().monitor_backend();
    let mut monitors = Vec::with_capacity(pointers.capacity());
    for index in 0..pointers.capacity() {
        // Java PointerBuffer.get(index) is absolute: position is ignored, limit is checked.
        let pointer = pointers.get_absolute(index)?;
        monitors.push(Monitor::new(pointer, monitor_backend.clone())?);
    }
    Ok(monitors)
}

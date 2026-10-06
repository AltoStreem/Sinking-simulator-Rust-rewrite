//! Monitor$scale$1.class: constructor-time content-scale query.
#![allow(dead_code)]
use crate::monitor::MonitorBackend;
pub(crate) fn invoke(backend: &mut dyn MonitorBackend, monitor: i64) -> [f32; 2] {
    backend.content_scale(monitor)
}

//! FBO.java: binding/error-check/hook ordering and deferred deletion.
#![allow(dead_code)]
use crate::{
    framebuffer_target::FramebufferTarget,
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex};
pub(crate) trait FramebufferBackend: Send {
    fn create_framebuffer(&mut self) -> i32;
    fn bind_framebuffer(&mut self, target: i32, id: i32);
    fn check_error(&mut self, label: &str);
    fn viewport(&mut self) -> [i32; 4];
    fn draw_buffers(&mut self, attachments: &[i32]);
    fn set_viewport(&mut self, viewport: [i32; 4]);
    fn delete_framebuffer(&mut self, id: i32);
}
pub(crate) struct Fbo {
    resource: GlResource,
    pub(crate) backend: Arc<Mutex<dyn FramebufferBackend>>,
}
impl Fbo {
    pub fn new(
        backend: Arc<Mutex<dyn FramebufferBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_framebuffer();
        let cleanup = backend.clone();
        Self {
            resource: GlResource::new(id, context, runtime, move || {
                cleanup.lock().unwrap().delete_framebuffer(id)
            }),
            backend,
        }
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn bind(&self) {
        let mut backend = self.backend.lock().unwrap();
        backend.bind_framebuffer(36160, self.id());
        backend.check_error("FBO Bind");
    }
    pub fn bind_texture(&self, target: &dyn FramebufferTarget, attachment: i32) {
        self.bind();
        target.bind_to_framebuffer(attachment);
        self.unbind();
    }
    pub fn unbind(&self) {
        self.unbind_with(|| {});
    }
    pub fn unbind_with(&self, on_unbind: impl FnOnce()) {
        {
            let mut backend = self.backend.lock().unwrap();
            backend.bind_framebuffer(36160, 0);
            backend.check_error("FBO Unbind");
        }
        on_unbind();
        self.backend.lock().unwrap().check_error("FBO onUnbind");
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}

//! RenderBuffer.java storage, attachment and resource lifetime.
#![allow(dead_code)]
use crate::{
    framebuffer_target::FramebufferTarget,
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex};
pub(crate) trait RenderBufferBackend: Send {
    fn create_renderbuffer(&mut self) -> i32;
    fn bind_renderbuffer(&mut self, target: i32, id: i32);
    fn storage(&mut self, target: i32, format: i32, width: i32, height: i32);
    fn attach_renderbuffer(
        &mut self,
        framebuffer_target: i32,
        attachment: i32,
        renderbuffer_target: i32,
        id: i32,
    );
    fn delete_renderbuffer(&mut self, id: i32);
}
pub(crate) struct RenderBuffer {
    resource: GlResource,
    backend: Arc<Mutex<dyn RenderBufferBackend>>,
    pub width: i32,
    pub height: i32,
    pub format: i32,
}
impl RenderBuffer {
    pub fn new(
        width: i32,
        height: i32,
        format: i32,
        backend: Arc<Mutex<dyn RenderBufferBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_renderbuffer();
        let cleanup = backend.clone();
        let result = Self {
            resource: GlResource::new(id, context, runtime, move || {
                cleanup.lock().unwrap().delete_renderbuffer(id)
            }),
            backend,
            width,
            height,
            format,
        };
        result.bind();
        result
            .backend
            .lock()
            .unwrap()
            .storage(36161, format, width, height);
        result.unbind();
        result
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn bind(&self) {
        self.backend
            .lock()
            .unwrap()
            .bind_renderbuffer(36161, self.id());
    }
    pub fn unbind(&self) {
        self.backend.lock().unwrap().bind_renderbuffer(36161, 0);
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}
impl FramebufferTarget for RenderBuffer {
    fn bind_to_framebuffer(&self, attachment: i32) {
        self.backend
            .lock()
            .unwrap()
            .attach_renderbuffer(36160, attachment, 36161, self.id());
    }
}
impl crate::passes::target_pass::StencilTarget for RenderBuffer {
    fn size(&self) -> [i32; 2] {
        [self.width, self.height]
    }
}

/// Shared native buffer object behind the engine-thread pass interface.
pub(crate) struct SharedStencilTarget(pub Arc<RenderBuffer>);
impl crate::passes::target_pass::StencilTarget for SharedStencilTarget {
    fn size(&self) -> [i32; 2] {
        [self.0.width, self.0.height]
    }
    fn native_target(&self) -> Option<Arc<dyn FramebufferTarget>> {
        Some(self.0.clone())
    }
}

//! VAO.java: attachment deliberately leaves the VBO bound, as in the source.
#![allow(dead_code)]
use crate::{
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
    vbo::Vbo,
};
use std::sync::{Arc, Mutex};
pub(crate) trait VertexArrayBackend: Send {
    fn create_vertex_array(&mut self) -> i32;
    fn bind_vertex_array(&mut self, id: i32);
    fn vertex_attribute_pointer(
        &mut self,
        index: i32,
        components: i32,
        kind: i32,
        normalized: bool,
        stride: i32,
        offset: u64,
    );
    fn delete_vertex_array(&mut self, id: i32);
}
pub(crate) struct Vao {
    resource: GlResource,
    backend: Arc<Mutex<dyn VertexArrayBackend>>,
}
impl Vao {
    pub fn new(
        backend: Arc<Mutex<dyn VertexArrayBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_vertex_array();
        let cleanup = backend.clone();
        Self {
            resource: GlResource::new(id, context, runtime, move || {
                cleanup.lock().unwrap().delete_vertex_array(id)
            }),
            backend,
        }
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn bind(&self) {
        self.backend.lock().unwrap().bind_vertex_array(self.id());
    }
    pub fn unbind(&self) {
        self.backend.lock().unwrap().bind_vertex_array(0);
    }
    pub fn attach_vbo(&self, vbo: &Vbo) {
        self.bind();
        vbo.bind();
        self.unbind();
    }
    pub fn bind_vbo_as_attribute(
        &self,
        vbo: &Vbo,
        index: i32,
        component_size: i32,
        normalized: bool,
    ) {
        self.bind();
        vbo.bind();
        self.backend.lock().unwrap().vertex_attribute_pointer(
            index,
            component_size,
            vbo.buffer_type,
            normalized,
            0,
            0,
        );
        vbo.unbind();
        self.unbind();
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}

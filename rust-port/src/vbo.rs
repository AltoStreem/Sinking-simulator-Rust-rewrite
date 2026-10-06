//! VBO.java typed uploads, capacity metadata, binding and deferred destruction.
#![allow(dead_code)]
use crate::{
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::{
    ops::Range,
    sync::{Arc, Mutex},
};
#[derive(Clone, Copy)]
pub(crate) enum BufferData<'a> {
    Byte(&'a [i8]),
    Short(&'a [i16]),
    Int(&'a [i32]),
    Float(&'a [f32]),
    Double(&'a [f64]),
}
impl BufferData<'_> {
    pub fn len(&self) -> usize {
        match self {
            Self::Byte(v) => v.len(),
            Self::Short(v) => v.len(),
            Self::Int(v) => v.len(),
            Self::Float(v) => v.len(),
            Self::Double(v) => v.len(),
        }
    }
    pub fn gl_type(&self) -> i32 {
        match self {
            Self::Byte(_) => 5120,
            Self::Short(_) => 5122,
            Self::Int(_) => 5124,
            Self::Float(_) => 5126,
            Self::Double(_) => 5130,
        }
    }
    fn window(&self, range: Range<usize>) -> BufferData<'_> {
        match self {
            Self::Byte(v) => BufferData::Byte(&v[range]),
            Self::Short(v) => BufferData::Short(&v[range]),
            Self::Int(v) => BufferData::Int(&v[range]),
            Self::Float(v) => BufferData::Float(&v[range]),
            Self::Double(v) => BufferData::Double(&v[range]),
        }
    }
}
pub(crate) trait BufferBackend: Send {
    fn create_buffer(&mut self) -> i32;
    fn bind_buffer(&mut self, target: i32, id: i32);
    fn upload(&mut self, target: i32, data: BufferData<'_>, mode: i32);
    fn delete_buffer(&mut self, id: i32);
}
pub(crate) struct Vbo {
    resource: GlResource,
    backend: Arc<Mutex<dyn BufferBackend>>,
    pub buffer_type: i32,
    pub size: usize,
    pub target: i32,
    pub mode: i32,
}
impl Vbo {
    pub fn new(
        target: i32,
        buffer: BufferData<'_>,
        remaining: Range<usize>,
        mode: i32,
        backend: Arc<Mutex<dyn BufferBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_buffer();
        let cleanup = backend.clone();
        let resource = GlResource::new(id, context, runtime, move || {
            cleanup.lock().unwrap().delete_buffer(id)
        });
        let result = Self {
            resource,
            backend,
            buffer_type: buffer.gl_type(),
            size: buffer.len(),
            target,
            mode,
        };
        result.bind();
        result
            .backend
            .lock()
            .unwrap()
            .upload(target, buffer.window(remaining), mode);
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
            .bind_buffer(self.target, self.id());
    }
    pub fn unbind(&self) {
        self.backend.lock().unwrap().bind_buffer(self.target, 0);
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}

#[allow(dead_code)]
impl Vbo {
    pub fn from_float_buffer(
        target: i32,
        buffer: &crate::mem_util::NativeBuffer<f32>,
        mode: i32,
        backend: Arc<Mutex<dyn BufferBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        Self::new(
            target,
            BufferData::Float(buffer.data()),
            buffer.remaining_range(),
            mode,
            backend,
            context,
            runtime,
        )
    }
    pub fn from_int_buffer(
        target: i32,
        buffer: &crate::mem_util::NativeBuffer<i32>,
        mode: i32,
        backend: Arc<Mutex<dyn BufferBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        Self::new(
            target,
            BufferData::Int(buffer.data()),
            buffer.remaining_range(),
            mode,
            backend,
            context,
            runtime,
        )
    }
}

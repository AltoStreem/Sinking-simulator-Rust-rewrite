//! ShadedModel.java: program wrapping, validation and program-dependent model lifetime.
#![allow(dead_code)]
use crate::{
    i_drawable::IDrawable, model::SourceModel, resource::ResourceHandle,
    shader_program::ShaderProgram,
};
use std::sync::Arc;
pub(crate) trait ModelProgram: Send + Sync {
    fn start(&self);
    fn stop(&self);
    fn validate(&self, vao: i32);
    fn register_dependent(&self, resource: &ResourceHandle);
}
impl ModelProgram for ShaderProgram {
    fn start(&self) {
        ShaderProgram::start(self)
    }
    fn stop(&self) {
        ShaderProgram::stop(self)
    }
    fn validate(&self, vao: i32) {
        ShaderProgram::validate(self, vao)
    }
    fn register_dependent(&self, resource: &ResourceHandle) {
        ShaderProgram::register_dependent(self, resource)
    }
}
pub(crate) struct ShadedModel {
    pub model: SourceModel,
    pub shader: Arc<dyn ModelProgram>,
}
impl ShadedModel {
    pub fn new(model: SourceModel, shader: Arc<dyn ModelProgram>) -> Self {
        shader.validate(model.vao.id());
        shader.register_dependent(&model.resource_handle());
        Self { model, shader }
    }
    pub fn render_with(&self, shader: &dyn ModelProgram) {
        shader.start();
        self.model.render();
        shader.stop();
    }
    pub fn render_shaderless(&self) {
        self.model.render();
    }
}
impl IDrawable for ShadedModel {
    fn render(&self) {
        self.render_with(self.shader.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ModelBackend,
        resource::ResourceRuntime,
        vao::VertexArrayBackend,
        vbo::{BufferBackend, BufferData, Vbo},
    };
    use std::sync::Mutex;
    type Log = Arc<Mutex<Vec<String>>>;
    struct Backend(Log);
    impl Backend {
        fn log(&self, event: impl Into<String>) {
            self.0.lock().unwrap().push(event.into());
        }
    }
    impl BufferBackend for Backend {
        fn create_buffer(&mut self) -> i32 {
            self.log("create_buffer");
            17
        }
        fn bind_buffer(&mut self, target: i32, id: i32) {
            self.log(format!("buffer:{target}:{id}"));
        }
        fn upload(&mut self, target: i32, data: BufferData<'_>, mode: i32) {
            self.log(format!(
                "upload:{target}:{}:{}:{mode}",
                data.gl_type(),
                data.len()
            ));
        }
        fn delete_buffer(&mut self, id: i32) {
            self.log(format!("delete_buffer:{id}"));
        }
    }
    impl VertexArrayBackend for Backend {
        fn create_vertex_array(&mut self) -> i32 {
            self.log("create_vao");
            21
        }
        fn bind_vertex_array(&mut self, id: i32) {
            self.log(format!("vao:{id}"));
        }
        fn vertex_attribute_pointer(
            &mut self,
            index: i32,
            components: i32,
            kind: i32,
            normalized: bool,
            stride: i32,
            offset: u64,
        ) {
            self.log(format!(
                "pointer:{index}:{components}:{kind}:{normalized}:{stride}:{offset}"
            ));
        }
        fn delete_vertex_array(&mut self, id: i32) {
            self.log(format!("delete_vao:{id}"));
        }
    }
    impl ModelBackend for Backend {
        fn enable_attribute(&mut self, index: i32) {
            self.log(format!("enable:{index}"));
        }
        fn disable_attribute(&mut self, index: i32) {
            self.log(format!("disable:{index}"));
        }
        fn draw_elements(&mut self, style: i32, count: usize, kind: i32, offset: u64) {
            self.log(format!("draw:{style}:{count}:{kind}:{offset}"));
        }
    }
    struct Program {
        log: Log,
        lifetime: ResourceHandle,
    }
    impl ModelProgram for Program {
        fn start(&self) {
            self.log.lock().unwrap().push("start".into());
        }
        fn stop(&self) {
            self.log.lock().unwrap().push("stop".into());
        }
        fn validate(&self, vao: i32) {
            self.log.lock().unwrap().push(format!("validate:{vao}"));
        }
        fn register_dependent(&self, resource: &ResourceHandle) {
            self.lifetime.register_dependent(resource);
            self.log.lock().unwrap().push("dependent".into());
        }
    }
    #[test]
    fn model_array_constructor_uses_source_wrappers_and_full_capacity_uploads() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let model = SourceModel::from_arrays(
            &[0, 1, 0],
            &[1., 2., 3., 4.],
            2,
            4,
            backend.clone(),
            backend.clone(),
            backend,
            context,
            &runtime,
        );
        assert_eq!(model.vertices.size, 4);
        assert_eq!(model.indices.size, 3);
        assert!(
            log.lock()
                .unwrap()
                .contains(&"upload:34962:5126:4:35044".into())
        );
        assert!(
            log.lock()
                .unwrap()
                .contains(&"upload:34963:5124:3:35044".into())
        );
        log.lock().unwrap().clear();
        model.render();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "vao:21",
                "enable:0",
                "draw:4:3:5125:0",
                "disable:0",
                "vao:0"
            ]
        );
    }
    fn make_model(runtime: &ResourceRuntime, context: ResourceHandle, log: Log) -> SourceModel {
        let backend = Arc::new(Mutex::new(Backend(log)));
        let vertices = Arc::new(Vbo::new(
            34962,
            BufferData::Float(&[1., 2., 3., 4.]),
            0..4,
            35044,
            backend.clone(),
            context.clone(),
            runtime,
        ));
        // Upload only remaining elements, while draw count remains the capacity (three).
        let indices = Arc::new(Vbo::new(
            34963,
            BufferData::Int(&[0, 1, 0]),
            1..3,
            35044,
            backend.clone(),
            context.clone(),
            runtime,
        ));
        SourceModel::new(
            vertices,
            indices,
            4,
            2,
            backend.clone(),
            backend,
            context,
            runtime,
        )
    }
    #[test]
    fn vertex_array_setup_and_shaded_render_match_source_call_order() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let model = make_model(&runtime, context, log.clone());
        assert_eq!(
            &log.lock().unwrap()[8..],
            [
                "create_vao",
                "vao:21",
                "buffer:34963:17",
                "vao:0",
                "vao:21",
                "buffer:34962:17",
                "pointer:0:2:5126:false:0:0",
                "buffer:34962:0",
                "vao:0"
            ]
        );
        assert!(
            log.lock()
                .unwrap()
                .contains(&"upload:34963:5124:2:35044".into())
        );
        let program = Arc::new(Program {
            log: log.clone(),
            lifetime: runtime.allocate(&[], || {}),
        });
        let shaded = ShadedModel::new(model, program.clone());
        assert_eq!(&log.lock().unwrap()[17..], ["validate:21", "dependent"]);
        log.lock().unwrap().clear();
        shaded.render();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "start",
                "vao:21",
                "enable:0",
                "draw:4:3:5125:0",
                "disable:0",
                "vao:0",
                "stop"
            ]
        );
        log.lock().unwrap().clear();
        shaded.render_shaderless();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "vao:21",
                "enable:0",
                "draw:4:3:5125:0",
                "disable:0",
                "vao:0"
            ]
        );
        program.lifetime.close();
        assert!(shaded.model.freed());
        assert!(!shaded.model.vao.freed());
        assert!(!shaded.model.vertices.freed());
    }
    #[test]
    fn every_buffer_type_preserves_source_type_capacity_window_and_deferred_free() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let bytes = [1i8, 2, 3];
        let shorts = [1i16, 2, 3];
        let ints = [1i32, 2, 3];
        let floats = [1f32, 2., 3.];
        let doubles = [1f64, 2., 3.];
        for (data, kind) in [
            (BufferData::Byte(&bytes), 5120),
            (BufferData::Short(&shorts), 5122),
            (BufferData::Int(&ints), 5124),
            (BufferData::Float(&floats), 5126),
            (BufferData::Double(&doubles), 5130),
        ] {
            log.lock().unwrap().clear();
            let buffer = Vbo::new(
                34962,
                data,
                1..2,
                35044,
                backend.clone(),
                context.clone(),
                &runtime,
            );
            assert_eq!(buffer.size, 3);
            assert_eq!(buffer.buffer_type, kind);
            assert_eq!(
                *log.lock().unwrap(),
                vec![
                    "create_buffer".to_owned(),
                    "buffer:34962:17".into(),
                    format!("upload:34962:{kind}:1:35044"),
                    "buffer:34962:0".into()
                ]
            );
            log.lock().unwrap().clear();
            buffer.close();
            assert!(buffer.freed());
            assert!(log.lock().unwrap().is_empty());
            runtime.run_main();
            assert_eq!(*log.lock().unwrap(), ["delete_buffer:17"]);
        }
    }
}

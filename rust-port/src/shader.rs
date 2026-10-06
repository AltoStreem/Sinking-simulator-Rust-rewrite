//! Shader.java compilation/attachment contract and deferred GLResource destruction.
#![allow(dead_code)]
use crate::{
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex};
pub(crate) const POSITION: &str = "Position";
pub(crate) const TEXCOORD: &str = "TexCoord";
pub(crate) const V_TEXCOORD: &str = "vTexCoord";
pub(crate) trait ShaderBackend: Send {
    fn create_shader(&mut self, shader_type: i32) -> i32;
    fn shader_source(&mut self, id: i32, source: &str);
    fn compile_shader(&mut self, id: i32);
    fn shader_info_log(&mut self, id: i32) -> String;
    fn shader_compiled(&mut self, id: i32) -> bool;
    fn attach_shader(&mut self, program: i32, shader: i32);
    fn detach_shader(&mut self, program: i32, shader: i32);
    fn delete_shader(&mut self, id: i32);
}
pub(crate) struct Shader {
    resource: GlResource,
    pub shader_type: i32,
    pub bindings: Vec<String>,
    backend: Arc<Mutex<dyn ShaderBackend>>,
}
impl Shader {
    pub fn new(
        source: &str,
        shader_type: i32,
        bindings: Vec<String>,
        backend: Arc<Mutex<dyn ShaderBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Result<Self, String> {
        let id = backend.lock().unwrap().create_shader(shader_type);
        let cleanup = backend.clone();
        let resource = GlResource::new(id, context, runtime, move || {
            cleanup.lock().unwrap().delete_shader(id)
        });
        let result = Self {
            resource,
            shader_type,
            bindings,
            backend,
        };
        let mut backend = result.backend.lock().unwrap();
        backend.shader_source(id, source);
        backend.compile_shader(id);
        let log = backend.shader_info_log(id);
        let log = log.trim();
        if !log.is_empty() {
            println!("{log}");
        }
        if !backend.shader_compiled(id) {
            println!("{source}");
            return Err("Failed to create Shader".into());
        }
        drop(backend);
        Ok(result)
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn attach(&self, program: i32) {
        self.backend
            .lock()
            .unwrap()
            .attach_shader(program, self.id());
    }
    pub fn detach(&self, program: i32) {
        self.backend
            .lock()
            .unwrap()
            .detach_shader(program, self.id());
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
    pub fn file_shader_type(file_name: &str) -> i32 {
        if file_name.ends_with(".frag") {
            35632
        } else if file_name.ends_with(".vert") {
            35633
        } else if file_name.ends_with(".geom") {
            36313
        } else if file_name.ends_with(".comp") {
            37305
        } else {
            0
        }
    }
}
impl std::fmt::Display for Shader {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Shader{{id={}, type={}}}",
            self.id(),
            self.shader_type
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Backend {
        events: Arc<Mutex<Vec<String>>>,
        success: bool,
    }
    impl ShaderBackend for Backend {
        fn create_shader(&mut self, kind: i32) -> i32 {
            self.events.lock().unwrap().push(format!("create:{kind}"));
            7
        }
        fn shader_source(&mut self, id: i32, source: &str) {
            self.events
                .lock()
                .unwrap()
                .push(format!("source:{id}:{source}"));
        }
        fn compile_shader(&mut self, id: i32) {
            self.events.lock().unwrap().push(format!("compile:{id}"));
        }
        fn shader_info_log(&mut self, _: i32) -> String {
            " \n ".into()
        }
        fn shader_compiled(&mut self, _: i32) -> bool {
            self.success
        }
        fn attach_shader(&mut self, program: i32, shader: i32) {
            self.events
                .lock()
                .unwrap()
                .push(format!("attach:{program}:{shader}"));
        }
        fn detach_shader(&mut self, program: i32, shader: i32) {
            self.events
                .lock()
                .unwrap()
                .push(format!("detach:{program}:{shader}"));
        }
        fn delete_shader(&mut self, id: i32) {
            self.events.lock().unwrap().push(format!("delete:{id}"));
        }
    }
    #[test]
    fn shader_lifecycle_defers_cleanup_and_preserves_compilation_attachment_order() {
        let runtime = ResourceRuntime::default();
        let events = Arc::new(Mutex::new(Vec::new()));
        let context = runtime.allocate(&[], || {});
        let backend = Arc::new(Mutex::new(Backend {
            events: events.clone(),
            success: true,
        }));
        let shader = Shader::new(
            "source",
            35632,
            vec!["color".into()],
            backend,
            context.clone(),
            &runtime,
        )
        .unwrap();
        shader.attach(9);
        shader.detach(9);
        assert_eq!(
            &*events.lock().unwrap(),
            &[
                "create:35632",
                "source:7:source",
                "compile:7",
                "attach:9:7",
                "detach:9:7"
            ]
        );
        assert_eq!(shader.to_string(), "Shader{id=7, type=35632}");
        context.close();
        assert!(shader.freed());
        assert_eq!(events.lock().unwrap().len(), 5);
        runtime.run_main();
        assert_eq!(events.lock().unwrap().last().unwrap(), "delete:7");
    }
    #[test]
    fn failed_compilation_returns_source_error_and_defers_cleanup() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = Arc::new(Mutex::new(Vec::new()));
        let backend = Arc::new(Mutex::new(Backend {
            events: events.clone(),
            success: false,
        }));
        assert_eq!(
            Shader::new("bad", 35632, vec![], backend, context, &runtime)
                .err()
                .unwrap(),
            "Failed to create Shader"
        );
        assert_eq!(events.lock().unwrap().len(), 3);
        runtime.run_main();
        assert_eq!(events.lock().unwrap().last().unwrap(), "delete:7");
    }
    #[test]
    fn source_file_extensions_are_case_sensitive() {
        for (name, expected) in [
            ("x.frag", 35632),
            ("x.vert", 35633),
            ("x.geom", 36313),
            ("x.comp", 37305),
            ("x.FRAG", 0),
            ("x.wgsl", 0),
        ] {
            assert_eq!(Shader::file_shader_type(name), expected);
        }
    }
}

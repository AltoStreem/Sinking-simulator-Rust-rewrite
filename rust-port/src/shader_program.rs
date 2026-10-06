//! ShaderProgram.java linking, uniforms, reflection and deferred program disposal.
#![allow(dead_code)]
use crate::{
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
    shader::Shader,
};
use std::sync::{Arc, Mutex};
pub(crate) trait ProgramBackend: Send {
    fn create_program(&mut self) -> i32;
    fn bind_attribute(&mut self, id: i32, index: i32, name: &str);
    fn bind_fragment_output(&mut self, id: i32, index: i32, name: &str);
    fn link_program(&mut self, id: i32);
    fn program_info_log(&mut self, id: i32) -> String;
    fn use_program(&mut self, id: i32);
    fn uniform_location(&mut self, id: i32, name: &str) -> i32;
    fn attribute_location(&mut self, id: i32, name: &str) -> i32;
    fn active_attribute_count(&mut self, id: i32) -> i32;
    fn active_attribute(&mut self, id: i32, index: i32, max_bytes: usize) -> String;
    fn float_uniform(&mut self, location: i32, values: &[f32]);
    fn int_uniform(&mut self, location: i32, values: &[i32]);
    fn matrix_uniform(&mut self, location: i32, transposed: bool, values: &[f32; 16]);
    fn bind_vertex_array(&mut self, vao: i32);
    fn validate_program(&mut self, id: i32);
    fn delete_program(&mut self, id: i32);
}
pub(crate) struct ShaderProgram {
    resource: GlResource,
    pub shaders: Vec<Arc<Shader>>,
    backend: Arc<Mutex<dyn ProgramBackend>>,
}
impl ShaderProgram {
    pub fn new(
        shaders: Vec<Arc<Shader>>,
        backend: Arc<Mutex<dyn ProgramBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_program();
        let cleanup_backend = backend.clone();
        let cleanup_shaders = shaders.clone();
        let resource = GlResource::new(id, context, runtime, move || {
            for shader in &cleanup_shaders {
                shader.detach(id);
            }
            cleanup_backend.lock().unwrap().delete_program(id);
        });
        let result = Self {
            resource,
            shaders,
            backend,
        };
        let first = result
            .shaders
            .first()
            .expect("ShaderProgram needs at least one shader");
        let last = result.shaders.last().unwrap();
        {
            let mut backend = result.backend.lock().unwrap();
            for (index, name) in first.bindings.iter().enumerate() {
                backend.bind_attribute(id, index as i32, name);
            }
            for (index, name) in last.bindings.iter().enumerate() {
                backend.bind_fragment_output(id, index as i32, name);
            }
        }
        for shader in &result.shaders {
            shader.attach(id);
        }
        let mut backend = result.backend.lock().unwrap();
        backend.link_program(id);
        print_log(backend.program_info_log(id));
        drop(backend);
        // Java prints link logs without a link-status check.
        result
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn register_dependent(&self, dependent: &ResourceHandle) {
        self.resource.register_dependent(dependent);
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
    pub fn start(&self) {
        self.backend.lock().unwrap().use_program(self.id());
    }
    pub fn stop(&self) {
        self.backend.lock().unwrap().use_program(0);
    }
    pub fn uniform_location(&self, name: &str) -> i32 {
        self.backend
            .lock()
            .unwrap()
            .uniform_location(self.id(), name)
    }
    pub fn attribute_location(&self, name: &str) -> i32 {
        self.backend
            .lock()
            .unwrap()
            .attribute_location(self.id(), name)
    }
    pub fn attributes(&self) -> Vec<String> {
        let mut backend = self.backend.lock().unwrap();
        let count = backend.active_attribute_count(self.id());
        // Original Kotlin IntRange is inclusive, including index == count.
        (0..=count)
            .map(|index| backend.active_attribute(self.id(), index, 50))
            .collect()
    }
    pub fn set_floats(&self, location: i32, values: &[f32]) -> Result<(), String> {
        let mut backend = self.backend.lock().unwrap();
        backend.use_program(self.id());
        if !(1..=4).contains(&values.len()) {
            return Err(format!(
                "Maximum 4 vector components, {} given.",
                values.len()
            ));
        }
        backend.float_uniform(location, values);
        backend.use_program(0);
        Ok(())
    }
    pub fn set_ints(&self, location: i32, values: &[i32]) -> Result<(), String> {
        let mut backend = self.backend.lock().unwrap();
        backend.use_program(self.id());
        if !(1..=4).contains(&values.len()) {
            return Err(format!(
                "Maximum 4 vector components, {} given.",
                values.len()
            ));
        }
        backend.int_uniform(location, values);
        backend.use_program(0);
        Ok(())
    }
    pub fn set_matrix(&self, location: i32, transposed: bool, values: &[f32; 16]) {
        let mut backend = self.backend.lock().unwrap();
        backend.use_program(self.id());
        backend.matrix_uniform(location, transposed, values);
        backend.use_program(0);
    }
    pub fn validate(&self, vao: i32) {
        let mut backend = self.backend.lock().unwrap();
        backend.bind_vertex_array(vao);
        backend.validate_program(self.id());
        print_log(backend.program_info_log(self.id()));
        backend.bind_vertex_array(0);
    }
}
fn print_log(log: String) {
    let log = log.trim();
    if !log.is_empty() {
        println!("{log}");
    }
}
impl std::fmt::Display for ShaderProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "ShaderProgram{{id={}, shaders=[", self.id())?;
        for (index, shader) in self.shaders.iter().enumerate() {
            if index > 0 {
                write!(formatter, ", ")?;
            }
            write!(formatter, "{shader}")?;
        }
        write!(formatter, "]}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::ShaderBackend;
    type Events = Arc<Mutex<Vec<String>>>;
    struct Backend(Events);
    impl ShaderBackend for Backend {
        fn create_shader(&mut self, kind: i32) -> i32 {
            kind
        }
        fn shader_source(&mut self, _: i32, _: &str) {}
        fn compile_shader(&mut self, _: i32) {}
        fn shader_info_log(&mut self, _: i32) -> String {
            String::new()
        }
        fn shader_compiled(&mut self, _: i32) -> bool {
            true
        }
        fn attach_shader(&mut self, program: i32, shader: i32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("attach:{program}:{shader}"));
        }
        fn detach_shader(&mut self, program: i32, shader: i32) {
            self.0
                .lock()
                .unwrap()
                .push(format!("detach:{program}:{shader}"));
        }
        fn delete_shader(&mut self, id: i32) {
            self.0.lock().unwrap().push(format!("delete shader:{id}"));
        }
    }
    impl ProgramBackend for Backend {
        fn create_program(&mut self) -> i32 {
            self.0.lock().unwrap().push("create program".into());
            9
        }
        fn bind_attribute(&mut self, _: i32, index: i32, name: &str) {
            self.0
                .lock()
                .unwrap()
                .push(format!("attribute:{index}:{name}"));
        }
        fn bind_fragment_output(&mut self, _: i32, index: i32, name: &str) {
            self.0
                .lock()
                .unwrap()
                .push(format!("output:{index}:{name}"));
        }
        fn link_program(&mut self, _: i32) {
            self.0.lock().unwrap().push("link".into());
        }
        fn program_info_log(&mut self, _: i32) -> String {
            String::new()
        }
        fn use_program(&mut self, id: i32) {
            self.0.lock().unwrap().push(format!("use:{id}"));
        }
        fn uniform_location(&mut self, _: i32, _: &str) -> i32 {
            -1
        }
        fn attribute_location(&mut self, _: i32, _: &str) -> i32 {
            2
        }
        fn active_attribute_count(&mut self, _: i32) -> i32 {
            1
        }
        fn active_attribute(&mut self, _: i32, index: i32, max_bytes: usize) -> String {
            self.0
                .lock()
                .unwrap()
                .push(format!("active:{index}:{max_bytes}"));
            format!("name{index}")
        }
        fn float_uniform(&mut self, location: i32, values: &[f32]) {
            self.0
                .lock()
                .unwrap()
                .push(format!("float:{location}:{values:?}"));
        }
        fn int_uniform(&mut self, location: i32, values: &[i32]) {
            self.0
                .lock()
                .unwrap()
                .push(format!("int:{location}:{values:?}"));
        }
        fn matrix_uniform(&mut self, location: i32, transpose: bool, values: &[f32; 16]) {
            self.0
                .lock()
                .unwrap()
                .push(format!("matrix:{location}:{transpose}:{}", values[15]));
        }
        fn bind_vertex_array(&mut self, id: i32) {
            self.0.lock().unwrap().push(format!("vao:{id}"));
        }
        fn validate_program(&mut self, _: i32) {
            self.0.lock().unwrap().push("validate".into());
        }
        fn delete_program(&mut self, id: i32) {
            self.0.lock().unwrap().push(format!("delete program:{id}"));
        }
    }
    fn program(
        runtime: &ResourceRuntime,
        context: ResourceHandle,
        events: &Events,
    ) -> ShaderProgram {
        let vertex = Shader::new(
            "vertex",
            35633,
            vec!["Position".into()],
            Arc::new(Mutex::new(Backend(events.clone()))),
            context.clone(),
            runtime,
        )
        .unwrap();
        let fragment = Shader::new(
            "fragment",
            35632,
            vec!["color".into()],
            Arc::new(Mutex::new(Backend(events.clone()))),
            context.clone(),
            runtime,
        )
        .unwrap();
        ShaderProgram::new(
            vec![Arc::new(vertex), Arc::new(fragment)],
            Arc::new(Mutex::new(Backend(events.clone()))),
            context,
            runtime,
        )
    }
    #[test]
    fn source_program_binds_first_last_shader_before_attach_link_and_defers_detach_delete() {
        let runtime = ResourceRuntime::default();
        let events = Events::default();
        let context = runtime.allocate(&[], || {});
        let program = program(&runtime, context, &events);
        assert_eq!(
            &*events.lock().unwrap(),
            &[
                "create program",
                "attribute:0:Position",
                "output:0:color",
                "attach:9:35633",
                "attach:9:35632",
                "link"
            ]
        );
        program.close();
        assert!(program.freed());
        assert_eq!(events.lock().unwrap().len(), 6);
        runtime.run_main();
        assert_eq!(
            &events.lock().unwrap()[6..],
            &["detach:9:35633", "detach:9:35632", "delete program:9"]
        );
    }
    #[test]
    fn uniforms_reflection_and_validation_preserve_source_calls() {
        let runtime = ResourceRuntime::default();
        let events = Events::default();
        let context = runtime.allocate(&[], || {});
        let program = program(&runtime, context, &events);
        events.lock().unwrap().clear();
        program.set_floats(-1, &[1.0, 2.0]).unwrap();
        program.set_ints(2, &[3]).unwrap();
        program.set_matrix(4, true, &[5.0; 16]);
        assert_eq!(program.attributes(), ["name0", "name1"]);
        program.validate(17);
        assert_eq!(
            &*events.lock().unwrap(),
            &[
                "use:9",
                "float:-1:[1.0, 2.0]",
                "use:0",
                "use:9",
                "int:2:[3]",
                "use:0",
                "use:9",
                "matrix:4:true:5",
                "use:0",
                "active:0:50",
                "active:1:50",
                "vao:17",
                "validate",
                "vao:0"
            ]
        );
        events.lock().unwrap().clear();
        assert!(program.set_floats(0, &[]).is_err());
        assert_eq!(&*events.lock().unwrap(), &["use:9"]);
        program.close();
        runtime.run_main();
    }
}

//! Retained Ship.java static resources, in original initializer order.
#![allow(dead_code)]
use std::sync::{Arc, Mutex};
pub(crate) struct SourceShipShaders {
    pub black_texture: Arc<crate::texture_2d::SourceTexture2D>,
    pub geometry: Arc<crate::shader::Shader>,
    pub fragment: Arc<crate::shader::Shader>,
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend(Arc<Mutex<Vec<String>>>);
    impl crate::shader::ShaderBackend for Backend {
        fn create_shader(&mut self, kind: i32) -> i32 {
            self.0.lock().unwrap().push(format!("shader:{kind}"));
            kind
        }
        fn shader_source(&mut self, id: i32, source: &str) {
            match id {
                36313 => assert_eq!(source, crate::ship_geometry_shader::SOURCE),
                35632 => assert_eq!(source, crate::ship_fragment_shader::SOURCE),
                35633 => assert_eq!(source, crate::vertex_shaders_none::SOURCE),
                _ => panic!("unexpected shader kind"),
            }
        }
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
            self.0.lock().unwrap().push(format!("delete-shader:{id}"));
        }
    }
    #[test]
    fn ship_static_shader_literals_match_original_java() {
        let original =
            std::fs::read_to_string("../SS2/decompiled/com/wicpar/sinkingsimulator/ship/Ship.java")
                .unwrap();
        for (source, kind) in [
            (crate::ship_geometry_shader::SOURCE, 36313),
            (crate::ship_fragment_shader::SOURCE, 35632),
        ] {
            assert!(original.contains(&format!("new Shader({source:?}, {kind}")));
        }
    }
    #[test]
    fn ship_static_resources_initialize_in_order_and_are_shared_by_new_programs() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let textures = crate::render_fbo::source_tests::backend(log.clone());
        let shaders = Arc::new(Mutex::new(Backend(log.clone())));
        let check_log = log.clone();
        let resources = SourceShipShaders::new(
            textures,
            shaders.clone(),
            context.clone(),
            &runtime,
            move |message| check_log.lock().unwrap().push(format!("check:{message}")),
        )
        .unwrap();
        let setup = log.lock().unwrap().clone();
        let upload = setup
            .iter()
            .position(|e| e == "image2:1:3553:32856:[1, 1]:6408:5121:false")
            .unwrap();
        let check = setup
            .iter()
            .position(|e| e == "check:Black Texture")
            .unwrap();
        let geometry = setup.iter().position(|e| e == "shader:36313").unwrap();
        let fragment = setup.iter().position(|e| e == "shader:35632").unwrap();
        assert!(upload < check && check < geometry && geometry < fragment);
        assert!(!setup.iter().any(|e| e.starts_with("mips:")));
        let configuration: Vec<_> = setup
            .iter()
            .filter(|e| e.starts_with("parameter:"))
            .cloned()
            .collect();
        assert_eq!(
            configuration,
            [
                "parameter:3553:10240:9728",
                "parameter:3553:10241:9728",
                "parameter:3553:10242:10497",
                "parameter:3553:10243:10497"
            ]
        );
        assert_eq!(
            *resources
                .black_texture
                .img
                .as_ref()
                .unwrap()
                .lock()
                .unwrap(),
            [0, 0, 0, 0]
        );
        assert!(resources.geometry.bindings.is_empty() && resources.fragment.bindings.is_empty());
        let vertices =
            crate::vertex_shaders::SourceVertexShaders::new(shaders, context.clone(), &runtime);
        let vertex = vertices.none().unwrap();
        let programs = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            log.clone(),
        )));
        let construct = || {
            Arc::new(crate::shader_program::ShaderProgram::new(
                vec![
                    vertex.clone(),
                    resources.geometry.clone(),
                    resources.fragment.clone(),
                ],
                programs.clone(),
                context.clone(),
                &runtime,
            ))
        };
        let first = construct();
        let second = construct();
        assert!(!Arc::ptr_eq(&first, &second));
        for index in 0..3 {
            assert!(Arc::ptr_eq(&first.shaders[index], &second.shaders[index]));
        }
        assert_eq!(
            first
                .shaders
                .iter()
                .map(|shader| shader.shader_type)
                .collect::<Vec<_>>(),
            [35633, 36313, 35632]
        );
        context.close();
        runtime.run_main();
        assert!(log.lock().unwrap().contains(&"delete-shader:36313".into()));
    }
}
impl SourceShipShaders {
    pub fn new(
        textures: Arc<Mutex<dyn crate::texture::TextureBackend>>,
        shaders: Arc<Mutex<dyn crate::shader::ShaderBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        check_error: impl FnOnce(&str),
    ) -> Result<Self, String> {
        let black_texture = Arc::new(crate::ship_black_texture::create(
            textures,
            context.clone(),
            runtime,
        ));
        check_error("Black Texture");
        let geometry = Arc::new(crate::shader::Shader::new(
            crate::ship_geometry_shader::SOURCE,
            36313,
            vec![],
            shaders.clone(),
            context.clone(),
            runtime,
        )?);
        let fragment = Arc::new(crate::shader::Shader::new(
            crate::ship_fragment_shader::SOURCE,
            35632,
            vec![],
            shaders,
            context,
            runtime,
        )?);
        Ok(Self {
            black_texture,
            geometry,
            fragment,
        })
    }
}

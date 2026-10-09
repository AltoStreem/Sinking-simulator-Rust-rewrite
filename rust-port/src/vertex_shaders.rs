//! Arithmetic translation of VertexShaders' three recovered shader bodies.
//! Bevy's Mesh2d shader supplies GPU position/UV attributes; these operations
//! also provide the original transform contract to CPU-created pass geometry.
use bevy::prelude::{Mat4, Vec2, Vec3, Vec4};
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VertexOutput {
    pub position: Vec4,
    pub uv: Vec2,
}
pub(crate) struct VertexShaders;
/// Source Kotlin synchronized lazy delegates. A failed initializer leaves the
/// slot empty, allowing the next getter to retry instead of caching an error.
#[allow(dead_code)]
pub(crate) struct SourceVertexShaders {
    backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: crate::resource::ResourceRuntime,
    slots: [std::sync::Mutex<Option<std::sync::Arc<crate::shader::Shader>>>; 3],
}
#[allow(dead_code)]
impl SourceVertexShaders {
    pub fn new(
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        Self {
            backend,
            context,
            runtime: runtime.clone(),
            slots: std::array::from_fn(|_| std::sync::Mutex::new(None)),
        }
    }
    fn get(
        &self,
        index: usize,
        source: &str,
        bindings: &[&str],
    ) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        let mut slot = self.slots[index].lock().unwrap();
        if let Some(shader) = &*slot {
            return Ok(shader.clone());
        }
        let shader = std::sync::Arc::new(crate::shader::Shader::new(
            source,
            35633,
            bindings.iter().map(|name| (*name).to_owned()).collect(),
            self.backend.clone(),
            self.context.clone(),
            &self.runtime,
        )?);
        *slot = Some(shader.clone());
        Ok(shader)
    }
    pub fn none(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(
            0,
            crate::vertex_shaders_none::SOURCE,
            crate::vertex_shaders_none::BINDINGS,
        )
    }
    pub fn nothing(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(
            1,
            crate::vertex_shaders_nothing::SOURCE,
            crate::vertex_shaders_nothing::BINDINGS,
        )
    }
    pub fn transform(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(
            2,
            crate::vertex_shaders_transform::SOURCE,
            crate::vertex_shaders_transform::BINDINGS,
        )
    }
}
impl VertexShaders {
    pub fn none(position: Vec3, uv: Vec2) -> VertexOutput {
        VertexOutput {
            position: position.extend(1.0),
            uv,
        }
    }
    pub fn nothing(position: Vec3) -> Vec4 {
        position.extend(1.0)
    }
    pub fn transform(position: Vec3, uv: Vec2, transform: Mat4) -> VertexOutput {
        VertexOutput {
            position: transform * position.extend(1.0),
            uv,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[derive(Default)]
    struct Compiler {
        sources: Vec<String>,
        fail: bool,
    }
    impl crate::shader::ShaderBackend for Compiler {
        fn create_shader(&mut self, kind: i32) -> i32 {
            assert_eq!(kind, 35633);
            self.sources.len() as i32 + 1
        }
        fn shader_source(&mut self, _: i32, source: &str) {
            self.sources.push(source.to_owned());
        }
        fn compile_shader(&mut self, _: i32) {}
        fn shader_info_log(&mut self, _: i32) -> String {
            String::new()
        }
        fn shader_compiled(&mut self, _: i32) -> bool {
            !self.fail
        }
        fn attach_shader(&mut self, _: i32, _: i32) {}
        fn detach_shader(&mut self, _: i32, _: i32) {}
        fn delete_shader(&mut self, _: i32) {}
    }
    #[test]
    fn source_lazy_delegates_compile_exact_bodies_once_and_retry_failure() {
        let runtime = crate::resource::ResourceRuntime::default();
        let backend = Arc::new(Mutex::new(Compiler::default()));
        let shaders =
            SourceVertexShaders::new(backend.clone(), runtime.allocate(&[], || {}), &runtime);
        assert!(backend.lock().unwrap().sources.is_empty());
        backend.lock().unwrap().fail = true;
        assert!(shaders.none().is_err());
        backend.lock().unwrap().fail = false;
        let none = shaders.none().unwrap();
        assert!(Arc::ptr_eq(&none, &shaders.none().unwrap()));
        let nothing = shaders.nothing().unwrap();
        let transform = shaders.transform().unwrap();
        assert!(Arc::ptr_eq(&nothing, &shaders.nothing().unwrap()));
        assert!(Arc::ptr_eq(&transform, &shaders.transform().unwrap()));
        assert!(!Arc::ptr_eq(&none, &transform));
        assert_eq!(none.bindings, ["Position", "TexCoord"]);
        assert_eq!(nothing.bindings, ["Position"]);
        assert_eq!(transform.bindings, ["Position", "TexCoord"]);
        assert_eq!(
            backend.lock().unwrap().sources,
            [
                crate::vertex_shaders_none::SOURCE,
                crate::vertex_shaders_none::SOURCE,
                crate::vertex_shaders_nothing::SOURCE,
                crate::vertex_shaders_transform::SOURCE
            ]
        );
    }
    #[test]
    fn source_vertex_shaders_preserve_uvs_and_homogeneous_transform() {
        let p = Vec3::new(2.0, 3.0, 4.0);
        let uv = Vec2::new(0.25, 0.75);
        assert_eq!(
            VertexShaders::none(p, uv),
            VertexOutput {
                position: p.extend(1.0),
                uv
            }
        );
        assert_eq!(VertexShaders::nothing(p), p.extend(1.0));
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 3.0, 1.0),
            bevy::prelude::Quat::IDENTITY,
            Vec3::new(10.0, -4.0, 0.0),
        );
        assert_eq!(
            VertexShaders::transform(p, uv, transform),
            VertexOutput {
                position: Vec4::new(14.0, 5.0, 4.0, 1.0),
                uv
            }
        );
    }
}

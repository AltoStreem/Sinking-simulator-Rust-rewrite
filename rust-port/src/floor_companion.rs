//! Recovered Floor$Companion.class retained shader getter.
/// The original companion owns one compiled fragment shader. Retain this owner
/// alongside the source shader registry so constructing another Floor reuses
/// the shader while linking a new program.
#[allow(dead_code)]
pub(crate) struct FloorShader {
    shader: std::sync::Arc<crate::shader::Shader>,
}
#[allow(dead_code)]
impl FloorShader {
    pub const SOURCE: &'static str = "\n    #version 150 core\n\n    in vec2 vTexCoord;\n    out vec4 FragColor;\n\n    uniform mat4 inv;\n    uniform float floorHeight;\n\n    void main() {\n        vec2 screenCoord = vTexCoord * 2 - 1;\n        vec4 worldCoord = inv * vec4(screenCoord, 0, 1);\n\n        FragColor = mix(vec4(.5, .5, .5, 1), vec4(0), float(worldCoord.y > floorHeight));\n    }\n";
    pub fn new(
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        Ok(Self {
            shader: std::sync::Arc::new(crate::shader::Shader::new(
                Self::SOURCE,
                35632,
                vec![],
                backend,
                context,
                runtime,
            )?),
        })
    }
    pub fn shader(&self) -> std::sync::Arc<crate::shader::Shader> {
        self.get_floor_shader()
    }
    pub fn get_floor_shader(&self) -> std::sync::Arc<crate::shader::Shader> {
        self.shader.clone()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn companion_getter_returns_retained_shader() {
        let runtime = crate::resource::ResourceRuntime::default();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let backend = std::sync::Arc::new(std::sync::Mutex::new(
            crate::shader_program::tests::Backend(events),
        ));
        let owner =
            super::FloorShader::new(backend, runtime.allocate(&[], || {}), &runtime).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &owner.get_floor_shader(),
            &owner.shader()
        ));
    }
}

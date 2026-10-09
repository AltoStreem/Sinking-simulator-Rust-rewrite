//! Recovered Sea$Companion.class retained shader getter.
/// Sea.Companion retains a single fragment shader across Sea instances.
#[allow(dead_code)]
pub(crate) struct SeaShader {
    shader: std::sync::Arc<crate::shader::Shader>,
}
#[allow(dead_code)]
impl SeaShader {
    pub const SOURCE: &'static str = r#"
    #version 150 core

    in vec2 vTexCoord;
    out vec4 FragColor;

    uniform mat4 inv;
    uniform mat4 transform;
    uniform vec2 waveSize = vec2(20, 2.0);
    uniform float floorHeight;
    uniform float waterDarkness;
    uniform float time = 0;
    uniform vec2 resolution;
    uniform vec4 col = vec4(0, 0.278431373, 0.623529411, 0.5);

    uniform sampler2D tex;

    const float lightDepth = 1000;

    float water(const float xpos) {
        float invWave = 3.141592 / waveSize.x;
        return ((sin(xpos * invWave + time * .3) * .7 + sin(invWave * 3 * xpos - time) * .3) + 1) * .5 * waveSize.y;
    }

    void main() {
        vec2 screenCoord = vTexCoord * 2 - 1;
        vec4 worldCoord = inv * vec4(screenCoord, 0, 1);
        vec4 worldCoord2 = inv * vec4(0, screenCoord.y + 4/resolution.y, 0, 1);
        float waterdepth = water(worldCoord.x);

        float ref = ((transform * vec4(worldCoord.x, (2 * waterdepth - worldCoord2.y), 0, 1)).y + 1) * 0.5;
        vec4 reflection = texture(tex, vec2(vTexCoord.x, ref), distance(vTexCoord.y, ref) * 20);
        vec4 waterColor = mix(mix(col, reflection, reflection.a * smoothstep(.25, 0, distance(vTexCoord.y, ref)) * .4), vec4(0, 0, 0, 1), (waterdepth - worldCoord.y) / lightDepth * waterDarkness);

        vec4 color = mix(vec4(0), waterColor, smoothstep(worldCoord.y, worldCoord2.y, waterdepth));
        vec4 prev = texture(tex, vTexCoord.xy);
        FragColor = vec4(mix(prev.rgb, color.rgb, color.a), max(prev.a, color.a));
    }
"#;
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
        self.get_sea_shader()
    }
    pub fn get_sea_shader(&self) -> std::sync::Arc<crate::shader::Shader> {
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
        let owner = super::SeaShader::new(backend, runtime.allocate(&[], || {}), &runtime).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &owner.get_sea_shader(),
            &owner.shader()
        ));
    }
}

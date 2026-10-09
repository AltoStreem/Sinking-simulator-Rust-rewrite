//! Recovered Sky$Companion.class shader owners and bakeStars implementation.
/// Retained objects of Sky's static initializer, shared by all sky instances.
#[allow(dead_code)]
pub(crate) struct SourceSkyShaders {
    pub sky: std::sync::Arc<crate::shader::Shader>,
    pub star_field: std::sync::Arc<crate::shader_program::ShaderProgram>,
}
#[allow(dead_code)]
impl SourceSkyShaders {
    pub const SKY_SOURCE: &'static str = "\n    #version 150 core\n\n    in vec2 vTexCoord;\n    out vec4 FragColor;\n\n    uniform mat4 inv;\n    uniform float day = 1;\n    uniform vec2 resolution;\n\n    uniform sampler2D sky;\n    uniform sampler2D stars;\n\n    const vec4 col = vec4(0.529, 0.808, 0.980, 1);\n    const float pi = 3.141592;\n\n    void main()\n    {\n        vec2 screenCoord = vTexCoord * 2 - 1;\n        vec4 worldCoord = inv * vec4(screenCoord, 0, 1);\n        vec4 lim = inv * vec4(screenCoord.x, sign(worldCoord.y), 0, 1);\n        float ytex = 0;\n        if (worldCoord.y > 0) {\n            ytex = 0.5 - (worldCoord.y / lim.y) * 0.5;\n        } else {\n            ytex = 0.5 + (worldCoord.y / lim.y) * 0.5;\n        }\n\n        float StarVal = texture(stars, vTexCoord).x * smoothstep(0, 3, worldCoord.y) * (1 - day);\n        FragColor.xyz = texture(sky, vec2(day, ytex)).xyz + vec3(StarVal);\n        FragColor.w = 1;\n    }\n";
    pub fn new(
        vertices: &crate::vertex_shaders::SourceVertexShaders,
        shader_backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
        program_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::shader_program::ProgramBackend>,
        >,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        use std::sync::Arc;
        let sky = Arc::new(crate::shader::Shader::new(
            Self::SKY_SOURCE,
            35632,
            vec![],
            shader_backend.clone(),
            context.clone(),
            runtime,
        )?);
        let vertex = vertices.nothing()?;
        let field = Arc::new(crate::shader::Shader::new(
            crate::sky_star_field::SOURCE,
            35632,
            vec![],
            shader_backend,
            context.clone(),
            runtime,
        )?);
        let star_field = Arc::new(crate::shader_program::ShaderProgram::new(
            vec![vertex, field],
            program_backend,
            context,
            runtime,
        ));
        Ok(Self { sky, star_field })
    }
}

/// Sky.Companion.bakeStars through the converted texture, framebuffer, program
/// and position-only physics FullScreen classes. Blend is deliberately left
/// disabled, and the source draw scope does not restore state after a failure.
#[allow(dead_code)]
pub(crate) struct SourceStarBaker {
    program: std::sync::Arc<crate::shader_program::ShaderProgram>,
    fullscreen: std::sync::Arc<crate::model::SourceModel>,
    texture_backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
    framebuffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::fbo::FramebufferBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: crate::resource::ResourceRuntime,
    blend: Box<dyn Fn(bool)>,
}
#[allow(dead_code)]
impl SourceStarBaker {
    pub fn new(
        program: std::sync::Arc<crate::shader_program::ShaderProgram>,
        fullscreen: std::sync::Arc<crate::model::SourceModel>,
        texture_backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        framebuffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::fbo::FramebufferBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
        blend: impl Fn(bool) + 'static,
    ) -> Self {
        Self {
            program,
            fullscreen,
            texture_backend,
            framebuffer_backend,
            context,
            runtime: runtime.clone(),
            blend: Box::new(blend),
        }
    }
    pub fn make_stars(&self, size: [i32; 2]) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        let texture = std::sync::Arc::new(crate::texture_2d::SourceTexture2D::new(
            None,
            size,
            6408,
            32856,
            5121,
            false,
            std::sync::Arc::new(crate::sky_stars_texture::configure),
            self.texture_backend.clone(),
            self.context.clone(),
            &self.runtime,
        ));
        self.bake(texture.clone());
        texture
    }
    pub fn make_sky_stars(&self, size: [i32; 2]) -> std::rc::Rc<dyn crate::sky::SkyTexture> {
        std::rc::Rc::new(BakedStarTexture(self.make_stars(size)))
    }
    pub fn bake(&self, texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>) {
        use std::sync::{Arc, Mutex};
        let width = texture.width;
        let height = texture.height;
        let target: Arc<dyn crate::framebuffer_target::FramebufferTarget> = texture;
        let fbo = crate::fbo::Fbo::new(
            self.framebuffer_backend.clone(),
            self.context.clone(),
            &self.runtime,
        );
        let framebuffer = crate::textured_fbo::TexturedFbo::new(
            width,
            height,
            Arc::new(Mutex::new(vec![target].into_boxed_slice())),
            None,
            fbo,
        )
        .unwrap_or_else(|error| panic!("{error}"));
        framebuffer.draw(|| {
            (self.blend)(false);
            self.program.start();
            crate::i_drawable::IDrawable::render(self.fullscreen.as_ref());
            self.program.stop();
        });
    }
}
struct BakedStarTexture(std::sync::Arc<crate::texture_2d::SourceTexture2D>);
impl crate::sky::SkyTexture for BakedStarTexture {
    fn bind(&self, unit: i32) {
        self.0.texture.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.0.texture.unbind_unit(unit);
    }
}

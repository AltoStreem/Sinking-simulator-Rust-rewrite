//! UVModel.java: indexed Model geometry with attribute 1 texture coordinates.
use crate::model::Model;
use bevy::prelude::Mesh;
pub(crate) struct UVModel {
    pub model: Model,
}

/// UVModel.java's source VBO constructor and virtual attribute hooks.
#[allow(dead_code)]
pub(crate) struct SourceUVModel {
    pub shaded: crate::shaded_model::ShadedModel,
    texture_coords: std::sync::Arc<crate::vbo::Vbo>,
}
#[allow(dead_code)]
impl SourceUVModel {
    pub fn new(
        model: crate::model::SourceModel,
        texture_coords: std::sync::Arc<crate::vbo::Vbo>,
        shader: std::sync::Arc<dyn crate::shaded_model::ModelProgram>,
    ) -> Self {
        // Source ShadedModel validates/registers dependency before UV attribute binding.
        let shaded = crate::shaded_model::ShadedModel::new(model, shader);
        shaded
            .model
            .vao
            .bind_vbo_as_attribute(&texture_coords, 1, 2, false);
        Self {
            shaded,
            texture_coords,
        }
    }
    pub fn texture_coords(&self) -> std::sync::Arc<crate::vbo::Vbo> {
        self.texture_coords.clone()
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_arrays(
        indices: &[i32],
        vertices: &[f32],
        component_size: i32,
        uvs: &[f32],
        shader: std::sync::Arc<dyn crate::shaded_model::ModelProgram>,
        render_style: i32,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let index_buffer = crate::mem_util::wrap_int_buffer(indices);
        let vertex_buffer = crate::mem_util::wrap_float_buffer(vertices);
        let uv_buffer = crate::mem_util::wrap_float_buffer(uvs);
        Self::from_buffers(
            &index_buffer,
            &vertex_buffer,
            component_size,
            &uv_buffer,
            shader,
            render_style,
            buffer_backend,
            vao_backend,
            backend,
            context,
            runtime,
        )
    }
    /// The source NIO overload uploads remaining elements but retains capacity
    /// for VBO size/draw count; constructing geometry must not rewind the caller.
    #[allow(clippy::too_many_arguments)]
    pub fn from_buffers(
        indices: &crate::mem_util::NativeBuffer<i32>,
        vertices: &crate::mem_util::NativeBuffer<f32>,
        component_size: i32,
        uvs: &crate::mem_util::NativeBuffer<f32>,
        shader: std::sync::Arc<dyn crate::shaded_model::ModelProgram>,
        render_style: i32,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        use std::sync::Arc;
        // All three source argument VBOs are constructed before Model's VAO.
        let vertices = Arc::new(crate::vbo::Vbo::from_float_buffer(
            34962,
            vertices,
            35044,
            buffer_backend.clone(),
            context.clone(),
            runtime,
        ));
        let indices = Arc::new(crate::vbo::Vbo::from_int_buffer(
            34963,
            indices,
            35044,
            buffer_backend.clone(),
            context.clone(),
            runtime,
        ));
        let uvs = Arc::new(crate::vbo::Vbo::from_float_buffer(
            34962,
            uvs,
            35044,
            buffer_backend,
            context.clone(),
            runtime,
        ));
        let model = crate::model::SourceModel::new(
            vertices,
            indices,
            render_style,
            component_size,
            vao_backend,
            backend,
            context,
            runtime,
        );
        Self::new(model, uvs, shader)
    }
    pub fn render_with(&self, shader: &dyn crate::shaded_model::ModelProgram) {
        shader.start();
        self.render_shaderless();
        shader.stop();
    }
    pub fn render_shaderless(&self) {
        self.shaded.model.render_attributes(&[1]);
    }
}
impl crate::i_drawable::IDrawable for SourceUVModel {
    fn render(&self) {
        self.render_with(self.shaded.shader.as_ref());
    }
}
impl UVModel {
    pub fn new(
        indices: &[u32],
        vertices: &[f32],
        component_size: usize,
        uvs: &[f32],
        render_style: u32,
    ) -> Result<Self, String> {
        let mut model = Model::new(indices, vertices, component_size, render_style)?;
        if uvs.len() != vertices.len() / component_size * 2 {
            return Err("source UV and position counts differ".into());
        }
        model.mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            uvs.chunks_exact(2)
                .map(|uv| [uv[0], uv[1]])
                .collect::<Vec<_>>(),
        );
        Ok(Self { model })
    }
    pub fn into_mesh(self) -> Mesh {
        self.model.mesh
    }
}

//! Model.java's indexed position geometry mapped to Bevy Mesh ownership.
//! ECS Mesh2d/Material2d replace the original bind/draw/unbind calls.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::Mesh,
};
pub(crate) struct Model {
    pub mesh: Mesh,
}
impl Model {
    pub fn new(
        indices: &[u32],
        vertices: &[f32],
        component_size: usize,
        render_style: u32,
    ) -> Result<Self, String> {
        if !(1..=4).contains(&component_size) || vertices.len() % component_size != 0 {
            return Err("invalid source vertex component count".into());
        }
        let positions: Vec<[f32; 3]> = vertices
            .chunks_exact(component_size)
            .map(|p| {
                [
                    p[0],
                    p.get(1).copied().unwrap_or(0.0),
                    p.get(2).copied().unwrap_or(0.0),
                ]
            })
            .collect();
        if indices
            .iter()
            .any(|index| *index as usize >= positions.len())
        {
            return Err("source index outside vertex buffer".into());
        }
        let (topology, indices) = match render_style {
            0 => (PrimitiveTopology::PointList, indices.to_vec()),
            1 => (PrimitiveTopology::LineList, indices.to_vec()),
            2 => {
                let mut lines: Vec<u32> = indices
                    .windows(2)
                    .flat_map(|pair| [pair[0], pair[1]])
                    .collect();
                if let (Some(first), Some(last)) = (indices.first(), indices.last()) {
                    lines.extend([*last, *first]);
                }
                (PrimitiveTopology::LineList, lines)
            }
            3 => (PrimitiveTopology::LineStrip, indices.to_vec()),
            4 => (PrimitiveTopology::TriangleList, indices.to_vec()),
            5 => (PrimitiveTopology::TriangleStrip, indices.to_vec()),
            6 => (
                PrimitiveTopology::TriangleList,
                indices
                    .get(1..)
                    .unwrap_or(&[])
                    .windows(2)
                    .flat_map(|pair| [indices[0], pair[0], pair[1]])
                    .collect(),
            ),
            _ => return Err(format!("unsupported source primitive {render_style}")),
        };
        let mut mesh = Mesh::new(topology, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(Indices::U32(indices));
        Ok(Self { mesh })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_model_converts_loop_and_fan_without_changing_vertex_order() {
        let verts = [-1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0];
        let loop_model = Model::new(&[0, 1, 2, 3], &verts, 2, 2).unwrap();
        assert_eq!(
            loop_model.mesh.indices(),
            Some(&Indices::U32(vec![0, 1, 1, 2, 2, 3, 3, 0]))
        );
        let fan = Model::new(&[0, 1, 2, 3], &verts, 2, 6).unwrap();
        assert_eq!(
            fan.mesh.indices(),
            Some(&Indices::U32(vec![0, 1, 2, 0, 2, 3]))
        );
    }
}

/// Source Model render/lifecycle path. The Bevy mesh adapter above remains the active game path.
pub(crate) trait ModelBackend: Send {
    fn enable_attribute(&mut self, index: i32);
    fn disable_attribute(&mut self, index: i32);
    fn draw_elements(&mut self, style: i32, count: usize, kind: i32, offset: u64);
}
#[allow(dead_code)]
pub(crate) struct SourceModel {
    pub vao: std::sync::Arc<crate::vao::Vao>,
    pub vertices: std::sync::Arc<crate::vbo::Vbo>,
    pub indices: std::sync::Arc<crate::vbo::Vbo>,
    pub render_style: i32,
    pub component_size: i32,
    lifetime: crate::resource::ResourceHandle,
    backend: std::sync::Arc<std::sync::Mutex<dyn ModelBackend>>,
}
#[allow(dead_code)]
impl SourceModel {
    pub fn new(
        vertices: std::sync::Arc<crate::vbo::Vbo>,
        indices: std::sync::Arc<crate::vbo::Vbo>,
        render_style: i32,
        component_size: i32,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        // Model Resource has no dependencies and empty free(); VAO/VBO are retained references.
        let lifetime = runtime.allocate(&[], || {});
        let vao = std::sync::Arc::new(crate::vao::Vao::new(vao_backend, context, runtime));
        vao.attach_vbo(&indices);
        vao.bind_vbo_as_attribute(&vertices, 0, component_size, false);
        Self {
            vao,
            vertices,
            indices,
            render_style,
            component_size,
            lifetime,
            backend,
        }
    }
    pub fn resource_handle(&self) -> crate::resource::ResourceHandle {
        self.lifetime.clone()
    }
    pub fn close(&self) {
        self.lifetime.close();
    }
    pub fn freed(&self) -> bool {
        self.lifetime.freed()
    }
}
impl crate::i_drawable::IDrawable for SourceModel {
    fn render(&self) {
        self.vao.bind();
        let mut backend = self.backend.lock().unwrap();
        backend.enable_attribute(0);
        backend.draw_elements(self.render_style, self.indices.size, 5125, 0);
        backend.disable_attribute(0);
        drop(backend);
        self.vao.unbind();
    }
}

#[allow(dead_code)]
impl SourceModel {
    pub fn from_arrays(
        indices: &[i32],
        vertices: &[f32],
        component_size: i32,
        render_style: i32,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let verts = crate::mem_util::wrap_float_buffer(vertices);
        let inds = crate::mem_util::wrap_int_buffer(indices);
        let vertices = std::sync::Arc::new(crate::vbo::Vbo::from_float_buffer(
            34962,
            &verts,
            35044,
            buffer_backend.clone(),
            context.clone(),
            runtime,
        ));
        let indices = std::sync::Arc::new(crate::vbo::Vbo::from_int_buffer(
            34963,
            &inds,
            35044,
            buffer_backend,
            context.clone(),
            runtime,
        ));
        Self::new(
            vertices,
            indices,
            render_style,
            component_size,
            vao_backend,
            backend,
            context,
            runtime,
        )
    }
}

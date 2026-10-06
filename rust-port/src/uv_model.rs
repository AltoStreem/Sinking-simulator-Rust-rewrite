//! UVModel.java: indexed Model geometry with attribute 1 texture coordinates.
use crate::model::Model;
use bevy::prelude::Mesh;
pub(crate) struct UVModel {
    pub model: Model,
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

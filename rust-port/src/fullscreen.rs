//! Fullscreen.java's exact six indices, four clip-space corners and UVs.
use crate::{uv_model::UVModel, vertex_shaders::VertexShaders};
use bevy::{mesh::VertexAttributeValues, prelude::*};
pub(crate) struct Fullscreen;
impl Fullscreen {
    pub const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
    pub const VERTICES: [f32; 8] = [-1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0];
    pub const UVS: [f32; 8] = [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0];
    pub fn mesh() -> Mesh {
        UVModel::new(&Self::INDICES, &Self::VERTICES, 2, &Self::UVS, 4)
            .expect("source fullscreen geometry")
            .into_mesh()
    }
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn source(
        shader: std::sync::Arc<dyn crate::shaded_model::ModelProgram>,
        buffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vao_backend: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> crate::uv_model::SourceUVModel {
        crate::uv_model::SourceUVModel::from_arrays(
            &[0, 1, 2, 0, 2, 3],
            &Self::VERTICES,
            2,
            &Self::UVS,
            shader,
            4,
            buffer_backend,
            vao_backend,
            backend,
            context,
            runtime,
        )
    }
    /// Fit the source clip-space quad into the orthographic world rectangle.
    /// Flip UV Y for Bevy's top-left image convention when sampling pass maps.
    pub fn fit(mesh: &mut Mesh, area: Rect) {
        let transform = Mat4::from_scale_rotation_translation(
            (area.size() * 0.5).extend(1.0),
            Quat::IDENTITY,
            area.center().extend(0.0),
        );
        if let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for (position, source) in positions.iter_mut().zip(Self::VERTICES.chunks_exact(2)) {
                *position = VertexShaders::transform(
                    Vec3::new(source[0], source[1], 0.0),
                    Vec2::ZERO,
                    transform,
                )
                .position
                .truncate()
                .to_array();
            }
        }
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            Self::UVS
                .chunks_exact(2)
                .map(|uv| [uv[0], 1.0 - uv[1]])
                .collect::<Vec<_>>(),
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_fullscreen_preserves_geometry_and_fits_camera_rectangle() {
        let mut mesh = Fullscreen::mesh();
        assert_eq!(
            mesh.indices(),
            Some(&bevy::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]))
        );
        Fullscreen::fit(
            &mut mesh,
            Rect::from_corners(Vec2::new(-20.0, -10.0), Vec2::new(40.0, 50.0)),
        );
        let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        assert_eq!(
            p,
            &vec![
                [-20.0, 50.0, 0.0],
                [-20.0, -10.0, 0.0],
                [40.0, -10.0, 0.0],
                [40.0, 50.0, 0.0]
            ]
        );
    }
}

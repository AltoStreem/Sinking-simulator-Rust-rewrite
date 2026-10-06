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

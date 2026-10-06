//! Source `ShipStruts.java` vertex/index construction and shader link filtering.
use crate::ship_data::ShipData;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};

#[derive(Component)]
pub(crate) struct ShipStruts {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) dimensions: (usize, usize),
    pub(crate) masks: Vec<[u32; 4]>,
}

impl ShipStruts {
    pub(crate) fn create_vertices(dat: &ShipData) -> Vec<[f32; 2]> {
        (0..dat.height)
            .flat_map(|y| (0..dat.width).map(move |x| [x as f32, y as f32]))
            .collect()
    }

    pub(crate) fn create_indices(dat: &ShipData) -> Vec<u32> {
        let occupied: Vec<_> = dat.material_buffer.iter().map(Option::is_some).collect();
        Self::indices(dat.width as usize, dat.height as usize, &occupied, None)
    }

    pub(crate) fn live_indices(
        width: usize,
        height: usize,
        occupied: &[bool],
        masks: &[[u32; 4]],
    ) -> Vec<u32> {
        assert_eq!(masks.len(), width * height);
        Self::indices(width, height, occupied, Some(masks))
    }

    fn indices(
        width: usize,
        height: usize,
        occupied: &[bool],
        masks: Option<&[[u32; 4]]>,
    ) -> Vec<u32> {
        assert_eq!(occupied.len(), width * height);
        let mut indices = Vec::new();
        // Kotlin loops over X then Y. Each undirected edge is emitted once:
        // east, southeast, south, southwest (mask bits 0,1,2,3).
        for x in 0..width {
            for y in 0..height {
                let a = y * width + x;
                if !occupied[a] {
                    continue;
                }
                for (bit, (dx, dy)) in [(1isize, 0isize), (1, 1), (0, 1), (-1, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                        continue;
                    }
                    let b = ny as usize * width + nx as usize;
                    if occupied[b] && masks.is_none_or(|m| m[a][1] & (1 << bit) != 0) {
                        indices.extend([a as u32, b as u32]);
                    }
                }
            }
        }
        indices
    }
}

pub(crate) fn build_mesh(structure: &crate::ShipStructure, masks: &[[u32; 4]]) -> Mesh {
    let width = structure.texel_width;
    let height = structure.texel_height;
    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let positions: Vec<[f32; 3]> = structure
        .texel_positions
        .iter()
        .map(|p| [p.x, p.y, 0.0])
        .collect();
    let uvs: Vec<[f32; 2]> = (0..width * height)
        .map(|i| {
            [
                (i % width) as f32 / width as f32 + 0.5 / width as f32,
                (i / width) as f32 / height as f32 + 0.5 / height as f32,
            ]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(ShipStruts::live_indices(
        width,
        height,
        &structure.texel_solid,
        masks,
    )));
    mesh
}

pub(crate) fn sync_ship_struts(
    structure: Res<crate::ShipStructure>,
    snapshot: Res<crate::GpuShipPhysicsSnapshot>,
    mut meshes: ResMut<Assets<Mesh>>,
    surfaces: Query<&Transform, (With<crate::ShipMesh>, Without<ShipStruts>)>,
    mut struts: Query<(&mut ShipStruts, &mut Transform), Without<crate::ShipMesh>>,
) {
    let masks = crate::current_render_masks(&structure, &snapshot);
    let dimensions = (structure.texel_width, structure.texel_height);
    for (mut state, mut transform) in &mut struts {
        if let Ok(surface) = surfaces.single() {
            *transform = *surface;
            // Draw the line pass behind filled surfaces. A future stencil pass
            // is still needed for exact source compositing of translucent art.
            transform.translation.z -= 0.001;
        }
        let Some(mut mesh) = meshes.get_mut(&state.mesh) else {
            continue;
        };
        if state.dimensions != dimensions || state.masks != masks {
            *mesh = build_mesh(&structure, &masks);
            state.dimensions = dimensions;
            state.masks.clone_from(&masks);
        } else if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for (vertex, position) in vertices.iter_mut().zip(&structure.texel_positions) {
                *vertex = [position.x, position.y, 0.0];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_geometry_struts_preserve_source_order_and_mask_channel() {
        let occupied = [true; 4];
        let masks = [[8, 7, 0, 0], [8, 28, 0, 0], [8, 193, 0, 0], [8, 112, 0, 0]];
        assert_eq!(
            ShipStruts::live_indices(2, 2, &occupied, &masks),
            vec![0, 1, 0, 3, 0, 2, 2, 3, 1, 3, 1, 2]
        );
        let mut broken = masks;
        broken[0][1] &= !2;
        assert_eq!(
            ShipStruts::live_indices(2, 2, &occupied, &broken),
            vec![0, 1, 0, 2, 2, 3, 1, 3, 1, 2]
        );
        assert!(ShipStruts::live_indices(2, 2, &occupied, &[[8, 0, 255, 0]; 4]).is_empty());
    }
}

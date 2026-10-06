//! Source `Ship.java` companion geometry and geometry-shader topology.
//! Bevy has no geometry shaders; emit the same triangles on the CPU using
//! the live mask channel Z, then upload them as a triangle-list mesh.

use crate::ship_data::ShipData;

pub(crate) struct Ship;

impl Ship {
    /// Original point primitives: one cell anchor whenever any corner has a
    /// material. This preserves the source's row-major primitive order.
    pub(crate) fn create_indices(dat: &ShipData) -> Vec<u32> {
        let width = dat.width as usize;
        let height = dat.height as usize;
        let mut indices = Vec::new();
        for y in 0..height.saturating_sub(1) {
            for x in 0..width.saturating_sub(1) {
                let i = y * width + x;
                if [i, i + 1, i + width, i + width + 1]
                    .iter()
                    .any(|&corner| dat.material_buffer[corner].is_some())
                {
                    indices.push((y * (width - 1) + x) as u32);
                }
            }
        }
        indices
    }

    pub(crate) fn create_vertices(dat: &ShipData) -> Vec<[f32; 2]> {
        (0..dat.height.saturating_sub(1))
            .flat_map(|y| (0..dat.width.saturating_sub(1)).map(move |x| [x as f32, y as f32]))
            .collect()
    }

    /// Direct translation of ShipShader's full quad and make013/make132/
    /// make012/make032 predicates. Vertex IDs refer to the source texels,
    /// whose UVs sample their centers rather than an expanded pixel border.
    pub(crate) fn triangle_indices(width: usize, height: usize, masks: &[[u32; 4]]) -> Vec<u32> {
        assert_eq!(masks.len(), width * height);
        let mut indices = Vec::new();
        for y in 0..height.saturating_sub(1) {
            for x in 0..width.saturating_sub(1) {
                let tl = y * width + x;
                // Source offsets: 0=(0,1), 1=(0,0), 2=(1,1), 3=(1,0).
                let corners = [tl + width, tl, tl + width + 1, tl + 1];
                let s = corners.map(|i| masks[i][2]);
                let mut emit = |a: usize, b: usize, c: usize| {
                    indices.extend([corners[a] as u32, corners[b] as u32, corners[c] as u32]);
                };
                if s[1] & 7 == 7 && s[0] & 1 != 0 && s[3] & 12 == 12 {
                    // GLSL triangle strip [0,1,2,3], with alternating winding.
                    emit(0, 1, 2);
                    emit(2, 1, 3);
                } else {
                    if s[1] & 5 == 5 && s[3] & 8 != 0 {
                        emit(0, 1, 3);
                    }
                    if s[1] & 3 == 3 && s[3] & 4 != 0 {
                        emit(1, 3, 2);
                    }
                    if s[1] & 6 == 6 && s[0] & 1 != 0 {
                        emit(0, 1, 2);
                    }
                    if s[3] & 12 == 12 && s[0] & 1 != 0 {
                        emit(0, 3, 2);
                    }
                }
            }
        }
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_geometry_full_quad_matches_triangle_strip() {
        let masks = [
            [8, 7, 7, 0],
            [8, 28, 28, 0],
            [8, 193, 193, 0],
            [8, 112, 112, 0],
        ];
        assert_eq!(Ship::triangle_indices(2, 2, &masks), vec![2, 0, 3, 3, 0, 1]);
    }

    #[test]
    fn source_geometry_broken_diagonal_changes_surface_and_empty_links_remove_it() {
        // Only source make013 is permitted: top-left east/south and
        // top-right southwest remain connected.
        let masks = [[8, 5, 5, 0], [8, 8, 8, 0], [8, 0, 0, 0], [8, 0, 0, 0]];
        assert_eq!(Ship::triangle_indices(2, 2, &masks), vec![2, 0, 1]);
        let disconnected = [[8, 0, 0, 0]; 4];
        assert!(Ship::triangle_indices(2, 2, &disconnected).is_empty());
    }
}

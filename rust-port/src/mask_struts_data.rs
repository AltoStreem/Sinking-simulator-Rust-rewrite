//! Translation of SS2 `MaskStrutsDataHolder` pixel masks and spring links.
use crate::ShipStructure;

/// Pack occupancy/material flags and reciprocal spring direction masks.
pub(super) fn gpu_mask_data(structure: &ShipStructure) -> Vec<[u32; 4]> {
    let solid: Vec<bool> = structure
        .texel_solid
        .iter()
        .enumerate()
        .map(|(index, &is_solid)| {
            let x = index % structure.texel_width;
            let y = index / structure.texel_width;
            let proxy =
                (y / crate::PHYSICS_NODE_PIXELS) * structure.width + x / crate::PHYSICS_NODE_PIXELS;
            is_solid && !structure.breached[proxy]
        })
        .collect();
    let water_solid: Vec<bool> = structure
        .texel_materials
        .iter()
        .zip(&solid)
        .map(|(material, &is_solid)| is_solid && material.is_some_and(|material| !material.rope))
        .collect();
    let struts = build_strut_masks(&solid, structure.texel_width, structure.texel_height);
    let water_struts =
        build_strut_masks(&water_solid, structure.texel_width, structure.texel_height);
    structure
        .texel_materials
        .iter()
        .zip(struts)
        .zip(water_struts)
        .zip(solid)
        .map(|(((material, struts), water_struts), is_solid)| {
            let flags = if is_solid {
                material.map_or(0, |material| {
                    8 | (u32::from(material.ground) << 2)
                        | (u32::from(material.hull) << 1)
                        | u32::from(material.rope)
                })
            } else {
                0
            };
            [flags, u32::from(struts), u32::from(water_struts), 0]
        })
        .collect()
}

/// Create one reciprocal bit for each solid 8-neighbour spring edge.
pub(super) fn build_strut_masks(solid: &[bool], width: usize, height: usize) -> Vec<u8> {
    if width == 0 || height == 0 || solid.len() != width * height {
        return vec![0; solid.len()];
    }
    const FORWARD_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (1, 1), (0, 1), (-1, 1)];
    let mut masks = vec![0u8; solid.len()];
    for y in 0..height {
        for x in 0..width {
            let a = y * width + x;
            if !solid[a] {
                continue;
            }
            for (direction, (dx, dy)) in FORWARD_DIRECTIONS.iter().copied().enumerate() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    continue;
                }
                let b = ny as usize * width + nx as usize;
                if solid[b] {
                    masks[a] |= 1 << direction;
                    masks[b] |= 1 << (direction + 4);
                }
            }
        }
    }
    masks
}

/// Live plane followed by the stable input plane used by the source final pass.
pub(super) fn gpu_mask_storage(mut live: Vec<[u32; 4]>) -> Vec<[u32; 4]> {
    live.extend_from_within(..);
    live
}

/// CPU reference of ShipPhysics finalPass's opposite-link intersection.
#[cfg(test)]
fn reconcile_links(width: usize, height: usize, input: &[[u32; 4]]) -> Vec<[u32; 4]> {
    let mut output = input.to_vec();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            for (direction, (dx, dy)) in crate::EIGHT_NEIGHBORS.iter().copied().enumerate() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                let other = if nx >= 0 && ny >= 0 && nx < width as isize && ny < height as isize {
                    input[ny as usize * width + nx as usize][1]
                } else {
                    0
                };
                if other & (1 << ((direction + 4) & 7)) == 0 {
                    output[index][1] &= !(1 << direction);
                    output[index][2] &= !(1 << direction);
                }
            }
        }
    }
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_reciprocal_pass_cuts_the_other_end_and_preserves_flags() {
        let input = [[8, 1, 1, 0], [10, 0, 0, 0]];
        assert_eq!(
            reconcile_links(2, 1, &input),
            vec![[8, 0, 0, 0], [10, 0, 0, 0]]
        );
        let input = [[8, 1, 1, 0], [10, 16, 16, 0]];
        assert_eq!(reconcile_links(2, 1, &input), input);
        assert_eq!(gpu_mask_storage(input.to_vec()), [input, input].concat());
    }
}

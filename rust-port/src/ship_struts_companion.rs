//! ShipStruts$Companion: X-major link order and signed source vertex allocation.
#![allow(dead_code)]
use crate::ship_companion::ShipGeometryData;
pub(crate) fn create_indices(dat: &impl ShipGeometryData) -> Result<Vec<i32>, String> {
    let width = dat.width();
    let height = dat.height();
    let offsets = [1, width.wrapping_add(1), width, width.wrapping_sub(1)];
    let exclusions = [
        (width.wrapping_sub(1), -1),
        (width.wrapping_sub(1), height.wrapping_sub(1)),
        (-1, height.wrapping_sub(1)),
        (0, height.wrapping_sub(1)),
    ];
    let mut indices = vec![];
    for x in 0..width {
        for y in 0..height {
            let index = y.wrapping_mul(width).wrapping_add(x);
            if !dat.occupied(index)? {
                continue;
            }
            for (offset, (excluded_x, excluded_y)) in offsets.into_iter().zip(exclusions) {
                if x != excluded_x && y != excluded_y {
                    let other = index.wrapping_add(offset);
                    if dat.occupied(other)? {
                        indices.extend([index, other]);
                    }
                }
            }
        }
    }
    Ok(indices)
}
pub(crate) fn create_vertices(dat: &impl ShipGeometryData) -> Result<Vec<f32>, String> {
    let count = dat.width().wrapping_mul(dat.height()).wrapping_mul(2);
    let count =
        usize::try_from(count).map_err(|_| "negative source vertex array size".to_owned())?;
    let mut vertices = vec![0.; count];
    for (i, value) in vertices.iter_mut().enumerate() {
        let index = i as i32 / 2;
        let divisor = dat.width();
        if divisor == 0 {
            return Err("source integer division by zero".into());
        }
        *value = if i % 2 == 0 {
            index.wrapping_rem(divisor)
        } else {
            index.wrapping_div(divisor)
        } as f32;
    }
    Ok(vertices)
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Data(i32, i32, Vec<bool>);
    impl ShipGeometryData for Data {
        fn width(&self) -> i32 {
            self.0
        }
        fn height(&self) -> i32 {
            self.1
        }
        fn occupied(&self, index: i32) -> Result<bool, String> {
            usize::try_from(index)
                .ok()
                .and_then(|i| self.2.get(i))
                .copied()
                .ok_or_else(|| "bounds".into())
        }
    }
    #[test]
    fn source_x_major_edges_and_vertex_row_order_are_independent() {
        let dat = Data(2, 2, vec![true; 4]);
        assert_eq!(
            create_indices(&dat).unwrap(),
            [0, 1, 0, 3, 0, 2, 2, 3, 1, 3, 1, 2]
        );
        assert_eq!(
            create_vertices(&dat).unwrap(),
            [0., 0., 1., 0., 0., 1., 1., 1.]
        );
        assert!(
            create_indices(&Data(2, 2, vec![false; 4]))
                .unwrap()
                .is_empty()
        );
        assert!(create_indices(&Data(2, 2, vec![true])).is_err());
    }
    #[test]
    fn source_negative_ranges_and_wrapped_allocations_are_preserved() {
        assert!(create_indices(&Data(-1, -2, vec![])).unwrap().is_empty());
        assert_eq!(
            create_vertices(&Data(-1, -2, vec![])).unwrap(),
            [0., 0., 0., -1.]
        );
        assert!(
            create_vertices(&Data(-1, 2, vec![]))
                .unwrap_err()
                .contains("negative")
        );
        assert!(
            create_vertices(&Data(65536, 65536, vec![]))
                .unwrap()
                .is_empty()
        );
        assert!(create_vertices(&Data(0, 2, vec![])).unwrap().is_empty());
    }
}

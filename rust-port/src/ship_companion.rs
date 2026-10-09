//! Ship$Companion.createIndices/createVertices: signed JVM arithmetic and ordered list reads.
#![allow(dead_code)]
pub(crate) trait ShipGeometryData {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn occupied(&self, index: i32) -> Result<bool, String>;
}
impl<P: crate::ship_data::SourceMaterialLookup> ShipGeometryData
    for crate::ship_data::SourceShipData<P>
{
    fn width(&self) -> i32 {
        self.width
    }
    fn height(&self) -> i32 {
        self.height
    }
    fn occupied(&self, index: i32) -> Result<bool, String> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.material_buffer.borrow().get(i).map(Option::is_some))
            .ok_or_else(|| "source material list index out of bounds".into())
    }
}
impl ShipGeometryData for crate::ship_data::ShipData {
    fn width(&self) -> i32 {
        self.width as i32
    }
    fn height(&self) -> i32 {
        self.height as i32
    }
    fn occupied(&self, index: i32) -> Result<bool, String> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.material_buffer.get(i).map(Option::is_some))
            .ok_or_else(|| "source material list index out of bounds".into())
    }
}
pub(crate) fn create_indices(dat: &impl ShipGeometryData) -> Result<Vec<i32>, String> {
    let count = dat
        .width()
        .wrapping_sub(1)
        .wrapping_mul(dat.height().wrapping_sub(1));
    let count =
        usize::try_from(count).map_err(|_| "negative source index array size".to_owned())?;
    // Java constructs the complete temporary array before filtering out -1.
    let mut indices = vec![0; count];
    for (index, value) in indices.iter_mut().enumerate() {
        let divisor = dat.width().wrapping_sub(1);
        if divisor == 0 {
            return Err("source integer division by zero".into());
        }
        let x = (index as i32).wrapping_rem(divisor);
        let y = (index as i32).wrapping_div(divisor);
        let top = y.wrapping_mul(dat.width()).wrapping_add(x);
        // Java && short-circuits. Do not prevalidate the whole material list.
        let absent = !dat.occupied(top)?
            && !dat.occupied(top.wrapping_add(1))?
            && !dat.occupied(y.wrapping_add(1).wrapping_mul(dat.width()).wrapping_add(x))?
            && !dat.occupied(
                y.wrapping_add(1)
                    .wrapping_mul(dat.width())
                    .wrapping_add(x)
                    .wrapping_add(1),
            )?;
        *value = if absent { -1 } else { index as i32 };
    }
    Ok(indices.into_iter().filter(|index| *index != -1).collect())
}
pub(crate) fn create_vertices(dat: &impl ShipGeometryData) -> Result<Vec<f32>, String> {
    let count = dat
        .width()
        .wrapping_sub(1)
        .wrapping_mul(dat.height().wrapping_sub(1))
        .wrapping_mul(2);
    let count =
        usize::try_from(count).map_err(|_| "negative source vertex array size".to_owned())?;
    let mut vertices = vec![0.; count];
    for (i, value) in vertices.iter_mut().enumerate() {
        let index = i as i32 / 2;
        let divisor = dat.width().wrapping_sub(1);
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
    use std::cell::RefCell;
    struct Data {
        width: i32,
        height: i32,
        materials: Vec<bool>,
        reads: RefCell<Vec<i32>>,
    }
    impl ShipGeometryData for Data {
        fn width(&self) -> i32 {
            self.width
        }
        fn height(&self) -> i32 {
            self.height
        }
        fn occupied(&self, index: i32) -> Result<bool, String> {
            self.reads.borrow_mut().push(index);
            usize::try_from(index)
                .ok()
                .and_then(|i| self.materials.get(i))
                .copied()
                .ok_or_else(|| "bounds".into())
        }
    }
    #[test]
    fn source_cell_order_and_short_circuit_reads_match_java() {
        let dat = Data {
            width: 3,
            height: 2,
            materials: vec![false, false, true, false, false, false],
            reads: RefCell::new(vec![]),
        };
        assert_eq!(create_indices(&dat).unwrap(), [1]);
        assert_eq!(*dat.reads.borrow(), [0, 1, 3, 4, 1, 2]);
        assert_eq!(create_vertices(&dat).unwrap(), [0., 0., 1., 0.]);
        let truncated = Data {
            width: 2,
            height: 2,
            materials: vec![true],
            reads: RefCell::new(vec![]),
        };
        assert_eq!(create_indices(&truncated).unwrap(), [0]);
        assert_eq!(*truncated.reads.borrow(), [0]);
    }
    #[test]
    fn source_signed_dimensions_preserve_zero_overflow_and_failure_behavior() {
        let mut dat = Data {
            width: 0,
            height: 0,
            materials: vec![true],
            reads: RefCell::new(vec![]),
        };
        assert_eq!(create_indices(&dat).unwrap(), [0]);
        assert_eq!(create_vertices(&dat).unwrap(), [0., 0.]);
        dat.height = 2;
        assert!(create_indices(&dat).unwrap_err().contains("negative"));
        assert!(create_vertices(&dat).unwrap_err().contains("negative"));
        dat.width = 1;
        assert!(create_indices(&dat).unwrap().is_empty());
        assert!(create_vertices(&dat).unwrap().is_empty());
        dat.width = 65537;
        dat.height = 65537; // JVM multiplication wraps to zero.
        assert!(create_indices(&dat).unwrap().is_empty());
        assert!(create_vertices(&dat).unwrap().is_empty());
    }
    #[test]
    fn source_geometry_reads_the_retained_material_list_after_external_mutation() {
        use std::{
            rc::Rc,
            sync::{Arc, Mutex},
        };
        let materials = Rc::new(RefCell::new(vec![None; 4]));
        let dat = crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(crate::materials::SourceMaterials::default()),
            materials.clone(),
            2,
            2,
        );
        assert!(create_indices(&dat).unwrap().is_empty());
        assert!(
            crate::ship_struts_companion::create_indices(&dat)
                .unwrap()
                .is_empty()
        );
        materials.borrow_mut()[0] = Some(Arc::new(crate::materials::Material::default()));
        materials.borrow_mut()[3] = Some(Arc::new(crate::materials::Material::default()));
        assert_eq!(create_indices(&dat).unwrap(), [0]);
        assert_eq!(
            crate::ship_struts_companion::create_indices(&dat).unwrap(),
            [0, 3]
        );
        materials.borrow_mut().truncate(1);
        assert_eq!(create_indices(&dat).unwrap(), [0]); // First occupied corner short-circuits.
        assert!(crate::ship_struts_companion::create_indices(&dat).is_err());
    }
}

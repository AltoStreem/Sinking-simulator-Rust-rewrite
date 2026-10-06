//! Translation of all four KotlinHelpersKt mapInPlace overloads.
//! Rust slices cover both object arrays and primitive float arrays. Transforms
//! borrow the old element; replacements retain the original slice allocation.
#![allow(dead_code)]
pub fn map_in_place<T>(array: &mut [T], mut transform: impl FnMut(&T) -> T) -> &mut [T] {
    for element in array.iter_mut() {
        *element = transform(element);
    }
    array
}
pub fn map_in_place_indexed<T>(
    array: &mut [T],
    mut transform: impl FnMut(usize, &T) -> T,
) -> &mut [T] {
    for (index, element) in array.iter_mut().enumerate() {
        *element = transform(index, element);
    }
    array
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mapping_keeps_allocation_and_left_to_right_order() {
        let mut array = [1.0, 2.0, 3.0];
        let original = array.as_ptr();
        let mut order = Vec::new();
        let result = map_in_place_indexed(&mut array, |index, value| {
            order.push(index);
            *value + index as f32
        });
        assert_eq!(result.as_ptr(), original);
        assert_eq!(result, &[1.0, 3.0, 5.0]);
        assert_eq!(order, vec![0, 1, 2]);
        assert_eq!(map_in_place(result, |value| value * 2.0), &[2.0, 6.0, 10.0]);
    }
    #[test]
    fn object_and_empty_arrays() {
        let mut array = [String::from("ship")];
        assert_eq!(
            map_in_place(&mut array, |value| format!("{value}!")),
            &[String::from("ship!")]
        );
        let mut empty: [f32; 0] = [];
        map_in_place(&mut empty, |_| panic!("empty callback"));
    }
}

//! EnumSet.java bitmap operations and companion constructors.
use super::{
    bitmap_enum::BitmapEnum, mutable_enum_set::MutableEnumSet, static_enum_set::StaticEnumSet,
};
use std::{cell::RefCell, rc::Rc};
/// Retains the source array by reference; constructors cache their own filtered values/maps.
pub(crate) type EnumArray<T> = Rc<RefCell<Vec<T>>>;
pub(crate) fn bitmap_of<T: BitmapEnum>(values: &[T]) -> i32 {
    values.iter().fold(0, |mask, value| mask | value.int())
}
pub(crate) trait EnumSet<T: BitmapEnum> {
    fn bitmap(&self) -> i32;
    fn size(&self) -> usize {
        self.bitmap().count_ones() as usize
    }
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
    fn contains(&self, value: &T) -> bool {
        value.int() & self.bitmap() != 0
    }
    // This reversed containment is the original DefaultImpls behavior.
    fn contains_all_bitmap(&self, other: i32) -> bool {
        self.bitmap() & other == self.bitmap()
    }
    fn contains_all(&self, values: &[T]) -> bool {
        self.contains_all_bitmap(bitmap_of(values))
    }
}
pub(crate) fn of<T: BitmapEnum + Clone>(values: &[T], universe: EnumArray<T>) -> StaticEnumSet<T> {
    StaticEnumSet::new(bitmap_of(values), universe)
}
pub(crate) fn of_bitmap<T: BitmapEnum + Clone>(
    bitmap: i32,
    universe: EnumArray<T>,
) -> StaticEnumSet<T> {
    let all = bitmap_of(&universe.borrow());
    StaticEnumSet::new(bitmap & all, universe)
}
pub(crate) fn all<T: BitmapEnum + Clone>(universe: EnumArray<T>) -> StaticEnumSet<T> {
    let bitmap = bitmap_of(&universe.borrow());
    StaticEnumSet::new(bitmap, universe)
}
pub(crate) fn none<T: BitmapEnum + Clone>(universe: EnumArray<T>) -> StaticEnumSet<T> {
    StaticEnumSet::new(0, universe)
}
pub(crate) fn mutable_of<T: BitmapEnum + Clone>(
    values: &[T],
    universe: EnumArray<T>,
) -> MutableEnumSet<T> {
    MutableEnumSet::new(bitmap_of(values), universe)
}
pub(crate) fn mutable_all<T: BitmapEnum + Clone>(universe: EnumArray<T>) -> MutableEnumSet<T> {
    let bitmap = bitmap_of(&universe.borrow());
    MutableEnumSet::new(bitmap, universe)
}
pub(crate) fn mutable_none<T: BitmapEnum + Clone>(universe: EnumArray<T>) -> MutableEnumSet<T> {
    MutableEnumSet::new(0, universe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{int_enum::IntEnum, mutable_enum_set::IteratorError};
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Flag {
        A,
        B,
        C,
        Combined,
        Zero,
    }
    impl IntEnum for Flag {
        fn int(&self) -> i32 {
            match self {
                Self::A => 1,
                Self::B => 2,
                Self::C => 4,
                Self::Combined => 3,
                Self::Zero => 0,
            }
        }
    }
    impl BitmapEnum for Flag {}
    fn universe() -> EnumArray<Flag> {
        Rc::new(RefCell::new(vec![Flag::A, Flag::B, Flag::C]))
    }
    #[test]
    fn contains_all_and_companion_masks_match_source_not_standard_set_semantics() {
        let set = of(&[Flag::A], universe());
        assert!(set.contains_all(&[Flag::A, Flag::B]));
        assert!(!set.contains_all(&[]));
        assert!(none(universe()).contains_all(&[]));
        assert!(set.contains(&Flag::Combined));
        assert!(!set.contains(&Flag::Zero));
        assert_eq!(of_bitmap(-1, universe()).bitmap(), 7);
        assert_eq!(all(universe()).to_array(), vec![Flag::A, Flag::B, Flag::C]);
        assert_eq!(mutable_all(universe()).bitmap(), 7);
        assert_eq!(mutable_none(universe()).bitmap(), 0);
    }
    #[test]
    fn static_set_preserves_array_order_snapshot_and_bitcount_size() {
        let array = Rc::new(RefCell::new(vec![
            Flag::B,
            Flag::A,
            Flag::A,
            Flag::Combined,
            Flag::Zero,
        ]));
        let set = StaticEnumSet::new(1, array.clone());
        assert_eq!(set.size(), 1);
        assert_eq!(set.to_array(), vec![Flag::A, Flag::A, Flag::Combined]);
        let unknown = StaticEnumSet::new(8, universe());
        assert_eq!(unknown.size(), 1);
        assert!(!unknown.is_empty());
        assert!(unknown.to_array().is_empty());
        array.borrow_mut().clear();
        assert_eq!(set.to_array(), vec![Flag::A, Flag::A, Flag::Combined]);
        let other = StaticEnumSet::new(1, array);
        assert!(set == other); // Equality uses the live source array, not the filtered snapshot.
        assert_eq!(set.java_hash_code(|v| v.int()), 32);
    }
    #[test]
    fn mutable_bitmap_operations_report_changes_and_retain_source_array() {
        let array = universe();
        let mut set = mutable_of(&[Flag::A], array.clone());
        assert!(!set.add(&Flag::A));
        assert!(set.add(&Flag::B));
        assert!(set.add_all(&[Flag::C]));
        assert_eq!(set.size(), 3);
        assert!(set.remove_all_bitmap(2));
        assert_eq!(set.bitmap(), 5);
        assert!(set.retain_all(&[Flag::C]));
        assert_eq!(set.bitmap(), 4);
        assert!(!set.remove(&Flag::A));
        assert!(set.remove(&Flag::C));
        set.set_bitmap(-1);
        assert_eq!(set.size(), 32);
        set.clear();
        assert!(set.is_empty());
        assert!(Rc::ptr_eq(&set.enums(), &array));
    }
    #[test]
    fn iterator_preserves_repeated_next_remove_and_highest_bit_boundaries() {
        let mut set = MutableEnumSet::new(5, universe());
        let mut it = set.iterator();
        assert!(it.has_next());
        assert_eq!(it.next(), Ok(Flag::A));
        assert_eq!(it.next(), Ok(Flag::A));
        it.remove();
        assert!(it.has_next());
        assert_eq!(it.next(), Ok(Flag::C));
        assert!(!it.has_next());
        assert_eq!(it.next(), Ok(Flag::C));
        it.remove();
        assert_eq!(it.next(), Err(IteratorError::NullValue));
        let mut sign = MutableEnumSet::new(i32::MIN, universe());
        let mut it = sign.iterator();
        assert!(!it.has_next());
        assert_eq!(it.next(), Err(IteratorError::NullValue));
        let mut empty = MutableEnumSet::new(0, universe());
        let mut it = empty.iterator();
        assert!(!it.has_next());
        assert_eq!(it.next(), Ok(Flag::A));
    }
}

//! MutableEnumSet.java including its non-advancing iterator (confirmed in class bytecode).
use super::{
    bitmap_enum::BitmapEnum,
    enum_set::{EnumArray, EnumSet, bitmap_of},
};
use std::collections::HashMap;
pub(crate) struct MutableEnumSet<T: BitmapEnum> {
    bitmap: i32,
    enums: EnumArray<T>,
    values: HashMap<i32, T>,
}
impl<T: BitmapEnum + Clone> MutableEnumSet<T> {
    pub fn new(bitmap: i32, enums: EnumArray<T>) -> Self {
        let values = enums
            .borrow()
            .iter()
            .map(|value| (value.int(), value.clone()))
            .collect();
        Self {
            bitmap,
            enums,
            values,
        }
    }
    pub fn enums(&self) -> EnumArray<T> {
        self.enums.clone()
    }
    pub fn set_bitmap(&mut self, bitmap: i32) {
        self.bitmap = bitmap;
    }
    pub fn add(&mut self, value: &T) -> bool {
        self.add_all_bitmap(value.int())
    }
    pub fn add_all(&mut self, values: &[T]) -> bool {
        self.add_all_bitmap(bitmap_of(values))
    }
    pub fn add_all_bitmap(&mut self, other: i32) -> bool {
        let old = self.bitmap;
        self.bitmap |= other;
        old != self.bitmap
    }
    pub fn remove(&mut self, value: &T) -> bool {
        self.remove_all_bitmap(value.int())
    }
    pub fn remove_all(&mut self, values: &[T]) -> bool {
        self.remove_all_bitmap(bitmap_of(values))
    }
    pub fn remove_all_bitmap(&mut self, other: i32) -> bool {
        let old = self.bitmap;
        self.bitmap &= !other;
        old != self.bitmap
    }
    pub fn retain_all(&mut self, values: &[T]) -> bool {
        self.retain_all_bitmap(bitmap_of(values))
    }
    pub fn retain_all_bitmap(&mut self, other: i32) -> bool {
        let old = self.bitmap;
        self.bitmap &= other;
        old != self.bitmap
    }
    pub fn clear(&mut self) {
        self.bitmap = 0;
    }
    pub fn iterator(&mut self) -> MutableEnumSetIterator<'_, T> {
        let max = if self.bitmap == 0 {
            0
        } else {
            (1u32 << (31 - (self.bitmap as u32).leading_zeros())) as i32
        };
        MutableEnumSetIterator {
            set: self,
            max,
            current: 0,
        }
    }
    pub fn java_hash_code(&self, hash: impl Fn(&T) -> i32) -> i32 {
        self.enums
            .borrow()
            .iter()
            .fold(1i32, |acc, value| {
                acc.wrapping_mul(31).wrapping_add(hash(value))
            })
            .wrapping_add(self.bitmap.wrapping_mul(31))
    }
}
impl<T: BitmapEnum> EnumSet<T> for MutableEnumSet<T> {
    fn bitmap(&self) -> i32 {
        self.bitmap
    }
}
impl<T: BitmapEnum + PartialEq> PartialEq for MutableEnumSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bitmap == other.bitmap && *self.enums.borrow() == *other.enums.borrow()
    }
}
impl<T: BitmapEnum + Eq> Eq for MutableEnumSet<T> {}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum IteratorError {
    NullValue,
    NonTerminatingSourceShift,
}
/// Explicit Java-style methods: intentionally not Rust Iterator, which promises different conventions.
pub(crate) struct MutableEnumSetIterator<'a, T: BitmapEnum> {
    set: &'a mut MutableEnumSet<T>,
    max: i32,
    current: i32,
}
impl<T: BitmapEnum + Clone> MutableEnumSetIterator<'_, T> {
    pub fn has_next(&self) -> bool {
        self.current < self.max
    }
    pub fn next(&mut self) -> Result<T, IteratorError> {
        while self.current & self.set.bitmap == 0 && self.current <= self.max {
            let next = if self.current == 0 {
                1
            } else {
                self.current.wrapping_shl(1)
            };
            // Certain source masks wrap forever through zero. Report that exact pathology.
            if next == 0 {
                return Err(IteratorError::NonTerminatingSourceShift);
            }
            self.current = next;
        }
        self.set
            .values
            .get(&self.current)
            .cloned()
            .ok_or(IteratorError::NullValue)
    }
    pub fn remove(&mut self) {
        self.set.bitmap &= !self.current;
    }
}

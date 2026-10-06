//! StaticEnumSet.java: source array retention and construction-time value snapshot.
use super::{
    bitmap_enum::BitmapEnum,
    enum_set::{EnumArray, EnumSet},
};
pub(crate) struct StaticEnumSet<T: BitmapEnum> {
    bitmap: i32,
    enums: EnumArray<T>,
    values: Vec<T>,
}
impl<T: BitmapEnum + Clone> StaticEnumSet<T> {
    pub fn new(bitmap: i32, enums: EnumArray<T>) -> Self {
        let values = enums
            .borrow()
            .iter()
            .filter(|value| value.int() & bitmap != 0)
            .cloned()
            .collect();
        Self {
            bitmap,
            enums,
            values,
        }
    }
    pub fn iterator(&self) -> std::slice::Iter<'_, T> {
        if self.is_empty() {
            [].iter()
        } else {
            self.values.iter()
        }
    }
    pub fn to_array(&self) -> Vec<T> {
        self.iterator().cloned().collect()
    }
    /// JVM Arrays.hashCode with enum identity hashes supplied by the caller.
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
impl<T: BitmapEnum> EnumSet<T> for StaticEnumSet<T> {
    fn bitmap(&self) -> i32 {
        self.bitmap
    }
}
impl<T: BitmapEnum + PartialEq> PartialEq for StaticEnumSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bitmap == other.bitmap && *self.enums.borrow() == *other.enums.borrow()
    }
}
impl<T: BitmapEnum + Eq> Eq for StaticEnumSet<T> {}
impl<T: BitmapEnum + Clone + std::fmt::Display> std::fmt::Display for StaticEnumSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (index, value) in self.iterator().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{value}")?;
        }
        f.write_str("]")
    }
}

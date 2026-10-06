//! MemUtil.java array wrapping, native byte encoding and NUL-terminated string pointers.
#![allow(dead_code)]
use std::{
    ffi::{CStr, c_char},
    ops::Range,
};
#[derive(Clone, Debug)]
pub(crate) struct NativeBuffer<T> {
    storage: Vec<T>,
    position: usize,
    limit: usize,
}
impl<T> NativeBuffer<T> {
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn set_position(&mut self, position: usize) -> Result<(), String> {
        if position > self.limit {
            return Err("position exceeds limit".into());
        }
        self.position = position;
        Ok(())
    }
    pub fn set_limit(&mut self, limit: usize) -> Result<(), String> {
        if limit > self.capacity() {
            return Err("limit exceeds capacity".into());
        }
        self.limit = limit;
        self.position = self.position.min(limit);
        Ok(())
    }
    pub fn remaining_range(&self) -> Range<usize> {
        self.position..self.limit
    }
    pub fn data(&self) -> &[T] {
        &self.storage
    }
    pub fn remaining(&self) -> &[T] {
        &self.storage[self.remaining_range()]
    }
    pub fn rewind(&mut self) {
        self.position = 0;
    }
}
fn wrap<T: Clone>(values: &[T]) -> NativeBuffer<T> {
    NativeBuffer {
        storage: values.to_vec(),
        position: 0,
        limit: values.len(),
    }
}
pub(crate) fn wrap_float_buffer(values: &[f32]) -> NativeBuffer<f32> {
    wrap(values)
}
pub(crate) fn wrap_int_buffer(values: &[i32]) -> NativeBuffer<i32> {
    wrap(values)
}
pub(crate) fn wrap_byte_buffer(values: &[u8]) -> NativeBuffer<u8> {
    wrap(values)
}
pub(crate) fn wrap_float_bytes(values: &[f32]) -> NativeBuffer<u8> {
    let bytes: Vec<_> = values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    wrap(&bytes)
}
pub(crate) fn wrap_int_bytes(values: &[i32]) -> NativeBuffer<u8> {
    let bytes: Vec<_> = values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    wrap(&bytes)
}
/// # Safety
/// `names` must point to `count` readable native pointers; each must point to a
/// readable NUL-terminated UTF-8 byte string for the duration of this call.
pub(crate) unsafe fn char_array_array_to_string_array(
    count: i32,
    names: *const *const c_char,
) -> Result<Vec<String>, String> {
    if count < 0 {
        return Err("negative source pointer-buffer capacity".into());
    }
    if count > 0 && names.is_null() {
        return Err("null source string-pointer array".into());
    }
    let mut strings = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let pointer = unsafe { names.add(index).read() };
        if pointer.is_null() {
            return Err("null source string pointer".into());
        }
        strings.push(
            unsafe { CStr::from_ptr(pointer) }
                .to_string_lossy()
                .into_owned(),
        );
    }
    Ok(strings)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wraps_copy_arrays_and_preserve_capacity_position_limit_and_remaining() {
        let mut values = [1, 2, 3];
        let mut buffer = wrap_int_buffer(&values);
        values[0] = 9;
        assert_eq!(buffer.data(), [1, 2, 3]);
        assert_eq!(
            (buffer.capacity(), buffer.position(), buffer.limit()),
            (3, 0, 3)
        );
        buffer.set_position(1).unwrap();
        buffer.set_limit(2).unwrap();
        assert_eq!(buffer.remaining(), [2]);
        assert_eq!(buffer.remaining_range(), 1..2);
        assert!(buffer.set_position(3).is_err());
        assert!(buffer.set_limit(4).is_err());
        buffer.set_limit(0).unwrap();
        assert_eq!(buffer.position(), 0);
        buffer.set_limit(3).unwrap();
        buffer.set_position(2).unwrap();
        buffer.rewind();
        assert_eq!(buffer.remaining(), [1, 2, 3]);
        assert_eq!(wrap_float_buffer(&[1., 2.]).remaining(), [1., 2.]);
        assert_eq!(wrap_byte_buffer(&[0, 255]).remaining(), [0, 255]);
        assert_eq!(wrap_int_buffer(&[]).capacity(), 0);
    }
    #[test]
    fn byte_wrappers_preserve_native_order_and_float_bit_patterns() {
        let bits = [0x80000000u32, 0x7fc01234, 0x3f800000];
        let floats = bits.map(f32::from_bits);
        let expected: Vec<_> = bits.iter().flat_map(|v| v.to_ne_bytes()).collect();
        assert_eq!(wrap_float_bytes(&floats).remaining(), expected);
        let ints = [i32::MIN, -1, 0x12345678];
        let expected: Vec<_> = ints.iter().flat_map(|v| v.to_ne_bytes()).collect();
        assert_eq!(wrap_int_bytes(&ints).remaining(), expected);
    }
    #[test]
    fn native_pointer_strings_decode_in_source_pointer_order() {
        let strings = [
            std::ffi::CString::new("first").unwrap(),
            std::ffi::CString::new("水").unwrap(),
        ];
        let pointers = strings.each_ref().map(|v| v.as_ptr());
        assert_eq!(
            unsafe { char_array_array_to_string_array(2, pointers.as_ptr()) }.unwrap(),
            ["first", "水"]
        );
        assert!(
            unsafe { char_array_array_to_string_array(0, std::ptr::null()) }
                .unwrap()
                .is_empty()
        );
        assert!(unsafe { char_array_array_to_string_array(-1, std::ptr::null()) }.is_err());
    }
}

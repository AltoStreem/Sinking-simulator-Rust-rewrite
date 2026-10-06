//! Toolbox$reloadFiles$$inlined$sortedBy$1.class: compareValues of thumbnail names.
//! Java String ordering is lexicographic UTF-16 code-unit ordering, not Unicode scalar/UTF-8 order.
pub fn compare(a: &[u16], b: &[u16]) -> std::cmp::Ordering {
    a.cmp(b)
}

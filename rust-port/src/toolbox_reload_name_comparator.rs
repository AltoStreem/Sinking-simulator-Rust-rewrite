//! Toolbox$reloadFiles$$inlined$sortedBy$1.class: compareValues of thumbnail names.
//! Java String ordering is lexicographic UTF-16 code-unit ordering, not Unicode scalar/UTF-8 order.
use std::cmp::Ordering;

/// Kotlin ComparisonsKt.compareValues: nullable values order before non-null values.
pub fn compare_values<T: Ord + ?Sized>(a: Option<&T>, b: Option<&T>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b),
    }
}

/// The generated Comparator.compare bridge delegates to compareValues on the
/// selected name property. The JVM's Object-to-ShipThumbnail casts are represented
/// by this typed name projection; its non-null contract is enforced by the owner.
pub fn compare_nullable_names(a: Option<&[u16]>, b: Option<&[u16]>) -> Ordering {
    compare_values(a, b)
}

/// Normal ship browser path: ShipThumbnail.getName() is non-null in the source API.
pub fn compare(a: &[u16], b: &[u16]) -> Ordering {
    compare_nullable_names(Some(a), Some(b))
}

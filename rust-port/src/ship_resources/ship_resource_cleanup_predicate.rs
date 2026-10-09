//! `ShipResource$Companion$1$1.class`: remove dead weak references and
//! release live resources whose last image/texture access was over 10s ago.
//! Decompiled directly from the original class with CFR 0.152.

use std::sync::{Mutex, Weak};

pub(crate) fn idle_expired(now_ms: i64, last_used_ms: i64) -> bool {
    // Java long subtraction wraps; the comparison is signed and strict.
    now_ms.wrapping_sub(last_used_ms) > 10_000
}

pub(crate) fn test<T>(
    reference: &Weak<Mutex<T>>,
    now_ms: i64,
    last_used: impl FnOnce(&T) -> i64,
    free_resources: impl FnOnce(&mut T),
) -> bool {
    let Some(resource) = reference.upgrade() else {
        return true;
    };
    let mut resource = resource.lock().unwrap_or_else(|error| error.into_inner());
    if idle_expired(now_ms, last_used(&resource)) {
        free_resources(&mut resource);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn cleanup_retains_live_references_and_uses_strict_idle_threshold() {
        let resource = Arc::new(Mutex::new((100_i64, 0)));
        let weak = Arc::downgrade(&resource);
        assert!(!test(&weak, 10_100, |r| r.0, |r| r.1 += 1));
        assert_eq!(resource.lock().unwrap().1, 0);
        assert!(!test(&weak, 10_101, |r| r.0, |r| r.1 += 1));
        assert_eq!(resource.lock().unwrap().1, 1);
        drop(resource);
        assert!(test(&weak, 10_102, |r| r.0, |_| panic!("dead resource")));
    }

    #[test]
    fn clock_rollback_and_long_overflow_follow_java_arithmetic() {
        assert!(!idle_expired(1, 2));
        assert!(!idle_expired(i64::MAX, -1));
        assert!(idle_expired(i64::MIN + 10_001, i64::MAX));
    }
}

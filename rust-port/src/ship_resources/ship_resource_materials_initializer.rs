//! Recovered ShipResource$materials$2.class.
use super::resource_type::ResourceType;

pub(crate) fn invoke<T>(
    kind: impl FnOnce() -> ResourceType,
    generate: impl FnOnce() -> T,
) -> Option<T> {
    if kind() == ResourceType::Materials {
        Some(generate())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn reads_type_once_and_generates_only_for_materials_preserving_identity() {
        for kind in ResourceType::values() {
            let calls = RefCell::new(Vec::new());
            let palette = Rc::new(());
            let result = invoke(
                || {
                    calls.borrow_mut().push("type");
                    kind
                },
                || {
                    calls.borrow_mut().push("generate");
                    palette.clone()
                },
            );
            if kind == ResourceType::Materials {
                assert!(Rc::ptr_eq(&palette, &result.unwrap()));
                assert_eq!(*calls.borrow(), ["type", "generate"]);
            } else {
                assert!(result.is_none());
                assert_eq!(*calls.borrow(), ["type"]);
            }
        }
    }
}

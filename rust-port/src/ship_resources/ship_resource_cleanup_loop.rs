//! ShipResource$Companion$1: removeIf, delay 1000ms, repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Delay {
    Complete,
    Suspended,
}
pub(crate) trait Operations {
    fn remove_expired(&mut self) -> Result<(), String>;
    fn delay(&mut self, millis: i64) -> Result<Delay, String>;
}
pub(crate) struct CleanupLoop<S> {
    pub label: i32,
    scope: Option<S>,
    saved_scope: Option<S>,
}
impl<S: Clone> CleanupLoop<S> {
    pub fn new(scope: Option<S>) -> Self {
        Self {
            label: 0,
            scope,
            saved_scope: None,
        }
    }
    pub fn create(&self, scope: Option<S>) -> Self {
        Self::new(scope)
    }
    pub fn invoke_suspend(
        &mut self,
        incoming: Result<(), String>,
        operations: &mut impl Operations,
    ) -> Result<Delay, String> {
        let scope = match self.label {
            0 => {
                incoming?;
                self.scope.clone()
            }
            1 => {
                let scope = self.saved_scope.clone();
                incoming?;
                scope
            }
            _ => return Err("call to 'resume' before 'invoke' with coroutine".into()),
        };
        loop {
            operations.remove_expired()?;
            self.saved_scope = scope.clone();
            self.label = 1;
            if operations.delay(1000)? == Delay::Suspended {
                return Ok(Delay::Suspended);
            }
        }
    }
}

/// Original synchronized ArrayList.removeIf evaluates every predicate before
/// compacting the list. Predicate failure preserves the list, while resource
/// releases already performed by the predicate remain observable.
pub(crate) struct RegistryOperations<'a, T, P, D> {
    pub references: &'a mut Vec<std::sync::Weak<T>>,
    pub predicate: P,
    pub delay: D,
}
impl<T, P, D> Operations for RegistryOperations<'_, T, P, D>
where
    P: FnMut(&std::sync::Weak<T>) -> Result<bool, String>,
    D: FnMut(i64) -> Result<Delay, String>,
{
    fn remove_expired(&mut self) -> Result<(), String> {
        let removals: Result<Vec<_>, _> = self.references.iter().map(&mut self.predicate).collect();
        let removals = removals?;
        let mut index = 0;
        self.references.retain(|_| {
            let keep = !removals[index];
            index += 1;
            keep
        });
        Ok(())
    }
    fn delay(&mut self, millis: i64) -> Result<Delay, String> {
        (self.delay)(millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc, sync::Arc};
    #[test]
    fn cleanup_then_delay_resume_and_failure_match_continuation() {
        let alive = Arc::new(());
        let dead = {
            let value = Arc::new(());
            Arc::downgrade(&value)
        };
        let mut references = vec![dead, Arc::downgrade(&alive)];
        let events = Rc::new(RefCell::new(Vec::new()));
        let predicate_events = events.clone();
        let delay_events = events.clone();
        let mut immediate = 1;
        let mut ops = RegistryOperations {
            references: &mut references,
            predicate: move |reference: &std::sync::Weak<()>| {
                predicate_events.borrow_mut().push("predicate".to_owned());
                Ok(reference.upgrade().is_none())
            },
            delay: move |millis| {
                delay_events.borrow_mut().push(format!("delay:{millis}"));
                if immediate > 0 {
                    immediate -= 1;
                    Ok(Delay::Complete)
                } else {
                    Ok(Delay::Suspended)
                }
            },
        };
        let scope = Rc::new(());
        let mut callback = CleanupLoop::new(Some(scope.clone()));
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut ops).unwrap(),
            Delay::Suspended
        );
        assert_eq!(
            *events.borrow(),
            [
                "predicate",
                "predicate",
                "delay:1000",
                "predicate",
                "delay:1000"
            ]
        );
        assert!(Rc::ptr_eq(callback.saved_scope.as_ref().unwrap(), &scope));
        assert_eq!(ops.references.len(), 1);
        events.borrow_mut().clear();
        callback.invoke_suspend(Ok(()), &mut ops).unwrap();
        assert_eq!(*events.borrow(), ["predicate", "delay:1000"]);
        events.borrow_mut().clear();
        assert!(
            callback
                .invoke_suspend(Err("cancelled".into()), &mut ops)
                .is_err()
        );
        assert!(events.borrow().is_empty());
        callback.label = 2;
        assert!(
            callback
                .invoke_suspend(Ok(()), &mut ops)
                .unwrap_err()
                .contains("resume")
        );
        assert_eq!(callback.create(None).label, 0);
    }
    #[test]
    fn failed_remove_if_does_not_compact_or_delay() {
        let alive = Arc::new(());
        let mut references = vec![Arc::downgrade(&alive), Arc::downgrade(&alive)];
        let mut calls = 0;
        let mut ops = RegistryOperations {
            references: &mut references,
            predicate: |_: &std::sync::Weak<()>| {
                calls += 1;
                if calls == 1 {
                    Ok(true)
                } else {
                    Err("predicate failed".into())
                }
            },
            delay: |_| panic!("delay after failed predicate"),
        };
        let mut callback = CleanupLoop::<()>::new(None);
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut ops).unwrap_err(),
            "predicate failed"
        );
        assert_eq!(callback.label, 0);
        assert_eq!(ops.references.len(), 2);
    }
}

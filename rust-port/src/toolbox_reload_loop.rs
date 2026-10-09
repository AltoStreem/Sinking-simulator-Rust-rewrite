//! Recovered Toolbox$1.class: reload, delay one second, repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Delay {
    Complete,
    Suspended,
}
pub(crate) trait Operations {
    fn reload_files(&mut self) -> Result<(), String>;
    fn delay(&mut self, millis: i64) -> Result<Delay, String>;
}
pub(crate) struct ReloadLoop<S> {
    pub label: i32,
    scope: Option<S>,
    saved_scope: Option<S>,
}
impl<S: Clone> ReloadLoop<S> {
    pub fn new(scope: Option<S>) -> Self {
        Self {
            label: 0,
            scope,
            saved_scope: None,
        }
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
            operations.reload_files()?;
            self.saved_scope = scope.clone();
            self.label = 1;
            if operations.delay(1000)? == Delay::Suspended {
                return Ok(Delay::Suspended);
            }
        }
    }
}
pub(crate) struct NativeOperations<'a, D> {
    pub toolbox: &'a mut crate::toolbox::SourceToolbox,
    pub files: &'a mut crate::toolbox_reload_filesystem::FilesystemReloadBackend,
    pub delay: D,
}
impl<D: FnMut(i64) -> Result<Delay, String>> Operations for NativeOperations<'_, D> {
    fn reload_files(&mut self) -> Result<(), String> {
        self.toolbox.reload_files(self.files)
    }
    fn delay(&mut self, millis: i64) -> Result<Delay, String> {
        (self.delay)(millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Ops {
        events: Vec<String>,
        immediate: usize,
        fail_reload: bool,
    }
    impl Operations for Ops {
        fn reload_files(&mut self) -> Result<(), String> {
            self.events.push("reload".into());
            if self.fail_reload {
                Err("scan failed".into())
            } else {
                Ok(())
            }
        }
        fn delay(&mut self, millis: i64) -> Result<Delay, String> {
            self.events.push(format!("delay:{millis}"));
            if self.immediate == 0 {
                Ok(Delay::Suspended)
            } else {
                self.immediate -= 1;
                Ok(Delay::Complete)
            }
        }
    }
    #[test]
    fn reload_precedes_each_delay_and_resume_reloads_without_extra_delay() {
        let scope = std::rc::Rc::new(());
        let mut callback = ReloadLoop::new(Some(scope.clone()));
        let mut operations = Ops {
            events: vec![],
            immediate: 2,
            fail_reload: false,
        };
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut operations).unwrap(),
            Delay::Suspended
        );
        assert_eq!(
            operations.events,
            [
                "reload",
                "delay:1000",
                "reload",
                "delay:1000",
                "reload",
                "delay:1000"
            ]
        );
        assert!(std::rc::Rc::ptr_eq(
            callback.saved_scope.as_ref().unwrap(),
            &scope
        ));
        operations.events.clear();
        callback.invoke_suspend(Ok(()), &mut operations).unwrap();
        assert_eq!(operations.events, ["reload", "delay:1000"]);
        operations.events.clear();
        assert!(
            callback
                .invoke_suspend(Err("cancelled".into()), &mut operations)
                .is_err()
        );
        assert!(operations.events.is_empty());
    }
    #[test]
    fn initial_failure_scan_failure_and_invalid_label_preserve_source_boundaries() {
        let mut callback = ReloadLoop::<()>::new(None);
        let mut operations = Ops {
            events: vec![],
            immediate: 0,
            fail_reload: true,
        };
        assert!(
            callback
                .invoke_suspend(Err("incoming".into()), &mut operations)
                .is_err()
        );
        assert!(operations.events.is_empty());
        assert_eq!(callback.label, 0);
        assert!(callback.invoke_suspend(Ok(()), &mut operations).is_err());
        assert_eq!(operations.events, ["reload"]);
        assert_eq!(callback.label, 0);
        callback.label = 2;
        assert!(
            callback
                .invoke_suspend(Ok(()), &mut operations)
                .unwrap_err()
                .contains("resume")
        );
    }
}

//! Main$main$6.class: ten-second shutdown watchdog continuation.
//! Native launch uses a worker thread; exact GlobalScope scheduling and JVM ABI remain pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Suspended,
    Complete,
}
pub(crate) trait Operations {
    fn delay(&mut self, millis: i64) -> Result<Outcome, String>;
    fn exit(&mut self, status: i32);
}
pub(crate) struct Watchdog<S> {
    pub label: i32,
    scope: Option<S>,
    saved_scope: Option<S>,
}
impl<S: Clone> Watchdog<S> {
    pub fn new(scope: Option<S>) -> Self {
        Self {
            label: 0,
            scope,
            saved_scope: None,
        }
    }
    pub fn invoke_suspend(
        &mut self,
        result: Result<(), String>,
        ops: &mut impl Operations,
    ) -> Result<Outcome, String> {
        match self.label {
            0 => {
                result?;
                self.saved_scope = self.scope.clone();
                self.label = 1;
                if ops.delay(10_000)? == Outcome::Suspended {
                    return Ok(Outcome::Suspended);
                }
            }
            1 => {
                let _scope = self.saved_scope.clone();
                result?;
            }
            _ => return Err("call to 'resume' before 'invoke' with coroutine".into()),
        }
        ops.exit(76);
        Ok(Outcome::Complete)
    }
}
pub(crate) fn launch() -> std::io::Result<std::thread::JoinHandle<()>> {
    launch_with(
        || std::thread::sleep(std::time::Duration::from_millis(10_000)),
        |status| std::process::exit(status),
    )
}
pub(crate) fn launch_with(
    delay: impl FnOnce() + Send + 'static,
    exit: impl FnOnce(i32) + Send + 'static,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("SS2 shutdown watchdog".into())
        .spawn(move || {
            delay();
            exit(76);
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Ops {
        events: Vec<String>,
        delay: Outcome,
    }
    impl Operations for Ops {
        fn delay(&mut self, millis: i64) -> Result<Outcome, String> {
            self.events.push(format!("delay:{millis}"));
            Ok(self.delay)
        }
        fn exit(&mut self, status: i32) {
            self.events.push(format!("exit:{status}"));
        }
    }
    #[test]
    fn suspension_saves_scope_and_only_resume_exits() {
        let scope = std::rc::Rc::new(());
        let mut callback = Watchdog::new(Some(scope.clone()));
        let mut ops = Ops {
            events: vec![],
            delay: Outcome::Suspended,
        };
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut ops),
            Ok(Outcome::Suspended)
        );
        assert_eq!(callback.label, 1);
        assert!(std::rc::Rc::ptr_eq(
            callback.saved_scope.as_ref().unwrap(),
            &scope
        ));
        assert_eq!(ops.events, ["delay:10000"]);
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut ops),
            Ok(Outcome::Complete)
        );
        assert_eq!(ops.events, ["delay:10000", "exit:76"]);
    }
    #[test]
    fn immediate_delay_completion_and_failure_boundaries_follow_original_labels() {
        let mut callback = Watchdog::<()>::new(None);
        let mut ops = Ops {
            events: vec![],
            delay: Outcome::Complete,
        };
        assert_eq!(
            callback.invoke_suspend(Err("incoming".into()), &mut ops),
            Err("incoming".into())
        );
        assert_eq!(callback.label, 0);
        assert!(ops.events.is_empty());
        assert_eq!(
            callback.invoke_suspend(Ok(()), &mut ops),
            Ok(Outcome::Complete)
        );
        assert_eq!(ops.events, ["delay:10000", "exit:76"]);
        ops.events.clear();
        assert_eq!(
            callback.invoke_suspend(Err("cancelled".into()), &mut ops),
            Err("cancelled".into())
        );
        assert!(ops.events.is_empty());
        callback.label = 2;
        assert!(
            callback
                .invoke_suspend(Ok(()), &mut ops)
                .unwrap_err()
                .contains("resume")
        );
        assert!(ops.events.is_empty());
    }
    #[test]
    fn worker_launcher_delays_then_exits_without_blocking_caller() {
        let main = std::thread::current().id();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let delayed = events.clone();
        let exited = events.clone();
        launch_with(
            move || {
                assert_ne!(std::thread::current().id(), main);
                delayed.lock().unwrap().push(10000);
            },
            move |status| exited.lock().unwrap().push(status),
        )
        .unwrap()
        .join()
        .unwrap();
        assert_eq!(*events.lock().unwrap(), [10000, 76]);
    }
}

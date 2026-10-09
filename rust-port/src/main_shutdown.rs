//! Main.java after Window.start: close GLFW, launch watchdog, closeAll, runMain.
pub(crate) trait Operations {
    fn close_glfw(&mut self);
    fn launch_watchdog(&mut self) -> Result<(), String>;
    fn close_all(&mut self);
    fn run_main(&mut self);
}
pub(crate) fn finish(ops: &mut impl Operations) -> Result<(), String> {
    ops.close_glfw();
    ops.launch_watchdog()?;
    ops.close_all();
    ops.run_main();
    Ok(())
}
pub(crate) struct NativeOperations<'a> {
    pub glfw: &'a crate::glfw::Glfw,
    pub runtime: &'a crate::resource::ResourceRuntime,
    pub launch: Box<dyn FnMut() -> Result<(), String> + 'a>,
}
impl<'a> NativeOperations<'a> {
    pub fn new(glfw: &'a crate::glfw::Glfw, runtime: &'a crate::resource::ResourceRuntime) -> Self {
        Self {
            glfw,
            runtime,
            launch: Box::new(|| {
                // Discarding the handle detaches the watchdog, as source ignores its Job.
                crate::main_shutdown_watchdog::launch()
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            }),
        }
    }
}
impl Operations for NativeOperations<'_> {
    fn close_glfw(&mut self) {
        self.glfw.close();
    }
    fn launch_watchdog(&mut self) -> Result<(), String> {
        (self.launch)()
    }
    fn close_all(&mut self) {
        self.runtime.close_all();
    }
    fn run_main(&mut self) {
        self.runtime.run_main();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Ops {
        events: Vec<&'static str>,
        fail_launch: bool,
    }
    impl Operations for Ops {
        fn close_glfw(&mut self) {
            self.events.push("glfw.close");
        }
        fn launch_watchdog(&mut self) -> Result<(), String> {
            self.events.push("launch");
            if self.fail_launch {
                Err("launch failure".into())
            } else {
                Ok(())
            }
        }
        fn close_all(&mut self) {
            self.events.push("closeAll");
        }
        fn run_main(&mut self) {
            self.events.push("runMain");
        }
    }
    #[test]
    fn cleanup_order_and_launch_failure_keep_original_partial_effects() {
        let mut ops = Ops {
            events: vec![],
            fail_launch: false,
        };
        finish(&mut ops).unwrap();
        assert_eq!(ops.events, ["glfw.close", "launch", "closeAll", "runMain"]);
        ops.events.clear();
        ops.fail_launch = true;
        assert_eq!(finish(&mut ops), Err("launch failure".into()));
        assert_eq!(ops.events, ["glfw.close", "launch"]);
    }
}

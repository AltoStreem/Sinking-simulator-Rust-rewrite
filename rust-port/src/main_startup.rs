//! Main.java startup order, excluding Steam operations by user instruction.
//! Backend methods retain constructed objects for later phases; native platform
//! composition and JVM exception boundaries remain pending.
pub(crate) trait Backend {
    fn logging(&mut self) -> Result<i64, String>;
    fn clock(&mut self) -> i64;
    fn print(&mut self, text: String);
    fn glfw(&mut self) -> Result<(), String>;
    fn window(&mut self, title: &str) -> Result<(), String>;
    fn icons(&mut self, paths: &[&str]) -> Result<(), String>;
    fn counter(&mut self) -> Result<(), String>;
    fn debug(&mut self, args: &[String]) -> Result<(), String>;
    fn audio_context_start(&mut self) -> Result<(), String>;
    fn empty_tracks_player_publish(&mut self) -> Result<(), String>;
    fn volume(&mut self, value: f32);
    fn launch_music(&mut self) -> Result<(), String>;
    fn camera(&mut self) -> Result<(), String>;
    fn materials_publish(&mut self, path: &str) -> Result<(), String>;
    fn sky(&mut self, path: &str) -> Result<(), String>;
    fn initial_ship_publish_append(&mut self, path: &str) -> Result<(), String>;
    fn gui(&mut self) -> Result<(), String>;
    fn gui_handle(&mut self);
    fn register_input_stacks(&mut self);
    fn screen_fbo(&mut self, depth: bool, stencil: bool) -> Result<(), String>;
    fn sea(&mut self) -> Result<(), String>;
    fn floor(&mut self) -> Result<(), String>;
}
fn elapsed(backend: &mut impl Backend, base: i64, phase: &str) {
    let millis = backend.clock().wrapping_sub(base);
    backend.print(format!("{phase} done after {millis} millis"));
}
pub(crate) fn initialize(args: &[String], backend: &mut impl Backend) -> Result<i64, String> {
    let base = backend.logging()?;
    backend.glfw()?;
    backend.window("Loading...")?;
    backend.icons(&["ico32.png", "ico64.png", "ico128.png"])?;
    backend.counter()?;
    backend.debug(args)?;
    elapsed(backend, base, "GL");
    backend.audio_context_start()?;
    backend.empty_tracks_player_publish()?;
    backend.volume(0.25);
    backend.launch_music()?;
    backend.camera()?;
    backend.materials_publish("config/materials.json")?;
    elapsed(backend, base, "Materials");
    backend.sky("config/sky.png")?;
    let start = backend.clock();
    backend.initial_ship_publish_append("ships/pacmaster")?;
    let seconds = backend.clock().wrapping_sub(start) as f32 / 1000.0;
    backend.print(format!(
        "Ship done in {}",
        crate::java_string::java_float_to_string(seconds)
    ));
    backend.gui()?;
    elapsed(backend, base, "GUI");
    backend.gui_handle();
    elapsed(backend, base, "GUI init");
    backend.register_input_stacks();
    backend.screen_fbo(true, true)?;
    elapsed(backend, base, "FB");
    backend.sea()?;
    backend.floor()?;
    elapsed(backend, base, "Init");
    Ok(base)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Recorder {
        events: Vec<String>,
        time: i64,
        fail: Option<&'static str>,
    }
    impl Recorder {
        fn event(&mut self, text: impl Into<String>) -> Result<(), String> {
            let text = text.into();
            let fails = self.fail.is_some_and(|phase| text == phase);
            self.events.push(text);
            if fails {
                Err("startup failure".into())
            } else {
                Ok(())
            }
        }
    }
    impl Backend for Recorder {
        fn logging(&mut self) -> Result<i64, String> {
            self.event("logging")?;
            self.time = 1000;
            Ok(self.time)
        }
        fn clock(&mut self) -> i64 {
            self.time += 100;
            self.time
        }
        fn print(&mut self, text: String) {
            self.events.push(text);
        }
        fn glfw(&mut self) -> Result<(), String> {
            self.event("glfw")
        }
        fn window(&mut self, title: &str) -> Result<(), String> {
            self.event(format!("window:{title}"))
        }
        fn icons(&mut self, paths: &[&str]) -> Result<(), String> {
            self.event(format!("icons:{}", paths.join(",")))
        }
        fn counter(&mut self) -> Result<(), String> {
            self.event("counter")
        }
        fn debug(&mut self, args: &[String]) -> Result<(), String> {
            self.event(format!("debug:{}", args.join(",")))
        }
        fn audio_context_start(&mut self) -> Result<(), String> {
            self.event("audio")
        }
        fn empty_tracks_player_publish(&mut self) -> Result<(), String> {
            self.event("emptyTracks/player/publish")
        }
        fn volume(&mut self, value: f32) {
            self.events.push(format!("volume:{value}"));
        }
        fn launch_music(&mut self) -> Result<(), String> {
            self.event("launchMusic")
        }
        fn camera(&mut self) -> Result<(), String> {
            self.event("camera")
        }
        fn materials_publish(&mut self, path: &str) -> Result<(), String> {
            self.event(format!("materials:{path}"))
        }
        fn sky(&mut self, path: &str) -> Result<(), String> {
            self.event(format!("sky:{path}"))
        }
        fn initial_ship_publish_append(&mut self, path: &str) -> Result<(), String> {
            self.event(format!("ship:{path}"))
        }
        fn gui(&mut self) -> Result<(), String> {
            self.event("gui")
        }
        fn gui_handle(&mut self) {
            self.events.push("gui.handle".into());
        }
        fn register_input_stacks(&mut self) {
            self.events.push("inputStacks".into());
        }
        fn screen_fbo(&mut self, depth: bool, stencil: bool) -> Result<(), String> {
            self.event(format!("screen:{depth}:{stencil}"))
        }
        fn sea(&mut self) -> Result<(), String> {
            self.event("sea")
        }
        fn floor(&mut self) -> Result<(), String> {
            self.event("floor")
        }
    }
    #[test]
    fn initialization_order_paths_options_volume_and_timing_match_source() {
        let mut backend = Recorder::default();
        assert_eq!(initialize(&["-debug".into()], &mut backend), Ok(1000));
        assert_eq!(
            backend.events,
            [
                "logging",
                "glfw",
                "window:Loading...",
                "icons:ico32.png,ico64.png,ico128.png",
                "counter",
                "debug:-debug",
                "GL done after 100 millis",
                "audio",
                "emptyTracks/player/publish",
                "volume:0.25",
                "launchMusic",
                "camera",
                "materials:config/materials.json",
                "Materials done after 200 millis",
                "sky:config/sky.png",
                "ship:ships/pacmaster",
                "Ship done in 0.1",
                "gui",
                "GUI done after 500 millis",
                "gui.handle",
                "GUI init done after 600 millis",
                "inputStacks",
                "screen:true:true",
                "FB done after 700 millis",
                "sea",
                "floor",
                "Init done after 800 millis"
            ]
        );
    }
    #[test]
    fn construction_failures_stop_at_each_original_phase_without_rollback() {
        let mut success = Recorder::default();
        initialize(&[], &mut success).unwrap();
        for phase in [
            "logging",
            "glfw",
            "window:Loading...",
            "counter",
            "debug:",
            "audio",
            "emptyTracks/player/publish",
            "launchMusic",
            "camera",
            "materials:config/materials.json",
            "sky:config/sky.png",
            "ship:ships/pacmaster",
            "gui",
            "screen:true:true",
            "sea",
            "floor",
        ] {
            let mut backend = Recorder {
                fail: Some(phase),
                ..Default::default()
            };
            assert_eq!(initialize(&[], &mut backend), Err("startup failure".into()));
            let end = success
                .events
                .iter()
                .position(|event| event == phase)
                .unwrap();
            assert_eq!(backend.events, success.events[..=end]);
        }
    }
}

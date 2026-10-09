//! Main.java's Window.start draw body. Steam operations are excluded by request.
//! Window.start owns the outer clear, swap and event polling; this is its draw callback.
#![allow(dead_code)]
use crate::game_parameters::SourceGameParameterProvider;
use std::{cell::RefCell, rc::Rc};

pub(crate) trait MainFrameBackend {
    fn resources(&mut self);
    fn music(&mut self);
    fn gui_handle(&mut self);
    fn screen_size(&mut self) -> [i32; 2];
    fn parameters(&mut self) -> Rc<RefCell<SourceGameParameterProvider>>;
    fn ship_time(&mut self, time: f32);
    fn ship_update(&mut self);
    fn scene(&mut self);
    fn sea_time(&mut self, time: f32);
    fn sea_render(&mut self);
    fn gui_render(&mut self);
    fn counter(&mut self);
}

/// Fresh global getters and Java float operation order are intentional.
pub(crate) fn draw(backend: &mut impl MainFrameBackend) -> bool {
    backend.resources();
    backend.music();
    backend.gui_handle();
    let size = backend.screen_size();
    if size[0] == 0 || size[1] == 0 {
        return false;
    }
    let time = backend.parameters().borrow().time();
    let cycle = backend.parameters().borrow().daycycle();
    if cycle {
        let length = backend.parameters().borrow().cycle_length();
        let angle = time * std::f32::consts::PI * 2.0 / length % (std::f32::consts::PI * 2.0);
        let receiver = backend.parameters();
        let cosine = (angle as f64).cos() as f32;
        receiver.borrow_mut().set_day(cosine * 0.5 + 0.5);
    }
    backend.ship_time(time);
    backend.ship_update();
    backend.scene();
    backend.sea_time(time);
    backend.sea_render();
    backend.gui_render();
    backend.counter();
    // The inline callback captures one provider for both getTime and setTime.
    let receiver = backend.parameters();
    let next = receiver.borrow().time() + 0.016666668;
    receiver.borrow_mut().set_time(next);
    true
}

pub(crate) trait MainGuiFrame {
    fn handle(&mut self);
    fn render(&mut self);
}
impl<B: crate::gui::GuiBackend, R: crate::gui::ResetShipOperations> MainGuiFrame
    for crate::gui::SourceGui<B, R>
{
    fn handle(&mut self) {
        crate::gui::SourceGui::handle(self);
    }
    fn render(&mut self) {
        crate::gui::SourceGui::render(self);
    }
}

/// Native composition of the translated scene classes, with late Main globals.
pub(crate) struct NativeMainFrame {
    pub resources: crate::resource::ResourceRuntime,
    pub player: Rc<dyn Fn() -> Rc<RefCell<crate::music_player::SourceMusicPlayer>>>,
    pub gui: Rc<RefCell<dyn MainGuiFrame>>,
    pub window: Rc<crate::window::SourceWindow>,
    pub ship: Rc<dyn Fn() -> Rc<RefCell<crate::source_ship::SourceShip>>>,
    pub screen: Rc<crate::screen_fbo::SourceScreenFbo>,
    pub state: Rc<RefCell<dyn crate::source_ship::ShipSceneStateBackend>>,
    pub sky: Rc<crate::sky::SourceSky>,
    pub floor: Rc<crate::floor::SourceFloor>,
    pub sea: Rc<crate::sea::SourceSea>,
    pub counter: Rc<crate::time_sync::SourceTimeSync>,
}
impl NativeMainFrame {
    pub fn use_main_globals(&mut self) {
        self.ship = Rc::new(crate::main_globals::get_global_ship);
        self.player = Rc::new(crate::main_globals::get_player);
    }
    pub fn callback(frame: Rc<RefCell<Self>>) -> Rc<dyn Fn(&crate::window::SourceWindow)> {
        Rc::new(move |_| {
            draw(&mut *frame.borrow_mut());
        })
    }
}
impl MainFrameBackend for NativeMainFrame {
    fn resources(&mut self) {
        self.resources.run_main();
    }
    fn music(&mut self) {
        (self.player)().borrow_mut().update();
    }
    fn gui_handle(&mut self) {
        self.gui.borrow_mut().handle();
    }
    fn screen_size(&mut self) -> [i32; 2] {
        self.window.screen_size()
    }
    fn parameters(&mut self) -> Rc<RefCell<SourceGameParameterProvider>> {
        crate::game_parameter_provider_kt::get_game_parameter_provider()
    }
    fn ship_time(&mut self, time: f32) {
        (self.ship)().borrow().set_time(time);
    }
    fn ship_update(&mut self) {
        (self.ship)()
            .borrow()
            .update()
            .unwrap_or_else(|e| panic!("{e}"));
    }
    fn scene(&mut self) {
        use crate::i_drawable::IDrawable;
        self.screen.framebuffer.borrow().framebuffer.draw(|| {
            self.state.borrow_mut().clear(16384);
            self.sky.render();
            (self.ship)().borrow().render();
            self.floor.render();
        });
    }
    fn sea_time(&mut self, time: f32) {
        self.sea.set_time(time);
    }
    fn sea_render(&mut self) {
        self.sea.render();
    }
    fn gui_render(&mut self) {
        self.gui.borrow_mut().render();
    }
    fn counter(&mut self) {
        self.counter.update();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend {
        log: Vec<String>,
        size: [i32; 2],
        provider: Rc<RefCell<SourceGameParameterProvider>>,
        replacement: Option<(usize, Rc<RefCell<SourceGameParameterProvider>>)>,
        gets: usize,
        fail: bool,
    }
    impl Backend {
        fn new() -> Self {
            Self {
                log: vec![],
                size: [400, 200],
                provider: Rc::new(RefCell::new(SourceGameParameterProvider::default())),
                replacement: None,
                gets: 0,
                fail: false,
            }
        }
        fn record(&mut self, name: &str) {
            self.log.push(name.into());
        }
    }
    impl MainFrameBackend for Backend {
        fn resources(&mut self) {
            self.record("resources");
        }
        fn music(&mut self) {
            self.record("music");
        }
        fn gui_handle(&mut self) {
            self.record("gui-handle");
        }
        fn screen_size(&mut self) -> [i32; 2] {
            self.record("size");
            self.size
        }
        fn parameters(&mut self) -> Rc<RefCell<SourceGameParameterProvider>> {
            self.gets += 1;
            self.record("parameters");
            if let Some((at, next)) = &self.replacement {
                if *at == self.gets {
                    self.provider = next.clone();
                }
            }
            self.provider.clone()
        }
        fn ship_time(&mut self, t: f32) {
            self.log.push(format!("ship-time:{t}"));
        }
        fn ship_update(&mut self) {
            self.record("ship-update");
            if self.fail {
                panic!("physics");
            }
        }
        fn scene(&mut self) {
            self.record("scene");
        }
        fn sea_time(&mut self, t: f32) {
            self.log.push(format!("sea-time:{t}"));
        }
        fn sea_render(&mut self) {
            self.record("sea-render");
        }
        fn gui_render(&mut self) {
            self.record("gui-render");
        }
        fn counter(&mut self) {
            self.record("counter");
        }
    }
    #[test]
    fn frame_order_clock_and_minimized_gate_follow_main() {
        let mut b = Backend::new();
        b.provider.borrow_mut().set_time(30.);
        assert!(draw(&mut b));
        assert_eq!(
            b.log,
            [
                "resources",
                "music",
                "gui-handle",
                "size",
                "parameters",
                "parameters",
                "parameters",
                "parameters",
                "ship-time:30",
                "ship-update",
                "scene",
                "sea-time:30",
                "sea-render",
                "gui-render",
                "counter",
                "parameters"
            ]
        );
        let angle = 30.0_f32 * std::f32::consts::PI * 2.0 / 120.0 % (std::f32::consts::PI * 2.0);
        assert_eq!(
            b.provider.borrow().day(),
            (angle as f64).cos() as f32 * 0.5 + 0.5
        );
        assert_eq!(b.provider.borrow().time(), 30.0_f32 + 0.016666668);
        for size in [[0, 200], [400, 0], [0, 0]] {
            b.size = size;
            b.log.clear();
            let time = b.provider.borrow().time();
            assert!(!draw(&mut b));
            assert_eq!(b.log, ["resources", "music", "gui-handle", "size"]);
            assert_eq!(b.provider.borrow().time(), time);
        }
        b.size = [-1, -2];
        b.log.clear();
        assert!(draw(&mut b)); // Signed nonzero dimensions are not clamped.
    }
    #[test]
    fn clock_increment_reads_the_captured_receiver_and_no_cycle_skips_day() {
        let mut b = Backend::new();
        let original = b.provider.clone();
        original.borrow_mut().set_daycycle(false);
        original.borrow_mut().set_time(2.);
        original.borrow_mut().set_day(0.25);
        let next = Rc::new(RefCell::new(SourceGameParameterProvider::default()));
        next.borrow_mut().set_time(100.);
        b.replacement = Some((4, next.clone())); // Source never performs a fourth lookup.
        draw(&mut b);
        assert_eq!(original.borrow().day(), 0.25);
        assert_eq!(original.borrow().time(), 2.0_f32 + 0.016666668);
        assert_eq!(next.borrow().time(), 100.);
        assert_eq!(b.gets, 3);
        assert!(Rc::ptr_eq(&b.provider, &original));

        // Replacing the provider at the final capture updates that new object
        // using its own time while ship/sea retain this frame's earlier time.
        b.replacement = Some((b.gets + 3, next.clone()));
        b.log.clear();
        draw(&mut b);
        assert_eq!(next.borrow().time(), 100.0_f32 + 0.016666668);
        assert_eq!(original.borrow().time(), 2.0_f32 + 0.016666668);
    }
    #[test]
    fn invalid_cycle_values_propagate_and_physics_failure_stops_remaining_frame() {
        let mut b = Backend::new();
        b.provider.borrow_mut().set_cycle_length(0.);
        b.fail = true;
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| draw(&mut b))).is_err());
        assert!(b.provider.borrow().day().is_nan());
        assert_eq!(b.provider.borrow().time(), 0.);
        assert_eq!(b.log.last().unwrap(), "ship-update");
        assert!(!b.log.iter().any(|s| s == "scene"));
    }
}

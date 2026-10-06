//! GUI.java input routing and frame/render methods through required native GUI/ship adapters.
//! Original construction order is translated through required GUI/tool adapters; concrete native wiring remains pending.
use crate::{
    camera_control::SourceCameraControl, gui_kt::GuiKt, input_handler::InputHandler,
    texture_2d::SourceTexture2D, tools::tool::SourceTool, window::SourceWindow,
};
use std::{any::Any, cell::RefCell, rc::Rc};
pub type GuiTool = RefCell<dyn SourceTool<SourceWindow, Texture = SourceTexture2D>>;
pub trait GuiBackend {
    type Font;
    type DrawData;
    fn mouse_button_callback(&mut self, window: i64, button: i32, action: i32, mods: i32);
    fn cursor_pos_callback(&mut self, window: i64, x: f64, y: f64);
    fn cursor_enter_callback(&mut self, window: i64, enter: bool);
    fn scroll_callback(&mut self, window: i64, x: f64, y: f64);
    fn char_callback(&mut self, window: i64, codepoint: i32);
    fn key_callback(&mut self, window: i64, key: i32, scancode: i32, action: i32, mods: i32);
    fn want_capture_mouse(&mut self) -> bool;
    fn want_capture_keyboard(&mut self) -> bool;
    fn check_error(&mut self, message: &str, fatal: bool);
    fn glfw_new_frame(&mut self);
    fn gl3_new_frame(&mut self);
    fn imgui_new_frame(&mut self);
    fn toolbox_render(&mut self);
    fn imgui_render(&mut self);
    fn draw_data(&mut self) -> Option<Self::DrawData>;
    fn put_framebuffer_scale(&mut self, data: &mut Self::DrawData, x: f32, y: f32);
    fn render_draw_data(&mut self, data: Self::DrawData);
    fn print_throwable(&mut self, payload: &(dyn Any + Send));
    fn gl3_shutdown(&mut self);
    fn glfw_shutdown(&mut self);
    fn context_destroy(&mut self);
}
/// Must allocate native objects only when the corresponding creation method is called.
pub trait GuiConstructionBackend: GuiBackend {
    fn create_context_default(&mut self);
    fn wrap_glfw_window(&mut self, window: i64);
    fn create_glfw_implementation(&mut self, install_callbacks: bool);
    fn create_gl3_implementation_default(&mut self);
    /// Resolve Window.getGlfw().getMonitor().getScale(), preserving its cached mutable scale.
    fn monitor_scale(&mut self, window: &SourceWindow) -> [f32; 2];
    fn scale_all_style_sizes(&mut self, scale: f32);
    fn set_font_global_scale(&mut self, scale: f32);
    /// Config and glyph ranges are the source defaults (null).
    fn add_font_from_file_ttf(&mut self, path: &str, size: f32) -> Option<Rc<Self::Font>>;
    fn create_toolbox(&mut self);
}
pub trait GuiToolFactory {
    fn break_tool(
        &mut self,
        control: Rc<RefCell<SourceCameraControl>>,
        provider: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
    ) -> Result<Rc<GuiTool>, String>;
    fn flood_tool(
        &mut self,
        control: Rc<RefCell<SourceCameraControl>>,
        provider: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
    ) -> Result<Rc<GuiTool>, String>;
    fn dry_tool(
        &mut self,
        control: Rc<RefCell<SourceCameraControl>>,
        provider: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
    ) -> Result<Rc<GuiTool>, String>;
    fn move_tool(&mut self) -> Result<Rc<GuiTool>, String>;
}
pub trait ResetShipOperations {
    type Ship;
    type Thumbnail;
    fn global_ship(&mut self) -> Rc<Self::Ship>;
    fn thumbnail(&mut self, ship: &Self::Ship) -> Self::Thumbnail;
    fn close_ship(&mut self, ship: &Self::Ship);
    fn construct_ship(
        &mut self,
        thumbnail: Self::Thumbnail,
        control: Rc<RefCell<SourceCameraControl>>,
    ) -> Rc<Self::Ship>;
    fn set_global_ship(&mut self, ship: Rc<Self::Ship>);
    fn clear_ship_list(&mut self);
    fn add_ship(&mut self, ship: Rc<Self::Ship>);
}
pub struct SourceGui<B: GuiBackend, R: ResetShipOperations> {
    window: Rc<SourceWindow>,
    camera_control: Rc<RefCell<SourceCameraControl>>,
    globals: Rc<RefCell<GuiKt<GuiTool, B::Font>>>,
    backend: Rc<RefCell<B>>,
    reset: R,
    fbscale: f32,
    fontscale: Option<f32>,
    resource: Option<crate::resource::ResourceHandle>,
}
impl<B: GuiBackend, R: ResetShipOperations> SourceGui<B, R> {
    pub fn from_parts(
        window: Rc<SourceWindow>,
        camera_control: Rc<RefCell<SourceCameraControl>>,
        globals: Rc<RefCell<GuiKt<GuiTool, B::Font>>>,
        backend: B,
        reset: R,
        fbscale: f32,
    ) -> Self {
        Self {
            window,
            camera_control,
            globals,
            backend: Rc::new(RefCell::new(backend)),
            reset,
            fbscale,
            fontscale: None,
            resource: None,
        }
    }
    pub fn resource(&self) -> crate::resource::ResourceHandle {
        self.resource
            .as_ref()
            .expect("GUI resource absent in assembly path")
            .clone()
    }
    pub fn close(&self) {
        self.resource().close();
    }
    pub fn freed(&self) -> bool {
        self.resource().freed()
    }
    pub fn window(&self) -> Rc<SourceWindow> {
        self.window.clone()
    }
    pub fn camera_control(&self) -> Rc<RefCell<SourceCameraControl>> {
        self.camera_control.clone()
    }
    pub fn handlers(&self) -> Vec<Rc<GuiTool>> {
        let list = self.globals.borrow().tool_list();
        let handlers = list.borrow().iter().filter_map(Clone::clone).collect();
        handlers
    }
    fn broadcast(
        &self,
        mut invoke: impl FnMut(&mut dyn SourceTool<SourceWindow, Texture = SourceTexture2D>) -> bool,
    ) -> bool {
        // Do not short circuit: every original handler receives the same blocked argument.
        let results: Vec<_> = self
            .handlers()
            .iter()
            .map(|tool| invoke(&mut *tool.borrow_mut()))
            .collect();
        results.into_iter().any(|result| result)
    }
    pub fn handle(&mut self) {
        self.backend.borrow_mut().check_error("GUI handle", false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.backend.borrow_mut().glfw_new_frame();
            self.backend.borrow_mut().gl3_new_frame();
            self.backend.borrow_mut().imgui_new_frame();
            self.backend.borrow_mut().toolbox_render();
            self.backend.borrow_mut().imgui_render();
        }));
        if let Err(payload) = result {
            self.backend.borrow_mut().print_throwable(payload.as_ref());
        }
    }
    pub fn render(&mut self) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let tool = self.globals.borrow().current_tool();
            if let Some(tool) = tool {
                tool.borrow_mut().update();
            }
            let draw_data = self.backend.borrow_mut().draw_data();
            if let Some(mut data) = draw_data {
                self.backend.borrow_mut().put_framebuffer_scale(
                    &mut data,
                    self.fbscale,
                    self.fbscale,
                );
                self.backend.borrow_mut().render_draw_data(data);
            }
        }));
        if let Err(payload) = result {
            self.backend.borrow_mut().print_throwable(payload.as_ref());
        }
    }
    pub fn free(&mut self) {
        self.backend.borrow_mut().gl3_shutdown();
        self.backend.borrow_mut().glfw_shutdown();
        self.backend.borrow_mut().context_destroy();
    }
}
impl<B: GuiBackend, R: ResetShipOperations> InputHandler<SourceWindow> for SourceGui<B, R> {
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        win: &SourceWindow,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        self.backend
            .borrow_mut()
            .mouse_button_callback(win.id, button, action, mods);
        if self.backend.borrow_mut().want_capture_mouse() {
            true
        } else {
            self.broadcast(|h| h.on_mouse_button(blocked, win, button, action, mods))
        }
    }
    fn on_cursor_pos(&mut self, blocked: bool, win: &SourceWindow, x: f64, y: f64) -> bool {
        self.backend.borrow_mut().cursor_pos_callback(win.id, x, y);
        if self.backend.borrow_mut().want_capture_mouse() {
            true
        } else {
            self.broadcast(|h| h.on_cursor_pos(blocked, win, x, y))
        }
    }
    fn on_cursor_enter(&mut self, blocked: bool, win: &SourceWindow, enter: bool) -> bool {
        self.backend
            .borrow_mut()
            .cursor_enter_callback(win.id, enter);
        if self.backend.borrow_mut().want_capture_mouse() {
            true
        } else {
            self.broadcast(|h| h.on_cursor_enter(blocked, win, enter))
        }
    }
    fn on_scroll(&mut self, blocked: bool, win: &SourceWindow, x: f64, y: f64) -> bool {
        self.backend.borrow_mut().scroll_callback(win.id, x, y);
        if self.backend.borrow_mut().want_capture_mouse() {
            true
        } else {
            self.broadcast(|h| h.on_scroll(blocked, win, x, y))
        }
    }
    fn on_char(&mut self, blocked: bool, win: &SourceWindow, codepoint: i32) -> bool {
        self.backend.borrow_mut().char_callback(win.id, codepoint);
        if self.backend.borrow_mut().want_capture_keyboard() {
            true
        } else {
            self.broadcast(|h| h.on_char(blocked, win, codepoint))
        }
    }
    fn on_char_mods(
        &mut self,
        blocked: bool,
        win: &SourceWindow,
        codepoint: i32,
        mods: i32,
    ) -> bool {
        if mods & 1 == 1 {
            let c = codepoint as u16;
            if c == b'r' as u16 || c == b'R' as u16 {
                let ship = self.reset.global_ship();
                let thumbnail = self.reset.thumbnail(&ship);
                let ship = self.reset.global_ship();
                self.reset.close_ship(&ship);
                let replacement = self
                    .reset
                    .construct_ship(thumbnail, self.camera_control.clone());
                self.reset.set_global_ship(replacement);
                self.reset.clear_ship_list();
                let ship = self.reset.global_ship();
                self.reset.add_ship(ship);
            }
            true
        } else if self.backend.borrow_mut().want_capture_keyboard() {
            true
        } else {
            self.broadcast(|h| h.on_char_mods(blocked, win, codepoint, mods))
        }
    }
    fn on_key(
        &mut self,
        blocked: bool,
        win: &SourceWindow,
        key: i32,
        scancode: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        self.backend
            .borrow_mut()
            .key_callback(win.id, key, scancode, action, mods);
        if self.backend.borrow_mut().want_capture_keyboard() {
            return true;
        }
        if action == 1 && key > 48 && {
            let count = self.globals.borrow().tool_list().borrow().len() as i32;
            key < 48i32.wrapping_add(count)
        } {
            let index = key.wrapping_sub(48);
            let mut globals = self.globals.borrow_mut();
            let current = globals.current_tool_index();
            globals.set_current_tool_index(if current == index { 0 } else { index });
            true
        } else {
            self.broadcast(|h| h.on_key(blocked, win, key, scancode, action, mods))
        }
    }
    fn on_position(&mut self, blocked: bool, win: &SourceWindow, y: i32, x: i32) -> bool {
        self.broadcast(|h| h.on_position(blocked, win, y, x))
    }
    fn on_size(&mut self, blocked: bool, win: &SourceWindow, height: i32, width: i32) -> bool {
        self.broadcast(|h| h.on_size(blocked, win, height, width))
    }
    fn on_close(&mut self, blocked: bool, win: &SourceWindow) -> bool {
        self.broadcast(|h| h.on_close(blocked, win))
    }
    fn on_refresh(&mut self, blocked: bool, win: &SourceWindow) -> bool {
        self.broadcast(|h| h.on_refresh(blocked, win))
    }
    fn on_focus(&mut self, blocked: bool, win: &SourceWindow, focus: bool) -> bool {
        self.broadcast(|h| h.on_focus(blocked, win, focus))
    }
    fn on_iconify(&mut self, blocked: bool, win: &SourceWindow, iconify: bool) -> bool {
        self.broadcast(|h| h.on_iconify(blocked, win, iconify))
    }
    fn on_framebuffer_size(
        &mut self,
        blocked: bool,
        win: &SourceWindow,
        width: i32,
        height: i32,
    ) -> bool {
        self.broadcast(|h| h.on_framebuffer_size(blocked, win, width, height))
    }
    fn on_drop(&mut self, blocked: bool, win: &SourceWindow, data: &[String]) -> bool {
        self.broadcast(|h| h.on_drop(blocked, win, data))
    }
    fn before_events(&mut self, win: &SourceWindow) {
        for tool in self.handlers() {
            tool.borrow_mut().before_events(win);
        }
    }
    fn after_events(&mut self, win: &SourceWindow) {
        for tool in self.handlers() {
            tool.borrow_mut().after_events(win);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    type Log = Rc<RefCell<Vec<String>>>;
    struct Backend {
        log: Log,
        mouse: bool,
        keyboard: bool,
        draw: bool,
        fail: Option<&'static str>,
    }
    impl Backend {
        fn event(&mut self, name: &str) {
            self.log.borrow_mut().push(name.into());
            if self.fail == Some(name) {
                panic!("{name}");
            }
        }
    }
    impl GuiBackend for Backend {
        type Font = ();
        type DrawData = [f32; 2];
        fn mouse_button_callback(&mut self, window: i64, button: i32, action: i32, mods: i32) {
            self.event(&format!("native:mouse:{window}:{button}:{action}:{mods}"));
        }
        fn cursor_pos_callback(&mut self, window: i64, x: f64, y: f64) {
            self.event(&format!("native:cursor:{window}:{x}:{y}"));
        }
        fn cursor_enter_callback(&mut self, window: i64, enter: bool) {
            self.event(&format!("native:enter:{window}:{enter}"));
        }
        fn scroll_callback(&mut self, window: i64, x: f64, y: f64) {
            self.event(&format!("native:scroll:{window}:{x}:{y}"));
        }
        fn char_callback(&mut self, window: i64, c: i32) {
            self.event(&format!("native:char:{window}:{c}"));
        }
        fn key_callback(&mut self, window: i64, key: i32, scan: i32, action: i32, mods: i32) {
            self.event(&format!("native:key:{window}:{key}:{scan}:{action}:{mods}"));
        }
        fn want_capture_mouse(&mut self) -> bool {
            self.event("capture:mouse");
            self.mouse
        }
        fn want_capture_keyboard(&mut self) -> bool {
            self.event("capture:keyboard");
            self.keyboard
        }
        fn check_error(&mut self, message: &str, fatal: bool) {
            assert_eq!(message, "GUI handle");
            assert!(!fatal);
            self.event("error-check");
        }
        fn glfw_new_frame(&mut self) {
            self.event("glfw-frame");
        }
        fn gl3_new_frame(&mut self) {
            self.event("gl3-frame");
        }
        fn imgui_new_frame(&mut self) {
            self.event("imgui-frame");
        }
        fn toolbox_render(&mut self) {
            self.event("toolbox");
        }
        fn imgui_render(&mut self) {
            self.event("imgui-render");
        }
        fn draw_data(&mut self) -> Option<[f32; 2]> {
            self.event("draw-data");
            self.draw.then_some([0.0; 2])
        }
        fn put_framebuffer_scale(&mut self, data: &mut [f32; 2], x: f32, y: f32) {
            self.event("put-scale");
            *data = [x, y];
        }
        fn render_draw_data(&mut self, data: [f32; 2]) {
            assert_eq!(data, [2.5, 2.5]);
            self.event("draw");
        }
        fn print_throwable(&mut self, _: &(dyn Any + Send)) {
            self.event("print-throwable");
        }
        fn gl3_shutdown(&mut self) {
            self.event("gl3-shutdown");
        }
        fn glfw_shutdown(&mut self) {
            self.event("glfw-shutdown");
        }
        fn context_destroy(&mut self) {
            self.event("destroy");
        }
    }
    impl GuiConstructionBackend for Backend {
        fn create_context_default(&mut self) {
            self.event("create-context");
        }
        fn wrap_glfw_window(&mut self, window: i64) {
            self.event(&format!("wrap-window:{window}"));
        }
        fn create_glfw_implementation(&mut self, install_callbacks: bool) {
            assert!(install_callbacks);
            self.event("create-glfw");
        }
        fn create_gl3_implementation_default(&mut self) {
            self.event("create-gl3");
        }
        fn monitor_scale(&mut self, window: &SourceWindow) -> [f32; 2] {
            self.event(&format!("monitor-scale:{}", window.id));
            [3.0, 4.0]
        }
        fn scale_all_style_sizes(&mut self, scale: f32) {
            self.event(&format!("style-scale:{scale}"));
        }
        fn set_font_global_scale(&mut self, scale: f32) {
            self.event(&format!("font-scale:{scale}"));
        }
        fn add_font_from_file_ttf(&mut self, path: &str, size: f32) -> Option<Rc<()>> {
            assert_eq!(path, "FiraSans-Regular.ttf");
            self.event(&format!("font:{size}"));
            if (self.fail == Some("null18") && size == 54.0)
                || (self.fail == Some("null12") && size == 36.0)
            {
                None
            } else {
                Some(Rc::new(()))
            }
        }
        fn create_toolbox(&mut self) {
            self.event("create-toolbox");
        }
    }
    struct Reset {
        log: Log,
        ship: Rc<u8>,
    }
    impl ResetShipOperations for Reset {
        type Ship = u8;
        type Thumbnail = u8;
        fn global_ship(&mut self) -> Rc<u8> {
            self.log
                .borrow_mut()
                .push(format!("ship:get:{}", self.ship));
            self.ship.clone()
        }
        fn thumbnail(&mut self, ship: &u8) -> u8 {
            self.log.borrow_mut().push(format!("ship:thumbnail:{ship}"));
            *ship
        }
        fn close_ship(&mut self, ship: &u8) {
            self.log.borrow_mut().push(format!("ship:close:{ship}"));
        }
        fn construct_ship(&mut self, thumb: u8, _: Rc<RefCell<SourceCameraControl>>) -> Rc<u8> {
            self.log
                .borrow_mut()
                .push(format!("ship:construct:{thumb}"));
            Rc::new(2)
        }
        fn set_global_ship(&mut self, ship: Rc<u8>) {
            self.log.borrow_mut().push(format!("ship:set:{ship}"));
            self.ship = ship;
        }
        fn clear_ship_list(&mut self) {
            self.log.borrow_mut().push("ship:clear".into());
        }
        fn add_ship(&mut self, ship: Rc<u8>) {
            self.log.borrow_mut().push(format!("ship:add:{ship}"));
        }
    }
    struct Tool {
        id: u8,
        log: Log,
        fail: bool,
    }
    impl InputHandler<SourceWindow> for Tool {
        fn on_mouse_button(
            &mut self,
            blocked: bool,
            _: &SourceWindow,
            _: i32,
            _: i32,
            _: i32,
        ) -> bool {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:mouse:{blocked}", self.id));
            self.id == 1
        }
        fn on_key(
            &mut self,
            blocked: bool,
            _: &SourceWindow,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> bool {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:key:{blocked}", self.id));
            self.id == 1
        }
        fn on_size(&mut self, blocked: bool, _: &SourceWindow, h: i32, w: i32) -> bool {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:size:{blocked}:{h}:{w}", self.id));
            false
        }
        fn on_char_mods(&mut self, blocked: bool, _: &SourceWindow, c: i32, mods: i32) -> bool {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:charmods:{blocked}:{c}:{mods}", self.id));
            blocked
        }
        fn before_events(&mut self, _: &SourceWindow) {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:before", self.id));
        }
        fn after_events(&mut self, _: &SourceWindow) {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:after", self.id));
        }
    }
    impl SourceTool<SourceWindow> for Tool {
        type Texture = SourceTexture2D;
        fn texture(&self) -> &SourceTexture2D {
            panic!("unused recording texture")
        }
        fn active_texture(&self) -> &SourceTexture2D {
            panic!("unused recording texture")
        }
        fn name(&self) -> &str {
            "recording tool"
        }
        fn update(&mut self) {
            self.log
                .borrow_mut()
                .push(format!("tool:{}:update", self.id));
            if self.fail {
                panic!("update");
            }
        }
    }
    fn fixture() -> (SourceGui<Backend, Reset>, Log) {
        let (window, _, _) = crate::window::tests::fixture();
        let log = Log::default();
        let mut globals = GuiKt::default();
        let a: Rc<GuiTool> = Rc::new(RefCell::new(Tool {
            id: 1,
            log: log.clone(),
            fail: false,
        }));
        let b: Rc<GuiTool> = Rc::new(RefCell::new(Tool {
            id: 2,
            log: log.clone(),
            fail: false,
        }));
        globals.access_set_tool_list(Some(Rc::new(RefCell::new(vec![
            None,
            Some(a),
            Some(b),
            None,
            None,
        ]))));
        let control = Rc::new(RefCell::new(SourceCameraControl::with_camera(
            window.clone(),
            Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200)),
        )));
        let backend = Backend {
            log: log.clone(),
            mouse: false,
            keyboard: false,
            draw: true,
            fail: None,
        };
        let reset = Reset {
            log: log.clone(),
            ship: Rc::new(1),
        };
        (
            SourceGui::from_parts(
                window,
                control,
                Rc::new(RefCell::new(globals)),
                backend,
                reset,
                2.5,
            ),
            log,
        )
    }
    struct Factory {
        log: Log,
        providers: Vec<Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>>,
        control: Rc<RefCell<SourceCameraControl>>,
        globals: Rc<RefCell<GuiKt<GuiTool, ()>>>,
        previous: crate::gui_kt::ToolList<GuiTool>,
        fail: Option<u8>,
    }
    impl Factory {
        fn brush(
            &mut self,
            id: u8,
            control: Rc<RefCell<SourceCameraControl>>,
            provider: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
        ) -> Result<Rc<GuiTool>, String> {
            assert!(Rc::ptr_eq(&self.control, &control));
            assert!(Rc::ptr_eq(&provider, &self.providers[(id - 1) as usize]));
            assert!(Rc::ptr_eq(
                &self.previous,
                &self.globals.borrow().tool_list()
            ));
            self.log.borrow_mut().push(format!("construct-tool:{id}"));
            if self.fail == Some(id) {
                return Err("tool constructor failed".into());
            }
            if id < 3 {
                crate::game_parameter_provider_kt::set_game_parameter_provider(
                    self.providers[id as usize].clone(),
                );
            }
            Ok(Rc::new(RefCell::new(Tool {
                id,
                log: self.log.clone(),
                fail: false,
            })))
        }
    }
    impl GuiToolFactory for Factory {
        fn break_tool(
            &mut self,
            control: Rc<RefCell<SourceCameraControl>>,
            p: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
        ) -> Result<Rc<GuiTool>, String> {
            self.brush(1, control, p)
        }
        fn flood_tool(
            &mut self,
            control: Rc<RefCell<SourceCameraControl>>,
            p: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
        ) -> Result<Rc<GuiTool>, String> {
            self.brush(2, control, p)
        }
        fn dry_tool(
            &mut self,
            control: Rc<RefCell<SourceCameraControl>>,
            p: Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>,
        ) -> Result<Rc<GuiTool>, String> {
            self.brush(3, control, p)
        }
        fn move_tool(&mut self) -> Result<Rc<GuiTool>, String> {
            self.log.borrow_mut().push("construct-tool:4".into());
            assert!(Rc::ptr_eq(
                &self.previous,
                &self.globals.borrow().tool_list()
            ));
            Ok(Rc::new(RefCell::new(Tool {
                id: 4,
                log: self.log.clone(),
                fail: false,
            })))
        }
    }
    fn constructor_fixture() -> (
        Rc<SourceWindow>,
        crate::resource::ResourceRuntime,
        Factory,
        Backend,
        Reset,
    ) {
        let (window, resources, _) = crate::window::tests::fixture();
        let log = Log::default();
        let control = Rc::new(RefCell::new(SourceCameraControl::with_camera(
            window.clone(),
            Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200)),
        )));
        let globals = Rc::new(RefCell::new(GuiKt::default()));
        let previous = Rc::new(RefCell::new(vec![None]));
        globals
            .borrow_mut()
            .access_set_tool_list(Some(previous.clone()));
        let factory = Factory {
            log: log.clone(),
            providers: (0..3)
                .map(|_| {
                    Rc::new(RefCell::new(
                        crate::game_parameters::SourceGameParameterProvider::default(),
                    ))
                })
                .collect(),
            control,
            globals,
            previous,
            fail: None,
        };
        let backend = Backend {
            log: log.clone(),
            mouse: false,
            keyboard: false,
            draw: true,
            fail: None,
        };
        let reset = Reset {
            log,
            ship: Rc::new(1),
        };
        (window, resources, factory, backend, reset)
    }
    #[test]
    fn constructor_preserves_dpi_tool_font_order_live_provider_reads_and_window_dependency() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let (window, resources, mut factory, backend, reset) = constructor_fixture();
        let log = factory.log.clone();
        let globals = factory.globals.clone();
        let control = factory.control.clone();
        crate::game_parameter_provider_kt::set_game_parameter_provider(
            factory.providers[0].clone(),
        );
        let gui = SourceGui::new(
            window.clone(),
            control.clone(),
            globals.clone(),
            backend,
            reset,
            &mut factory,
            &resources,
        )
        .unwrap();
        assert_eq!(
            *log.borrow(),
            [
                "create-context",
                "wrap-window:7",
                "create-glfw",
                "create-gl3",
                "monitor-scale:7",
                "style-scale:1.5",
                "font-scale:0.5",
                "construct-tool:1",
                "construct-tool:2",
                "construct-tool:3",
                "construct-tool:4",
                "font:54",
                "font:36",
                "create-toolbox"
            ]
        );
        assert_eq!(gui.fbscale, 2.0);
        assert_eq!(gui.fontscale, Some(3.0));
        assert_eq!(globals.borrow().scale(), 1.5);
        let list = globals.borrow().tool_list();
        assert!(!Rc::ptr_eq(&list, &factory.previous));
        assert_eq!(list.borrow().len(), 5);
        assert!(list.borrow()[0].is_none());
        assert_eq!(gui.handlers().len(), 4);
        assert!(Rc::ptr_eq(&control, &gui.camera_control()));
        assert!(Rc::ptr_eq(&window, &gui.window()));
        assert!(!Rc::ptr_eq(
            &globals.borrow().f18(),
            &globals.borrow().f12()
        ));
        log.borrow_mut().clear();
        window.close();
        assert!(gui.freed());
        assert!(log.borrow().is_empty());
        resources.run_main();
        assert_eq!(*log.borrow(), ["gl3-shutdown", "glfw-shutdown", "destroy"]);
        log.borrow_mut().clear();
        gui.close();
        resources.run_main();
        assert!(log.borrow().is_empty());
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
    #[test]
    fn failed_tool_and_font_construction_preserve_original_publication_boundaries() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        for failure in ["tool", "null18", "null12"] {
            let (window, resources, mut factory, mut backend, reset) = constructor_fixture();
            let globals = factory.globals.clone();
            let log = factory.log.clone();
            let old18 = Rc::new(());
            let old12 = Rc::new(());
            globals.borrow_mut().access_set_f18(Some(old18.clone()));
            globals.borrow_mut().access_set_f12(Some(old12.clone()));
            if failure == "tool" {
                factory.fail = Some(2);
            } else {
                backend.fail = Some(failure);
            }
            crate::game_parameter_provider_kt::set_game_parameter_provider(
                factory.providers[0].clone(),
            );
            let control = factory.control.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                SourceGui::new(
                    window.clone(),
                    control,
                    globals.clone(),
                    backend,
                    reset,
                    &mut factory,
                    &resources,
                )
            }));
            if failure == "tool" {
                assert!(result.unwrap().is_err());
                assert!(Rc::ptr_eq(&factory.previous, &globals.borrow().tool_list()));
                assert!(Rc::ptr_eq(&old18, &globals.borrow().f18()));
                assert_eq!(log.borrow().last().unwrap(), "construct-tool:2");
            } else {
                assert!(result.is_err());
                assert_eq!(globals.borrow().tool_list().borrow().len(), 5);
                if failure == "null18" {
                    assert!(Rc::ptr_eq(&old18, &globals.borrow().f18()));
                } else {
                    assert!(!Rc::ptr_eq(&old18, &globals.borrow().f18()));
                }
                assert_eq!(
                    log.borrow().last().unwrap(),
                    if failure == "null18" {
                        "font:54"
                    } else {
                        "font:36"
                    }
                );
            }
            assert!(Rc::ptr_eq(&old12, &globals.borrow().f12()));
            log.borrow_mut().clear();
            resources.run_main();
            assert_eq!(*log.borrow(), ["gl3-shutdown", "glfw-shutdown", "destroy"]);
        }
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
    #[test]
    fn native_callbacks_precede_capture_and_broadcast_reaches_all_tools() {
        let (mut gui, log) = fixture();
        let win = gui.window();
        assert!(Rc::ptr_eq(&win, &gui.window()));
        assert!(Rc::ptr_eq(&gui.camera_control, &gui.camera_control()));
        assert!(gui.on_mouse_button(false, &win, 0, 1, 0));
        assert_eq!(
            *log.borrow(),
            [
                "native:mouse:7:0:1:0",
                "capture:mouse",
                "tool:1:mouse:false",
                "tool:2:mouse:false"
            ]
        );
        log.borrow_mut().clear();
        gui.backend.borrow_mut().mouse = true;
        assert!(gui.on_mouse_button(false, &win, 0, 0, 0));
        assert_eq!(*log.borrow(), ["native:mouse:7:0:0:0", "capture:mouse"]);
        log.borrow_mut().clear();
        assert!(!gui.on_size(true, &win, 10, 20));
        assert_eq!(
            *log.borrow(),
            ["tool:1:size:true:10:20", "tool:2:size:true:10:20"]
        );
        log.borrow_mut().clear();
        gui.before_events(&win);
        gui.after_events(&win);
        assert_eq!(
            *log.borrow(),
            [
                "tool:1:before",
                "tool:2:before",
                "tool:1:after",
                "tool:2:after"
            ]
        );
    }
    #[test]
    fn number_keys_toggle_tool_index_even_when_blocked_but_not_when_captured() {
        let (mut gui, log) = fixture();
        let win = gui.window();
        assert!(gui.on_key(true, &win, 49, 3, 1, 0));
        assert_eq!(gui.globals.borrow().current_tool_index(), 1);
        assert_eq!(*log.borrow(), ["native:key:7:49:3:1:0", "capture:keyboard"]);
        assert!(gui.on_key(false, &win, 49, 3, 1, 0));
        assert_eq!(gui.globals.borrow().current_tool_index(), 0);
        assert!(gui.on_key(false, &win, 52, 3, 1, 0));
        assert_eq!(gui.globals.borrow().current_tool_index(), 4);
        log.borrow_mut().clear();
        assert!(gui.on_key(false, &win, 53, 3, 1, 0));
        assert_eq!(
            *log.borrow(),
            [
                "native:key:7:53:3:1:0",
                "capture:keyboard",
                "tool:1:key:false",
                "tool:2:key:false"
            ]
        );
        gui.backend.borrow_mut().keyboard = true;
        assert!(gui.on_key(false, &win, 50, 3, 1, 0));
        assert_eq!(gui.globals.borrow().current_tool_index(), 4);
        gui.backend.borrow_mut().keyboard = false;
        log.borrow_mut().clear();
        assert!(gui.on_key(false, &win, 49, 3, 2, 0));
        assert_eq!(
            *log.borrow(),
            [
                "native:key:7:49:3:2:0",
                "capture:keyboard",
                "tool:1:key:false",
                "tool:2:key:false"
            ]
        );
    }
    #[test]
    fn shifted_characters_consume_input_and_reset_uses_live_global_getters() {
        let (mut gui, log) = fixture();
        let win = gui.window();
        gui.backend.borrow_mut().keyboard = true;
        assert!(gui.on_char_mods(true, &win, 0x10072, 1));
        assert_eq!(
            *log.borrow(),
            [
                "ship:get:1",
                "ship:thumbnail:1",
                "ship:get:1",
                "ship:close:1",
                "ship:construct:1",
                "ship:set:2",
                "ship:clear",
                "ship:get:2",
                "ship:add:2"
            ]
        );
        log.borrow_mut().clear();
        assert!(gui.on_char_mods(false, &win, 120, 1));
        assert!(log.borrow().is_empty());
        gui.backend.borrow_mut().keyboard = false;
        assert!(gui.on_char_mods(true, &win, 114, 0));
        assert_eq!(
            *log.borrow(),
            [
                "capture:keyboard",
                "tool:1:charmods:true:114:0",
                "tool:2:charmods:true:114:0"
            ]
        );
    }
    #[test]
    fn frame_render_and_shutdown_preserve_order_and_catch_only_source_boundaries() {
        let (mut gui, log) = fixture();
        gui.handle();
        assert_eq!(
            *log.borrow(),
            [
                "error-check",
                "glfw-frame",
                "gl3-frame",
                "imgui-frame",
                "toolbox",
                "imgui-render"
            ]
        );
        log.borrow_mut().clear();
        gui.globals.borrow_mut().set_current_tool_index(2);
        gui.render();
        assert_eq!(
            *log.borrow(),
            ["tool:2:update", "draw-data", "put-scale", "draw"]
        );
        log.borrow_mut().clear();
        gui.backend.borrow_mut().fail = Some("toolbox");
        gui.handle();
        assert_eq!(
            *log.borrow(),
            [
                "error-check",
                "glfw-frame",
                "gl3-frame",
                "imgui-frame",
                "toolbox",
                "print-throwable"
            ]
        );
        log.borrow_mut().clear();
        gui.backend.borrow_mut().fail = Some("error-check");
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.handle())).is_err());
        assert_eq!(*log.borrow(), ["error-check"]);
        log.borrow_mut().clear();
        gui.backend.borrow_mut().fail = None;
        gui.backend.borrow_mut().draw = false;
        gui.render();
        assert_eq!(*log.borrow(), ["tool:2:update", "draw-data"]);
        log.borrow_mut().clear();
        gui.globals.borrow_mut().set_current_tool_index(99);
        gui.render();
        assert_eq!(*log.borrow(), ["print-throwable"]);
        log.borrow_mut().clear();
        gui.free();
        assert_eq!(*log.borrow(), ["gl3-shutdown", "glfw-shutdown", "destroy"]);
        log.borrow_mut().clear();
        gui.backend.borrow_mut().fail = Some("gl3-shutdown");
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.free())).is_err());
        assert_eq!(*log.borrow(), ["gl3-shutdown"]);
    }
}
impl<B: GuiConstructionBackend + 'static, R: ResetShipOperations> SourceGui<B, R> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window: Rc<SourceWindow>,
        camera_control: Rc<RefCell<SourceCameraControl>>,
        globals: Rc<RefCell<GuiKt<GuiTool, B::Font>>>,
        backend: B,
        reset: R,
        factory: &mut impl GuiToolFactory,
        resources: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        let backend = Rc::new(RefCell::new(backend));
        let cleanup = backend.clone();
        let resource = resources.allocate_local(&[window.resource()], move || {
            cleanup.borrow_mut().gl3_shutdown();
            cleanup.borrow_mut().glfw_shutdown();
            cleanup.borrow_mut().context_destroy();
        });
        backend.borrow_mut().create_context_default();
        backend.borrow_mut().wrap_glfw_window(window.id);
        backend.borrow_mut().create_glfw_implementation(true);
        backend.borrow_mut().create_gl3_implementation_default();
        let control_window = camera_control.borrow().window();
        let size = control_window.screen_size();
        let framebuffer = control_window.framebuffer_size();
        let ratio = [
            framebuffer[0] as f32 / size[0] as f32,
            framebuffer[1] as f32 / size[1] as f32,
        ];
        let fbscale = ratio[0];
        let fontscale = backend.borrow_mut().monitor_scale(&window)[0];
        globals.borrow_mut().access_set_scale(fontscale / fbscale);
        backend
            .borrow_mut()
            .scale_all_style_sizes(globals.borrow().scale());
        backend.borrow_mut().set_font_global_scale(1.0 / fbscale);
        // Each constructor reads the global provider independently, as the original argument evaluation does.
        let break_tool = factory.break_tool(
            camera_control.clone(),
            crate::game_parameter_provider_kt::get_game_parameter_provider(),
        )?;
        let flood_tool = factory.flood_tool(
            camera_control.clone(),
            crate::game_parameter_provider_kt::get_game_parameter_provider(),
        )?;
        let dry_tool = factory.dry_tool(
            camera_control.clone(),
            crate::game_parameter_provider_kt::get_game_parameter_provider(),
        )?;
        let move_tool = factory.move_tool()?;
        globals
            .borrow_mut()
            .access_set_tool_list(Some(Rc::new(RefCell::new(vec![
                None,
                Some(break_tool),
                Some(flood_tool),
                Some(dry_tool),
                Some(move_tool),
            ]))));
        let f18 = backend
            .borrow_mut()
            .add_font_from_file_ttf("FiraSans-Regular.ttf", 18.0 * fontscale)
            .expect("KotlinNullPointerException: f18");
        globals.borrow_mut().access_set_f18(Some(f18));
        let f12 = backend
            .borrow_mut()
            .add_font_from_file_ttf("FiraSans-Regular.ttf", 12.0 * fontscale)
            .expect("KotlinNullPointerException: f12");
        globals.borrow_mut().access_set_f12(Some(f12));
        backend.borrow_mut().create_toolbox();
        Ok(Self {
            window,
            camera_control,
            globals,
            backend,
            reset,
            fbscale,
            fontscale: Some(fontscale),
            resource: Some(resource),
        })
    }
}

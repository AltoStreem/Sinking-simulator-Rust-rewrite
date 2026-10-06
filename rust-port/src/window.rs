//! Window.java callback dispatch, native property queries and event-loop sequencing.
//! Native construction/context/icon setup remains pending; this wraps an existing handle.
#![allow(dead_code)]
use crate::{
    input_handler::InputHandler,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
#[derive(Clone, Debug)]
pub(crate) enum WindowEvent {
    Position {
        xpos: i32,
        ypos: i32,
    },
    Size {
        width: i32,
        height: i32,
    },
    Close {},
    Refresh {},
    Focus {
        focus: bool,
    },
    Iconify {
        iconify: bool,
    },
    FramebufferSize {
        width: i32,
        height: i32,
    },
    Key {
        key: i32,
        scancode: i32,
        action: i32,
        mods: i32,
    },
    Char {
        codepoint: i32,
    },
    CharMods {
        codepoint: i32,
        mods: i32,
    },
    MouseButton {
        button: i32,
        action: i32,
        mods: i32,
    },
    CursorPos {
        xpos: f64,
        ypos: f64,
    },
    CursorEnter {
        enter: bool,
    },
    Scroll {
        x: f64,
        y: f64,
    },
    Drop {
        data: Vec<String>,
    },
}
pub(crate) trait WindowBackend: Send {
    fn set_title(&mut self, id: i64, title: &str);
    fn position(&mut self, id: i64) -> [i32; 2];
    fn set_position(&mut self, id: i64, pos: [i32; 2]);
    fn framebuffer_size(&mut self, id: i64) -> [i32; 2];
    fn screen_size(&mut self, id: i64) -> [i32; 2];
    fn mouse_position(&mut self, id: i64) -> [f64; 2];
    fn should_close(&mut self, id: i64) -> bool;
    fn set_should_close(&mut self, id: i64, value: bool);
    fn set_clear_color(&mut self, color: [f32; 4]);
    fn clear(&mut self, mask: i32);
    fn swap_buffers(&mut self, id: i64);
    fn poll_events(&mut self) -> Vec<WindowEvent>;
    fn free_callbacks(&mut self, id: i64);
    fn destroy_window(&mut self, id: i64);
}
pub(crate) type Handler = Rc<RefCell<dyn InputHandler<SourceWindow>>>;
pub(crate) struct SourceWindow {
    pub id: i64,
    title: RefCell<String>,
    backend: Arc<Mutex<dyn WindowBackend>>,
    lifetime: ResourceHandle,
    glfw: ResourceHandle,
    gl_context: RefCell<Option<Arc<crate::gl_context::GlContext>>>,
    pub handler_stacks: RefCell<Vec<Vec<Handler>>>,
    pub before_events_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow)>>>,
    pub after_events_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow)>>>,
    pub position_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32)>>>,
    pub size_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32)>>>,
    pub close_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow)>>>,
    pub refresh_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow)>>>,
    pub focus_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, bool)>>>,
    pub iconify_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, bool)>>>,
    pub framebuffer_size_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32)>>>,
    pub key_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32, i32, i32)>>>,
    pub char_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32)>>>,
    pub char_mods_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32)>>>,
    pub mouse_button_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, i32, i32, i32)>>>,
    pub cursor_pos_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, f64, f64)>>>,
    pub cursor_enter_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, bool)>>>,
    pub scroll_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, f64, f64)>>>,
    pub drop_callbacks: RefCell<Vec<Rc<dyn Fn(&SourceWindow, &[String])>>>,
}
impl SourceWindow {
    pub fn resource(&self) -> ResourceHandle {
        self.lifetime.clone()
    }
    pub fn gl_context(&self) -> Arc<crate::gl_context::GlContext> {
        self.gl_context
            .borrow()
            .clone()
            .expect("lateinit property glContext has not been initialized")
    }
    pub fn set_gl_context(&self, context: Arc<crate::gl_context::GlContext>) {
        *self.gl_context.borrow_mut() = Some(context);
    }
    pub fn from_handle(
        id: i64,
        title: String,
        backend: Arc<Mutex<dyn WindowBackend>>,
        glfw: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let cleanup = backend.clone();
        let lifetime = runtime.allocate(&[glfw.clone()], move || {
            let mut backend = cleanup.lock().unwrap();
            backend.free_callbacks(id);
            backend.destroy_window(id);
        });
        Self {
            id,
            title: RefCell::new(title),
            backend,
            lifetime,
            glfw,
            gl_context: RefCell::new(None),
            handler_stacks: RefCell::new(vec![]),
            before_events_callbacks: RefCell::new(vec![]),
            after_events_callbacks: RefCell::new(vec![]),
            position_callbacks: RefCell::new(vec![]),
            size_callbacks: RefCell::new(vec![]),
            close_callbacks: RefCell::new(vec![]),
            refresh_callbacks: RefCell::new(vec![]),
            focus_callbacks: RefCell::new(vec![]),
            iconify_callbacks: RefCell::new(vec![]),
            framebuffer_size_callbacks: RefCell::new(vec![]),
            key_callbacks: RefCell::new(vec![]),
            char_callbacks: RefCell::new(vec![]),
            char_mods_callbacks: RefCell::new(vec![]),
            mouse_button_callbacks: RefCell::new(vec![]),
            cursor_pos_callbacks: RefCell::new(vec![]),
            cursor_enter_callbacks: RefCell::new(vec![]),
            scroll_callbacks: RefCell::new(vec![]),
            drop_callbacks: RefCell::new(vec![]),
        }
    }
    pub fn emit_position(&self, xpos: i32, ypos: i32) {
        for callback in self.position_callbacks.borrow().iter() {
            callback(self, xpos, ypos);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_position(blocked, self, ypos, xpos);
            }
        }
    }
    pub fn emit_size(&self, width: i32, height: i32) {
        for callback in self.size_callbacks.borrow().iter() {
            callback(self, width, height);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_size(blocked, self, height, width);
            }
        }
    }
    pub fn emit_close(&self) {
        for callback in self.close_callbacks.borrow().iter() {
            callback(self);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_close(blocked, self);
            }
        }
    }
    pub fn emit_refresh(&self) {
        for callback in self.refresh_callbacks.borrow().iter() {
            callback(self);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_refresh(blocked, self);
            }
        }
    }
    pub fn emit_focus(&self, focus: bool) {
        for callback in self.focus_callbacks.borrow().iter() {
            callback(self, focus);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_focus(blocked, self, focus);
            }
        }
    }
    pub fn emit_iconify(&self, iconify: bool) {
        for callback in self.iconify_callbacks.borrow().iter() {
            callback(self, iconify);
        }
    }
    pub fn emit_framebuffer_size(&self, width: i32, height: i32) {
        for callback in self.framebuffer_size_callbacks.borrow().iter() {
            callback(self, width, height);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler
                    .borrow_mut()
                    .on_framebuffer_size(blocked, self, width, height);
            }
        }
    }
    pub fn emit_key(&self, key: i32, scancode: i32, action: i32, mods: i32) {
        for callback in self.key_callbacks.borrow().iter() {
            callback(self, key, scancode, action, mods);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler
                    .borrow_mut()
                    .on_key(blocked, self, key, scancode, action, mods);
            }
        }
    }
    pub fn emit_char(&self, codepoint: i32) {
        for callback in self.char_callbacks.borrow().iter() {
            callback(self, codepoint);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_char(blocked, self, codepoint);
            }
        }
    }
    pub fn emit_char_mods(&self, codepoint: i32, mods: i32) {
        for callback in self.char_mods_callbacks.borrow().iter() {
            callback(self, codepoint, mods);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler
                    .borrow_mut()
                    .on_char_mods(blocked, self, codepoint, mods);
            }
        }
    }
    pub fn emit_mouse_button(&self, button: i32, action: i32, mods: i32) {
        for callback in self.mouse_button_callbacks.borrow().iter() {
            callback(self, button, action, mods);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler
                    .borrow_mut()
                    .on_mouse_button(blocked, self, button, action, mods);
            }
        }
    }
    pub fn emit_cursor_pos(&self, xpos: f64, ypos: f64) {
        for callback in self.cursor_pos_callbacks.borrow().iter() {
            callback(self, xpos, ypos);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler
                    .borrow_mut()
                    .on_cursor_pos(blocked, self, xpos, ypos);
            }
        }
    }
    pub fn emit_cursor_enter(&self, enter: bool) {
        for callback in self.cursor_enter_callbacks.borrow().iter() {
            callback(self, enter);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_cursor_enter(blocked, self, enter);
            }
        }
    }
    pub fn emit_scroll(&self, x: f64, y: f64) {
        for callback in self.scroll_callbacks.borrow().iter() {
            callback(self, x, y);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_scroll(blocked, self, x, y);
            }
        }
    }
    pub fn emit_drop(&self, data: &[String]) {
        for callback in self.drop_callbacks.borrow().iter() {
            callback(self, data);
        }
        for stack in self.handler_stacks.borrow().iter() {
            let mut blocked = false;
            for handler in stack {
                blocked = handler.borrow_mut().on_drop(blocked, self, data);
            }
        }
    }
    pub fn dispatch(&self, event: WindowEvent) {
        match event {
            WindowEvent::Position { xpos, ypos } => self.emit_position(xpos, ypos),
            WindowEvent::Size { width, height } => self.emit_size(width, height),
            WindowEvent::Close {} => self.emit_close(),
            WindowEvent::Refresh {} => self.emit_refresh(),
            WindowEvent::Focus { focus } => self.emit_focus(focus),
            WindowEvent::Iconify { iconify } => self.emit_iconify(iconify),
            WindowEvent::FramebufferSize { width, height } => {
                self.emit_framebuffer_size(width, height)
            }
            WindowEvent::Key {
                key,
                scancode,
                action,
                mods,
            } => self.emit_key(key, scancode, action, mods),
            WindowEvent::Char { codepoint } => self.emit_char(codepoint),
            WindowEvent::CharMods { codepoint, mods } => self.emit_char_mods(codepoint, mods),
            WindowEvent::MouseButton {
                button,
                action,
                mods,
            } => self.emit_mouse_button(button, action, mods),
            WindowEvent::CursorPos { xpos, ypos } => self.emit_cursor_pos(xpos, ypos),
            WindowEvent::CursorEnter { enter } => self.emit_cursor_enter(enter),
            WindowEvent::Scroll { x, y } => self.emit_scroll(x, y),
            WindowEvent::Drop { data } => self.emit_drop(&data),
        }
    }
    pub fn title(&self) -> String {
        self.title.borrow().clone()
    }
    pub fn set_title(&self, title: String) {
        *self.title.borrow_mut() = title.clone();
        self.backend.lock().unwrap().set_title(self.id, &title);
    }
    pub fn position(&self) -> [i32; 2] {
        self.backend.lock().unwrap().position(self.id)
    }
    pub fn set_position(&self, pos: [i32; 2]) {
        self.backend.lock().unwrap().set_position(self.id, pos);
    }
    pub fn framebuffer_size(&self) -> [i32; 2] {
        self.backend.lock().unwrap().framebuffer_size(self.id)
    }
    pub fn screen_size(&self) -> [i32; 2] {
        self.backend.lock().unwrap().screen_size(self.id)
    }
    pub fn mouse_position(&self) -> [f64; 2] {
        self.backend.lock().unwrap().mouse_position(self.id)
    }
    pub fn mouse_position_relative(&self) -> [f64; 2] {
        let mouse = self.mouse_position();
        let screen = self.screen_size();
        [
            (mouse[0] - screen[0] as f64 * 0.5) * 2. / screen[0] as f64,
            (mouse[1] - screen[1] as f64 * 0.5) * 2. / screen[1] as f64,
        ]
    }
    pub fn should_close(&self) -> bool {
        self.backend.lock().unwrap().should_close(self.id)
    }
    pub fn set_should_close(&self, value: bool) {
        self.backend
            .lock()
            .unwrap()
            .set_should_close(self.id, value);
    }
    pub fn close(&self) {
        self.lifetime.close();
    }
    pub fn freed(&self) -> bool {
        self.lifetime.freed()
    }
    pub fn before_events(&self) {
        for callback in self.before_events_callbacks.borrow().iter() {
            callback(self);
        }
        for stack in self.handler_stacks.borrow().iter() {
            for handler in stack {
                handler.borrow_mut().before_events(self);
            }
        }
    }
    pub fn after_events(&self) {
        for callback in self.after_events_callbacks.borrow().iter() {
            callback(self);
        }
        for stack in self.handler_stacks.borrow().iter() {
            for handler in stack {
                handler.borrow_mut().after_events(self);
            }
        }
    }
    pub fn start_default(self: &Rc<Self>, draw: Rc<dyn Fn(&SourceWindow)>) {
        self.start([0.; 4], draw);
    }
    pub fn start(self: &Rc<Self>, clear: [f32; 4], draw: Rc<dyn Fn(&SourceWindow)>) {
        self.backend.lock().unwrap().set_clear_color(clear);
        let refresh = draw.clone();
        let original = self.clone();
        self.refresh_callbacks.borrow_mut().push(Rc::new(move |_| {
            original.backend.lock().unwrap().clear(17664);
            refresh(&original);
            original.backend.lock().unwrap().swap_buffers(original.id);
        }));
        while !self.should_close() {
            self.backend.lock().unwrap().clear(17664);
            draw(self);
            self.backend.lock().unwrap().swap_buffers(self.id);
            self.before_events();
            let events = self.backend.lock().unwrap().poll_events();
            for event in events {
                self.dispatch(event);
            }
            self.after_events();
        }
        self.glfw.close();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    type Log = Arc<Mutex<Vec<String>>>;
    struct Backend {
        log: Log,
        closed: bool,
    }
    impl Backend {
        fn record(&self, s: impl Into<String>) {
            self.log.lock().unwrap().push(s.into());
        }
    }
    impl WindowBackend for Backend {
        fn set_title(&mut self, _: i64, title: &str) {
            self.record(format!("title:{title}"));
        }
        fn position(&mut self, _: i64) -> [i32; 2] {
            [11, 22]
        }
        fn set_position(&mut self, _: i64, pos: [i32; 2]) {
            self.record(format!("position:{pos:?}"));
        }
        fn framebuffer_size(&mut self, _: i64) -> [i32; 2] {
            [800, 400]
        }
        fn screen_size(&mut self, _: i64) -> [i32; 2] {
            self.record("screen");
            [400, 200]
        }
        fn mouse_position(&mut self, _: i64) -> [f64; 2] {
            self.record("mouse");
            [300., 50.]
        }
        fn should_close(&mut self, _: i64) -> bool {
            self.record("should-close");
            self.closed
        }
        fn set_should_close(&mut self, _: i64, value: bool) {
            self.closed = value;
        }
        fn set_clear_color(&mut self, color: [f32; 4]) {
            self.record(format!("color:{color:?}"));
        }
        fn clear(&mut self, mask: i32) {
            self.record(format!("clear:{mask}"));
        }
        fn swap_buffers(&mut self, id: i64) {
            self.record(format!("swap:{id}"));
        }
        fn poll_events(&mut self) -> Vec<WindowEvent> {
            self.record("poll");
            self.closed = true;
            vec![WindowEvent::Refresh {}]
        }
        fn free_callbacks(&mut self, id: i64) {
            self.record(format!("free-callbacks:{id}"));
        }
        fn destroy_window(&mut self, id: i64) {
            self.record(format!("destroy:{id}"));
        }
    }
    pub(crate) fn fixture() -> (Rc<SourceWindow>, ResourceRuntime, Log) {
        let runtime = ResourceRuntime::default();
        let log = Log::default();
        let parent_log = log.clone();
        let parent = runtime.allocate(&[], move || {
            parent_log.lock().unwrap().push("free-glfw".into())
        });
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            closed: false,
        }));
        (
            Rc::new(SourceWindow::from_handle(
                7,
                "initial".into(),
                backend,
                parent,
                &runtime,
            )),
            runtime,
            log,
        )
    }
    struct HandlerRecorder {
        name: &'static str,
        output: bool,
        log: Log,
    }
    impl InputHandler<SourceWindow> for HandlerRecorder {
        fn on_position(&mut self, blocked: bool, _: &SourceWindow, y: i32, x: i32) -> bool {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}:{blocked}:{y}:{x}", self.name));
            self.output
        }
        fn on_iconify(&mut self, _: bool, _: &SourceWindow, _: bool) -> bool {
            panic!("Window does not forward iconify to handlers")
        }
        fn before_events(&mut self, _: &SourceWindow) {
            self.log.lock().unwrap().push("handler-before".into());
        }
        fn after_events(&mut self, _: &SourceWindow) {
            self.log.lock().unwrap().push("handler-after".into());
        }
        fn on_refresh(&mut self, blocked: bool, _: &SourceWindow) -> bool {
            self.log.lock().unwrap().push("handler-refresh".into());
            blocked
        }
    }
    #[test]
    fn callbacks_precede_handler_folds_and_stacks_reset_blocking() {
        let (window, _, log) = fixture();
        let callback_log = log.clone();
        window
            .position_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, x, y| {
                callback_log
                    .lock()
                    .unwrap()
                    .push(format!("callback:{x}:{y}"))
            }));
        let handler = |name, output| -> Handler {
            Rc::new(RefCell::new(HandlerRecorder {
                name,
                output,
                log: log.clone(),
            }))
        };
        window.handler_stacks.borrow_mut().extend([
            vec![handler("a", true), handler("b", false), handler("c", true)],
            vec![handler("d", false)],
        ]);
        window.dispatch(WindowEvent::Position { xpos: 12, ypos: 34 });
        assert_eq!(
            *log.lock().unwrap(),
            [
                "callback:12:34",
                "a:false:34:12",
                "b:true:34:12",
                "c:false:34:12",
                "d:false:34:12"
            ]
        );
        window.emit_iconify(true);
    }
    #[test]
    fn native_queries_and_relative_mouse_preserve_source_coordinates() {
        let (window, runtime, log) = fixture();
        assert_eq!(window.mouse_position_relative(), [0.5, -0.5]);
        assert_eq!(*log.lock().unwrap(), ["mouse", "screen"]);
        assert_eq!(window.position(), [11, 22]);
        assert_eq!(window.framebuffer_size(), [800, 400]);
        window.set_title("changed".into());
        assert_eq!(window.title(), "changed");
        window.set_should_close(true);
        assert!(window.should_close());
        window.close();
        assert!(window.freed());
        runtime.run_main();
        assert!(
            log.lock()
                .unwrap()
                .windows(2)
                .any(|pair| pair == ["free-callbacks:7", "destroy:7"])
        );
    }
    #[test]
    fn frame_refresh_hooks_and_parent_cleanup_follow_source_order() {
        let (window, runtime, log) = fixture();
        let before = log.clone();
        window
            .before_events_callbacks
            .borrow_mut()
            .push(Rc::new(move |_| {
                before.lock().unwrap().push("before".into())
            }));
        let after = log.clone();
        window
            .after_events_callbacks
            .borrow_mut()
            .push(Rc::new(move |_| after.lock().unwrap().push("after".into())));
        window
            .handler_stacks
            .borrow_mut()
            .push(vec![Rc::new(RefCell::new(HandlerRecorder {
                name: "a",
                output: false,
                log: log.clone(),
            }))]);
        let draw_log = log.clone();
        window.start_default(Rc::new(move |win| {
            draw_log.lock().unwrap().push(format!("draw:{}", win.id))
        }));
        assert!(window.freed());
        assert_eq!(
            *log.lock().unwrap(),
            [
                "color:[0.0, 0.0, 0.0, 0.0]",
                "should-close",
                "clear:17664",
                "draw:7",
                "swap:7",
                "before",
                "handler-before",
                "poll",
                "clear:17664",
                "draw:7",
                "swap:7",
                "handler-refresh",
                "after",
                "handler-after",
                "should-close"
            ]
        );
        runtime.run_main();
        assert_eq!(
            &log.lock().unwrap()[15..],
            ["free-callbacks:7", "destroy:7", "free-glfw"]
        );
        window.refresh_callbacks.borrow_mut().clear(); // Break captured Rc cycle; JVM uses GC.
    }
}

/// Native operations required by Window.java's constructor. Callback installation
/// must wire the existing SourceWindow dispatch methods, not change their order.
pub(crate) trait WindowConstructorBackend {
    fn register_resource_dependency(&mut self);
    fn default_window_hints(&mut self);
    fn window_hint(&mut self, hint: i32, value: i32);
    fn is_macos(&mut self) -> bool;
    fn create_window(&mut self, size: [i32; 2], title: &str, monitor: i64, share: i64) -> i64;
    fn install_callbacks(&mut self, window: i64, callbacks: &[WindowCallbackKind]);
    fn install_framebuffer_viewport_callback(&mut self, window: i64);
    fn monitor_size(&mut self) -> [i32; 2];
    fn set_window_position(&mut self, window: i64, position: [i32; 2]);
    fn make_context_current(&mut self, window: i64);
    fn get_or_create_context(&mut self, window: i64);
    fn swap_interval(&mut self, interval: i32);
    fn show_window(&mut self, window: i64);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowCallbackKind {
    Position,
    Size,
    Close,
    Refresh,
    Focus,
    Iconify,
    FramebufferSize,
    Key,
    Char,
    CharMods,
    MouseButton,
    CursorPos,
    CursorEnter,
    Scroll,
    Drop,
}
const CALLBACK_ORDER: [WindowCallbackKind; 15] = [
    WindowCallbackKind::Position,
    WindowCallbackKind::Size,
    WindowCallbackKind::Close,
    WindowCallbackKind::Refresh,
    WindowCallbackKind::Focus,
    WindowCallbackKind::Iconify,
    WindowCallbackKind::FramebufferSize,
    WindowCallbackKind::Key,
    WindowCallbackKind::Char,
    WindowCallbackKind::CharMods,
    WindowCallbackKind::MouseButton,
    WindowCallbackKind::CursorPos,
    WindowCallbackKind::CursorEnter,
    WindowCallbackKind::Scroll,
    WindowCallbackKind::Drop,
];
/// The default constructor queries the monitor before entering the main
/// constructor, which then queries it again to center the created window.
pub(crate) fn initialize_window(
    backend: &mut dyn WindowConstructorBackend,
    title: &str,
    size: Option<[i32; 2]>,
) -> Result<i64, &'static str> {
    let size = size.unwrap_or_else(|| {
        let monitor = backend.monitor_size();
        [
            (monitor[0] as f64 * 0.8) as i32,
            (monitor[1] as f64 * 0.8) as i32,
        ]
    });
    backend.register_resource_dependency();
    backend.default_window_hints();
    for (hint, value) in [(131076, 0), (131075, 1), (139271, 1)] {
        backend.window_hint(hint, value);
    }
    if backend.is_macos() {
        for (hint, value) in [
            (139266, 3),
            (139267, 2),
            (139272, 204801),
            (139270, 1),
            (143361, 1),
        ] {
            backend.window_hint(hint, value);
        }
    }
    let window = backend.create_window(size, title, 0, 0);
    if window == 0 {
        return Err("Could not create game Window.");
    }
    backend.install_callbacks(window, &CALLBACK_ORDER);
    backend.install_framebuffer_viewport_callback(window);
    let monitor = backend.monitor_size();
    backend.set_window_position(
        window,
        [
            monitor[0].wrapping_sub(size[0]) / 2,
            monitor[1].wrapping_sub(size[1]) / 2,
        ],
    );
    backend.make_context_current(window);
    backend.get_or_create_context(window);
    backend.swap_interval(0);
    backend.show_window(window);
    Ok(window)
}
#[cfg(test)]
mod constructor_tests {
    use super::*;
    #[derive(Default)]
    struct Backend {
        calls: Vec<String>,
        mac: bool,
        fail: bool,
    }
    impl WindowConstructorBackend for Backend {
        fn register_resource_dependency(&mut self) {
            self.calls.push("resource".into());
        }
        fn default_window_hints(&mut self) {
            self.calls.push("defaults".into());
        }
        fn window_hint(&mut self, h: i32, v: i32) {
            self.calls.push(format!("hint:{h}:{v}"));
        }
        fn is_macos(&mut self) -> bool {
            self.mac
        }
        fn create_window(&mut self, s: [i32; 2], t: &str, m: i64, share: i64) -> i64 {
            self.calls.push(format!("create:{s:?}:{t}:{m}:{share}"));
            if self.fail { 0 } else { 9 }
        }
        fn install_callbacks(&mut self, id: i64, c: &[WindowCallbackKind]) {
            assert_eq!(c, &CALLBACK_ORDER);
            self.calls.push(format!("callbacks:{id}:{}", c.len()));
        }
        fn install_framebuffer_viewport_callback(&mut self, id: i64) {
            self.calls.push(format!("viewport-callback:{id}"));
        }
        fn monitor_size(&mut self) -> [i32; 2] {
            self.calls.push("monitor".into());
            [1921, 1081]
        }
        fn set_window_position(&mut self, id: i64, p: [i32; 2]) {
            self.calls.push(format!("position:{id}:{p:?}"));
        }
        fn make_context_current(&mut self, id: i64) {
            self.calls.push(format!("current:{id}"));
        }
        fn get_or_create_context(&mut self, id: i64) {
            self.calls.push(format!("context:{id}"));
        }
        fn swap_interval(&mut self, i: i32) {
            self.calls.push(format!("interval:{i}"));
        }
        fn show_window(&mut self, id: i64) {
            self.calls.push(format!("show:{id}"));
        }
    }
    #[test]
    fn constructor_default_size_and_context_setup_match_source_order() {
        let mut backend = Backend::default();
        assert_eq!(initialize_window(&mut backend, "ship", None), Ok(9));
        assert_eq!(
            backend.calls,
            [
                "monitor",
                "resource",
                "defaults",
                "hint:131076:0",
                "hint:131075:1",
                "hint:139271:1",
                "create:[1536, 864]:ship:0:0",
                "callbacks:9:15",
                "viewport-callback:9",
                "monitor",
                "position:9:[192, 108]",
                "current:9",
                "context:9",
                "interval:0",
                "show:9"
            ]
        );
    }
    #[test]
    fn mac_hints_and_creation_failure_skip_all_post_creation_operations() {
        let mut backend = Backend {
            mac: true,
            fail: true,
            ..Default::default()
        };
        assert_eq!(
            initialize_window(&mut backend, "ship", Some([2000, 1200])),
            Err("Could not create game Window.")
        );
        assert_eq!(
            backend.calls,
            [
                "resource",
                "defaults",
                "hint:131076:0",
                "hint:131075:1",
                "hint:139271:1",
                "hint:139266:3",
                "hint:139267:2",
                "hint:139272:204801",
                "hint:139270:1",
                "hint:143361:1",
                "create:[2000, 1200]:ship:0:0"
            ]
        );
    }
}

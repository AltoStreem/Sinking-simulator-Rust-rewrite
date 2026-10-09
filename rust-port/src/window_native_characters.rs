//! Win32 character packets retained before Winit merges key text or UTF-16 units.
//! The bundled SS2 GLFW DLL reports each WM_CHAR unit separately.
use crate::window::WindowEvent;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Packet {
    pub codepoint: u32,
    pub mods: i32,
    pub plain: bool,
    pub composing: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Boundary {
    Key,
    Mouse,
    Cursor,
    Scroll,
    Focus,
    Size,
    Position,
    Close,
}
impl Boundary {
    fn event(event: &WindowEvent) -> Option<Self> {
        Some(match event {
            WindowEvent::Key { .. } => Self::Key,
            WindowEvent::MouseButton { .. } => Self::Mouse,
            WindowEvent::CursorPos { .. } => Self::Cursor,
            WindowEvent::Scroll { .. } => Self::Scroll,
            WindowEvent::Focus { .. } => Self::Focus,
            WindowEvent::Size { .. } => Self::Size,
            WindowEvent::Position { .. } => Self::Position,
            WindowEvent::Close { .. } => Self::Close,
            _ => return None,
        })
    }
}
#[derive(Debug)]
enum Entry {
    Boundary(Boundary),
    Character(Packet),
}
#[derive(Default)]
struct Queue {
    entries: std::collections::VecDeque<Entry>,
    composing: bool,
}
impl Queue {
    fn message(&mut self, message: u32, value: usize, mods: i32) -> Option<isize> {
        match message {
            0x102 | 0x106 | 0x109 => {
                if message == 0x109 && value == 0xffff {
                    return Some(1);
                }
                let point = value as u32; // Bundled DLL forwards unsigned wParam before JVM GUI narrowing.
                if point >= 32 && !(127..160).contains(&point) {
                    self.entries.push_back(Entry::Character(Packet {
                        codepoint: point,
                        mods: mods & 15,
                        plain: message != 0x106,
                        composing: self.composing,
                    }));
                }
            }
            0x10d => self.composing = true,
            0x10e => self.composing = false,
            0x100 | 0x101 | 0x104 | 0x105 => self.entries.push_back(Entry::Boundary(Boundary::Key)),
            0x201 | 0x202 | 0x204 | 0x205 | 0x207 | 0x208 | 0x20b | 0x20c => {
                self.entries.push_back(Entry::Boundary(Boundary::Mouse))
            }
            0x200 => self.entries.push_back(Entry::Boundary(Boundary::Cursor)),
            0x20a | 0x20e => self.entries.push_back(Entry::Boundary(Boundary::Scroll)),
            7 | 8 => self.entries.push_back(Entry::Boundary(Boundary::Focus)),
            5 => self.entries.push_back(Entry::Boundary(Boundary::Size)),
            3 => self.entries.push_back(Entry::Boundary(Boundary::Position)),
            0x10 => self.entries.push_back(Entry::Boundary(Boundary::Close)),
            _ => {}
        }
        None
    }
    fn before(&mut self, event: &WindowEvent) -> Vec<Packet> {
        let Some(kind) = Boundary::event(event) else {
            return Vec::new();
        };
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| matches!(entry,Entry::Boundary(boundary) if *boundary==kind))
        else {
            return Vec::new();
        };
        self.entries
            .drain(..=index)
            .filter_map(|entry| {
                if let Entry::Character(packet) = entry {
                    Some(packet)
                } else {
                    None
                }
            })
            .collect()
    }
    fn finish(&mut self) -> Vec<Packet> {
        self.entries
            .drain(..)
            .filter_map(|entry| {
                if let Entry::Character(packet) = entry {
                    Some(packet)
                } else {
                    None
                }
            })
            .collect()
    }
}
#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        ffi::c_void,
        sync::{Arc, Mutex},
    };
    type Callback =
        unsafe extern "system" fn(*mut c_void, u32, usize, isize, usize, usize) -> isize;
    #[link(name = "comctl32")]
    unsafe extern "system" {
        fn SetWindowSubclass(
            window: *mut c_void,
            callback: Callback,
            id: usize,
            data: usize,
        ) -> i32;
        fn RemoveWindowSubclass(window: *mut c_void, callback: Callback, id: usize) -> i32;
        fn DefSubclassProc(window: *mut c_void, message: u32, value: usize, param: isize) -> isize;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetKeyState(key: i32) -> i16;
    }
    unsafe extern "system" fn callback(
        window: *mut c_void,
        message: u32,
        value: usize,
        param: isize,
        id: usize,
        data: usize,
    ) -> isize {
        // data holds a strong Arc for the installed subclass. No ECS/callback work
        // occurs under this lock; Winit continues to own the window procedure.
        let queue = unsafe { &*(data as *const Mutex<Queue>) };
        let mut mods = 0;
        for (key, bit) in [(0x10, 1), (0x11, 2), (0x12, 4), (0x5b, 8), (0x5c, 8)] {
            if unsafe { GetKeyState(key) } < 0 {
                mods |= bit;
            }
        }
        let result = queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .message(message, value, mods);
        if message == 0x82 {
            unsafe {
                if RemoveWindowSubclass(window, callback, id) != 0 {
                    drop(Arc::from_raw(data as *const Mutex<Queue>));
                }
            }
        }
        // WM_UNICHAR capability probing is answered directly as the bundled GLFW does.
        if let Some(result) = result {
            return result;
        }
        unsafe { DefSubclassProc(window, message, value, param) }
    }
    pub(crate) struct Hook {
        window: isize,
        id: usize,
        queue: Arc<Mutex<Queue>>,
    }
    impl Hook {
        pub(crate) fn install(window: isize) -> Option<Self> {
            if window == 0 {
                return None;
            }
            let queue = Arc::new(Mutex::new(Queue::default()));
            let pointer = Arc::into_raw(queue.clone());
            let id = pointer as usize;
            if unsafe { SetWindowSubclass(window as *mut c_void, callback, id, id) } == 0 {
                unsafe {
                    drop(Arc::from_raw(pointer));
                }
                return None;
            }
            Some(Self { window, id, queue })
        }
        pub(crate) fn before(&self, event: &WindowEvent) -> Vec<Packet> {
            self.queue
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .before(event)
        }
        pub(crate) fn finish(&self) -> Vec<Packet> {
            self.queue
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .finish()
        }
    }
    impl Drop for Hook {
        fn drop(&mut self) {
            // NCDESTROY already releases the callback Arc. Otherwise release it only
            // after a successful removal; a failed removal keeps callback data alive.
            if unsafe { RemoveWindowSubclass(self.window as *mut c_void, callback, self.id) } != 0 {
                unsafe {
                    drop(Arc::from_raw(self.id as *const Mutex<Queue>));
                }
            }
        }
    }
}
#[cfg(windows)]
pub(crate) use native::Hook;
#[cfg(not(windows))]
pub(crate) struct Hook;
#[cfg(not(windows))]
impl Hook {
    pub(crate) fn install(_: isize) -> Option<Self> {
        None
    }
    pub(crate) fn before(&self, _: &WindowEvent) -> Vec<Packet> {
        Vec::new()
    }
    pub(crate) fn finish(&self) -> Vec<Packet> {
        Vec::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_glfw_character_packets_keep_units_system_flag_filter_and_probe() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../tools/source-glfw-characters.json")).unwrap();
        for case in reference.as_array().unwrap() {
            let mut queue = Queue::default();
            let mut returns = Vec::new();
            for message in case["messages"].as_array().unwrap() {
                returns.push(
                    queue
                        .message(
                            message[0].as_u64().unwrap() as u32,
                            message[1].as_u64().unwrap() as usize,
                            i32::from(case["shift"].as_bool().unwrap()),
                        )
                        .unwrap_or(0),
                );
            }
            let mut callbacks = Vec::new();
            for packet in queue.finish() {
                callbacks.push(serde_json::json!(["mods", packet.codepoint, packet.mods]));
                if packet.plain {
                    callbacks.push(serde_json::json!(["char", packet.codepoint]));
                }
            }
            assert_eq!(
                serde_json::json!(callbacks),
                case["callbacks"],
                "{}",
                case["case"]
            );
            assert_eq!(serde_json::json!(returns), case["return"]);
        }
    }
    #[test]
    fn character_boundaries_preserve_key_char_cursor_key_char_order() {
        let mut queue = Queue::default();
        for (message, value) in [
            (0x100, 65),
            (0x102, 65),
            (0x200, 0),
            (0x100, 66),
            (0x102, 66),
        ] {
            queue.message(message, value, 0);
        }
        assert!(
            queue
                .before(&WindowEvent::Key {
                    key: 65,
                    scancode: 30,
                    action: 1,
                    mods: 0
                })
                .is_empty()
        );
        assert_eq!(
            queue.before(&WindowEvent::CursorPos {
                xpos: 10.0,
                ypos: 20.0
            })[0]
                .codepoint,
            65
        );
        assert!(
            queue
                .before(&WindowEvent::Key {
                    key: 66,
                    scancode: 48,
                    action: 1,
                    mods: 0
                })
                .is_empty()
        );
        assert_eq!(queue.finish()[0].codepoint, 66);
        queue.message(0x10d, 0, 0);
        queue.message(0x102, 65, 0);
        queue.message(0x10e, 0, 0);
        assert!(queue.finish()[0].composing);
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "Creates its own hidden Win32 window to exercise the installed native subclass"]
    fn actual_native_tap_matches_bundled_glfw_character_callbacks() {
        use std::{cell::RefCell, ffi::c_void, rc::Rc};
        #[link(name = "user32")]
        unsafe extern "system" {
            fn CreateWindowExW(
                ex: u32,
                class: *const u16,
                title: *const u16,
                style: u32,
                x: i32,
                y: i32,
                w: i32,
                h: i32,
                parent: *mut c_void,
                menu: *mut c_void,
                instance: *mut c_void,
                param: *mut c_void,
            ) -> *mut c_void;
            fn DestroyWindow(window: *mut c_void) -> i32;
            fn SendMessageW(window: *mut c_void, message: u32, value: usize, param: isize)
            -> isize;
            fn GetKeyboardState(keys: *mut u8) -> i32;
            fn SetKeyboardState(keys: *const u8) -> i32;
        }
        struct Window(*mut c_void);
        impl Drop for Window {
            fn drop(&mut self) {
                unsafe {
                    DestroyWindow(self.0);
                }
            }
        }
        struct Keys([u8; 256]);
        impl Drop for Keys {
            fn drop(&mut self) {
                unsafe {
                    SetKeyboardState(self.0.as_ptr());
                }
            }
        }
        let mut keys = [0; 256];
        assert_ne!(unsafe { GetKeyboardState(keys.as_mut_ptr()) }, 0);
        let _keys = Keys(keys);
        let class: Vec<u16> = "STATIC".encode_utf16().chain([0]).collect();
        let title: Vec<u16> = "SS2 Rust native character verification"
            .encode_utf16()
            .chain([0])
            .collect();
        let window = Window(unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                0,
                0,
                0,
                32,
                32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        });
        assert!(!window.0.is_null());
        let hook = Hook::install(window.0 as isize)
            .expect("Native subclass must install on the window's thread");
        let (source, _, _) = crate::window::tests::fixture();
        let observed = Rc::new(RefCell::new(Vec::new()));
        let saved = observed.clone();
        source
            .char_mods_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c, m| {
                saved.borrow_mut().push(serde_json::json!(["mods", c, m]))
            }));
        let saved = observed.clone();
        source
            .char_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c| {
                saved.borrow_mut().push(serde_json::json!(["char", c]))
            }));
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../tools/source-glfw-characters.json")).unwrap();
        let mut reset_count = 0;
        for case in reference.as_array().unwrap() {
            let mut keys = [0; 256];
            if case["shift"].as_bool().unwrap() {
                keys[0x10] = 0x80;
                keys[0xa0] = 0x80;
            }
            assert_ne!(unsafe { SetKeyboardState(keys.as_ptr()) }, 0);
            observed.borrow_mut().clear();
            let mut results = Vec::new();
            for message in case["messages"].as_array().unwrap() {
                results.push(unsafe {
                    SendMessageW(
                        window.0,
                        message[0].as_u64().unwrap() as u32,
                        message[1].as_u64().unwrap() as usize,
                        0,
                    )
                });
            }
            for packet in hook.finish() {
                crate::window_characters::native_packet(
                    &source,
                    packet,
                    |_| {},
                    || reset_count += 1,
                );
            }
            assert_eq!(
                serde_json::json!(*observed.borrow()),
                case["callbacks"],
                "{}",
                case["case"]
            );
            assert_eq!(
                results.last().copied().unwrap(),
                case["return"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()
                    .as_i64()
                    .unwrap() as isize
            );
            println!(
                "Native subclass matched original GLFW case {}",
                case["case"]
            );
        }
        assert_eq!(
            reset_count, 3,
            "Source char narrowing aliases both WM_CHAR/WM_UNICHAR U+10052 to R, plus Shift+R"
        );
        drop(window);
        drop(hook); // Test NCDESTROY followed by the Rust resource drop too.
    }
}

//! Translation of GUIKt.java state and description(), isolated from native ImGui drawing.
//! The original singleton is represented by one retained instance; the Bevy/native GUI adapter is pending.
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait DescriptionBackend {
    fn item_id(&mut self) -> i32;
    fn current_time_millis(&mut self) -> i64;
    fn is_item_hovered(&mut self, flags: i32) -> bool;
    fn begin_tooltip(&mut self);
    fn font_size(&mut self) -> f32;
    fn push_text_wrap_pos(&mut self, position: f32);
    fn text_ex(&mut self, text: &str, flags: i32, end: Option<usize>);
    fn pop_text_wrap_pos(&mut self);
    fn end_tooltip(&mut self);
}
/// Java finally executes even if a native backend operation throws. A cleanup panic replaces the body panic.
fn source_finally<B>(backend: &mut B, body: impl FnOnce(&mut B), cleanup: impl FnOnce(&mut B)) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(backend)));
    cleanup(backend);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}
pub type ToolList<T> = Rc<RefCell<Vec<Option<Rc<T>>>>>;
pub struct GuiKt<T: ?Sized, F: ?Sized> {
    scale: f32,
    tool_list: Option<ToolList<T>>,
    current_tool_index: i32,
    f18: Option<Rc<F>>,
    f12: Option<Rc<F>>,
    descriptions: DescriptionState,
}
impl<T: ?Sized, F: ?Sized> Default for GuiKt<T, F> {
    fn default() -> Self {
        Self {
            scale: 1.0,
            tool_list: None,
            current_tool_index: 0,
            f18: None,
            f12: None,
            descriptions: DescriptionState::default(),
        }
    }
}
impl<T: ?Sized, F: ?Sized> GuiKt<T, F> {
    pub fn scale(&self) -> f32 {
        self.scale
    }
    pub fn access_set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
    pub fn tool_list(&self) -> ToolList<T> {
        self.tool_list
            .as_ref()
            .expect("UninitializedPropertyAccessException: toolList")
            .clone()
    }
    pub fn access_set_tool_list(&mut self, list: Option<ToolList<T>>) {
        self.tool_list = list;
    }
    pub fn current_tool_index(&self) -> i32 {
        self.current_tool_index
    }
    pub fn set_current_tool_index(&mut self, index: i32) {
        self.current_tool_index = index;
    }
    pub fn current_tool(&self) -> Option<Rc<T>> {
        let list = self.tool_list();
        let list = list.borrow();
        list.get(self.current_tool_index as usize)
            .expect("IndexOutOfBoundsException: currentToolIndex")
            .clone()
    }
    pub fn f18(&self) -> Rc<F> {
        self.f18
            .as_ref()
            .expect("UninitializedPropertyAccessException: f18")
            .clone()
    }
    pub fn f12(&self) -> Rc<F> {
        self.f12
            .as_ref()
            .expect("UninitializedPropertyAccessException: f12")
            .clone()
    }
    pub fn access_set_f18(&mut self, font: Option<Rc<F>>) {
        self.f18 = font;
    }
    pub fn access_set_f12(&mut self, font: Option<Rc<F>>) {
        self.f12 = font;
    }
    pub fn description(&mut self, desc: &str, backend: &mut impl DescriptionBackend) {
        self.descriptions.description(desc, backend);
    }
}
/// Shared source description timer, usable by the native and active renderers.
#[derive(Default)]
pub(crate) struct DescriptionState {
    last_hovered: HashMap<i32, i64>,
}
impl DescriptionState {
    pub(crate) fn description(&mut self, desc: &str, backend: &mut impl DescriptionBackend) {
        let id = backend.item_id();
        let last = *self
            .last_hovered
            .entry(id)
            .or_insert_with(|| backend.current_time_millis());
        if backend.is_item_hovered(0) {
            if last < backend.current_time_millis().wrapping_sub(300) {
                backend.begin_tooltip();
                source_finally(
                    backend,
                    |backend| {
                        let wrap = backend.font_size() * 35.0;
                        backend.push_text_wrap_pos(wrap);
                        source_finally(
                            backend,
                            |backend| backend.text_ex(desc, 0, None),
                            |backend| backend.pop_text_wrap_pos(),
                        );
                    },
                    |backend| backend.end_tooltip(),
                );
            }
        } else {
            self.last_hovered.insert(id, backend.current_time_millis());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    struct Backend {
        id: i32,
        hovered: bool,
        times: VecDeque<i64>,
        log: Vec<String>,
        fail: Option<&'static str>,
    }
    impl Backend {
        fn event(&mut self, event: &str) {
            self.log.push(event.into());
            if self.fail == Some(event) {
                panic!("{event}");
            }
        }
    }
    impl DescriptionBackend for Backend {
        fn item_id(&mut self) -> i32 {
            self.event("id");
            self.id
        }
        fn current_time_millis(&mut self) -> i64 {
            self.event("clock");
            self.times.pop_front().unwrap()
        }
        fn is_item_hovered(&mut self, flags: i32) -> bool {
            assert_eq!(flags, 0);
            self.event("hover");
            self.hovered
        }
        fn begin_tooltip(&mut self) {
            self.event("begin");
        }
        fn font_size(&mut self) -> f32 {
            self.event("font");
            18.0
        }
        fn push_text_wrap_pos(&mut self, position: f32) {
            assert_eq!(position, 630.0);
            self.event("push");
        }
        fn text_ex(&mut self, text: &str, flags: i32, end: Option<usize>) {
            assert_eq!(text, "description");
            assert_eq!(flags, 0);
            assert_eq!(end, None);
            self.event("text");
        }
        fn pop_text_wrap_pos(&mut self) {
            self.event("pop");
        }
        fn end_tooltip(&mut self) {
            self.event("end");
        }
    }
    fn backend(times: &[i64]) -> Backend {
        Backend {
            id: 7,
            hovered: true,
            times: times.iter().copied().collect(),
            log: vec![],
            fail: None,
        }
    }
    #[test]
    fn global_state_defaults_aliases_nullable_entries_and_uninitialized_getters() {
        let mut gui = GuiKt::<String, String>::default();
        assert_eq!(gui.scale(), 1.0);
        assert_eq!(gui.current_tool_index(), 0);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.tool_list())).is_err()
        );
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.f18())).is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.f12())).is_err());
        let tool = Rc::new("tool".to_string());
        let list = Rc::new(RefCell::new(vec![Some(tool.clone()), None]));
        gui.access_set_tool_list(Some(list.clone()));
        assert!(Rc::ptr_eq(&list, &gui.tool_list()));
        assert!(Rc::ptr_eq(&tool, &gui.current_tool().unwrap()));
        gui.set_current_tool_index(1);
        assert!(gui.current_tool().is_none());
        gui.set_current_tool_index(-1);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.current_tool())).is_err()
        );
        gui.set_current_tool_index(2);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.current_tool())).is_err()
        );
        list.borrow_mut().push(Some(tool.clone()));
        assert!(Rc::ptr_eq(&tool, &gui.current_tool().unwrap()));
        gui.access_set_scale(f32::NAN);
        assert!(gui.scale().is_nan());
        let font = Rc::new("font".to_string());
        gui.access_set_f18(Some(font.clone()));
        gui.access_set_f12(Some(font.clone()));
        assert!(Rc::ptr_eq(&font, &gui.f18()));
        assert!(Rc::ptr_eq(&font, &gui.f12()));
        gui.access_set_f12(None);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gui.f12())).is_err());
    }
    #[test]
    fn descriptions_observe_strict_delay_reset_and_original_clock_read_order() {
        let mut gui = GuiKt::<(), ()>::default();
        let mut backend = backend(&[1000, 1300, 1301, 1500, 1800, 1801]);
        gui.description("description", &mut backend);
        assert_eq!(backend.log, ["id", "clock", "hover", "clock"]);
        backend.log.clear();
        gui.description("description", &mut backend);
        assert_eq!(
            backend.log,
            [
                "id", "hover", "clock", "begin", "font", "push", "text", "pop", "end"
            ]
        );
        backend.log.clear();
        backend.hovered = false;
        gui.description("description", &mut backend);
        assert_eq!(backend.log, ["id", "hover", "clock"]);
        backend.log.clear();
        backend.hovered = true;
        gui.description("description", &mut backend);
        assert_eq!(backend.log, ["id", "hover", "clock"]);
        backend.log.clear();
        gui.description("description", &mut backend);
        assert_eq!(
            backend.log,
            [
                "id", "hover", "clock", "begin", "font", "push", "text", "pop", "end"
            ]
        );
        assert!(backend.times.is_empty());
    }
    #[test]
    fn tooltip_finally_cleanup_matches_each_throw_boundary() {
        for (failure, expected) in [
            ("begin", vec!["begin"]),
            ("font", vec!["begin", "font", "end"]),
            ("push", vec!["begin", "font", "push", "end"]),
            ("text", vec!["begin", "font", "push", "text", "pop", "end"]),
            ("pop", vec!["begin", "font", "push", "text", "pop", "end"]),
            ("end", vec!["begin", "font", "push", "text", "pop", "end"]),
        ] {
            let mut gui = GuiKt::<(), ()>::default();
            let mut backend = backend(&[0, 301]);
            backend.fail = Some(failure);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || gui.description("description", &mut backend)
                ))
                .is_err()
            );
            let draw: Vec<_> = backend.log.iter().skip(4).map(String::as_str).collect();
            assert_eq!(draw, expected);
        }
    }
}

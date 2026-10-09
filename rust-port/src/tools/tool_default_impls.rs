//! Recovered Tool$DefaultImpls.class: explicit InputHandler default calls.
#![allow(dead_code, unused_variables)]
use crate::input_handler::InputHandler;
struct DefaultReceiver;
impl InputHandler<crate::window::SourceWindow> for DefaultReceiver {}

pub(crate) fn on_position<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    ypos: i32,
    xpos: i32,
) -> bool {
    InputHandler::on_position(&mut DefaultReceiver, blocked, win, ypos, xpos)
}
pub(crate) fn on_size<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    height: i32,
    width: i32,
) -> bool {
    InputHandler::on_size(&mut DefaultReceiver, blocked, win, height, width)
}
pub(crate) fn on_close<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
) -> bool {
    InputHandler::on_close(&mut DefaultReceiver, blocked, win)
}
pub(crate) fn on_refresh<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
) -> bool {
    InputHandler::on_refresh(&mut DefaultReceiver, blocked, win)
}
pub(crate) fn on_focus<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    focus: bool,
) -> bool {
    InputHandler::on_focus(&mut DefaultReceiver, blocked, win, focus)
}
pub(crate) fn on_iconify<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    iconify: bool,
) -> bool {
    InputHandler::on_iconify(&mut DefaultReceiver, blocked, win, iconify)
}
pub(crate) fn on_framebuffer_size<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    width: i32,
    height: i32,
) -> bool {
    InputHandler::on_framebuffer_size(&mut DefaultReceiver, blocked, win, width, height)
}
pub(crate) fn on_key<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    key: i32,
    scancode: i32,
    action: i32,
    mods: i32,
) -> bool {
    InputHandler::on_key(
        &mut DefaultReceiver,
        blocked,
        win,
        key,
        scancode,
        action,
        mods,
    )
}
pub(crate) fn on_char<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    codepoint: i32,
) -> bool {
    InputHandler::on_char(&mut DefaultReceiver, blocked, win, codepoint)
}
pub(crate) fn on_char_mods<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    codepoint: i32,
    mods: i32,
) -> bool {
    InputHandler::on_char_mods(&mut DefaultReceiver, blocked, win, codepoint, mods)
}
pub(crate) fn on_mouse_button<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    button: i32,
    action: i32,
    mods: i32,
) -> bool {
    InputHandler::on_mouse_button(&mut DefaultReceiver, blocked, win, button, action, mods)
}
pub(crate) fn on_cursor_pos<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    xpos: f64,
    ypos: f64,
) -> bool {
    InputHandler::on_cursor_pos(&mut DefaultReceiver, blocked, win, xpos, ypos)
}
pub(crate) fn on_cursor_enter<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    enter: bool,
) -> bool {
    InputHandler::on_cursor_enter(&mut DefaultReceiver, blocked, win, enter)
}
pub(crate) fn on_scroll<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    x: f64,
    y: f64,
) -> bool {
    InputHandler::on_scroll(&mut DefaultReceiver, blocked, win, x, y)
}
pub(crate) fn on_drop<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    blocked: bool,
    win: &crate::window::SourceWindow,
    data: &[String],
) -> bool {
    InputHandler::on_drop(&mut DefaultReceiver, blocked, win, data)
}
pub(crate) fn before_events<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    win: &crate::window::SourceWindow,
) {
    InputHandler::before_events(&mut DefaultReceiver, win);
}
pub(crate) fn after_events<T: InputHandler<crate::window::SourceWindow> + ?Sized>(
    _tool: &mut T,
    win: &crate::window::SourceWindow,
) {
    InputHandler::after_events(&mut DefaultReceiver, win);
}

#[cfg(test)]
mod tests {
    use super::*;
    struct ToolReceiver;
    impl InputHandler<crate::window::SourceWindow> for ToolReceiver {
        fn on_close(&mut self, _: bool, _: &crate::window::SourceWindow) -> bool {
            panic!("must bypass override")
        }
        fn before_events(&mut self, _: &crate::window::SourceWindow) {
            panic!("must bypass override")
        }
        fn after_events(&mut self, _: &crate::window::SourceWindow) {
            panic!("must bypass override")
        }
    }
    #[test]
    fn explicit_defaults_preserve_blocking_and_bypass_overrides() {
        let mut tool = ToolReceiver;
        let (window, _runtime, _log) = crate::window::tests::fixture();
        for blocked in [false, true] {
            let outcomes = [
                on_position(&mut tool, blocked, &window, -1, 2),
                on_size(&mut tool, blocked, &window, 0, -3),
                on_close(&mut tool, blocked, &window),
                on_refresh(&mut tool, blocked, &window),
                on_focus(&mut tool, blocked, &window, false),
                on_iconify(&mut tool, blocked, &window, true),
                on_framebuffer_size(&mut tool, blocked, &window, -1, 0),
                on_key(&mut tool, blocked, &window, -1, 2, 3, 4),
                on_char(&mut tool, blocked, &window, -1),
                on_char_mods(&mut tool, blocked, &window, -1, 2),
                on_mouse_button(&mut tool, blocked, &window, -1, 2, 3),
                on_cursor_pos(&mut tool, blocked, &window, f64::NAN, f64::INFINITY),
                on_cursor_enter(&mut tool, blocked, &window, false),
                on_scroll(&mut tool, blocked, &window, f64::NAN, f64::NEG_INFINITY),
                on_drop(&mut tool, blocked, &window, &["a".into()]),
            ];
            assert!(outcomes.into_iter().all(|result| result == blocked));
        }
        before_events(&mut tool, &window);
        after_events(&mut tool, &window);
    }
}

//! InputHandler.java callback defaults. W is the source window's eventual Rust representation.
#![allow(dead_code, unused_variables)]
pub(crate) trait InputHandler<W: ?Sized> {
    fn on_position(&mut self, blocked: bool, win: &W, ypos: i32, xpos: i32) -> bool {
        blocked
    }
    fn on_size(&mut self, blocked: bool, win: &W, height: i32, width: i32) -> bool {
        blocked
    }
    fn on_close(&mut self, blocked: bool, win: &W) -> bool {
        blocked
    }
    fn on_refresh(&mut self, blocked: bool, win: &W) -> bool {
        blocked
    }
    fn on_focus(&mut self, blocked: bool, win: &W, focus: bool) -> bool {
        blocked
    }
    fn on_iconify(&mut self, blocked: bool, win: &W, iconify: bool) -> bool {
        blocked
    }
    fn on_framebuffer_size(&mut self, blocked: bool, win: &W, width: i32, height: i32) -> bool {
        blocked
    }
    fn on_key(
        &mut self,
        blocked: bool,
        win: &W,
        key: i32,
        scancode: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        blocked
    }
    fn on_char(&mut self, blocked: bool, win: &W, codepoint: i32) -> bool {
        blocked
    }
    fn on_char_mods(&mut self, blocked: bool, win: &W, codepoint: i32, mods: i32) -> bool {
        blocked
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        win: &W,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        blocked
    }
    fn on_cursor_pos(&mut self, blocked: bool, win: &W, xpos: f64, ypos: f64) -> bool {
        blocked
    }
    fn on_cursor_enter(&mut self, blocked: bool, win: &W, enter: bool) -> bool {
        blocked
    }
    fn on_scroll(&mut self, blocked: bool, win: &W, x: f64, y: f64) -> bool {
        blocked
    }
    fn on_drop(&mut self, blocked: bool, win: &W, data: &[String]) -> bool {
        blocked
    }
    fn before_events(&mut self, win: &W) {}
    fn after_events(&mut self, win: &W) {}
}

//! Remaining compiled `Window` GLFW callback bodies, separated from the event loop.
//! Every callback list is invoked before the input-handler folds. Each fold starts
//! with `false`, even when the previous handler stack returned `true`.
use super::SourceWindow;

pub(super) fn position(window: &SourceWindow, xpos: i32, ypos: i32) {
    for callback in window.position_callbacks.borrow().iter() {
        callback(window, xpos, ypos);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_position(blocked, window, ypos, xpos);
        }
    }
}

pub(super) fn refresh(window: &SourceWindow) {
    for callback in window.refresh_callbacks.borrow().iter() {
        callback(window);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_refresh(blocked, window);
        }
    }
}

pub(super) fn close(window: &SourceWindow) {
    for callback in window.close_callbacks.borrow().iter() {
        callback(window);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_close(blocked, window);
        }
    }
}

pub(super) fn focus(window: &SourceWindow, focus: bool) {
    for callback in window.focus_callbacks.borrow().iter() {
        callback(window, focus);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_focus(blocked, window, focus);
        }
    }
}

pub(super) fn iconify(window: &SourceWindow, iconify: bool) {
    // The original lambda notifies callbacks only; it never visits handler stacks.
    for callback in window.iconify_callbacks.borrow().iter() {
        callback(window, iconify);
    }
}

pub(super) fn framebuffer_size(window: &SourceWindow, width: i32, height: i32) {
    for callback in window.framebuffer_size_callbacks.borrow().iter() {
        callback(window, width, height);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_framebuffer_size(blocked, window, width, height);
        }
    }
}

pub(super) fn key(window: &SourceWindow, key: i32, scancode: i32, action: i32, mods: i32) {
    for callback in window.key_callbacks.borrow().iter() {
        callback(window, key, scancode, action, mods);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_key(blocked, window, key, scancode, action, mods);
        }
    }
}

pub(super) fn character(window: &SourceWindow, codepoint: i32) {
    for callback in window.char_callbacks.borrow().iter() {
        callback(window, codepoint);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_char(blocked, window, codepoint);
        }
    }
}

pub(super) fn character_mods(window: &SourceWindow, codepoint: i32, mods: i32) {
    for callback in window.char_mods_callbacks.borrow().iter() {
        callback(window, codepoint, mods);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler
                .borrow_mut()
                .on_char_mods(blocked, window, codepoint, mods);
        }
    }
}

pub(super) fn cursor_enter(window: &SourceWindow, enter: bool) {
    for callback in window.cursor_enter_callbacks.borrow().iter() {
        callback(window, enter);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_cursor_enter(blocked, window, enter);
        }
    }
}

pub(super) fn drop(window: &SourceWindow, data: &[String]) {
    for callback in window.drop_callbacks.borrow().iter() {
        callback(window, data);
    }
    for stack in window.handler_stacks.borrow().iter() {
        let mut blocked = false;
        for handler in stack {
            blocked = handler.borrow_mut().on_drop(blocked, window, data);
        }
    }
}

//! InputHandlerDelegate.java broadcasts each original blocked flag before aggregating results.
#![allow(dead_code)]
use crate::input_handler::InputHandler;
pub(crate) trait InputHandlerDelegate<W: ?Sized> {
    fn handlers(&mut self) -> &mut [Box<dyn InputHandler<W>>];
    fn on_position(&mut self, blocked: bool, win: &W, ypos: i32, xpos: i32) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_position(blocked, win, ypos, xpos))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_size(&mut self, blocked: bool, win: &W, height: i32, width: i32) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_size(blocked, win, height, width))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_close(&mut self, blocked: bool, win: &W) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_close(blocked, win))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_refresh(&mut self, blocked: bool, win: &W) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_refresh(blocked, win))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_focus(&mut self, blocked: bool, win: &W, focus: bool) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_focus(blocked, win, focus))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_iconify(&mut self, blocked: bool, win: &W, iconify: bool) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_iconify(blocked, win, iconify))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_framebuffer_size(&mut self, blocked: bool, win: &W, width: i32, height: i32) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_framebuffer_size(blocked, win, width, height))
            .collect();
        results.into_iter().any(|result| result)
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
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_key(blocked, win, key, scancode, action, mods))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_char(&mut self, blocked: bool, win: &W, codepoint: i32) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_char(blocked, win, codepoint))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_char_mods(&mut self, blocked: bool, win: &W, codepoint: i32, mods: i32) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_char_mods(blocked, win, codepoint, mods))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        win: &W,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_mouse_button(blocked, win, button, action, mods))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_cursor_pos(&mut self, blocked: bool, win: &W, xpos: f64, ypos: f64) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_cursor_pos(blocked, win, xpos, ypos))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_cursor_enter(&mut self, blocked: bool, win: &W, enter: bool) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_cursor_enter(blocked, win, enter))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_scroll(&mut self, blocked: bool, win: &W, x: f64, y: f64) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_scroll(blocked, win, x, y))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn on_drop(&mut self, blocked: bool, win: &W, data: &[String]) -> bool {
        let results: Vec<bool> = self
            .handlers()
            .iter_mut()
            .map(|handler| handler.on_drop(blocked, win, data))
            .collect();
        results.into_iter().any(|result| result)
    }
    fn before_events(&mut self, win: &W) {
        for handler in self.handlers() {
            handler.before_events(win);
        }
    }
    fn after_events(&mut self, win: &W) {
        for handler in self.handlers() {
            handler.after_events(win);
        }
    }
}
impl<W: ?Sized, T: InputHandlerDelegate<W>> InputHandler<W> for T {
    fn on_position(&mut self, blocked: bool, win: &W, ypos: i32, xpos: i32) -> bool {
        InputHandlerDelegate::on_position(self, blocked, win, ypos, xpos)
    }
    fn on_size(&mut self, blocked: bool, win: &W, height: i32, width: i32) -> bool {
        InputHandlerDelegate::on_size(self, blocked, win, height, width)
    }
    fn on_close(&mut self, blocked: bool, win: &W) -> bool {
        InputHandlerDelegate::on_close(self, blocked, win)
    }
    fn on_refresh(&mut self, blocked: bool, win: &W) -> bool {
        InputHandlerDelegate::on_refresh(self, blocked, win)
    }
    fn on_focus(&mut self, blocked: bool, win: &W, focus: bool) -> bool {
        InputHandlerDelegate::on_focus(self, blocked, win, focus)
    }
    fn on_iconify(&mut self, blocked: bool, win: &W, iconify: bool) -> bool {
        InputHandlerDelegate::on_iconify(self, blocked, win, iconify)
    }
    fn on_framebuffer_size(&mut self, blocked: bool, win: &W, width: i32, height: i32) -> bool {
        InputHandlerDelegate::on_framebuffer_size(self, blocked, win, width, height)
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
        InputHandlerDelegate::on_key(self, blocked, win, key, scancode, action, mods)
    }
    fn on_char(&mut self, blocked: bool, win: &W, codepoint: i32) -> bool {
        InputHandlerDelegate::on_char(self, blocked, win, codepoint)
    }
    fn on_char_mods(&mut self, blocked: bool, win: &W, codepoint: i32, mods: i32) -> bool {
        InputHandlerDelegate::on_char_mods(self, blocked, win, codepoint, mods)
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        win: &W,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        InputHandlerDelegate::on_mouse_button(self, blocked, win, button, action, mods)
    }
    fn on_cursor_pos(&mut self, blocked: bool, win: &W, xpos: f64, ypos: f64) -> bool {
        InputHandlerDelegate::on_cursor_pos(self, blocked, win, xpos, ypos)
    }
    fn on_cursor_enter(&mut self, blocked: bool, win: &W, enter: bool) -> bool {
        InputHandlerDelegate::on_cursor_enter(self, blocked, win, enter)
    }
    fn on_scroll(&mut self, blocked: bool, win: &W, x: f64, y: f64) -> bool {
        InputHandlerDelegate::on_scroll(self, blocked, win, x, y)
    }
    fn on_drop(&mut self, blocked: bool, win: &W, data: &[String]) -> bool {
        InputHandlerDelegate::on_drop(self, blocked, win, data)
    }
    fn before_events(&mut self, win: &W) {
        InputHandlerDelegate::before_events(self, win)
    }
    fn after_events(&mut self, win: &W) {
        InputHandlerDelegate::after_events(self, win)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    type Log = Rc<RefCell<Vec<(usize, String, bool, u64, String)>>>;
    struct Recorder {
        id: usize,
        result: bool,
        log: Log,
    }
    impl Recorder {
        fn event(&self, name: String, blocked: bool, win: u64, payload: String) {
            self.log
                .borrow_mut()
                .push((self.id, name, blocked, win, payload));
        }
    }
    impl InputHandler<u64> for Recorder {
        fn on_position(&mut self, blocked: bool, win: &u64, ypos: i32, xpos: i32) -> bool {
            self.event(
                format!("on_position"),
                blocked,
                *win,
                format!("{:?}", (ypos, xpos,)),
            );
            self.result
        }
        fn on_size(&mut self, blocked: bool, win: &u64, height: i32, width: i32) -> bool {
            self.event(
                format!("on_size"),
                blocked,
                *win,
                format!("{:?}", (height, width,)),
            );
            self.result
        }
        fn on_close(&mut self, blocked: bool, win: &u64) -> bool {
            self.event(format!("on_close"), blocked, *win, format!("{:?}", ()));
            self.result
        }
        fn on_refresh(&mut self, blocked: bool, win: &u64) -> bool {
            self.event(format!("on_refresh"), blocked, *win, format!("{:?}", ()));
            self.result
        }
        fn on_focus(&mut self, blocked: bool, win: &u64, focus: bool) -> bool {
            self.event(
                format!("on_focus"),
                blocked,
                *win,
                format!("{:?}", (focus,)),
            );
            self.result
        }
        fn on_iconify(&mut self, blocked: bool, win: &u64, iconify: bool) -> bool {
            self.event(
                format!("on_iconify"),
                blocked,
                *win,
                format!("{:?}", (iconify,)),
            );
            self.result
        }
        fn on_framebuffer_size(
            &mut self,
            blocked: bool,
            win: &u64,
            width: i32,
            height: i32,
        ) -> bool {
            self.event(
                format!("on_framebuffer_size"),
                blocked,
                *win,
                format!("{:?}", (width, height,)),
            );
            self.result
        }
        fn on_key(
            &mut self,
            blocked: bool,
            win: &u64,
            key: i32,
            scancode: i32,
            action: i32,
            mods: i32,
        ) -> bool {
            self.event(
                format!("on_key"),
                blocked,
                *win,
                format!("{:?}", (key, scancode, action, mods,)),
            );
            self.result
        }
        fn on_char(&mut self, blocked: bool, win: &u64, codepoint: i32) -> bool {
            self.event(
                format!("on_char"),
                blocked,
                *win,
                format!("{:?}", (codepoint,)),
            );
            self.result
        }
        fn on_char_mods(&mut self, blocked: bool, win: &u64, codepoint: i32, mods: i32) -> bool {
            self.event(
                format!("on_char_mods"),
                blocked,
                *win,
                format!("{:?}", (codepoint, mods,)),
            );
            self.result
        }
        fn on_mouse_button(
            &mut self,
            blocked: bool,
            win: &u64,
            button: i32,
            action: i32,
            mods: i32,
        ) -> bool {
            self.event(
                format!("on_mouse_button"),
                blocked,
                *win,
                format!("{:?}", (button, action, mods,)),
            );
            self.result
        }
        fn on_cursor_pos(&mut self, blocked: bool, win: &u64, xpos: f64, ypos: f64) -> bool {
            self.event(
                format!("on_cursor_pos"),
                blocked,
                *win,
                format!("{:?}", (xpos, ypos,)),
            );
            self.result
        }
        fn on_cursor_enter(&mut self, blocked: bool, win: &u64, enter: bool) -> bool {
            self.event(
                format!("on_cursor_enter"),
                blocked,
                *win,
                format!("{:?}", (enter,)),
            );
            self.result
        }
        fn on_scroll(&mut self, blocked: bool, win: &u64, x: f64, y: f64) -> bool {
            self.event(
                format!("on_scroll"),
                blocked,
                *win,
                format!("{:?}", (x, y,)),
            );
            self.result
        }
        fn on_drop(&mut self, blocked: bool, win: &u64, data: &[String]) -> bool {
            self.event(format!("on_drop"), blocked, *win, format!("{:?}", (data,)));
            self.result
        }
        fn before_events(&mut self, win: &u64) {
            self.event("before_events".into(), false, *win, String::new());
        }
        fn after_events(&mut self, win: &u64) {
            self.event("after_events".into(), false, *win, String::new());
        }
    }
    struct Group {
        handlers: Vec<Box<dyn InputHandler<u64>>>,
    }
    impl InputHandlerDelegate<u64> for Group {
        fn handlers(&mut self) -> &mut [Box<dyn InputHandler<u64>>] {
            &mut self.handlers
        }
    }
    fn group(results: &[bool], log: Log) -> Group {
        Group {
            handlers: results
                .iter()
                .enumerate()
                .map(|(id, result)| {
                    Box::new(Recorder {
                        id,
                        result: *result,
                        log: log.clone(),
                    }) as Box<dyn InputHandler<u64>>
                })
                .collect(),
        }
    }
    struct DefaultHandler;
    impl InputHandler<u64> for DefaultHandler {}
    fn every_event(handler: &mut dyn InputHandler<u64>, blocked: bool) -> Vec<bool> {
        let win = 71;
        let dropped = vec!["a.ship".into(), "b.ship".into()];
        vec![
            handler.on_position(blocked, &win, 3, 4),
            handler.on_size(blocked, &win, 5, 6),
            handler.on_close(blocked, &win),
            handler.on_refresh(blocked, &win),
            handler.on_focus(blocked, &win, true),
            handler.on_iconify(blocked, &win, false),
            handler.on_framebuffer_size(blocked, &win, 7, 8),
            handler.on_key(blocked, &win, 9, 10, 11, 12),
            handler.on_char(blocked, &win, 13),
            handler.on_char_mods(blocked, &win, 14, 15),
            handler.on_mouse_button(blocked, &win, 16, 17, 18),
            handler.on_cursor_pos(blocked, &win, 19.5, 20.5),
            handler.on_cursor_enter(blocked, &win, true),
            handler.on_scroll(blocked, &win, 21.5, 22.5),
            handler.on_drop(blocked, &win, &dropped),
        ]
    }
    #[test]
    fn default_handlers_preserve_blocked_flag_for_all_fifteen_events() {
        assert_eq!(every_event(&mut DefaultHandler, false), vec![false; 15]);
        assert_eq!(every_event(&mut DefaultHandler, true), vec![true; 15]);
        DefaultHandler.before_events(&71);
        DefaultHandler.after_events(&71);
    }
    #[test]
    fn delegate_broadcasts_original_blocked_and_all_payloads_without_short_circuit() {
        let log = Rc::new(RefCell::new(vec![]));
        let mut g = group(&[true, false, true], log.clone());
        assert_eq!(every_event(&mut g, false), vec![true; 15]);
        let events = log.borrow();
        assert_eq!(events.len(), 45);
        for chunk in events.chunks_exact(3) {
            assert_eq!(chunk.iter().map(|v| v.0).collect::<Vec<_>>(), [0, 1, 2]);
            assert!(chunk.iter().all(|v| !v.2 && v.3 == 71));
            assert!(chunk.iter().all(|v| v.1 == chunk[0].1 && v.4 == chunk[0].4));
        }
        let first: Vec<_> = events
            .chunks_exact(3)
            .map(|chunk| (chunk[0].1.as_str(), chunk[0].4.as_str()))
            .collect();
        assert_eq!(
            first,
            [
                ("on_position", "(3, 4)"),
                ("on_size", "(5, 6)"),
                ("on_close", "()"),
                ("on_refresh", "()"),
                ("on_focus", "(true,)"),
                ("on_iconify", "(false,)"),
                ("on_framebuffer_size", "(7, 8)"),
                ("on_key", "(9, 10, 11, 12)"),
                ("on_char", "(13,)"),
                ("on_char_mods", "(14, 15)"),
                ("on_mouse_button", "(16, 17, 18)"),
                ("on_cursor_pos", "(19.5, 20.5)"),
                ("on_cursor_enter", "(true,)"),
                ("on_scroll", "(21.5, 22.5)"),
                ("on_drop", "([\"a.ship\", \"b.ship\"],)")
            ]
        );
    }
    #[test]
    fn empty_delegate_returns_false_and_event_hooks_run_in_handler_order() {
        let log = Rc::new(RefCell::new(vec![]));
        let mut empty = group(&[], log.clone());
        assert_eq!(every_event(&mut empty, true), vec![false; 15]);
        let mut g = group(&[false, false], log.clone());
        assert_eq!(every_event(&mut g, true), vec![false; 15]);
        assert!(log.borrow().iter().all(|v| v.2));
        log.borrow_mut().clear();
        InputHandler::before_events(&mut g, &71);
        InputHandler::after_events(&mut g, &71);
        assert_eq!(
            log.borrow()
                .iter()
                .map(|v| (v.0, v.1.clone()))
                .collect::<Vec<_>>(),
            [
                (0, "before_events".into()),
                (1, "before_events".into()),
                (0, "after_events".into()),
                (1, "after_events".into())
            ]
        );
    }
}

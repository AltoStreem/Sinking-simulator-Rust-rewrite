//! Toolbox.render window scaffolding and ShipScroll sizing, translated from the source.
//! The active Bevy renderer still needs a backend that applies these scopes and clipping.
use crate::{
    toolbox::SourceToolbox,
    toolbox_render_3::{SizeCallbackBackend, SizeConstraintCallback},
};
use bevy::math::Vec2;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCondition {
    Always,
    Once,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowFlag {
    NoDecoration,
    NoBackground,
    AlwaysAutoResize,
    NoSavedSettings,
    NoFocusOnAppearing,
    NoNav,
    NoMove,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarFlag {
    TabListPopupButton,
    FittingPolicyResizeDown,
}
pub const TOOLS_FLAGS: &[WindowFlag] = &[
    WindowFlag::NoDecoration,
    WindowFlag::NoBackground,
    WindowFlag::AlwaysAutoResize,
    WindowFlag::NoSavedSettings,
    WindowFlag::NoFocusOnAppearing,
    WindowFlag::NoNav,
    WindowFlag::NoMove,
];

/// Symbolic flags are mapped by the renderer to its native ImGui equivalents.
pub trait LayoutBackend: SizeCallbackBackend {
    fn set_next_window_position(&mut self, position: Vec2, condition: WindowCondition, pivot: Vec2);
    fn set_next_window_size_constraints(
        &mut self,
        minimum: Vec2,
        maximum: Vec2,
        callback: SizeConstraintCallback,
    );
    fn begin_window(&mut self, name: &str, open: Option<&mut bool>, flags: &[WindowFlag]) -> bool;
    fn end_window(&mut self);
    fn window_height(&mut self) -> f32;
    fn begin_tab_bar(&mut self, name: &str, flags: &[TabBarFlag]) -> bool;
    fn end_tab_bar(&mut self);
    fn content_region_available(&mut self) -> Vec2;
    fn is_left_mouse_dragging(&mut self, threshold: f32) -> bool;
    fn window_position(&mut self) -> Vec2;
    fn window_padding(&mut self) -> Vec2;
    fn cursor_position_y(&mut self) -> f32;
    fn begin_child(&mut self, name: &str, size: Vec2, border: bool, flags: i32) -> bool;
    fn end_child(&mut self);
}

/// Frame-local FloatRef shared with the retained window size callback.
pub struct LayoutFrame {
    pub padding: f32,
    pub tools_height: Rc<Cell<f32>>,
}
impl LayoutFrame {
    pub fn ship_scroll_size(&self, backend: &mut impl LayoutBackend) -> Vec2 {
        let width = backend.content_region_available().x;
        let height = if !backend.is_left_mouse_dragging(0.0) {
            // The JVM converts display height to int before the remaining float arithmetic.
            let display_height = backend.display_size().y as i32 as f32;
            display_height
                - backend.window_position().y
                - backend.window_padding().y
                - backend.cursor_position_y()
                - self.padding
                - self.tools_height.get()
        } else {
            backend.content_region_available().y
        };
        // Preserve zero/negative values: beginChild resolves these relative to remaining space.
        Vec2::new(width, height)
    }

    pub fn render_ship_scroll<B: LayoutBackend>(
        &self,
        backend: &mut B,
        contents: impl FnOnce(&mut B),
    ) {
        let size = self.ship_scroll_size(backend);
        if backend.begin_child("ShipScroll", size, false, 0) {
            finally(backend, contents, |b| b.end_child());
        } else {
            backend.end_child();
        }
    }
}

fn finally<B>(backend: &mut B, body: impl FnOnce(&mut B), end: impl FnOnce(&mut B)) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(backend)));
    end(backend);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

impl SourceToolbox {
    /// Native source render order: Tools, constrained Toolbox/ToolboxTabs, then ShipUpload.
    /// Tab bodies use the existing settings/browser ports; no Steam path is introduced here.
    pub fn render_layout<B: LayoutBackend>(
        &mut self,
        backend: &mut B,
        tools: impl FnOnce(&mut Self, &mut B),
        tabs: impl FnOnce(&mut Self, &LayoutFrame, &mut B),
        ship_upload: impl FnOnce(&mut B),
    ) {
        let frame = LayoutFrame {
            tools_height: Rc::new(Cell::new(0.0)),
            padding: 10.0 * backend.gui_scale(),
        };
        if self.tools_visible() {
            let y = backend.display_size().y as i32 as f32 - frame.padding;
            backend.set_next_window_position(
                Vec2::new(frame.padding, y),
                WindowCondition::Always,
                Vec2::new(0.0, 1.0),
            );
            let mut open = self.tools_visible();
            let visible = backend.begin_window("Tools", Some(&mut open), TOOLS_FLAGS);
            self.set_tools_visible(open);
            if visible {
                finally(backend, |b| tools(self, b), |b| b.end_window());
            } else {
                backend.end_window();
            }
            // The original reads this after end(), even when begin() returns false.
            frame
                .tools_height
                .set(backend.window_height() + frame.padding);
        }
        backend.set_next_window_position(
            Vec2::splat(frame.padding),
            WindowCondition::Once,
            Vec2::ZERO,
        );
        backend.set_next_window_size_constraints(
            Vec2::splat(-1.0),
            Vec2::splat(-1.0),
            SizeConstraintCallback {
                padding: frame.padding,
                tools_height: frame.tools_height.clone(),
            },
        );
        if backend.begin_window("Toolbox", None, &[WindowFlag::AlwaysAutoResize]) {
            finally(
                backend,
                |b| {
                    if b.begin_tab_bar(
                        "ToolboxTabs",
                        &[
                            TabBarFlag::TabListPopupButton,
                            TabBarFlag::FittingPolicyResizeDown,
                        ],
                    ) {
                        finally(b, |b| tabs(self, &frame, b), |b| b.end_tab_bar());
                    }
                },
                |b| b.end_window(),
            );
        } else {
            backend.end_window();
        }
        ship_upload(backend);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Backend {
        events: Vec<String>,
        callback: Option<SizeConstraintCallback>,
        tools_begin: bool,
        toolbox_begin: bool,
        tabs_begin: bool,
        child_begin: bool,
        dragging: bool,
        close_tools: bool,
        display_y: f32,
    }
    impl SizeCallbackBackend for Backend {
        fn display_size(&mut self) -> Vec2 {
            self.events.push("display".into());
            Vec2::new(1280.0, self.display_y)
        }
        fn gui_scale(&mut self) -> f32 {
            self.events.push("scale".into());
            1.5
        }
    }
    impl LayoutBackend for Backend {
        fn set_next_window_position(&mut self, p: Vec2, c: WindowCondition, pivot: Vec2) {
            self.events.push(format!("position:{p:?}:{c:?}:{pivot:?}"));
        }
        fn set_next_window_size_constraints(
            &mut self,
            min: Vec2,
            max: Vec2,
            cb: SizeConstraintCallback,
        ) {
            assert_eq!(min, Vec2::splat(-1.0));
            assert_eq!(max, min);
            self.events.push("constraints".into());
            self.callback = Some(cb);
        }
        fn begin_window(
            &mut self,
            name: &str,
            open: Option<&mut bool>,
            flags: &[WindowFlag],
        ) -> bool {
            self.events.push(format!("begin:{name}"));
            if name == "Tools" {
                assert_eq!(flags, TOOLS_FLAGS);
                let open = open.unwrap();
                if self.close_tools {
                    *open = false;
                }
                self.tools_begin
            } else {
                assert!(open.is_none());
                assert_eq!(flags, &[WindowFlag::AlwaysAutoResize]);
                self.toolbox_begin
            }
        }
        fn end_window(&mut self) {
            self.events.push("end-window".into());
        }
        fn window_height(&mut self) -> f32 {
            self.events.push("height".into());
            160.0
        }
        fn begin_tab_bar(&mut self, name: &str, flags: &[TabBarFlag]) -> bool {
            assert_eq!(name, "ToolboxTabs");
            assert_eq!(
                flags,
                &[
                    TabBarFlag::TabListPopupButton,
                    TabBarFlag::FittingPolicyResizeDown
                ]
            );
            self.events.push("begin-tabs".into());
            self.tabs_begin
        }
        fn end_tab_bar(&mut self) {
            self.events.push("end-tabs".into());
        }
        fn content_region_available(&mut self) -> Vec2 {
            self.events.push("available".into());
            Vec2::new(320.0, 99.0)
        }
        fn is_left_mouse_dragging(&mut self, threshold: f32) -> bool {
            assert_eq!(threshold, 0.0);
            self.events.push("dragging".into());
            self.dragging
        }
        fn window_position(&mut self) -> Vec2 {
            self.events.push("window-position".into());
            Vec2::new(20.0, 30.0)
        }
        fn window_padding(&mut self) -> Vec2 {
            self.events.push("padding".into());
            Vec2::splat(8.0)
        }
        fn cursor_position_y(&mut self) -> f32 {
            self.events.push("cursor".into());
            100.0
        }
        fn begin_child(&mut self, name: &str, size: Vec2, border: bool, flags: i32) -> bool {
            assert_eq!(name, "ShipScroll");
            assert!(!border);
            assert_eq!(flags, 0);
            self.events.push(format!("child:{size:?}"));
            self.child_begin
        }
        fn end_child(&mut self) {
            self.events.push("end-child".into());
        }
    }
    #[test]
    fn source_frame_orders_windows_reserves_height_and_keeps_callback_alias() {
        let mut toolbox = SourceToolbox::default();
        let mut b = Backend {
            display_y: 720.9,
            tools_begin: true,
            toolbox_begin: true,
            tabs_begin: true,
            child_begin: true,
            ..Default::default()
        };
        toolbox.render_layout(
            &mut b,
            |_, b| b.events.push("tools".into()),
            |_, frame, b| {
                assert_eq!(frame.tools_height.get(), 175.0);
                assert!(Rc::ptr_eq(
                    &frame.tools_height,
                    &b.callback.as_ref().unwrap().tools_height
                ));
                frame.render_ship_scroll(b, |b| b.events.push("ships".into()));
            },
            |b| b.events.push("upload".into()),
        );
        assert_eq!(
            b.events,
            [
                "scale",
                "display",
                "position:Vec2(15.0, 705.0):Always:Vec2(0.0, 1.0)",
                "begin:Tools",
                "tools",
                "end-window",
                "height",
                "position:Vec2(15.0, 15.0):Once:Vec2(0.0, 0.0)",
                "constraints",
                "begin:Toolbox",
                "begin-tabs",
                "available",
                "dragging",
                "display",
                "window-position",
                "padding",
                "cursor",
                "child:Vec2(320.0, 392.0)",
                "ships",
                "end-child",
                "end-tabs",
                "end-window",
                "upload"
            ]
        );
    }
    #[test]
    fn hidden_tools_and_collapsed_windows_preserve_source_scope_rules() {
        let mut toolbox = SourceToolbox::default();
        toolbox.set_tools_visible(false);
        let mut b = Backend::default();
        toolbox.render_layout(
            &mut b,
            |_, _| panic!("hidden"),
            |_, _, _| panic!("collapsed"),
            |b| b.events.push("upload".into()),
        );
        assert_eq!(b.callback.as_ref().unwrap().tools_height.get(), 0.0);
        assert_eq!(b.events.last().unwrap(), "upload");
        assert_eq!(
            b.events
                .iter()
                .filter(|e| e.as_str() == "end-window")
                .count(),
            1
        );
        let mut b = Backend {
            close_tools: true,
            ..Default::default()
        };
        toolbox.set_tools_visible(true);
        toolbox.render_layout(
            &mut b,
            |_, _| panic!("collapsed"),
            |_, _, _| panic!("collapsed"),
            |_| {},
        );
        assert!(!toolbox.tools_visible());
        assert_eq!(b.callback.as_ref().unwrap().tools_height.get(), 175.0);
    }
    #[test]
    fn ship_scroll_drag_branch_and_negative_height_are_not_normalized() {
        let frame = LayoutFrame {
            padding: 15.0,
            tools_height: Rc::new(Cell::new(175.0)),
        };
        let mut b = Backend {
            dragging: true,
            ..Default::default()
        };
        assert_eq!(frame.ship_scroll_size(&mut b), Vec2::new(320.0, 99.0));
        assert_eq!(b.events, ["available", "dragging", "available"]);
        b.dragging = false;
        b.display_y = 50.9;
        b.events.clear();
        assert_eq!(frame.ship_scroll_size(&mut b), Vec2::new(320.0, -278.0));
        frame.render_ship_scroll(&mut b, |_| panic!("collapsed child"));
        assert_eq!(b.events.last().unwrap(), "end-child");
    }
    #[test]
    fn failing_child_unwinds_all_open_scopes_and_does_not_render_upload() {
        let mut toolbox = SourceToolbox::default();
        toolbox.set_tools_visible(false);
        let mut b = Backend {
            toolbox_begin: true,
            tabs_begin: true,
            child_begin: true,
            ..Default::default()
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            toolbox.render_layout(
                &mut b,
                |_, _| {},
                |_, frame, b| frame.render_ship_scroll(b, |_| panic!("body")),
                |_| panic!("upload should not run"),
            );
        }));
        assert!(result.is_err());
        assert_eq!(
            &b.events[b.events.len() - 3..],
            ["end-child", "end-tabs", "end-window"]
        );
    }
}

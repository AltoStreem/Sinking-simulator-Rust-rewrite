//! dslfix.class: original tab scopes, texture sizing and thumbnail centering helpers.
use crate::toolbox_references::BoolReference;
use bevy::math::Vec2;
use std::{cell::RefCell, rc::Rc};

pub trait TextureDimensions {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}
impl TextureDimensions for crate::texture_2d::SourceTexture2D {
    fn width(&self) -> i32 {
        self.width
    }
    fn height(&self) -> i32 {
        self.height
    }
}
pub trait GeometryBackend {
    fn gui_scale(&mut self) -> f32;
    fn window_content_region_width(&mut self) -> f32;
    fn window_height(&mut self) -> f32;
    fn frame_padding(&mut self) -> Vec2;
    fn scroll_max_y(&mut self) -> f32;
    fn window_width(&mut self) -> f32;
    fn scrollbar_size(&mut self) -> f32;
    fn set_cursor_pos_x(&mut self, x: f32);
}
pub trait TabBackend {
    fn begin_tab_bar(&mut self, id: &str, flags: i32) -> bool;
    fn end_tab_bar(&mut self);
    fn begin_tab_item_property(
        &mut self,
        label: &str,
        open: Option<&dyn BoolReference>,
        flags: i32,
    ) -> bool;
    fn begin_tab_item_array(
        &mut self,
        label: &str,
        open: Rc<RefCell<Vec<bool>>>,
        index: i32,
        flags: i32,
    ) -> bool;
    fn end_tab_item(&mut self);
}
fn scope<B>(backend: &mut B, body: impl FnOnce(&mut B), end: impl FnOnce(&mut B)) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(backend)));
    end(backend);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}
pub struct DslFix;
impl DslFix {
    pub fn tab_bar<B: TabBackend>(
        backend: &mut B,
        id: &str,
        flags: i32,
        body: impl FnOnce(&mut B),
    ) {
        if backend.begin_tab_bar(id, flags) {
            scope(backend, body, |b| b.end_tab_bar());
        }
    }
    pub fn tab_bar_default<B: TabBackend>(
        backend: &mut B,
        id: &str,
        flags: i32,
        mask: i32,
        body: impl FnOnce(&mut B),
    ) {
        Self::tab_bar(backend, id, if mask & 2 != 0 { 0 } else { flags }, body);
    }
    pub fn tab_item<B: TabBackend>(
        backend: &mut B,
        label: &str,
        open: Option<&dyn BoolReference>,
        flags: i32,
        body: impl FnOnce(&mut B),
    ) {
        if backend.begin_tab_item_property(label, open, flags) {
            scope(backend, body, |b| b.end_tab_item());
        }
    }
    pub fn tab_item_default<B: TabBackend>(
        backend: &mut B,
        label: &str,
        open: Option<&dyn BoolReference>,
        flags: i32,
        mask: i32,
        body: impl FnOnce(&mut B),
    ) {
        Self::tab_item(
            backend,
            label,
            if mask & 2 != 0 { None } else { open },
            if mask & 4 != 0 { 0 } else { flags },
            body,
        );
    }
    pub fn tab_item_array<B: TabBackend>(
        backend: &mut B,
        label: &str,
        open: Rc<RefCell<Vec<bool>>>,
        index: i32,
        flags: i32,
        body: impl FnOnce(&mut B),
    ) {
        if backend.begin_tab_item_array(label, open, index, flags) {
            scope(backend, body, |b| b.end_tab_item());
        }
    }
    pub fn tab_item_array_default<B: TabBackend>(
        backend: &mut B,
        label: &str,
        open: Rc<RefCell<Vec<bool>>>,
        index: i32,
        flags: i32,
        mask: i32,
        body: impl FnOnce(&mut B),
    ) {
        Self::tab_item_array(
            backend,
            label,
            open,
            index,
            if mask & 8 != 0 { 0 } else { flags },
            body,
        );
    }
    pub fn adjust_texture_size(
        texture: &impl TextureDimensions,
        max_height: f64,
        padding: Vec2,
        backend: &mut impl GeometryBackend,
    ) -> Vec2 {
        let width = texture.width();
        let height = texture.height();
        let dim = Vec2::new(width as f32, height as f32) * backend.gui_scale();
        let ratio = dim.y as f64 / dim.x as f64;
        let inv_ratio = 1.0 / ratio;
        let size = if dim.x < backend.window_content_region_width() && (dim.y as f64) < max_height {
            dim
        } else if ratio * (backend.window_content_region_width() as f64) > max_height {
            Vec2::new((inv_ratio * max_height) as f32, max_height as f32)
        } else {
            let x = backend.window_content_region_width();
            let y = (ratio * (backend.window_content_region_width() as f64)) as f32;
            Vec2::new(x, y)
        };
        size - padding * 2.0
    }
    pub fn adjust_texture_size_default(
        texture: &impl TextureDimensions,
        mut max_height: f64,
        mut padding: Vec2,
        mask: i32,
        backend: &mut impl GeometryBackend,
    ) -> Vec2 {
        if mask & 2 != 0 {
            max_height = backend.window_height() as f64 * 0.5;
        }
        if mask & 4 != 0 {
            padding = backend.frame_padding();
        }
        Self::adjust_texture_size(texture, max_height, padding, backend)
    }
    pub fn center_next_element(size: Vec2, padding: Vec2, backend: &mut impl GeometryBackend) {
        let x = if backend.scroll_max_y() > 0.0 {
            (backend.window_width() - backend.scrollbar_size() - size.x - 2.0 * padding.x) / 2.0
        } else {
            (backend.window_width() - size.x - 2.0 * padding.x) / 2.0
        };
        backend.set_cursor_pos_x(x);
    }
    pub fn center_next_element_default(
        size: Vec2,
        padding: Vec2,
        mask: i32,
        backend: &mut impl GeometryBackend,
    ) {
        let padding = if mask & 2 != 0 {
            backend.frame_padding()
        } else {
            padding
        };
        Self::center_next_element(size, padding, backend);
    }
}

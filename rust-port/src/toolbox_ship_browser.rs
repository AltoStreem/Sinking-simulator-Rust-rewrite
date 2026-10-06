//! Toolbox.render ShipScroll contents: UTF-16 filtering, thumbnail drawing and source ship replacement order.
use crate::toolbox::SourceToolbox;
use bevy::math::{Vec2, Vec4};
use std::{cell::RefCell, rc::Rc};

pub trait BrowserThumbnail {
    type Texture: crate::dslfix::TextureDimensions;
    fn name_utf16(&self) -> Vec<u16>;
    /// getResource(TEXTURE, default layer)?.getTexture()
    fn default_texture(&self) -> Option<Rc<Self::Texture>>;
}
pub trait BrowserBackend<T: BrowserThumbnail>: crate::dslfix::GeometryBackend {
    fn frame_padding_y(&mut self) -> f32;
    fn image_button(
        &mut self,
        texture: &T::Texture,
        size: Vec2,
        uv0: Vec2,
        uv1: Vec2,
        padding: i32,
        background: Vec4,
        tint: Vec4,
    ) -> bool;
    /// Original GUIKt.description(name), with UTF-16 input retained for native binding.
    fn description_utf16(&mut self, name: &[u16]);
}
pub trait BrowserShipOperations<T> {
    type Ship;
    type Control;
    fn global_ship(&mut self) -> Rc<Self::Ship>;
    fn close_ship(&mut self, ship: &Self::Ship);
    fn camera_control(&mut self, ship: &Self::Ship) -> Rc<Self::Control>;
    fn construct_ship(&mut self, thumbnail: Rc<T>, control: Rc<Self::Control>) -> Rc<Self::Ship>;
    fn set_global_ship(&mut self, ship: Rc<Self::Ship>);
    fn clear_ship_list(&mut self);
    fn add_ship(&mut self, ship: Rc<Self::Ship>);
}
impl SourceToolbox {
    pub fn render_ship_scroll_contents<T: BrowserThumbnail>(
        &self,
        thumbnails: &[Rc<T>],
        backend: &mut impl BrowserBackend<T>,
        ships: &mut impl BrowserShipOperations<T>,
    ) {
        let filter = self.ship_filter();
        let filter = filter.borrow();
        // Arrays.copyOfRange(0, indexOf(NUL)) fails for an unterminated filter; do not silently accept it.
        let end = filter
            .iter()
            .position(|&c| c == 0)
            .expect("IllegalArgumentException: shipFilter has no NUL terminator");
        let search = filter[..end].to_vec();
        drop(filter);
        // Kotlin filter eagerly builds a new list before drawing any entry.
        let filtered: Vec<_> = thumbnails
            .iter()
            .filter(|thumbnail| {
                crate::jvm_character::contains_ignore_case(&thumbnail.name_utf16(), &search)
            })
            .cloned()
            .collect();
        for thumbnail in filtered {
            let Some(texture) = thumbnail.default_texture() else {
                continue;
            };
            let padding = backend.frame_padding_y() as i32; // JVM float-to-int truncation/saturation.
            let padding_vec = Vec2::splat(padding as f32);
            let size = crate::dslfix::DslFix::adjust_texture_size_default(
                texture.as_ref(),
                0.0,
                padding_vec,
                2,
                backend,
            );
            crate::dslfix::DslFix::center_next_element(size, padding_vec, backend);
            if backend.image_button(
                &texture,
                size,
                Vec2::ZERO,
                Vec2::ONE,
                padding,
                Vec4::ZERO,
                Vec4::ONE,
            ) {
                let previous = ships.global_ship();
                ships.close_ship(&previous);
                // Original getter occurs after close; it can resolve a different global Ship.
                let current = ships.global_ship();
                let control = ships.camera_control(&current);
                let replacement = ships.construct_ship(thumbnail.clone(), control);
                ships.set_global_ship(replacement);
                ships.clear_ship_list();
                let current = ships.global_ship();
                ships.add_ship(current);
            }
            backend.description_utf16(&thumbnail.name_utf16());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    type Log = Rc<RefCell<Vec<String>>>;
    struct Thumb {
        name: String,
        texture: Option<Rc<i32>>,
        log: Log,
    }
    impl BrowserThumbnail for Thumb {
        type Texture = i32;
        fn name_utf16(&self) -> Vec<u16> {
            self.log.borrow_mut().push(format!("name:{}", self.name));
            self.name.encode_utf16().collect()
        }
        fn default_texture(&self) -> Option<Rc<i32>> {
            self.log.borrow_mut().push(format!("texture:{}", self.name));
            self.texture.clone()
        }
    }
    struct Backend {
        log: Log,
        clicked: bool,
    }
    impl crate::dslfix::TextureDimensions for i32 {
        fn width(&self) -> i32 {
            200
        }
        fn height(&self) -> i32 {
            50
        }
    }
    impl crate::dslfix::GeometryBackend for Backend {
        fn gui_scale(&mut self) -> f32 {
            self.log.borrow_mut().push("adjust".into());
            1.0
        }
        fn window_content_region_width(&mut self) -> f32 {
            1000.0
        }
        fn window_height(&mut self) -> f32 {
            1000.0
        }
        fn frame_padding(&mut self) -> Vec2 {
            panic!("explicit padding must be retained")
        }
        fn scroll_max_y(&mut self) -> f32 {
            0.0
        }
        fn window_width(&mut self) -> f32 {
            1000.0
        }
        fn scrollbar_size(&mut self) -> f32 {
            panic!("no vertical scrollbar")
        }
        fn set_cursor_pos_x(&mut self, x: f32) {
            assert_eq!(x, 400.0);
            self.log.borrow_mut().push("center".into());
        }
    }
    impl BrowserBackend<Thumb> for Backend {
        fn frame_padding_y(&mut self) -> f32 {
            self.log.borrow_mut().push("padding".into());
            3.9
        }
        fn image_button(
            &mut self,
            _: &i32,
            size: Vec2,
            uv0: Vec2,
            uv1: Vec2,
            padding: i32,
            bg: Vec4,
            tint: Vec4,
        ) -> bool {
            assert_eq!(size, Vec2::new(194.0, 44.0));
            assert_eq!((uv0, uv1), (Vec2::ZERO, Vec2::ONE));
            assert_eq!(padding, 3);
            assert_eq!((bg, tint), (Vec4::ZERO, Vec4::ONE));
            self.log.borrow_mut().push("button".into());
            self.clicked
        }
        fn description_utf16(&mut self, name: &[u16]) {
            self.log
                .borrow_mut()
                .push(format!("description:{}", String::from_utf16(name).unwrap()));
        }
    }
    struct Ships {
        log: Log,
        current: Rc<i32>,
        list: Vec<Rc<i32>>,
        fail: bool,
    }
    impl BrowserShipOperations<Thumb> for Ships {
        type Ship = i32;
        type Control = i32;
        fn global_ship(&mut self) -> Rc<i32> {
            self.log
                .borrow_mut()
                .push(format!("global:{}", self.current));
            self.current.clone()
        }
        fn close_ship(&mut self, ship: &i32) {
            self.log.borrow_mut().push(format!("close:{ship}"));
            self.current = Rc::new(2);
        }
        fn camera_control(&mut self, ship: &i32) -> Rc<i32> {
            assert_eq!(*ship, 2);
            self.log.borrow_mut().push("control:2".into());
            Rc::new(20)
        }
        fn construct_ship(&mut self, _: Rc<Thumb>, control: Rc<i32>) -> Rc<i32> {
            assert_eq!(*control, 20);
            self.log.borrow_mut().push("construct".into());
            if self.fail {
                panic!("construction failed");
            }
            Rc::new(3)
        }
        fn set_global_ship(&mut self, ship: Rc<i32>) {
            self.log.borrow_mut().push("set:3".into());
            self.current = ship;
        }
        fn clear_ship_list(&mut self) {
            self.log.borrow_mut().push("clear".into());
            self.list.clear();
        }
        fn add_ship(&mut self, ship: Rc<i32>) {
            self.log.borrow_mut().push("add:3".into());
            assert!(Rc::ptr_eq(&ship, &self.current));
            self.list.push(ship);
        }
    }
    fn fixtures() -> (SourceToolbox, Vec<Rc<Thumb>>, Backend, Ships, Log) {
        let toolbox = SourceToolbox::default();
        let log = Rc::new(RefCell::new(Vec::new()));
        let thumbs = vec![
            Rc::new(Thumb {
                name: "TITANIC".into(),
                texture: Some(Rc::new(9)),
                log: log.clone(),
            }),
            Rc::new(Thumb {
                name: "Missing".into(),
                texture: None,
                log: log.clone(),
            }),
        ];
        let backend = Backend {
            log: log.clone(),
            clicked: true,
        };
        let ships = Ships {
            log: log.clone(),
            current: Rc::new(1),
            list: vec![Rc::new(1)],
            fail: false,
        };
        (toolbox, thumbs, backend, ships, log)
    }
    #[test]
    fn toolbox_browser_filters_eagerly_draws_defaults_and_requeries_global_after_close() {
        let (toolbox, thumbs, mut backend, mut ships, log) = fixtures();
        assert_eq!(toolbox.ship_filter().borrow().len(), 256);
        toolbox.render_ship_scroll_contents(&thumbs, &mut backend, &mut ships);
        assert_eq!(
            &*log.borrow(),
            &[
                "name:TITANIC",
                "name:Missing",
                "texture:TITANIC",
                "padding",
                "adjust",
                "center",
                "button",
                "global:1",
                "close:1",
                "global:2",
                "control:2",
                "construct",
                "set:3",
                "clear",
                "global:3",
                "add:3",
                "name:TITANIC",
                "description:TITANIC",
                "texture:Missing"
            ]
        );
        assert_eq!(ships.list.len(), 1);
        assert!(Rc::ptr_eq(&ships.list[0], &ships.current));
    }
    #[test]
    fn toolbox_browser_filter_alias_replacement_and_unterminated_failure() {
        let (mut toolbox, thumbs, mut backend, mut ships, log) = fixtures();
        let replacement = Rc::new(RefCell::new(vec![116, 105, 0]));
        toolbox.access_set_ship_filter(replacement.clone());
        assert!(Rc::ptr_eq(&replacement, &toolbox.ship_filter()));
        backend.clicked = false;
        toolbox.render_ship_scroll_contents(&thumbs, &mut backend, &mut ships);
        assert!(
            !log.borrow()
                .iter()
                .any(|item| item == "texture:Missing" || item.starts_with("global:"))
        );
        *replacement.borrow_mut() = vec![65];
        let count = log.borrow().len();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || toolbox.render_ship_scroll_contents(&thumbs, &mut backend, &mut ships)
            ))
            .is_err()
        );
        assert_eq!(log.borrow().len(), count);
    }
    #[test]
    fn toolbox_browser_failed_ship_construction_stops_before_publication_and_tooltip() {
        let (toolbox, thumbs, mut backend, mut ships, log) = fixtures();
        ships.fail = true;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || toolbox.render_ship_scroll_contents(&thumbs, &mut backend, &mut ships)
            ))
            .is_err()
        );
        assert_eq!(log.borrow().last().unwrap(), "construct");
        assert_eq!(*ships.current, 2);
        assert_eq!(*ships.list[0], 1);
    }
}

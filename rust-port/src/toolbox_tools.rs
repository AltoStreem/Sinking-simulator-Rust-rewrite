//! Toolbox.render Tools window contents. Native renderer binding remains pending.
use crate::{
    gui_kt::GuiKt,
    toolbox::SourceToolbox,
    toolbox_references::{IntReference, LayerObject},
    toolbox_settings::SettingsBackend,
    tools::tool::SourceTool,
    window::SourceWindow,
};
use bevy::math::{Vec2, Vec4};
use std::cell::RefCell;

pub(crate) trait ToolsBackend<Texture, Font: ?Sized>: SettingsBackend {
    // Two separate global-ship getters occur in the source argument evaluation order.
    fn global_ship_layer_receiver(&mut self) -> LayerObject;
    fn global_ship_layer_names(&mut self) -> Vec<String>;
    fn layer_combo(
        &mut self,
        label: &str,
        current: &dyn IntReference,
        names: &[String],
        count: i32,
        popup_height: i32,
    ) -> bool;
    fn texture_id(&mut self, texture: &Texture) -> i32;
    fn frame_height(&mut self) -> f32;
    fn image_button(
        &mut self,
        texture: i32,
        size: Vec2,
        uv0: Vec2,
        uv1: Vec2,
        padding: i32,
        background: Vec4,
        tint: Vec4,
    ) -> bool;
    fn same_line(&mut self, offset: f32, spacing: f32);
    fn push_font(&mut self, font: &Font);
    fn pop_font(&mut self);
}

impl SourceToolbox {
    pub(crate) fn render_tools<T: ?Sized + SourceTool<SourceWindow>, F: ?Sized>(
        gui: &mut GuiKt<RefCell<T>, F>,
        backend: &mut impl ToolsBackend<T::Texture, F>,
    ) {
        let current = crate::toolbox_references::toolbox_render_2_1::PropertyReference {
            receiver: backend.global_ship_layer_receiver(),
        };
        let names = backend.global_ship_layer_names();
        // widgetsComboBox.combo$default mask 24 substitutes names.length and -1.
        backend.layer_combo("Show Layer", &current, &names, names.len() as i32, -1);
        let list = gui.tool_list();
        let list = list.borrow();
        for (index, entry) in list.iter().enumerate() {
            let index =
                i32::try_from(index).expect("ArithmeticException: Index overflow has happened.");
            let Some(tool) = entry else {
                continue;
            };
            let tool = tool.borrow();
            let texture = if gui.current_tool_index() == index {
                tool.active_texture()
            } else {
                tool.texture()
            };
            let texture = backend.texture_id(texture);
            // Vec2's synthetic mask 2 duplicates x into y, making the icon square.
            let size = Vec2::splat(backend.frame_height() * 3.0 - 4.0 * gui.scale());
            let padding = (2.0 * gui.scale()) as i32;
            if backend.image_button(
                texture,
                size,
                Vec2::ZERO,
                Vec2::ONE,
                padding,
                Vec4::ZERO,
                Vec4::ONE,
            ) {
                gui.set_current_tool_index(if gui.current_tool_index() == index {
                    0
                } else {
                    index
                });
            }
            gui.description(tool.name(), backend);
            if index != gui.tool_list().borrow().len() as i32 - 1 {
                // sameLine$default uses offset 0 and spacing -1 (current style spacing).
                backend.same_line(0.0, -1.0);
            }
        }
        Self::render_tool_size(backend);
        let font = gui.f12();
        backend.push_font(font.as_ref());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            backend.text("You can hide the tools in the graphics settings");
        }));
        backend.pop_font();
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gui_kt::DescriptionBackend,
        toolbox_references::{self as properties, BoolReference, FloatReference, Value},
    };
    use std::rc::Rc;
    struct TestTool(i32);
    impl crate::input_handler::InputHandler<SourceWindow> for TestTool {}
    impl SourceTool<SourceWindow> for TestTool {
        type Texture = i32;
        fn texture(&self) -> &i32 {
            &self.0
        }
        fn active_texture(&self) -> &i32 {
            static ACTIVE: i32 = 99;
            &ACTIVE
        }
        fn name(&self) -> &str {
            "test tool"
        }
        fn update(&mut self) {}
    }
    struct Layer(i32);
    impl properties::LayerReceiver for Layer {
        fn current_layer(&self) -> i32 {
            self.0
        }
        fn set_current_layer(&mut self, value: i32) {
            self.0 = value;
        }
    }
    struct Backend {
        layer: LayerObject,
        events: Vec<String>,
        textures: Vec<i32>,
        sizes: Vec<Vec2>,
        clicks: Vec<bool>,
        fail_hint: bool,
    }
    impl DescriptionBackend for Backend {
        fn item_id(&mut self) -> i32 {
            self.textures.len() as i32
        }
        fn current_time_millis(&mut self) -> i64 {
            0
        }
        fn is_item_hovered(&mut self, _: i32) -> bool {
            false
        }
        fn begin_tooltip(&mut self) {
            unreachable!()
        }
        fn font_size(&mut self) -> f32 {
            unreachable!()
        }
        fn push_text_wrap_pos(&mut self, _: f32) {
            unreachable!()
        }
        fn text_ex(&mut self, _: &str, _: i32, _: Option<usize>) {
            unreachable!()
        }
        fn pop_text_wrap_pos(&mut self) {
            unreachable!()
        }
        fn end_tooltip(&mut self) {
            unreachable!()
        }
    }
    impl SettingsBackend for Backend {
        fn drag_float(
            &mut self,
            label: &str,
            property: &dyn FloatReference,
            speed: f32,
            min: f32,
            max: f32,
            format: &str,
            power: f32,
        ) -> bool {
            assert_eq!(
                (label, speed, min, max, format, power),
                ("Tool Size", 0.1, 0.0, 1000.0, "%.3f", 1.0)
            );
            self.events.push("tool-size".into());
            property.set(Some(Value::Number(properties::Number::Float(2.5))));
            true
        }
        fn drag_int(
            &mut self,
            _: &str,
            _: &dyn IntReference,
            _: f32,
            _: i32,
            _: i32,
            _: &str,
        ) -> bool {
            unreachable!()
        }
        fn checkbox(&mut self, _: &str, _: &dyn BoolReference) -> bool {
            unreachable!()
        }
        fn color_picker4(
            &mut self,
            _: &str,
            _: Rc<RefCell<Vec4>>,
            _: i32,
            _: Option<[f32; 4]>,
        ) -> bool {
            unreachable!()
        }
        fn text(&mut self, text: &str) {
            assert_eq!(text, "You can hide the tools in the graphics settings");
            self.events.push("hint".into());
            if self.fail_hint {
                panic!("hint");
            }
        }
    }
    impl ToolsBackend<i32, i32> for Backend {
        fn global_ship_layer_receiver(&mut self) -> LayerObject {
            self.events.push("ship-property".into());
            self.layer.clone()
        }
        fn global_ship_layer_names(&mut self) -> Vec<String> {
            self.events.push("ship-names".into());
            vec!["Default".into(), "exterior".into()]
        }
        fn layer_combo(
            &mut self,
            label: &str,
            current: &dyn IntReference,
            names: &[String],
            count: i32,
            popup_height: i32,
        ) -> bool {
            assert_eq!((label, count, popup_height), ("Show Layer", 2, -1));
            assert_eq!(names, &["Default", "exterior"]);
            current.set(Some(Value::Number(properties::Number::Int(1))));
            self.events.push("combo".into());
            true
        }
        fn texture_id(&mut self, texture: &i32) -> i32 {
            self.events.push("texture-id".into());
            *texture
        }
        fn frame_height(&mut self) -> f32 {
            self.events.push("frame-height".into());
            28.0
        }
        fn image_button(
            &mut self,
            texture: i32,
            size: Vec2,
            uv0: Vec2,
            uv1: Vec2,
            padding: i32,
            background: Vec4,
            tint: Vec4,
        ) -> bool {
            assert_eq!(
                (uv0, uv1, padding, background, tint),
                (Vec2::ZERO, Vec2::ONE, 3, Vec4::ZERO, Vec4::ONE)
            );
            self.textures.push(texture);
            self.sizes.push(size);
            self.events.push("icon".into());
            self.clicks.remove(0)
        }
        fn same_line(&mut self, offset: f32, spacing: f32) {
            assert_eq!((offset, spacing), (0.0, -1.0));
            self.events.push("same-line".into());
        }
        fn push_font(&mut self, font: &i32) {
            assert_eq!(*font, 12);
            self.events.push("push-font".into());
        }
        fn pop_font(&mut self) {
            self.events.push("pop-font".into());
        }
    }
    fn fixture() -> (GuiKt<RefCell<TestTool>, i32>, Backend) {
        let mut gui = GuiKt::default();
        gui.access_set_scale(1.75);
        gui.access_set_f12(Some(Rc::new(12)));
        gui.access_set_tool_list(Some(Rc::new(RefCell::new(vec![
            None,
            Some(Rc::new(RefCell::new(TestTool(1)))),
            None,
            Some(Rc::new(RefCell::new(TestTool(3)))),
        ]))));
        gui.set_current_tool_index(1);
        (
            gui,
            Backend {
                layer: Rc::new(RefCell::new(Layer(0))),
                events: vec![],
                textures: vec![],
                sizes: vec![],
                clicks: vec![true, false],
                fail_hint: false,
            },
        )
    }
    #[test]
    fn source_toolbar_selects_textures_toggles_tools_and_writes_live_properties() {
        let (mut gui, mut b) = fixture();
        let provider = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let saved = provider.borrow().tool();
        SourceToolbox::render_tools(&mut gui, &mut b);
        assert_eq!(gui.current_tool_index(), 0);
        assert_eq!(b.textures, [99, 3]);
        assert_eq!(b.sizes, [Vec2::splat(77.0); 2]);
        assert_eq!(b.layer.borrow().current_layer(), 1);
        assert_eq!(provider.borrow().tool(), 2.5);
        assert_eq!(
            b.events,
            [
                "ship-property",
                "ship-names",
                "combo",
                "texture-id",
                "frame-height",
                "icon",
                "same-line",
                "texture-id",
                "frame-height",
                "icon",
                "tool-size",
                "push-font",
                "hint",
                "pop-font"
            ]
        );
        b.clicks = vec![false, true];
        b.textures.clear();
        SourceToolbox::render_tools(&mut gui, &mut b);
        assert_eq!(b.textures, [1, 3]);
        assert_eq!(gui.current_tool_index(), 3);
        provider.borrow_mut().set_tool(saved);
    }
    #[test]
    fn hint_failure_always_restores_font() {
        let (mut gui, mut b) = fixture();
        b.fail_hint = true;
        let provider = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let saved = provider.borrow().tool();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SourceToolbox::render_tools(&mut gui, &mut b)
        }));
        assert!(result.is_err());
        assert_eq!(b.events.last().unwrap(), "pop-font");
        provider.borrow_mut().set_tool(saved);
    }
}

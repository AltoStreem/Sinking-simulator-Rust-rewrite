//! Physics/Graphics/Performance control bodies from Toolbox.render; outer windows/tabs and remaining pages are pending.
use crate::{
    gui_kt::{DescriptionBackend, GuiKt},
    toolbox::SourceToolbox,
    toolbox_references as properties,
};
use std::{cell::RefCell, rc::Rc};
pub trait SettingsBackend: DescriptionBackend {
    fn drag_float(
        &mut self,
        label: &str,
        property: &dyn properties::FloatReference,
        speed: f32,
        min: f32,
        max: f32,
        format: &str,
        power: f32,
    ) -> bool;
    fn drag_int(
        &mut self,
        label: &str,
        property: &dyn properties::IntReference,
        speed: f32,
        min: i32,
        max: i32,
        format: &str,
    ) -> bool;
    fn checkbox(&mut self, label: &str, property: &dyn properties::BoolReference) -> bool;
    fn color_picker4(
        &mut self,
        label: &str,
        color: Rc<RefCell<bevy::math::Vec4>>,
        flags: i32,
        reference_color: Option<[f32; 4]>,
    ) -> bool;
    fn text(&mut self, text: &str);
}
impl SourceToolbox {
    pub fn render_physics<T: ?Sized, F: ?Sized>(
        gui: &mut GuiKt<T, F>,
        backend: &mut impl SettingsBackend,
    ) {
        let property = properties::toolbox_render_4_1_2_1::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider()
                .borrow()
                .waves(),
        };
        backend.drag_float("Wave Width", &property, 0.05, 0.1, 1.0E7, "%.3f", 1.0);
        gui.description("The width of the waves (in meters)", backend);
        let property = properties::toolbox_render_4_1_2_2::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider()
                .borrow()
                .waves(),
        };
        backend.drag_float("Wave Height", &property, 0.05, 0.0, 1.0E7, "%.3f", 1.0);
        gui.description("The height of the waves (in meters)", backend);
        let property = properties::toolbox_render_4_1_2_3::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Sea Depth", &property, 0.5, -1.0E7, 1.0E7, "%.3f", 1.0);
        gui.description("The depth of the sea (in meters)", backend);
        let property = properties::toolbox_render_4_1_2_4::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Buoyancy", &property, 0.05, 0.0, 1.0E7, "%.3f", 1.0);
        gui.description("Multiplies how much the materials float", backend);
        let property = properties::toolbox_render_4_1_2_5::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Drag", &property, 0.05, 0.0, 1.0E7, "%.3f", 1.0);
        gui.description("Multiplies how thick the air and water is", backend);
        let property = properties::toolbox_render_4_1_2_6::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Water Flow", &property, 0.05, 0.0, 1000.0, "%.3f", 1.0);
        gui.description("Multiplies how fast the water flows within a ship", backend);
        let property = properties::toolbox_render_4_1_2_7::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Water Influx", &property, 0.05, 0.0, 1000.0, "%.3f", 1.0);
        gui.description("Multiplies how fast the water flows into a ship", backend);
        let property = properties::toolbox_render_4_1_2_8::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Water funk", &property, 0.05, 0.0, 2.0, "%.3f", 1.0);
        gui.description(
            "Makes the water go crazy (more bernoulli's velocity)",
            backend,
        );
        let property = properties::toolbox_render_4_1_2_9::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Gravity", &property, 0.1, -1000.0, 1000.0, "%.3f", 1.0);
        gui.description("The gravity", backend);
        let property = properties::toolbox_render_4_1_2_10::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Strength", &property, 0.05, 0.0, 1000.0, "%.3f", 1.0);
        gui.description("Multiplies the force necessary to break a link", backend);
        let property = properties::toolbox_render_4_1_2_11::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Rigidity", &property, 0.005, 0.0, 2.0, "%.3f", 1.0);
        gui.description(
            "Multiplies the force keeping a link at the same length",
            backend,
        );
        let property = properties::toolbox_render_4_1_2_12::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Dampening", &property, 0.005, 0.0, 2.0, "%.3f", 1.0);
        gui.description("Reduces how bouncy links are", backend);
        let property = properties::toolbox_render_4_1_2_13::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Water Weight", &property, 0.05, 0.0, 1000000.0, "%.3f", 1.0);
        gui.description("Multiplies how heavy the water is inside the ship", backend);
        let property = properties::toolbox_render_4_1_2_14::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Thickness", &property, 0.005, 1.0E-6, 1.0, "%.3f", 1.0);
        gui.description(
            "How thick the hull of the ship is (in % of the cross-section) (1 is 100%)",
            backend,
        );
    }
    pub fn render_graphics<T: ?Sized, F: ?Sized>(
        receiver: Rc<RefCell<Self>>,
        gui: &mut GuiKt<T, F>,
        backend: &mut impl SettingsBackend,
    ) {
        let visible = properties::toolbox_render_4_1_3_1::PropertyReference { receiver };
        backend.checkbox("Show Tools", &visible);
        let cycle = properties::toolbox_render_4_1_3_3::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.checkbox("Cycle", &cycle);
        gui.description("If the day-night cycle is active", backend);
        let property = properties::toolbox_render_4_1_3_5::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Cycle length", &property, 0.1, 0.0, 1000000.0, "%.3f", 1.0);
        gui.description("How long the day-night cycle is (in seconds)", backend);
        let property = properties::toolbox_render_4_1_3_6::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Day", &property, 0.01, 0.0, 1.0, "%.3f", 1.0);
        gui.description("The time of day", backend);
        let property = properties::toolbox_render_4_1_3_7::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Water Darkness", &property, 0.01, 0.0, 10.0, "%.3f", 1.0);
        gui.description("Multiplies how dark the sea gets in the depth", backend);
        let color = crate::game_parameter_provider_kt::get_game_parameter_provider()
            .borrow()
            .water_color();
        backend.color_picker4("Sea color", color, 65536 | 262144, None);
    }
    pub fn render_performance<T: ?Sized, F: ?Sized>(
        gui: &mut GuiKt<T, F>,
        backend: &mut impl SettingsBackend,
    ) {
        backend.text("Iterations per frame");
        let physics = properties::toolbox_render_4_1_5_1::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        let minimum = crate::game_parameter_provider_kt::get_game_parameter_provider()
            .borrow()
            .water_steps();
        backend.drag_int("Physics", &physics, 0.2, minimum, 1000, "%d");
        gui.description("How many times the physics are calculated per frame, it affects: strength, rigidity, bounciness", backend);
        let water = properties::toolbox_render_4_1_5_2::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        let maximum = crate::game_parameter_provider_kt::get_game_parameter_provider()
            .borrow()
            .physics_steps();
        backend.drag_int("Water flow", &water, 0.2, 1, maximum, "%d");
        gui.description("How many times the water inside the ship is calculated per frame, the higher the less buggy the water is.", backend);
    }
    pub fn render_tool_size(backend: &mut impl SettingsBackend) {
        let property = properties::toolbox_render_2_3::PropertyReference {
            receiver: crate::game_parameter_provider_kt::get_game_parameter_provider(),
        };
        backend.drag_float("Tool Size", &property, 0.1, 0.0, 1000.0, "%.3f", 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use properties::{Number, Value};
    struct Backend {
        calls: Vec<String>,
        floats: Vec<(String, f32, f32, f32)>,
        ints: Vec<(String, i32, i32)>,
        replacement: Option<Rc<RefCell<crate::game_parameters::SourceGameParameterProvider>>>,
    }
    impl DescriptionBackend for Backend {
        fn item_id(&mut self) -> i32 {
            self.calls.len() as i32
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
            p: &dyn properties::FloatReference,
            speed: f32,
            min: f32,
            max: f32,
            format: &str,
            power: f32,
        ) -> bool {
            assert_eq!(format, "%.3f");
            assert_eq!(power, 1.0);
            self.calls.push(format!("float:{label}:{}", p.name()));
            self.floats.push((label.into(), speed, min, max));
            p.set(Some(Value::Number(Number::Float(2.5))));
            if let Some(replacement) = self.replacement.take() {
                crate::game_parameter_provider_kt::set_game_parameter_provider(replacement);
            }
            true
        }
        fn drag_int(
            &mut self,
            label: &str,
            p: &dyn properties::IntReference,
            speed: f32,
            min: i32,
            max: i32,
            format: &str,
        ) -> bool {
            assert_eq!(speed, 0.2);
            assert_eq!(format, "%d");
            self.calls.push(format!("int:{label}:{}", p.name()));
            self.ints.push((label.into(), min, max));
            p.set(Some(Value::Number(Number::Int(200))));
            if let Some(replacement) = self.replacement.take() {
                crate::game_parameter_provider_kt::set_game_parameter_provider(replacement);
            }
            true
        }
        fn checkbox(&mut self, label: &str, p: &dyn properties::BoolReference) -> bool {
            self.calls.push(format!("checkbox:{label}:{}", p.name()));
            p.set(Some(Value::Boolean(false)));
            true
        }
        fn color_picker4(
            &mut self,
            label: &str,
            color: Rc<RefCell<bevy::math::Vec4>>,
            flags: i32,
            reference: Option<[f32; 4]>,
        ) -> bool {
            assert_eq!(flags, 327680);
            assert_eq!(reference, None);
            self.calls.push(format!("color:{label}"));
            *color.borrow_mut() = bevy::math::Vec4::new(0.1, 0.2, 0.3, 0.4);
            true
        }
        fn text(&mut self, text: &str) {
            self.calls.push(format!("text:{text}"));
        }
    }
    fn backend() -> Backend {
        Backend {
            calls: vec![],
            floats: vec![],
            ints: vec![],
            replacement: None,
        }
    }
    #[test]
    fn physics_uses_all_original_labels_ranges_and_live_global_receivers() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let first = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        let next = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        crate::game_parameter_provider_kt::set_game_parameter_provider(first.clone());
        let mut b = backend();
        b.replacement = Some(next.clone());
        SourceToolbox::render_physics(&mut GuiKt::<(), ()>::default(), &mut b);
        assert_eq!(
            b.calls,
            [
                "float:Wave Width:x",
                "float:Wave Height:y",
                "float:Sea Depth:seaFloor",
                "float:Buoyancy:buoyancy",
                "float:Drag:drag",
                "float:Water Flow:flow",
                "float:Water Influx:inflow",
                "float:Water funk:funk",
                "float:Gravity:gravity",
                "float:Strength:strength",
                "float:Rigidity:rigidity",
                "float:Dampening:dampening",
                "float:Water Weight:waterWeight",
                "float:Thickness:thickness"
            ]
        );
        assert_eq!(b.floats[0], ("Wave Width".into(), 0.05, 0.1, 1e7));
        assert_eq!(b.floats[2], ("Sea Depth".into(), 0.5, -1e7, 1e7));
        assert_eq!(b.floats[8], ("Gravity".into(), 0.1, -1000.0, 1000.0));
        assert_eq!(b.floats[13], ("Thickness".into(), 0.005, 1e-6, 1.0));
        assert_eq!(first.borrow().waves().borrow().x, 2.5);
        assert_eq!(first.borrow().waves().borrow().y, 1.0);
        assert_eq!(next.borrow().waves().borrow().x, 40.0);
        assert_eq!(next.borrow().waves().borrow().y, 2.5);
        assert_eq!(next.borrow().gravity(), 2.5);
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
    #[test]
    fn graphics_mutates_shared_visibility_cycle_and_exact_rgba_picker() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let provider = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        crate::game_parameter_provider_kt::set_game_parameter_provider(provider.clone());
        let toolbox = Rc::new(RefCell::new(SourceToolbox::default()));
        let mut b = backend();
        SourceToolbox::render_graphics(toolbox.clone(), &mut GuiKt::<(), ()>::default(), &mut b);
        assert_eq!(
            b.calls,
            [
                "checkbox:Show Tools:toolsVisible",
                "checkbox:Cycle:daycycle",
                "float:Cycle length:cycleLength",
                "float:Day:day",
                "float:Water Darkness:waterDarkness",
                "color:Sea color"
            ]
        );
        assert!(!toolbox.borrow().tools_visible());
        assert!(!provider.borrow().daycycle());
        assert_eq!(
            *provider.borrow().water_color().borrow(),
            bevy::math::Vec4::new(0.1, 0.2, 0.3, 0.4)
        );
        assert_eq!(b.floats[2], ("Water Darkness".into(), 0.01, 0.0, 10.0));
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
    #[test]
    fn performance_limits_and_tool_size_follow_current_provider_getters() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let first = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        let next = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        next.borrow_mut().set_physics_steps(777);
        crate::game_parameter_provider_kt::set_game_parameter_provider(first.clone());
        let mut b = backend();
        b.replacement = Some(next.clone());
        SourceToolbox::render_performance(&mut GuiKt::<(), ()>::default(), &mut b);
        assert_eq!(
            b.calls,
            [
                "text:Iterations per frame",
                "int:Physics:physicsSteps",
                "int:Water flow:waterSteps"
            ]
        );
        assert_eq!(
            b.ints,
            [("Physics".into(), 5, 1000), ("Water flow".into(), 1, 777)]
        );
        assert_eq!(first.borrow().physics_steps(), 200);
        assert_eq!(first.borrow().water_steps(), 5);
        assert_eq!(next.borrow().physics_steps(), 777);
        assert_eq!(next.borrow().water_steps(), 200);
        SourceToolbox::render_tool_size(&mut b);
        assert_eq!(next.borrow().tool(), 2.5);
        assert_eq!(
            b.floats.last().unwrap(),
            &("Tool Size".into(), 0.1, 0.0, 1000.0)
        );
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
}

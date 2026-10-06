//! Shared contracts for the separately converted Toolbox property-reference classes.
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}
impl Number {
    pub fn float_value(self) -> f32 {
        match self {
            Self::Byte(v) => v as f32,
            Self::Short(v) => v as f32,
            Self::Int(v) => v as f32,
            Self::Long(v) => v as f32,
            Self::Float(v) => v,
            Self::Double(v) => v as f32,
        }
    }
    pub fn int_value(self) -> i32 {
        match self {
            Self::Byte(v) => v as i32,
            Self::Short(v) => v as i32,
            Self::Int(v) => v,
            Self::Long(v) => v as i32,
            Self::Float(v) => v as i32,
            Self::Double(v) => v as i32,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Number(Number),
    Boolean(bool),
}
impl Value {
    pub fn number(value: Option<Self>) -> Number {
        match value.expect("NullPointerException: Number property") {
            Self::Number(n) => n,
            _ => panic!("ClassCastException: Number property"),
        }
    }
    pub fn boolean(value: Option<Self>) -> bool {
        match value.expect("NullPointerException: Boolean property") {
            Self::Boolean(v) => v,
            _ => panic!("ClassCastException: Boolean property"),
        }
    }
}
pub trait Metadata {
    fn name(&self) -> &'static str;
    fn signature(&self) -> &'static str;
    fn owner(&self) -> &'static str;
}
pub trait FloatReference: Metadata {
    fn get(&self) -> f32;
    fn set(&self, value: Option<Value>);
}
pub trait IntReference: Metadata {
    fn get(&self) -> i32;
    fn set(&self, value: Option<Value>);
}
pub trait BoolReference: Metadata {
    fn get(&self) -> bool;
    fn set(&self, value: Option<Value>);
}
/// The native Ship adapter must call the original getter/setter, preserving its effects.
pub trait LayerReceiver {
    fn current_layer(&self) -> i32;
    fn set_current_layer(&mut self, value: i32);
}
pub type LayerObject = Rc<RefCell<dyn LayerReceiver>>;
pub mod toolbox_render_1;
pub mod toolbox_render_2_1;
pub mod toolbox_render_2_3;
pub mod toolbox_render_4_1_2_1;
pub mod toolbox_render_4_1_2_10;
pub mod toolbox_render_4_1_2_11;
pub mod toolbox_render_4_1_2_12;
pub mod toolbox_render_4_1_2_13;
pub mod toolbox_render_4_1_2_14;
pub mod toolbox_render_4_1_2_2;
pub mod toolbox_render_4_1_2_3;
pub mod toolbox_render_4_1_2_4;
pub mod toolbox_render_4_1_2_5;
pub mod toolbox_render_4_1_2_6;
pub mod toolbox_render_4_1_2_7;
pub mod toolbox_render_4_1_2_8;
pub mod toolbox_render_4_1_2_9;
pub mod toolbox_render_4_1_3_1;
pub mod toolbox_render_4_1_3_3;
pub mod toolbox_render_4_1_3_5;
pub mod toolbox_render_4_1_3_6;
pub mod toolbox_render_4_1_3_7;
pub mod toolbox_render_4_1_5_1;
pub mod toolbox_render_4_1_5_2;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_parameters::SourceGameParameterProvider;
    struct Layer {
        current: i32,
        sets: usize,
    }
    impl LayerReceiver for Layer {
        fn current_layer(&self) -> i32 {
            self.current
        }
        fn set_current_layer(&mut self, value: i32) {
            self.current = value;
            self.sets += 1;
        }
    }
    #[test]
    fn all_original_property_classes_retain_metadata_and_mutate_receivers() {
        let provider = Rc::new(RefCell::new(SourceGameParameterProvider::default()));
        let wave = provider.borrow().waves();
        let toolbox = Rc::new(RefCell::new(crate::toolbox::SourceToolbox::default()));
        let layer = Rc::new(RefCell::new(Layer {
            current: 4,
            sets: 0,
        }));
        {
            let reference = toolbox_render_1::PropertyReference {
                receiver: toolbox.clone(),
            };
            assert_eq!(reference.name(), "toolsVisible");
            assert_eq!(reference.signature(), "getToolsVisible()Z");
            assert_eq!(reference.owner(), "com.wicpar.sinkingsimulator.gui.Toolbox");
            reference.set(Some(Value::Boolean(false)));
            assert!(!reference.get());
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_2_1::PropertyReference {
                receiver: layer.clone() as LayerObject,
            };
            assert_eq!(reference.name(), "currentLayer");
            assert_eq!(reference.signature(), "getCurrentLayer()I");
            assert_eq!(reference.owner(), "com.wicpar.sinkingsimulator.ship.Ship");
            reference.set(Some(Value::Number(Number::Long(0x100000002))));
            assert_eq!(reference.get(), 2);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_2_3::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "tool");
            assert_eq!(reference.signature(), "getTool()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_1::PropertyReference {
                receiver: wave.clone(),
            };
            assert_eq!(reference.name(), "x");
            assert_eq!(reference.signature(), "getX()Ljava/lang/Float;");
            assert_eq!(reference.owner(), "glm_.vec2.Vec2");
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_10::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "strength");
            assert_eq!(reference.signature(), "getStrength()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_11::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "rigidity");
            assert_eq!(reference.signature(), "getRigidity()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_12::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "dampening");
            assert_eq!(reference.signature(), "getDampening()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_13::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "waterWeight");
            assert_eq!(reference.signature(), "getWaterWeight()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_14::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "thickness");
            assert_eq!(reference.signature(), "getThickness()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_2::PropertyReference {
                receiver: wave.clone(),
            };
            assert_eq!(reference.name(), "y");
            assert_eq!(reference.signature(), "getY()Ljava/lang/Float;");
            assert_eq!(reference.owner(), "glm_.vec2.Vec2");
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_3::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "seaFloor");
            assert_eq!(reference.signature(), "getSeaFloor()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_4::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "buoyancy");
            assert_eq!(reference.signature(), "getBuoyancy()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_5::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "drag");
            assert_eq!(reference.signature(), "getDrag()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_6::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "flow");
            assert_eq!(reference.signature(), "getFlow()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_7::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "inflow");
            assert_eq!(reference.signature(), "getInflow()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_8::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "funk");
            assert_eq!(reference.signature(), "getFunk()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_2_9::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "gravity");
            assert_eq!(reference.signature(), "getGravity()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_3_1::PropertyReference {
                receiver: toolbox.clone(),
            };
            assert_eq!(reference.name(), "toolsVisible");
            assert_eq!(reference.signature(), "getToolsVisible()Z");
            assert_eq!(reference.owner(), "com.wicpar.sinkingsimulator.gui.Toolbox");
            reference.set(Some(Value::Boolean(false)));
            assert!(!reference.get());
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_3_3::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "daycycle");
            assert_eq!(reference.signature(), "getDaycycle()Z");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Boolean(false)));
            assert!(!reference.get());
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_3_5::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "cycleLength");
            assert_eq!(reference.signature(), "getCycleLength()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_3_6::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "day");
            assert_eq!(reference.signature(), "getDay()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_3_7::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "waterDarkness");
            assert_eq!(reference.signature(), "getWaterDarkness()F");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Double(-12.75))));
            assert_eq!(reference.get(), -12.75);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_5_1::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "physicsSteps");
            assert_eq!(reference.signature(), "getPhysicsSteps()I");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Long(0x100000002))));
            assert_eq!(reference.get(), 2);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        {
            let reference = toolbox_render_4_1_5_2::PropertyReference {
                receiver: provider.clone(),
            };
            assert_eq!(reference.name(), "waterSteps");
            assert_eq!(reference.signature(), "getWaterSteps()I");
            assert_eq!(
                reference.owner(),
                "com.wicpar.sinkingsimulator.GameParameterProvider"
            );
            reference.set(Some(Value::Number(Number::Long(0x100000002))));
            assert_eq!(reference.get(), 2);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reference.set(None)))
                    .is_err()
            );
        }
        assert_eq!(layer.borrow().sets, 1);
    }
    #[test]
    fn numeric_boxing_conversions_and_wrong_type_failures_match_source_categories() {
        assert_eq!(Number::Byte(-1).float_value(), -1.0);
        assert_eq!(Number::Short(-200).int_value(), -200);
        assert_eq!(Number::Long(i64::MAX).int_value(), -1);
        assert_eq!(Number::Float(f32::NAN).int_value(), 0);
        assert_eq!(Number::Double(f64::INFINITY).int_value(), i32::MAX);
        assert_eq!(Number::Double(f64::NEG_INFINITY).int_value(), i32::MIN);
        assert!(Number::Float(f32::NAN).float_value().is_nan());
        assert_eq!(
            Number::Double(-0.0).float_value().to_bits(),
            (-0.0f32).to_bits()
        );
        assert!(std::panic::catch_unwind(|| Value::number(Some(Value::Boolean(true)))).is_err());
        assert!(
            std::panic::catch_unwind(|| Value::boolean(Some(Value::Number(Number::Int(1)))))
                .is_err()
        );
    }
    #[test]
    fn references_keep_original_provider_and_vector_after_global_or_waves_replacement() {
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let provider = Rc::new(RefCell::new(SourceGameParameterProvider::default()));
        let waves = provider.borrow().waves();
        let x = toolbox_render_4_1_2_1::PropertyReference {
            receiver: waves.clone(),
        };
        let tool = toolbox_render_2_3::PropertyReference {
            receiver: provider.clone(),
        };
        crate::game_parameter_provider_kt::set_game_parameter_provider(Rc::new(RefCell::new(
            SourceGameParameterProvider::default(),
        )));
        provider
            .borrow_mut()
            .set_waves(Rc::new(RefCell::new(bevy::math::Vec2::new(7.0, 8.0))));
        x.set(Some(Value::Number(Number::Int(11))));
        tool.set(Some(Value::Number(Number::Float(2.5))));
        assert_eq!(waves.borrow().x, 11.0);
        assert_eq!(provider.borrow().waves().borrow().x, 7.0);
        assert_eq!(provider.borrow().tool(), 2.5);
        assert_eq!(
            crate::game_parameter_provider_kt::get_game_parameter_provider()
                .borrow()
                .tool(),
            1.0
        );
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
}

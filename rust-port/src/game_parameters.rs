//! Rust translation of the original SS2 `GameParameterProvider` data class.
//! Defaults and parameter order follow `SS2/decompiled/.../GameParameterProvider.java`.
use bevy::prelude::{Vec2, Vec4};

#[derive(Clone, Debug)]
pub struct GameParameterProvider {
    pub time: f32,
    pub daycycle: bool,
    pub cycle_length: f32,
    pub waves: Vec2,
    pub sea_floor: f32,
    pub buoyancy: f32,
    pub drag: f32,
    pub day: f32,
    pub flow: f32,
    pub inflow: f32,
    pub funk: f32,
    pub tool: f32,
    pub rigidity: f32,
    pub strength: f32,
    pub dampening: f32,
    pub water_weight: f32,
    pub gravity: f32,
    pub thickness: f32,
    pub physics_steps: u32,
    pub water_steps: u32,
    pub water_darkness: f32,
    pub water_color: Vec4,
}

impl Default for GameParameterProvider {
    fn default() -> Self {
        Self {
            time: 0.0,
            daycycle: true,
            cycle_length: 120.0,
            waves: Vec2::new(40.0, 1.0),
            sea_floor: 400.0,
            buoyancy: 1.0,
            drag: 1.0,
            day: 1.0,
            flow: 60.0,
            inflow: 1.0,
            funk: 0.0,
            tool: 1.0,
            rigidity: 1.0,
            strength: 1.0,
            dampening: 1.0,
            water_weight: 1.0,
            gravity: 9.81,
            thickness: 0.085,
            physics_steps: 50,
            water_steps: 5,
            water_darkness: 1.0,
            water_color: Vec4::new(0.0, 71.0 / 255.0, 159.0 / 255.0, 0.75),
        }
    }
}

// Source vector getters and copies retain mutable reference identity.
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
pub struct SourceGameParameterProvider {
    time: f32,
    daycycle: bool,
    cycle_length: f32,
    waves: Rc<RefCell<Vec2>>,
    sea_floor: f32,
    buoyancy: f32,
    drag: f32,
    day: f32,
    flow: f32,
    inflow: f32,
    funk: f32,
    tool: f32,
    rigidity: f32,
    strength: f32,
    dampening: f32,
    water_weight: f32,
    gravity: f32,
    thickness: f32,
    physics_steps: i32,
    water_steps: i32,
    water_darkness: f32,
    water_color: Rc<RefCell<Vec4>>,
}
impl Default for SourceGameParameterProvider {
    fn default() -> Self {
        Self {
            time: 0.0,
            daycycle: true,
            cycle_length: 120.0,
            waves: Rc::new(RefCell::new(Vec2::new(40.0, 1.0))),
            sea_floor: 400.0,
            buoyancy: 1.0,
            drag: 1.0,
            day: 1.0,
            flow: 60.0,
            inflow: 1.0,
            funk: 0.0,
            tool: 1.0,
            rigidity: 1.0,
            strength: 1.0,
            dampening: 1.0,
            water_weight: 1.0,
            gravity: 9.81,
            thickness: 0.085,
            physics_steps: 50,
            water_steps: 5,
            water_darkness: 1.0,
            water_color: Rc::new(RefCell::new(Vec4::new(
                0.0,
                0.278431373_f64 as f32,
                0.623529411_f64 as f32,
                0.75,
            ))),
        }
    }
}
impl SourceGameParameterProvider {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: f32,
        daycycle: bool,
        cycle_length: f32,
        waves: Rc<RefCell<Vec2>>,
        sea_floor: f32,
        buoyancy: f32,
        drag: f32,
        day: f32,
        flow: f32,
        inflow: f32,
        funk: f32,
        tool: f32,
        rigidity: f32,
        strength: f32,
        dampening: f32,
        water_weight: f32,
        gravity: f32,
        thickness: f32,
        physics_steps: i32,
        water_steps: i32,
        water_darkness: f32,
        water_color: Rc<RefCell<Vec4>>,
    ) -> Self {
        Self {
            time,
            daycycle,
            cycle_length,
            waves,
            sea_floor,
            buoyancy,
            drag,
            day,
            flow,
            inflow,
            funk,
            tool,
            rigidity,
            strength,
            dampening,
            water_weight,
            gravity,
            thickness,
            physics_steps,
            water_steps,
            water_darkness,
            water_color,
        }
    }
    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn component1(&self) -> f32 {
        self.time
    }
    pub fn set_time(&mut self, value: f32) {
        self.time = value;
    }
    pub fn daycycle(&self) -> bool {
        self.daycycle
    }
    pub fn component2(&self) -> bool {
        self.daycycle
    }
    pub fn set_daycycle(&mut self, value: bool) {
        self.daycycle = value;
    }
    pub fn cycle_length(&self) -> f32 {
        self.cycle_length
    }
    pub fn component3(&self) -> f32 {
        self.cycle_length
    }
    pub fn set_cycle_length(&mut self, value: f32) {
        self.cycle_length = value;
    }
    pub fn waves(&self) -> Rc<RefCell<Vec2>> {
        self.waves.clone()
    }
    pub fn component4(&self) -> Rc<RefCell<Vec2>> {
        self.waves.clone()
    }
    pub fn set_waves(&mut self, value: Rc<RefCell<Vec2>>) {
        self.waves = value;
    }
    pub fn sea_floor(&self) -> f32 {
        self.sea_floor
    }
    pub fn component5(&self) -> f32 {
        self.sea_floor
    }
    pub fn set_sea_floor(&mut self, value: f32) {
        self.sea_floor = value;
    }
    pub fn buoyancy(&self) -> f32 {
        self.buoyancy
    }
    pub fn component6(&self) -> f32 {
        self.buoyancy
    }
    pub fn set_buoyancy(&mut self, value: f32) {
        self.buoyancy = value;
    }
    pub fn drag(&self) -> f32 {
        self.drag
    }
    pub fn component7(&self) -> f32 {
        self.drag
    }
    pub fn set_drag(&mut self, value: f32) {
        self.drag = value;
    }
    pub fn day(&self) -> f32 {
        self.day
    }
    pub fn component8(&self) -> f32 {
        self.day
    }
    pub fn set_day(&mut self, value: f32) {
        self.day = value;
    }
    pub fn flow(&self) -> f32 {
        self.flow
    }
    pub fn component9(&self) -> f32 {
        self.flow
    }
    pub fn set_flow(&mut self, value: f32) {
        self.flow = value;
    }
    pub fn inflow(&self) -> f32 {
        self.inflow
    }
    pub fn component10(&self) -> f32 {
        self.inflow
    }
    pub fn set_inflow(&mut self, value: f32) {
        self.inflow = value;
    }
    pub fn funk(&self) -> f32 {
        self.funk
    }
    pub fn component11(&self) -> f32 {
        self.funk
    }
    pub fn set_funk(&mut self, value: f32) {
        self.funk = value;
    }
    pub fn tool(&self) -> f32 {
        self.tool
    }
    pub fn component12(&self) -> f32 {
        self.tool
    }
    pub fn set_tool(&mut self, value: f32) {
        self.tool = value;
    }
    pub fn rigidity(&self) -> f32 {
        self.rigidity
    }
    pub fn component13(&self) -> f32 {
        self.rigidity
    }
    pub fn set_rigidity(&mut self, value: f32) {
        self.rigidity = value;
    }
    pub fn strength(&self) -> f32 {
        self.strength
    }
    pub fn component14(&self) -> f32 {
        self.strength
    }
    pub fn set_strength(&mut self, value: f32) {
        self.strength = value;
    }
    pub fn dampening(&self) -> f32 {
        self.dampening
    }
    pub fn component15(&self) -> f32 {
        self.dampening
    }
    pub fn set_dampening(&mut self, value: f32) {
        self.dampening = value;
    }
    pub fn water_weight(&self) -> f32 {
        self.water_weight
    }
    pub fn component16(&self) -> f32 {
        self.water_weight
    }
    pub fn set_water_weight(&mut self, value: f32) {
        self.water_weight = value;
    }
    pub fn gravity(&self) -> f32 {
        self.gravity
    }
    pub fn component17(&self) -> f32 {
        self.gravity
    }
    pub fn set_gravity(&mut self, value: f32) {
        self.gravity = value;
    }
    pub fn thickness(&self) -> f32 {
        self.thickness
    }
    pub fn component18(&self) -> f32 {
        self.thickness
    }
    pub fn set_thickness(&mut self, value: f32) {
        self.thickness = value;
    }
    pub fn physics_steps(&self) -> i32 {
        self.physics_steps
    }
    pub fn component19(&self) -> i32 {
        self.physics_steps
    }
    pub fn set_physics_steps(&mut self, value: i32) {
        self.physics_steps = value;
    }
    pub fn water_steps(&self) -> i32 {
        self.water_steps
    }
    pub fn component20(&self) -> i32 {
        self.water_steps
    }
    pub fn set_water_steps(&mut self, value: i32) {
        self.water_steps = value;
    }
    pub fn water_darkness(&self) -> f32 {
        self.water_darkness
    }
    pub fn component21(&self) -> f32 {
        self.water_darkness
    }
    pub fn set_water_darkness(&mut self, value: f32) {
        self.water_darkness = value;
    }
    pub fn water_color(&self) -> Rc<RefCell<Vec4>> {
        self.water_color.clone()
    }
    pub fn component22(&self) -> Rc<RefCell<Vec4>> {
        self.water_color.clone()
    }
    pub fn with_defaults(mut arguments: Self, mask: i32) -> Self {
        if mask & 1 != 0 {
            arguments.time = 0.0;
        }
        if mask & 2 != 0 {
            arguments.daycycle = true;
        }
        if mask & 4 != 0 {
            arguments.cycle_length = 120.0;
        }
        if mask & 8 != 0 {
            arguments.waves = Rc::new(RefCell::new(Vec2::new(40.0, 1.0)));
        }
        if mask & 16 != 0 {
            arguments.sea_floor = 400.0;
        }
        if mask & 32 != 0 {
            arguments.buoyancy = 1.0;
        }
        if mask & 64 != 0 {
            arguments.drag = 1.0;
        }
        if mask & 128 != 0 {
            arguments.day = 1.0;
        }
        if mask & 256 != 0 {
            arguments.flow = 60.0;
        }
        if mask & 512 != 0 {
            arguments.inflow = 1.0;
        }
        if mask & 1024 != 0 {
            arguments.funk = 0.0;
        }
        if mask & 2048 != 0 {
            arguments.tool = 1.0;
        }
        if mask & 4096 != 0 {
            arguments.rigidity = 1.0;
        }
        if mask & 8192 != 0 {
            arguments.strength = 1.0;
        }
        if mask & 16384 != 0 {
            arguments.dampening = 1.0;
        }
        if mask & 32768 != 0 {
            arguments.water_weight = 1.0;
        }
        if mask & 65536 != 0 {
            arguments.gravity = 9.81;
        }
        if mask & 131072 != 0 {
            arguments.thickness = 0.085;
        }
        if mask & 262144 != 0 {
            arguments.physics_steps = 50;
        }
        if mask & 524288 != 0 {
            arguments.water_steps = 5;
        }
        if mask & 1048576 != 0 {
            arguments.water_darkness = 1.0;
        }
        if mask & 2097152 != 0 {
            arguments.water_color = Rc::new(RefCell::new(Vec4::new(
                0.0,
                0.278431373_f64 as f32,
                0.623529411_f64 as f32,
                0.75,
            )));
        }
        arguments
    }
    pub fn copy(&self, arguments: Self) -> Self {
        arguments
    }
    pub fn copy_default(&self, mut arguments: Self, mask: i32) -> Self {
        if mask & 1 != 0 {
            arguments.time = self.time;
        }
        if mask & 2 != 0 {
            arguments.daycycle = self.daycycle;
        }
        if mask & 4 != 0 {
            arguments.cycle_length = self.cycle_length;
        }
        if mask & 8 != 0 {
            arguments.waves = self.waves.clone();
        }
        if mask & 16 != 0 {
            arguments.sea_floor = self.sea_floor;
        }
        if mask & 32 != 0 {
            arguments.buoyancy = self.buoyancy;
        }
        if mask & 64 != 0 {
            arguments.drag = self.drag;
        }
        if mask & 128 != 0 {
            arguments.day = self.day;
        }
        if mask & 256 != 0 {
            arguments.flow = self.flow;
        }
        if mask & 512 != 0 {
            arguments.inflow = self.inflow;
        }
        if mask & 1024 != 0 {
            arguments.funk = self.funk;
        }
        if mask & 2048 != 0 {
            arguments.tool = self.tool;
        }
        if mask & 4096 != 0 {
            arguments.rigidity = self.rigidity;
        }
        if mask & 8192 != 0 {
            arguments.strength = self.strength;
        }
        if mask & 16384 != 0 {
            arguments.dampening = self.dampening;
        }
        if mask & 32768 != 0 {
            arguments.water_weight = self.water_weight;
        }
        if mask & 65536 != 0 {
            arguments.gravity = self.gravity;
        }
        if mask & 131072 != 0 {
            arguments.thickness = self.thickness;
        }
        if mask & 262144 != 0 {
            arguments.physics_steps = self.physics_steps;
        }
        if mask & 524288 != 0 {
            arguments.water_steps = self.water_steps;
        }
        if mask & 1048576 != 0 {
            arguments.water_darkness = self.water_darkness;
        }
        if mask & 2097152 != 0 {
            arguments.water_color = self.water_color.clone();
        }
        self.copy(arguments)
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn source_defaults_signed_setters_and_vector_aliases() {
        let mut p = SourceGameParameterProvider::default();
        assert_eq!(p.time(), 0.0);
        assert!(p.daycycle());
        assert_eq!(p.cycle_length(), 120.0);
        assert_eq!(*p.waves().borrow(), Vec2::new(40.0, 1.0));
        assert_eq!(p.sea_floor(), 400.0);
        assert_eq!(p.flow(), 60.0);
        assert_eq!(p.physics_steps(), 50);
        assert_eq!(p.water_steps(), 5);
        assert_eq!(p.gravity(), 9.81);
        assert_eq!(p.thickness(), 0.085);
        assert_eq!(p.water_color().borrow().w, 0.75);
        p.set_physics_steps(-50);
        p.set_water_steps(i32::MIN);
        p.set_tool(f32::NAN);
        p.set_cycle_length(-0.0);
        assert_eq!(p.component19(), -50);
        assert_eq!(p.component20(), i32::MIN);
        assert!(p.component12().is_nan());
        assert_eq!(p.component3().to_bits(), (-0.0f32).to_bits());
        let wave = p.waves();
        assert!(Rc::ptr_eq(&wave, &p.component4()));
        wave.borrow_mut().x = 3.0;
        assert_eq!(p.waves().borrow().x, 3.0);
        let replacement = Rc::new(RefCell::new(Vec2::ZERO));
        p.set_waves(replacement.clone());
        assert!(Rc::ptr_eq(&replacement, &p.waves()));
        let color = p.water_color();
        assert!(Rc::ptr_eq(&color, &p.component22()));
        color.borrow_mut().z = 0.2;
        assert_eq!(p.water_color().borrow().z, 0.2);
    }
    #[test]
    fn synthetic_masks_and_copies_preserve_or_replace_reference_identity() {
        let mut p = SourceGameParameterProvider::default();
        p.set_tool(3.0);
        p.set_time(11.0);
        let mut args = SourceGameParameterProvider::default();
        args.set_tool(7.0);
        args.set_time(22.0);
        let args_waves = args.waves();
        let args_color = args.water_color();
        let copied = p.copy_default(args, (1 << 11) | (1 << 3));
        assert_eq!(copied.tool(), 3.0);
        assert_eq!(copied.time(), 22.0);
        assert!(Rc::ptr_eq(&copied.waves(), &p.waves()));
        assert!(!Rc::ptr_eq(&copied.waves(), &args_waves));
        assert!(Rc::ptr_eq(&copied.water_color(), &args_color));
        let clone = p.copy_default(SourceGameParameterProvider::default(), 0x3fffff);
        assert!(Rc::ptr_eq(&clone.water_color(), &p.water_color()));
        clone.water_color().borrow_mut().w = 0.3;
        assert_eq!(p.water_color().borrow().w, 0.3);
        let reset = SourceGameParameterProvider::with_defaults(p.clone(), 0x3fffff);
        assert_eq!(reset.tool(), 1.0);
        assert_eq!(reset.time(), 0.0);
        assert!(!Rc::ptr_eq(&reset.waves(), &p.waves()));
        assert!(!Rc::ptr_eq(&reset.water_color(), &p.water_color()));
        assert_eq!(reset.water_color().borrow().w, 0.75);
        let retained = SourceGameParameterProvider::with_defaults(p.clone(), 1 << 11);
        assert_eq!(retained.time(), 11.0);
        assert_eq!(retained.tool(), 1.0);
        assert!(Rc::ptr_eq(&retained.waves(), &p.waves()));
    }
}

// Float.floatToIntBits, which canonicalizes NaN rather than retaining payload bits.
fn source_float_bits(value: f32) -> u32 {
    if value.is_nan() {
        0x7fc00000
    } else {
        value.to_bits()
    }
}
fn source_vector_hash(values: &[f32]) -> i32 {
    values.iter().fold(0i32, |hash, value| {
        hash.wrapping_mul(31)
            .wrapping_add(source_float_bits(*value) as i32)
    })
}
impl SourceGameParameterProvider {
    /// Float.compare equality for scalar fields; glm primitive comparisons for vector fields.
    /// Deliberately not Rust Eq/Hash: the original vectors consider signed zeros equal but hash them differently.
    pub fn equals(&self, other: Option<&Self>) -> bool {
        let Some(other) = other else {
            return false;
        };
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.daycycle != other.daycycle
            || self.physics_steps != other.physics_steps
            || self.water_steps != other.water_steps
        {
            return false;
        }
        let a = [
            self.time,
            self.cycle_length,
            self.sea_floor,
            self.buoyancy,
            self.drag,
            self.day,
            self.flow,
            self.inflow,
            self.funk,
            self.tool,
            self.rigidity,
            self.strength,
            self.dampening,
            self.water_weight,
            self.gravity,
            self.thickness,
            self.water_darkness,
        ];
        let b = [
            other.time,
            other.cycle_length,
            other.sea_floor,
            other.buoyancy,
            other.drag,
            other.day,
            other.flow,
            other.inflow,
            other.funk,
            other.tool,
            other.rigidity,
            other.strength,
            other.dampening,
            other.water_weight,
            other.gravity,
            other.thickness,
            other.water_darkness,
        ];
        if !a
            .iter()
            .zip(b)
            .all(|(a, b)| source_float_bits(*a) == source_float_bits(b))
        {
            return false;
        }
        // Intrinsics.areEqual delegates to glm Vec2/Vec4.equals; it does not shortcut vector identity.
        let wave = self.waves.borrow();
        let other_wave = other.waves.borrow();
        if wave.x != other_wave.x || wave.y != other_wave.y {
            return false;
        }
        let color = self.water_color.borrow();
        let other_color = other.water_color.borrow();
        color.x == other_color.x
            && color.y == other_color.y
            && color.z == other_color.z
            && color.w == other_color.w
    }
    /// Kotlin data-class `toString`, preserving source property and vector component order.
    pub fn source_to_string(&self) -> String {
        use crate::java_string::java_float_to_string as f;
        let waves = self.waves.borrow();
        let color = self.water_color.borrow();
        let waves = format!("{}, {}", f(waves.x), f(waves.y));
        let water_color = format!(
            "{}, {}, {}, {}",
            f(color.x),
            f(color.y),
            f(color.z),
            f(color.w)
        );
        format!(
            "GameParameterProvider(time={}, daycycle={}, cycleLength={}, waves={}, seaFloor={}, buoyancy={}, drag={}, day={}, flow={}, inflow={}, funk={}, tool={}, rigidity={}, strength={}, dampening={}, waterWeight={}, gravity={}, thickness={}, physicsSteps={}, waterSteps={}, waterDarkness={}, waterColor={})",
            f(self.time),
            self.daycycle,
            f(self.cycle_length),
            waves,
            f(self.sea_floor),
            f(self.buoyancy),
            f(self.drag),
            f(self.day),
            f(self.flow),
            f(self.inflow),
            f(self.funk),
            f(self.tool),
            f(self.rigidity),
            f(self.strength),
            f(self.dampening),
            f(self.water_weight),
            f(self.gravity),
            f(self.thickness),
            self.physics_steps,
            self.water_steps,
            f(self.water_darkness),
            water_color,
        )
    }
    pub fn hash_code(&self) -> i32 {
        let wave = self.waves.borrow();
        let color = self.water_color.borrow();
        let fields = [
            source_float_bits(self.time) as i32,
            i32::from(self.daycycle),
            source_float_bits(self.cycle_length) as i32,
            source_vector_hash(&[wave.x, wave.y]),
            source_float_bits(self.sea_floor) as i32,
            source_float_bits(self.buoyancy) as i32,
            source_float_bits(self.drag) as i32,
            source_float_bits(self.day) as i32,
            source_float_bits(self.flow) as i32,
            source_float_bits(self.inflow) as i32,
            source_float_bits(self.funk) as i32,
            source_float_bits(self.tool) as i32,
            source_float_bits(self.rigidity) as i32,
            source_float_bits(self.strength) as i32,
            source_float_bits(self.dampening) as i32,
            source_float_bits(self.water_weight) as i32,
            source_float_bits(self.gravity) as i32,
            source_float_bits(self.thickness) as i32,
            self.physics_steps,
            self.water_steps,
            source_float_bits(self.water_darkness) as i32,
            source_vector_hash(&[color.x, color.y, color.z, color.w]),
        ];
        fields.into_iter().fold(0i32, |hash, field| {
            hash.wrapping_mul(31).wrapping_add(field)
        })
    }
}

#[cfg(test)]
mod equality_tests {
    use super::*;
    #[test]
    fn scalar_nan_payloads_signed_zero_and_object_identity_follow_java() {
        let mut a = SourceGameParameterProvider::default();
        let mut b = a.clone();
        assert!(a.equals(Some(&b)));
        assert_eq!(a.hash_code(), b.hash_code());
        assert!(!a.equals(None));
        // Obtained by invoking the unmodified JVM class in sinkingsimulator-4.0-all.jar.
        assert_eq!(a.hash_code(), -975189733);
        a.set_time(f32::from_bits(0x7fc01234));
        b.set_time(f32::from_bits(0xff800001));
        assert!(a.equals(Some(&b)));
        assert_eq!(a.hash_code(), b.hash_code());
        assert_eq!(a.hash_code(), -1642084069);
        a.set_time(-0.0);
        b.set_time(0.0);
        assert!(!a.equals(Some(&b)));
        assert_ne!(a.hash_code(), b.hash_code());
        assert_eq!(a.hash_code(), 1172293915);
        assert_eq!(b.hash_code(), -975189733);
        assert!(a.equals(Some(&a)));
    }
    #[test]
    fn glm_vector_nan_and_signed_zero_preserve_original_hash_inconsistency() {
        let a = SourceGameParameterProvider::default();
        let b = SourceGameParameterProvider::default();
        a.waves().borrow_mut().x = -0.0;
        b.waves().borrow_mut().x = 0.0;
        assert!(a.equals(Some(&b)));
        assert_ne!(a.hash_code(), b.hash_code());
        assert_eq!(a.hash_code(), -67122917);
        assert_eq!(b.hash_code(), 2080360731);
        let shared = a.clone();
        a.waves().borrow_mut().x = f32::NAN;
        assert!(a.equals(Some(&a)));
        assert!(!a.equals(Some(&shared)));
        assert_eq!(a.hash_code(), shared.hash_code());
        a.waves().borrow_mut().x = 1.0;
        a.water_color().borrow_mut().w = f32::NAN;
        assert!(!a.equals(Some(&shared)));
    }
}

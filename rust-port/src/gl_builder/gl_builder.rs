//! GLBuilder.java's existing declaration/name generation; empty source operations stay empty.
use super::{
    gl_ivec2::GlIVec2, gl_reference::GlReference, gl_sampler_2d_type::GlSampler2DType,
    gl_type::GlType, gl_vec4::GlVec4,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
/// Retained identity replaces JVM object identity for holder lookup keys.
#[derive(Clone)]
pub(crate) struct HolderIdentity(Rc<()>);
impl HolderIdentity {
    pub fn new() -> Self {
        Self(Rc::new(()))
    }
    fn key(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}
type Names = Rc<RefCell<HashMap<usize, (HolderIdentity, String)>>>;
pub(crate) struct GlBuilder {
    pub width: i32,
    pub height: i32,
    inputs: Names,
}
impl GlBuilder {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            inputs: Default::default(),
        }
    }
    pub fn pass(&self) -> GlPassBuilder {
        GlPassBuilder {
            inputs: self.inputs.clone(),
            outputs: Default::default(),
        }
    }
}
pub(crate) struct GlPassBuilder {
    inputs: Names,
    outputs: Names,
}
impl GlPassBuilder {
    pub fn filter(&self) {}
    pub fn map(&self) {}
    pub fn context(&self) -> GlProgramContext {
        let mut context = GlProgramContext {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            global: Vec::new(),
            main: Vec::new(),
            frag_coord: GlReference::new(GlVec4, "gl_FragCoord"),
            index: GlReference::new(GlIVec2, ""),
            counter: 0,
        };
        // Source field initialization calls globalVar before inc = 1.
        let xy =
            super::gl_vector_util_kt::swizzle2(&context.frag_coord, &super::x::X, &super::y::Y);
        context.index = context.global_var(super::gl_vector_util_kt::cast_vector(&xy, GlIVec2));
        context.counter = 1;
        context
    }
}
pub(crate) struct GlProgramContext {
    inputs: Names,
    outputs: Names,
    pub global: Vec<String>,
    pub main: Vec<String>,
    pub frag_coord: GlReference<GlVec4>,
    pub index: GlReference<GlIVec2>,
    counter: i32,
}
impl GlProgramContext {
    pub fn build(&self) {}
    fn variable_name(&mut self, prefix: &str) -> String {
        let mut number = self.counter;
        self.counter = self.counter.wrapping_add(1);
        let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut name = format!("{prefix}_");
        while number > 0 {
            name.push(alphabet[(number % 52) as usize] as char);
            number /= 52;
        }
        name
    }
    fn holder_name(&mut self, holder: &HolderIdentity, input: bool) -> String {
        let names = if input {
            self.inputs.clone()
        } else {
            self.outputs.clone()
        };
        if let Some((_, name)) = names.borrow().get(&holder.key()) {
            return name.clone();
        }
        let name = self.variable_name(if input { "in" } else { "out" });
        names
            .borrow_mut()
            .insert(holder.key(), (holder.clone(), name.clone()));
        name
    }
    fn global_var<T: GlType>(&mut self, reference: GlReference<T>) -> GlReference<T> {
        let name = self.variable_name("v");
        self.global.push(format!(
            "{} {} = {};",
            reference.gl_type.type_name(),
            name,
            reference
        ));
        GlReference::new(reference.gl_type, name)
    }
    pub fn local_var<T: GlType>(&mut self, reference: GlReference<T>) -> GlReference<T> {
        let name = self.variable_name("v");
        self.main.push(format!(
            "{} {} = {};",
            reference.gl_type.type_name(),
            name,
            reference
        ));
        GlReference::new(reference.gl_type, name)
    }
    fn declare_input<T: GlSampler2DType>(
        &mut self,
        holder: &HolderIdentity,
        sampler: T,
    ) -> GlReference<T> {
        let name = self.holder_name(holder, true);
        self.global
            .push(format!("uniform {} {};", sampler.type_name(), name));
        GlReference::new(sampler, name)
    }
    pub fn input_float<T: super::float_type::FloatType>(
        &mut self,
        holder: &HolderIdentity,
        base: T,
    ) -> GlReference<super::gl_float_sampler_2d::GlFloatSampler2D<T>> {
        self.declare_input(
            holder,
            super::gl_float_sampler_2d::GlFloatSampler2D::new(base),
        )
    }
    pub fn input_int<T: super::int_type::IntType>(
        &mut self,
        holder: &HolderIdentity,
        base: T,
    ) -> GlReference<super::gl_int_sampler_2d::GlIntSampler2D<T>> {
        self.declare_input(holder, super::gl_int_sampler_2d::GlIntSampler2D::new(base))
    }
    pub fn input_uint<T: super::uint_type::UIntType>(
        &mut self,
        holder: &HolderIdentity,
        base: T,
    ) -> GlReference<super::gl_uint_sampler_2d::GlUIntSampler2D<T>> {
        self.declare_input(
            holder,
            super::gl_uint_sampler_2d::GlUIntSampler2D::new(base),
        )
    }
    pub fn output<T: GlType>(&mut self, holder: &HolderIdentity, gl_type: T) -> GlReference<T> {
        let name = self.holder_name(holder, false);
        self.global
            .push(format!("out {} {};", gl_type.type_name(), name));
        GlReference::new(gl_type, name)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_builder::{gl_float::GlFloat, gl_sampler_kt::sample};
    #[test]
    fn initialization_counter_and_declarations_match_source_field_order() {
        let builder = GlBuilder::new(3, 4);
        let pass = builder.pass();
        pass.filter();
        pass.map();
        let mut ctx = pass.context();
        assert_eq!(ctx.index.value, "v_");
        assert_eq!(ctx.global, ["ivec2 v_ = ivec2((gl_FragCoord.xY));"]);
        let holder = HolderIdentity::new();
        let input = ctx.input_float(&holder, GlFloat);
        assert_eq!(input.value, "in_b");
        assert_eq!(ctx.global[1], "uniform sampler2D in_b;");
        let sampled = sample(&input, &ctx.index);
        assert_eq!(sampled.value, "texelFetch(in_b, v_, 0)");
        let local = ctx.local_var(sampled);
        assert_eq!(local.value, "v_c");
        assert_eq!(ctx.main, ["float v_c = texelFetch(in_b, v_, 0);"]);
        ctx.build();
    }
    #[test]
    fn holder_identity_lookup_shares_inputs_and_repeats_source_declarations() {
        let builder = GlBuilder::new(1, 1);
        let pass = builder.pass();
        let holder = HolderIdentity::new();
        let mut a = pass.context();
        let first = a.input_float(&holder, GlFloat);
        let repeated = a.input_float(&holder.clone(), GlFloat);
        assert_eq!(first.value, repeated.value);
        assert_eq!(a.global[1], a.global[2]);
        let mut b = builder.pass().context();
        assert_eq!(b.input_float(&holder, GlFloat).value, first.value);
        let output = a.output(&holder, GlFloat);
        assert_eq!(output.value, "out_c");
        assert_eq!(a.global[3], "out float out_c;");
    }
    #[test]
    fn variable_names_use_source_least_significant_digit_first() {
        let mut ctx = GlBuilder::new(1, 1).pass().context();
        ctx.counter = 52;
        assert_eq!(
            ctx.local_var(GlReference::new(GlFloat, "1.0")).value,
            "v_ab"
        );
        ctx.counter = i32::MAX;
        ctx.variable_name("v");
        assert_eq!(ctx.variable_name("v"), "v_");
    }
}

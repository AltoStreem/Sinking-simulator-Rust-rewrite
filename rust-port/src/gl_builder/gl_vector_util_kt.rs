//! GLVectorUtilKt.java: source swizzles, component access, and scalar/vector casts.
use super::{
    gl_reference::GlReference, gl_type::GlType, gl_vector_component::GlVectorComponent,
    gl_vector_relatives::GlVectorRelatives, gl_vector_type::GlVectorType,
};
// The existing source GLSL families carry Float or Integer values.
pub(crate) trait ShaderNumber {}
impl ShaderNumber for f32 {}
impl ShaderNumber for i32 {}
type Four<V> = <<V as GlVectorType>::Relatives as GlVectorRelatives>::Four;
type Three<V> = <<V as GlVectorType>::Relatives as GlVectorRelatives>::Three;
type Two<V> = <<V as GlVectorType>::Relatives as GlVectorRelatives>::Two;
fn selection(value: &str, components: &[&dyn GlVectorComponent]) -> String {
    let letters: String = components
        .iter()
        .map(|component| component.component())
        .collect();
    format!("({value}.{letters})")
}
pub(crate) fn swizzle4<V: GlVectorType>(
    value: &GlReference<V>,
    x: &dyn GlVectorComponent,
    y: &dyn GlVectorComponent,
    z: &dyn GlVectorComponent,
    w: &dyn GlVectorComponent,
) -> GlReference<Four<V>>
where
    <V::Base as GlType>::Value: ShaderNumber,
{
    GlReference::new(
        value.gl_type.relatives().four(),
        selection(&value.value, &[x, y, z, w]),
    )
}
pub(crate) fn swizzle3<V: GlVectorType>(
    value: &GlReference<V>,
    x: &dyn GlVectorComponent,
    y: &dyn GlVectorComponent,
    z: &dyn GlVectorComponent,
) -> GlReference<Three<V>>
where
    <V::Base as GlType>::Value: ShaderNumber,
{
    GlReference::new(
        value.gl_type.relatives().three(),
        selection(&value.value, &[x, y, z]),
    )
}
pub(crate) fn swizzle2<V: GlVectorType>(
    value: &GlReference<V>,
    x: &dyn GlVectorComponent,
    y: &dyn GlVectorComponent,
) -> GlReference<Two<V>>
where
    <V::Base as GlType>::Value: ShaderNumber,
{
    GlReference::new(
        value.gl_type.relatives().two(),
        selection(&value.value, &[x, y]),
    )
}
pub(crate) fn get<V: GlVectorType>(
    value: &GlReference<V>,
    x: &dyn GlVectorComponent,
) -> GlReference<V::Base>
where
    <V::Base as GlType>::Value: ShaderNumber,
{
    GlReference::new(
        value.gl_type.relatives().base_type(),
        selection(&value.value, &[x]),
    )
}
pub(crate) fn cast_vector<U: GlVectorType, V: GlVectorType<Size = U::Size>>(
    value: &GlReference<U>,
    target: V,
) -> GlReference<V> {
    let expression = format!("{}({})", target.type_name(), value.value);
    GlReference::new(target, expression)
}
pub(crate) fn cast_scalar<U: GlType, V: GlType>(value: &GlReference<U>, target: V) -> GlReference<V>
where
    U::Value: ShaderNumber,
    V::Value: ShaderNumber,
{
    let expression = format!("{}({})", target.type_name(), value.value);
    GlReference::new(target, expression)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_builder::{
        gl_float::GlFloat, gl_int::GlInt, gl_ivec2::GlIVec2, gl_uint::GlUInt, gl_uvec2::GlUVec2,
        gl_vec2::GlVec2, gl_vec4::GlVec4, w::W, x::X, y::Y, z::Z,
    };
    #[test]
    fn swizzles_choose_source_relative_type_and_preserve_literal_component_letters() {
        let v = GlReference::new(GlVec4, "v");
        let two = swizzle2(&v, &X, &Y);
        assert_eq!(two.value, "(v.xY)");
        assert_eq!(two.gl_type.type_name(), "vec2");
        let three = swizzle3(&v, &Z, &Y, &X);
        assert_eq!(three.value, "(v.ZYx)");
        assert_eq!(three.gl_type.type_name(), "vec3");
        let four = swizzle4(&v, &W, &Z, &Y, &X);
        assert_eq!(four.value, "(v.WZYx)");
        assert_eq!(four.gl_type.type_name(), "vec4");
        let scalar = get(&v, &X);
        assert_eq!(scalar.value, "(v.x)");
        assert_eq!(scalar.gl_type.type_name(), "float");
        let u = GlReference::new(GlUVec2, "u");
        assert_eq!(get(&u, &Y).gl_type.type_name(), "int");
        // Source permits repeated components and does not validate component against input size.
        assert_eq!(swizzle4(&u, &W, &W, &W, &W).value, "(u.WWWW)");
    }
    #[test]
    fn casts_preserve_target_family_source_names_and_vector_size() {
        let v = GlReference::new(GlVec2, "(a+b)");
        let i = cast_vector(&v, GlIVec2);
        assert_eq!(i.value, "ivec2((a+b))");
        let u = cast_vector(&i, GlUVec2);
        assert_eq!(u.value, "uvec2(ivec2((a+b)))");
        let f = GlReference::new(GlFloat, "f");
        assert_eq!(cast_scalar(&f, GlInt).value, "int(f)");
        assert_eq!(cast_scalar(&f, GlUInt).value, "int(f)");
        assert_eq!(
            cast_scalar(&cast_scalar(&f, GlInt), GlFloat).value,
            "float(int(f))"
        );
    }
}

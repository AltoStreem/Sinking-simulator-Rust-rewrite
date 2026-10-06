//! Port of GLVectorRelatives.java.
use super::{
    four::Four, gl_type::GlType, gl_vector_type::GlVectorType, one::One, three::Three, two::Two,
};
pub(crate) trait GlVectorRelatives {
    type Base: GlType;
    type One: GlVectorType<Base = Self::Base, Size = One>;
    type Two: GlVectorType<Base = Self::Base, Size = Two>;
    type Three: GlVectorType<Base = Self::Base, Size = Three>;
    type Four: GlVectorType<Base = Self::Base, Size = Four>;
    fn base_type(&self) -> Self::Base;
    fn one(&self) -> Self::One;
    fn two(&self) -> Self::Two;
    fn three(&self) -> Self::Three;
    fn four(&self) -> Self::Four;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_builder::{
        float_relatives::FloatRelatives, gl_vector_component::GlVectorComponent,
        gl_vector_size::GlVectorSize, int_relatives::IntRelatives, uint_relatives::UIntRelatives,
    };
    fn check<V: GlVectorType>(vector: V, size: usize, name: &str) {
        assert_eq!(vector.vector_size().size(), size);
        assert_eq!(vector.type_name(), name);
        assert_eq!(vector.relatives().one().vector_size().size(), 1);
        assert_eq!(vector.relatives().four().vector_size().size(), 4);
    }
    #[test]
    fn all_vector_families_preserve_source_sizes_names_and_relatives() {
        let f = FloatRelatives;
        check(f.one(), 1, "vec2");
        check(f.two(), 2, "vec2");
        check(f.three(), 3, "vec3");
        check(f.four(), 4, "vec4");
        let i = IntRelatives;
        check(i.one(), 1, "ivec2");
        check(i.two(), 2, "ivec2");
        check(i.three(), 3, "ivec3");
        check(i.four(), 4, "ivec4");
        let u = UIntRelatives;
        check(u.one(), 1, "uvec2");
        check(u.two(), 2, "uvec2");
        check(u.three(), 3, "uvec3");
        check(u.four(), 4, "uvec4");
        assert_eq!(f.base_type().type_name(), "float");
        assert_eq!(i.base_type().type_name(), "int");
        assert_eq!(u.base_type().type_name(), "int");
    }
    #[test]
    fn source_component_letters_and_size_marker_inheritance_are_preserved() {
        use crate::gl_builder::{w::W, x::X, y::Y, z::Z};
        assert_eq!(
            [X.component(), Y.component(), Z.component(), W.component()],
            ['x', 'Y', 'Z', 'W']
        );
        fn increasing<T: crate::gl_builder::i2::I2>(_: T) {}
        fn decreasing<T: crate::gl_builder::d2::D2>(_: T) {}
        increasing(crate::gl_builder::two::Two);
        increasing(crate::gl_builder::three::Three);
        increasing(crate::gl_builder::four::Four);
        decreasing(crate::gl_builder::one::One);
        decreasing(crate::gl_builder::two::Two);
    }
}

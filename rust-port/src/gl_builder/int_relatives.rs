//! Port of IntRelatives.java.
#[derive(Clone, Copy)]
pub(crate) struct IntRelatives;
impl super::gl_vector_relatives::GlVectorRelatives for IntRelatives {
    type Base = super::gl_int::GlInt;
    fn base_type(&self) -> Self::Base {
        super::gl_int::GlInt
    }
    type One = super::gl_ivec1::GlIVec1;
    fn one(&self) -> Self::One {
        super::gl_ivec1::GlIVec1
    }
    type Two = super::gl_ivec2::GlIVec2;
    fn two(&self) -> Self::Two {
        super::gl_ivec2::GlIVec2
    }
    type Three = super::gl_ivec3::GlIVec3;
    fn three(&self) -> Self::Three {
        super::gl_ivec3::GlIVec3
    }
    type Four = super::gl_ivec4::GlIVec4;
    fn four(&self) -> Self::Four {
        super::gl_ivec4::GlIVec4
    }
}

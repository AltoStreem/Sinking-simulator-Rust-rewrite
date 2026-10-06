//! Port of UIntRelatives.java.
#[derive(Clone, Copy)]
pub(crate) struct UIntRelatives;
impl super::gl_vector_relatives::GlVectorRelatives for UIntRelatives {
    type Base = super::gl_uint::GlUInt;
    fn base_type(&self) -> Self::Base {
        super::gl_uint::GlUInt
    }
    type One = super::gl_uvec1::GlUVec1;
    fn one(&self) -> Self::One {
        super::gl_uvec1::GlUVec1
    }
    type Two = super::gl_uvec2::GlUVec2;
    fn two(&self) -> Self::Two {
        super::gl_uvec2::GlUVec2
    }
    type Three = super::gl_uvec3::GlUVec3;
    fn three(&self) -> Self::Three {
        super::gl_uvec3::GlUVec3
    }
    type Four = super::gl_uvec4::GlUVec4;
    fn four(&self) -> Self::Four {
        super::gl_uvec4::GlUVec4
    }
}

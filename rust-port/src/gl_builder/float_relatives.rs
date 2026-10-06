//! Port of FloatRelatives.java.
#[derive(Clone, Copy)]
pub(crate) struct FloatRelatives;
impl super::gl_vector_relatives::GlVectorRelatives for FloatRelatives {
    type Base = super::gl_float::GlFloat;
    fn base_type(&self) -> Self::Base {
        super::gl_float::GlFloat
    }
    type One = super::gl_vec1::GlVec1;
    fn one(&self) -> Self::One {
        super::gl_vec1::GlVec1
    }
    type Two = super::gl_vec2::GlVec2;
    fn two(&self) -> Self::Two {
        super::gl_vec2::GlVec2
    }
    type Three = super::gl_vec3::GlVec3;
    fn three(&self) -> Self::Three {
        super::gl_vec3::GlVec3
    }
    type Four = super::gl_vec4::GlVec4;
    fn four(&self) -> Self::Four {
        super::gl_vec4::GlVec4
    }
}

//! Port of W.java.
#[derive(Clone, Copy)]
pub(crate) struct W;
impl super::gl_vector_component::GlVectorComponent for W {
    fn component(&self) -> char {
        'W'
    }
}

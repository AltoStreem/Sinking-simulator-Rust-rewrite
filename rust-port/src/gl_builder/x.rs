//! Port of X.java.
#[derive(Clone, Copy)]
pub(crate) struct X;
impl super::gl_vector_component::GlVectorComponent for X {
    fn component(&self) -> char {
        'x'
    }
}

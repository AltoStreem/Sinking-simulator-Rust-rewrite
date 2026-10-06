//! Port of Y.java.
#[derive(Clone, Copy)]
pub(crate) struct Y;
impl super::gl_vector_component::GlVectorComponent for Y {
    fn component(&self) -> char {
        'Y'
    }
}

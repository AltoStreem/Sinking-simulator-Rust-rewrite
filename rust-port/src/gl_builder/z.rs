//! Port of Z.java.
#[derive(Clone, Copy)]
pub(crate) struct Z;
impl super::gl_vector_component::GlVectorComponent for Z {
    fn component(&self) -> char {
        'Z'
    }
}

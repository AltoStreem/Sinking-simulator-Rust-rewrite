//! ship.physics.FullScreen.java's shared position-only indexed quad.
#![allow(dead_code)]
pub(crate) struct SourcePhysicsFullscreen {
    pub model: std::sync::Arc<crate::model::SourceModel>,
}
impl SourcePhysicsFullscreen {
    pub fn new(
        buffers: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        arrays: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        draws: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        Self {
            model: std::sync::Arc::new(crate::model::SourceModel::from_arrays(
                &[0, 1, 2, 0, 2, 3],
                &[-1., 1., -1., -1., 1., -1., 1., 1.],
                2,
                4,
                buffers,
                arrays,
                draws,
                context,
                runtime,
            )),
        }
    }
}
impl crate::i_drawable::IDrawable for SourcePhysicsFullscreen {
    fn render(&self) {
        crate::i_drawable::IDrawable::render(self.model.as_ref());
    }
}

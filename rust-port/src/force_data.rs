//! ForceDataHolder.java extends the source two-component float holder.
use crate::float_data_holder::FloatDataHolder;

#[allow(dead_code)]
pub(crate) struct SourceForceDataHolder(
    pub(crate) crate::float_data_holder::SourceFloatDataHolder<2>,
);
#[allow(dead_code)]
impl SourceForceDataHolder {
    pub(crate) fn new<P: crate::ship_data::SourceMaterialLookup>(
        dat: &crate::ship_data::SourceShipData<P>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        Ok(Self(crate::float_data_holder::SourceFloatDataHolder::new(
            dat.width, dat.height, backend, context, runtime,
        )?))
    }
}
impl std::ops::Deref for SourceForceDataHolder {
    type Target = crate::float_data_holder::SourceFloatDataHolder<2>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl crate::gl_data_holder::SourceGlDataHolder for SourceForceDataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        crate::gl_data_holder::SourceGlDataHolder::source_texture(&self.0)
    }
}
pub(super) struct ForceDataHolder(FloatDataHolder<2>);
impl ForceDataHolder {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self(FloatDataHolder::new(width, height))
    }
    pub(super) fn into_storage(self) -> Vec<[f32; 4]> {
        self.0.into_storage()
    }
}

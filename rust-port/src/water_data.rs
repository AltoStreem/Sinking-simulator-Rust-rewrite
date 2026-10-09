//! WaterDataHolder.java extends the source four-component float holder.
use crate::float_data_holder::FloatDataHolder;

#[allow(dead_code)]
pub(crate) struct SourceWaterDataHolder(
    pub(crate) crate::float_data_holder::SourceFloatDataHolder<4>,
);
#[allow(dead_code)]
impl SourceWaterDataHolder {
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
impl std::ops::Deref for SourceWaterDataHolder {
    type Target = crate::float_data_holder::SourceFloatDataHolder<4>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl crate::gl_data_holder::SourceGlDataHolder for SourceWaterDataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        crate::gl_data_holder::SourceGlDataHolder::source_texture(&self.0)
    }
}
pub(super) struct WaterDataHolder(FloatDataHolder<4>);
impl WaterDataHolder {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self(FloatDataHolder::new(width, height))
    }
    pub(super) fn into_storage(self) -> Vec<[f32; 4]> {
        self.0.into_storage()
    }
    pub(super) fn into_state_planes(self, planes: usize) -> Vec<[f32; 4]> {
        self.0.into_state_planes(planes)
    }
}

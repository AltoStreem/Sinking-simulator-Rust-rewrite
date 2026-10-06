//! WaterDataHolder.java extends the source four-component float holder.
use crate::float_data_holder::FloatDataHolder;
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

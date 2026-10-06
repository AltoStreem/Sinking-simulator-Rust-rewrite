//! ForceDataHolder.java extends the source two-component float holder.
use crate::float_data_holder::FloatDataHolder;
pub(super) struct ForceDataHolder(FloatDataHolder<2>);
impl ForceDataHolder {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self(FloatDataHolder::new(width, height))
    }
    pub(super) fn into_storage(self) -> Vec<[f32; 4]> {
        self.0.into_storage()
    }
}

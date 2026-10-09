//! Recovered Sky$free$2.class: same receiver/owner/name/signature as Sky$3.
pub(crate) fn create(
    receiver: std::rc::Rc<crate::sky::SourceSkyState>,
) -> crate::camera_2d::SourceCameraCallback {
    crate::sky_camera_reference::create(receiver)
}

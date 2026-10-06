//! FramebufferTarget.java.
#![allow(dead_code)]
pub(crate) trait FramebufferTarget: Send + Sync {
    fn bind_to_framebuffer(&self, attachment: i32);
    fn texture(&self) -> Option<&crate::texture::Texture> {
        None
    }
}

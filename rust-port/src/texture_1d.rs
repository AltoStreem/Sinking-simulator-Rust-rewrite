//! Texture1D.java.
#![allow(dead_code)]
use crate::{
    resource::{ResourceHandle, ResourceRuntime},
    texture::{PixelBuffer, Texture, TextureBackend},
};
use std::sync::{Arc, Mutex};
pub(crate) struct Texture1D {
    pub texture: Texture,
    pub img: PixelBuffer,
}
impl Texture1D {
    pub fn new(
        img: PixelBuffer,
        size: i32,
        src_format: i32,
        internal_format: i32,
        data_format: i32,
        backend: Arc<Mutex<dyn TextureBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let texture = Texture::new(
            3552,
            Texture::empty_configuration(),
            backend,
            context,
            runtime,
        );
        texture.bind();
        texture.backend.lock().unwrap().image_1d(
            3552,
            internal_format,
            size,
            src_format,
            data_format,
            &img.lock().unwrap(),
        );
        texture.unbind();
        texture.reconfigure();
        Self { texture, img }
    }
}
impl crate::framebuffer_target::FramebufferTarget for Texture1D {
    fn texture(&self) -> Option<&crate::texture::Texture> {
        Some(&self.texture)
    }
    fn bind_to_framebuffer(&self, attachment: i32) {
        self.texture.bind_to_framebuffer(attachment);
    }
}

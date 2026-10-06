//! Texture2DArray.java: original image target 3553 is preserved despite binding target 35866.
#![allow(dead_code)]
use crate::{
    resource::{ResourceHandle, ResourceRuntime},
    texture::{Configure, PixelBuffer, Texture, TextureBackend},
};
use std::sync::{Arc, Mutex};
pub(crate) struct Texture2DArray {
    pub texture: Texture,
    pub img: Option<PixelBuffer>,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub format: i32,
    pub internal_format: i32,
}
impl Texture2DArray {
    pub fn new(
        img: Option<PixelBuffer>,
        size: [i32; 3],
        format: i32,
        internal_format: i32,
        data_format: i32,
        mipmap: bool,
        conf: Configure,
        backend: Arc<Mutex<dyn TextureBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let texture = Texture::new(35866, conf, backend, context, runtime);
        texture.bind();
        {
            let pixels = img.as_ref().map(|buffer| buffer.lock().unwrap());
            texture.backend.lock().unwrap().image_3d(
                3553,
                internal_format,
                size,
                format,
                data_format,
                pixels.as_deref().map(|pixels| pixels.as_slice()),
            );
        }
        if mipmap {
            texture
                .backend
                .lock()
                .unwrap()
                .generate_mipmaps(texture.target);
        }
        texture.unbind();
        texture.reconfigure();
        Self {
            texture,
            img,
            width: size[0],
            height: size[1],
            depth: size[2],
            format,
            internal_format,
        }
    }
}
impl crate::framebuffer_target::FramebufferTarget for Texture2DArray {
    fn texture(&self) -> Option<&crate::texture::Texture> {
        Some(&self.texture)
    }
    fn bind_to_framebuffer(&self, attachment: i32) {
        self.texture.bind_to_framebuffer(attachment);
    }
}

//! ImageData.java fields. OpenGL format numbers remain source metadata.
#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImageData {
    pub buffer: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub image_format: i32,
    pub format: i32,
}
impl ImageData {
    pub fn new(buffer: Vec<u8>, width: i32, height: i32, image_format: i32) -> Self {
        Self {
            buffer,
            width,
            height,
            image_format,
            format: 5121,
        }
    }
    pub fn into_rgba(self) -> Result<image::RgbaImage, String> {
        if self.image_format != 6408 || self.format != 5121 {
            return Err("RGBA unsigned-byte image required".into());
        }
        image::RgbaImage::from_raw(self.width as u32, self.height as u32, self.buffer)
            .ok_or_else(|| "Invalid image buffer dimensions".into())
    }
}

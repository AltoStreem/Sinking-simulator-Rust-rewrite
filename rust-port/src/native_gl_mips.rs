//! Windows implementation of the source RGBA8 glGenerateMipmap operation.
//! A private, invisible context belongs to each calling thread. Bevy receives
//! the resulting bytes; no foreign runtime or reference fixture is used.
use std::{cell::RefCell, ffi::c_void, ptr::null_mut};
type Handle = *mut c_void;
type Generate = unsafe extern "system" fn(u32);
#[repr(C)]
#[derive(Default)]
struct PixelFormat {
    size: u16,
    version: u16,
    flags: u32,
    pixel_type: u8,
    color_bits: u8,
    red_bits: u8,
    red_shift: u8,
    green_bits: u8,
    green_shift: u8,
    blue_bits: u8,
    blue_shift: u8,
    alpha_bits: u8,
    alpha_shift: u8,
    accum_bits: u8,
    accum_red_bits: u8,
    accum_green_bits: u8,
    accum_blue_bits: u8,
    accum_alpha_bits: u8,
    depth_bits: u8,
    stencil_bits: u8,
    aux_buffers: u8,
    layer_type: u8,
    reserved: u8,
    layer_mask: u32,
    visible_mask: u32,
    damage_mask: u32,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        param: Handle,
    ) -> Handle;
    fn GetDC(window: Handle) -> Handle;
    fn ReleaseDC(window: Handle, dc: Handle) -> i32;
    fn DestroyWindow(window: Handle) -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn ChoosePixelFormat(dc: Handle, format: *const PixelFormat) -> i32;
    fn SetPixelFormat(dc: Handle, index: i32, format: *const PixelFormat) -> i32;
}
#[link(name = "opengl32")]
unsafe extern "system" {
    fn wglCreateContext(dc: Handle) -> Handle;
    fn wglDeleteContext(context: Handle) -> i32;
    fn wglMakeCurrent(dc: Handle, context: Handle) -> i32;
    fn wglGetCurrentContext() -> Handle;
    fn wglGetCurrentDC() -> Handle;
    fn wglGetProcAddress(name: *const u8) -> *const c_void;
    fn glGenTextures(count: i32, texture: *mut u32);
    fn glDeleteTextures(count: i32, texture: *const u32);
    fn glBindTexture(target: u32, texture: u32);
    fn glTexImage2D(
        target: u32,
        level: i32,
        internal: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        kind: u32,
        pixels: *const c_void,
    );
    fn glGetTexImage(target: u32, level: i32, format: u32, kind: u32, pixels: *mut c_void);
    fn glGetError() -> u32;
}
struct Current {
    dc: Handle,
    context: Handle,
}
impl Current {
    fn bind(dc: Handle, context: Handle) -> Result<Self, String> {
        // SAFETY: handles belong to this thread; the previous pair is restored
        // before the private context can be destroyed, including error paths.
        unsafe {
            let previous = Self {
                dc: wglGetCurrentDC(),
                context: wglGetCurrentContext(),
            };
            if wglMakeCurrent(dc, context) == 0 {
                return Err("Cannot bind source GL mip context".into());
            }
            Ok(previous)
        }
    }
}
impl Drop for Current {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(self.dc, self.context);
        }
    }
}
struct Context {
    window: Handle,
    dc: Handle,
    context: Handle,
    generate: Option<Generate>,
}
impl Context {
    fn new() -> Result<Self, String> {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let title: Vec<u16> = "Source texture mip context\0".encode_utf16().collect();
        // SAFETY: the strings are terminated and live for the complete call.
        // No WS_VISIBLE flag is used and the context never leaves this thread.
        unsafe {
            let window = CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                0x80000000,
                0,
                0,
                16,
                16,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            );
            if window.is_null() {
                return Err("Cannot create source GL mip window".into());
            }
            let mut result = Self {
                window,
                dc: GetDC(window),
                context: null_mut(),
                generate: None,
            };
            if result.dc.is_null() {
                return Err("Cannot obtain source GL mip DC".into());
            }
            let format = PixelFormat {
                size: std::mem::size_of::<PixelFormat>() as u16,
                version: 1,
                flags: 0x25,
                color_bits: 24,
                alpha_bits: 8,
                ..Default::default()
            };
            let index = ChoosePixelFormat(result.dc, &format);
            if index == 0 || SetPixelFormat(result.dc, index, &format) == 0 {
                return Err("Cannot set source GL mip pixel format".into());
            }
            result.context = wglCreateContext(result.dc);
            if result.context.is_null() {
                return Err("Cannot create source GL mip context".into());
            }
            let _restore = Current::bind(result.dc, result.context)?;
            let function = wglGetProcAddress(c"glGenerateMipmap".as_ptr().cast());
            if function.is_null() || (function as usize) <= 3 || function as isize == -1 {
                return Err("Source glGenerateMipmap is unavailable".into());
            }
            result.generate = Some(std::mem::transmute::<*const c_void, Generate>(function));
            Ok(result)
        }
    }
    fn generate(&self, rgba: &image::RgbaImage) -> Result<(Vec<u8>, u32), String> {
        let mut width =
            i32::try_from(rgba.width()).map_err(|_| "Texture width exceeds GL range")?;
        let mut height =
            i32::try_from(rgba.height()).map_err(|_| "Texture height exceeds GL range")?;
        if width == 0 || height == 0 {
            return Err("Empty source mip texture".into());
        }
        let _restore = Current::bind(self.dc, self.context)?;
        struct Texture(u32);
        impl Drop for Texture {
            fn drop(&mut self) {
                unsafe {
                    glDeleteTextures(1, &self.0);
                }
            }
        }
        // SAFETY: the private context is current, the upload buffer is valid,
        // and each readback allocation has exactly width*height*4 bytes.
        unsafe {
            let mut texture = Texture(0);
            glGenTextures(1, &mut texture.0);
            glBindTexture(3553, texture.0);
            glTexImage2D(
                3553,
                0,
                32856,
                width,
                height,
                0,
                6408,
                5121,
                rgba.as_raw().as_ptr().cast(),
            );
            self.generate.expect("Initialized source GL function")(3553);
            let error = glGetError();
            if error != 0 {
                return Err(format!(
                    "Source GL mip upload/generation failed: {error:#x}"
                ));
            }
            let mut data = Vec::new();
            let mut level = 0;
            loop {
                let length = (width as usize)
                    .checked_mul(height as usize)
                    .and_then(|n| n.checked_mul(4))
                    .ok_or("Source mip allocation overflow")?;
                let offset = data.len();
                data.resize(offset + length, 0);
                glGetTexImage(
                    3553,
                    level,
                    6408,
                    5121,
                    data.as_mut_ptr().add(offset).cast(),
                );
                let error = glGetError();
                if error != 0 {
                    return Err(format!("Source GL mip operation failed: {error:#x}"));
                }
                level += 1;
                if width == 1 && height == 1 {
                    break;
                }
                width = (width / 2).max(1);
                height = (height / 2).max(1);
            }
            Ok((data, level as u32))
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() {
                if wglGetCurrentContext() == self.context {
                    wglMakeCurrent(null_mut(), null_mut());
                }
                wglDeleteContext(self.context);
            }
            if !self.dc.is_null() {
                ReleaseDC(self.window, self.dc);
            }
            DestroyWindow(self.window);
        }
    }
}
thread_local! {static CONTEXT:RefCell<Option<Context>>=const {RefCell::new(None)};}
pub(crate) fn generate(rgba: &image::RgbaImage) -> Result<(Vec<u8>, u32), String> {
    CONTEXT.with(|slot| {
        let mut context = slot.borrow_mut();
        if context.is_none() {
            *context = Some(Context::new()?);
        }
        context.as_ref().unwrap().generate(rgba)
    })
}
pub(crate) fn release_current_thread() {
    CONTEXT.with(|slot| drop(slot.borrow_mut().take()));
}
pub(crate) fn with_current<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    CONTEXT.with(|slot| {
        let mut context = slot.borrow_mut();
        if context.is_none() {
            *context = Some(Context::new()?);
        }
        let context = context.as_ref().unwrap();
        let _restore = Current::bind(context.dc, context.context)?;
        operation()
    })
}
pub(crate) fn procedure(name: &std::ffi::CStr) -> *const c_void {
    unsafe { wglGetProcAddress(name.as_ptr().cast()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_mip_context_restores_callers_context_and_recreates_after_release() {
        assert_eq!(
            std::mem::size_of::<PixelFormat>(),
            40,
            "Win32 PIXELFORMATDESCRIPTOR ABI"
        );
        let caller = Context::new().unwrap();
        let _restore = Current::bind(caller.dc, caller.context).unwrap();
        let pixels = image::RgbaImage::from_pixel(7, 5, image::Rgba([51, 102, 153, 128]));
        for _ in 0..2 {
            let (data, levels) = generate(&pixels).unwrap();
            assert_eq!(levels, 3);
            assert!(
                data.chunks_exact(4)
                    .all(|pixel| pixel == [51, 102, 153, 128])
            );
            unsafe {
                assert_eq!(wglGetCurrentContext(), caller.context);
                assert_eq!(wglGetCurrentDC(), caller.dc);
            }
            release_current_thread();
            unsafe {
                assert_eq!(wglGetCurrentContext(), caller.context);
            }
        }
        assert!(generate(&image::RgbaImage::new(0, 0)).is_err());
        unsafe {
            assert_eq!(wglGetCurrentContext(), caller.context);
        }
        release_current_thread();
    }
}

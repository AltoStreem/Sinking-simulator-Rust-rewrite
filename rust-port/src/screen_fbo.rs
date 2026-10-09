//! ScreenFBO.java's window-sized render target, adapted to Bevy image targets.
//! The scene target is copied into a regenerated mip chain before Sea renders.
use bevy::{prelude::*, render::render_resource::TextureFormat};
#[derive(Resource, Clone, bevy::render::extract_resource::ExtractResource)]
pub(crate) struct ScreenFbo {
    pub texture: Handle<Image>,
    pub filtered: Handle<Image>,
    pub size: UVec2,
    pub native_border: bool,
}
impl ScreenFbo {
    pub fn new(size: UVec2, images: &mut Assets<Image>) -> Self {
        Self::with_native_border(size, images, false)
    }
    pub fn with_native_border(
        size: UVec2,
        images: &mut Assets<Image>,
        native_border: bool,
    ) -> Self {
        Self {
            texture: images.add(Self::image(size, native_border)),
            filtered: images.add(Self::mip_image(size, native_border)),
            size,
            native_border,
        }
    }
    fn image(size: UVec2, native_border: bool) -> Image {
        let mut image = Image::new_target_texture(
            size.x.max(1),
            size.y.max(1),
            TextureFormat::Rgba8Unorm,
            None,
        );
        image.data = None;
        image.copy_on_resize = false;
        image.texture_descriptor.usage |= bevy::render::render_resource::TextureUsages::COPY_SRC;
        image.sampler = crate::main_screen_fbo_texture::sampler(native_border);
        image
    }
    fn mip_image(size: UVec2, native_border: bool) -> Image {
        let mut image = Self::image(size, native_border);
        image.texture_descriptor.mip_level_count = mip_count(size);
        image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;
        image
    }
    pub fn resize(&mut self, size: UVec2, images: &mut Assets<Image>) {
        if self.size != size {
            if let Some(mut image) = images.get_mut(&self.texture) {
                *image = Self::image(size, self.native_border);
            }
            if let Some(mut image) = images.get_mut(&self.filtered) {
                *image = Self::mip_image(size, self.native_border);
            }
            self.size = size;
        }
    }
}

fn mip_count(size: UVec2) -> u32 {
    32 - size.x.max(size.y).max(1).leading_zeros()
}

#[allow(dead_code)]
pub(crate) struct SourceScreenFbo {
    pub framebuffer: std::rc::Rc<std::cell::RefCell<crate::render_fbo::SourceRenderFbo>>,
    window: std::rc::Rc<crate::window::SourceWindow>,
    callback: std::rc::Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)>,
}
#[allow(dead_code)]
impl SourceScreenFbo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window: std::rc::Rc<crate::window::SourceWindow>,
        stencil_depth: bool,
        mipmap: bool,
        conf: crate::texture::Configure,
        texture_backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        renderbuffer_backend: std::sync::Arc<
            std::sync::Mutex<dyn crate::render_buffer::RenderBufferBackend>,
        >,
        framebuffer_backend: std::sync::Arc<std::sync::Mutex<dyn crate::fbo::FramebufferBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        // The original evaluates getFramebufferSize separately for x and y.
        let width = window.framebuffer_size()[0];
        let height = window.framebuffer_size()[1];
        let framebuffer = crate::render_fbo::SourceRenderFbo::new(
            [width, height],
            1,
            32856,
            stencil_depth,
            mipmap,
            conf,
            texture_backend,
            renderbuffer_backend,
            framebuffer_backend,
            context,
            runtime,
        )?;
        Ok(Self::from_parts(window, framebuffer))
    }
    pub fn from_parts(
        window: std::rc::Rc<crate::window::SourceWindow>,
        framebuffer: crate::render_fbo::SourceRenderFbo,
    ) -> Self {
        use std::{cell::RefCell, rc::Rc};
        let lifetime = framebuffer.framebuffer.fbo.resource_handle();
        let framebuffer = Rc::new(RefCell::new(framebuffer));
        let resize = framebuffer.clone();
        let callback: Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> =
            Rc::new(move |_, width, height| resize.borrow_mut().resize(width, height));
        window
            .framebuffer_size_callbacks
            .borrow_mut()
            .push(callback.clone());
        let cleanup_window = window.clone();
        let cleanup_callback = callback.clone();
        lifetime.before_free_local(move || {
            let mut callbacks = cleanup_window.framebuffer_size_callbacks.borrow_mut();
            if let Some(index) = callbacks
                .iter()
                .position(|callback| Rc::ptr_eq(callback, &cleanup_callback))
            {
                callbacks.remove(index);
            }
        });
        window.register_dependent(&lifetime);
        Self {
            framebuffer,
            window,
            callback,
        }
    }
    pub fn window(&self) -> std::rc::Rc<crate::window::SourceWindow> {
        self.window.clone()
    }
    pub fn framebuffer_callback(
        &self,
    ) -> std::rc::Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> {
        self.callback.clone()
    }
    pub fn texture(&self) -> std::sync::Arc<dyn crate::framebuffer_target::FramebufferTarget> {
        let framebuffer = self.framebuffer.borrow();
        let target = framebuffer.framebuffer.targets.lock().unwrap()[0].clone();
        assert!(
            target.texture().is_some(),
            "framebuffer target cannot be cast to Texture"
        );
        target
    }
    pub fn close(&self) {
        self.framebuffer.borrow().framebuffer.fbo.close();
    }
    pub fn freed(&self) -> bool {
        self.framebuffer.borrow().framebuffer.fbo.freed()
    }
}

struct ScreenTexture(std::sync::Arc<dyn crate::framebuffer_target::FramebufferTarget>);
impl crate::sea::SeaTexture for ScreenTexture {
    fn bind(&self, unit: i32) {
        self.0.texture().expect("Texture target").bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.0.texture().expect("Texture target").unbind_unit(unit);
    }
}
impl crate::sea::SeaFramebuffer for SourceScreenFbo {
    fn texture(&self) -> std::rc::Rc<dyn crate::sea::SeaTexture> {
        std::rc::Rc::new(ScreenTexture(self.texture()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_screen_constructor_texture_replacement_and_deferred_window_cleanup() {
        use std::{rc::Rc, sync::Arc};
        let (window, runtime, log) = crate::window::tests::fixture();
        let backend = crate::render_fbo::source_tests::backend(log.clone());
        let context = runtime.allocate(&[], || {});
        let screen = SourceScreenFbo::new(
            window.clone(),
            true,
            true,
            Arc::new(crate::main_screen_fbo_texture::configure),
            backend.clone(),
            backend.clone(),
            backend,
            context,
            &runtime,
        )
        .unwrap();
        assert!(Rc::ptr_eq(&window, &screen.window()));
        assert!(Rc::ptr_eq(
            &screen.framebuffer_callback(),
            &screen.framebuffer_callback()
        ));
        let first = screen.texture();
        assert!(Arc::ptr_eq(&first, &screen.texture()));
        assert_eq!(
            (
                screen.framebuffer.borrow().framebuffer.width,
                screen.framebuffer.borrow().framebuffer.height
            ),
            (800, 400)
        );
        assert!(screen.framebuffer.borrow().framebuffer.depth.is_some());
        let captured = crate::sea::SeaFramebuffer::texture(&screen);
        window.emit_framebuffer_size(0, 99);
        assert_eq!(
            (
                screen.framebuffer.borrow().framebuffer.width,
                screen.framebuffer.borrow().framebuffer.height
            ),
            (0, 99)
        );
        assert!(!Arc::ptr_eq(&first, &screen.texture()));
        log.lock().unwrap().clear();
        captured.bind(0);
        captured.unbind(0);
        assert!(log.lock().unwrap().iter().any(|e| e == "texture:3553:1"));
        let external: Rc<dyn Fn(&crate::window::SourceWindow, i32, i32)> = Rc::new(|_, _, _| {});
        window
            .framebuffer_size_callbacks
            .borrow_mut()
            .push(external.clone());
        log.lock().unwrap().clear();
        window.close();
        assert!(screen.freed());
        assert_eq!(window.framebuffer_size_callbacks.borrow().len(), 2);
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(window.framebuffer_size_callbacks.borrow().len(), 1);
        assert!(Rc::ptr_eq(
            &window.framebuffer_size_callbacks.borrow()[0],
            &external
        ));
        let events = log.lock().unwrap();
        let framebuffer = events.iter().position(|e| e == "delete_fbo:41").unwrap();
        let window_free = events.iter().position(|e| e == "free-callbacks:7").unwrap();
        assert!(framebuffer < window_free);
        drop(events);
        screen.close();
        runtime.run_main();
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|e| *e == "delete_fbo:41")
                .count(),
            1
        );
    }
    #[test]
    fn framebuffer_mip_chain_and_resize_keep_source_dimensions() {
        let mut images = Assets::default();
        let mut target = ScreenFbo::new(UVec2::new(2554, 1378), &mut images);
        let original = target.texture.clone();
        let filtered = target.filtered.clone();
        assert_eq!(
            images
                .get(&filtered)
                .unwrap()
                .texture_descriptor
                .mip_level_count,
            12
        );
        target.resize(UVec2::new(3, 7), &mut images);
        assert_eq!(target.texture, original);
        assert_eq!(target.filtered, filtered);
        assert_eq!(
            images
                .get(&filtered)
                .unwrap()
                .texture_descriptor
                .mip_level_count,
            3
        );
        assert_eq!(
            images.get(&filtered).unwrap().texture_descriptor.size.width,
            3
        );
    }
    #[test]
    fn native_transparent_border_survives_both_target_resize_paths() {
        use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerBorderColor};
        let mut images = Assets::default();
        let mut target = ScreenFbo::with_native_border(UVec2::new(32, 16), &mut images, true);
        for size in [UVec2::new(32, 16), UVec2::new(7, 3)] {
            target.resize(size, &mut images);
            assert!(target.native_border);
            for handle in [&target.texture, &target.filtered] {
                let ImageSampler::Descriptor(sampler) = &images.get(handle).unwrap().sampler else {
                    panic!()
                };
                assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToBorder);
                assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToBorder);
                assert_eq!(
                    sampler.border_color,
                    Some(ImageSamplerBorderColor::TransparentBlack)
                );
            }
        }
    }
    #[test]
    fn framebuffer_compute_shader_validates() {
        let shader = naga::front::wgsl::parse_str(include_str!(
            "../assets/shaders/framebuffer_mipmaps.wgsl"
        ))
        .unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&shader)
        .unwrap();
    }
}

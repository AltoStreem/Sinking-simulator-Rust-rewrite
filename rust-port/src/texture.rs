//! Texture.java binding, parameters, framebuffer attachment, readback and mipmaps.
#![allow(dead_code)]
use crate::{
    framebuffer_target::FramebufferTarget,
    gl_resource::GlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex};
pub(crate) type Configure = Arc<dyn Fn(&Texture) + Send + Sync>;
pub(crate) type PixelBuffer = Arc<Mutex<Vec<u8>>>;
pub(crate) trait TextureBackend: Send {
    fn create_texture(&mut self) -> i32;
    fn bind_texture(&mut self, target: i32, id: i32);
    fn active_texture(&mut self, unit: i32);
    fn parameter(&mut self, target: i32, parameter: i32, value: i32);
    fn image_1d(
        &mut self,
        target: i32,
        internal: i32,
        width: i32,
        format: i32,
        data_type: i32,
        data: &[u8],
    );
    fn image_2d(
        &mut self,
        target: i32,
        internal: i32,
        size: [i32; 2],
        format: i32,
        data_type: i32,
        data: Option<&[u8]>,
    );
    fn image_3d(
        &mut self,
        target: i32,
        internal: i32,
        size: [i32; 3],
        format: i32,
        data_type: i32,
        data: Option<&[u8]>,
    );
    fn get_image_floats(
        &mut self,
        target: i32,
        level: i32,
        format: i32,
        data_type: i32,
        output: &mut [f32],
    );
    fn framebuffer_texture(&mut self, target: i32, attachment: i32, id: i32, level: i32);
    fn check_error(&mut self, label: &str);
    fn generate_mipmaps(&mut self, target: i32);
    fn delete_texture(&mut self, id: i32);
}
pub(crate) struct Texture {
    resource: GlResource,
    pub target: i32,
    pub conf: Configure,
    pub(crate) backend: Arc<Mutex<dyn TextureBackend>>,
}
impl Texture {
    pub fn new(
        target: i32,
        conf: Configure,
        backend: Arc<Mutex<dyn TextureBackend>>,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
    ) -> Self {
        let id = backend.lock().unwrap().create_texture();
        let cleanup = backend.clone();
        Self {
            resource: GlResource::new(id, context, runtime, move || {
                cleanup.lock().unwrap().delete_texture(id)
            }),
            target,
            conf,
            backend,
        }
    }
    pub fn empty_configuration() -> Configure {
        Arc::new(|_| {})
    }
    pub fn id(&self) -> i32 {
        self.resource.id()
    }
    pub fn bind(&self) {
        self.backend
            .lock()
            .unwrap()
            .bind_texture(self.target, self.id());
    }
    pub fn unbind(&self) {
        self.backend.lock().unwrap().bind_texture(self.target, 0);
    }
    pub fn bind_unit(&self, index: i32) {
        let mut backend = self.backend.lock().unwrap();
        backend.active_texture(33984i32.wrapping_add(index));
        backend.bind_texture(self.target, self.id());
        backend.active_texture(33984);
    }
    pub fn unbind_unit(&self, index: i32) {
        let mut backend = self.backend.lock().unwrap();
        backend.active_texture(33984i32.wrapping_add(index));
        backend.bind_texture(self.target, 0);
        backend.active_texture(33984);
    }
    pub fn set_parameter(&self, parameter: i32, value: i32) {
        self.bind();
        self.backend
            .lock()
            .unwrap()
            .parameter(self.target, parameter, value);
        self.unbind();
    }
    pub fn download_floats(
        &self,
        width: i32,
        height: i32,
        components: i32,
    ) -> Result<Vec<f32>, String> {
        self.bind();
        let bytes = width
            .wrapping_mul(height)
            .wrapping_mul(4)
            .wrapping_mul(components);
        if bytes < 0 {
            return Err("negative source readback buffer capacity".into());
        }
        let mut output = vec![0.; bytes as usize / 4];
        self.backend.lock().unwrap().get_image_floats(
            3553,
            0,
            component_format(components),
            5126,
            &mut output,
        );
        self.unbind();
        Ok(output)
    }
    pub fn gen_mipmaps(&self) {
        self.bind();
        self.backend.lock().unwrap().generate_mipmaps(self.target);
        self.unbind();
    }
    pub fn reconfigure(&self) {
        (self.conf)(self);
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}
pub(crate) fn component_format(components: i32) -> i32 {
    match components {
        1 => 6403,
        2 => 33319,
        3 => 6407,
        4 => 6408,
        _ => 0,
    }
}
impl FramebufferTarget for Texture {
    fn texture(&self) -> Option<&crate::texture::Texture> {
        Some(self)
    }
    fn bind_to_framebuffer(&self, attachment: i32) {
        let mut backend = self.backend.lock().unwrap();
        backend.framebuffer_texture(36160, attachment, self.id(), 0);
        backend.check_error("Texture Bind To Framebuffer");
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        texture_1d::Texture1D, texture_2d::SourceTexture2D, texture_2d_array::Texture2DArray,
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct Backend(Log);
    pub(crate) fn backend(log: Arc<Mutex<Vec<String>>>) -> Arc<Mutex<dyn TextureBackend>> {
        Arc::new(Mutex::new(Backend(log)))
    }
    impl Backend {
        fn log(&self, event: impl Into<String>) {
            self.0.lock().unwrap().push(event.into());
        }
    }
    impl TextureBackend for Backend {
        fn create_texture(&mut self) -> i32 {
            self.log("create");
            61
        }
        fn bind_texture(&mut self, target: i32, id: i32) {
            self.log(format!("bind:{target}:{id}"));
        }
        fn active_texture(&mut self, unit: i32) {
            self.log(format!("active:{unit}"));
        }
        fn parameter(&mut self, target: i32, param: i32, value: i32) {
            self.log(format!("parameter:{target}:{param}:{value}"));
        }
        fn image_1d(
            &mut self,
            target: i32,
            internal: i32,
            width: i32,
            format: i32,
            kind: i32,
            data: &[u8],
        ) {
            self.log(format!(
                "image1:{target}:{internal}:{width}:{format}:{kind}:{}",
                data.len()
            ));
        }
        fn image_2d(
            &mut self,
            target: i32,
            internal: i32,
            size: [i32; 2],
            format: i32,
            kind: i32,
            data: Option<&[u8]>,
        ) {
            self.log(format!(
                "image2:{target}:{internal}:{size:?}:{format}:{kind}:{:?}",
                data.map(|d| d.len())
            ));
        }
        fn image_3d(
            &mut self,
            target: i32,
            internal: i32,
            size: [i32; 3],
            format: i32,
            kind: i32,
            data: Option<&[u8]>,
        ) {
            self.log(format!(
                "image3:{target}:{internal}:{size:?}:{format}:{kind}:{:?}",
                data.map(|d| d.len())
            ));
        }
        fn get_image_floats(
            &mut self,
            target: i32,
            level: i32,
            format: i32,
            kind: i32,
            output: &mut [f32],
        ) {
            self.log(format!(
                "read:{target}:{level}:{format}:{kind}:{}",
                output.len()
            ));
            for (i, v) in output.iter_mut().enumerate() {
                *v = i as f32;
            }
        }
        fn framebuffer_texture(&mut self, target: i32, attachment: i32, id: i32, level: i32) {
            self.log(format!("attach:{target}:{attachment}:{id}:{level}"));
        }
        fn check_error(&mut self, label: &str) {
            self.log(format!("check:{label}"));
        }
        fn generate_mipmaps(&mut self, target: i32) {
            self.log(format!("mips:{target}"));
        }
        fn delete_texture(&mut self, id: i32) {
            self.log(format!("delete:{id}"));
        }
    }
    #[test]
    fn texture_units_parameters_attachment_and_readback_preserve_source_order() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let texture = Texture::new(
            35866,
            Texture::empty_configuration(),
            backend,
            context,
            &runtime,
        );
        log.lock().unwrap().clear();
        texture.bind_unit(2);
        texture.unbind_unit(2);
        texture.set_parameter(10240, 9728);
        texture.bind_to_framebuffer(36064);
        texture.gen_mipmaps();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "active:33986",
                "bind:35866:61",
                "active:33984",
                "active:33986",
                "bind:35866:0",
                "active:33984",
                "bind:35866:61",
                "parameter:35866:10240:9728",
                "bind:35866:0",
                "attach:36160:36064:61:0",
                "check:Texture Bind To Framebuffer",
                "bind:35866:61",
                "mips:35866",
                "bind:35866:0"
            ]
        );
        log.lock().unwrap().clear();
        assert_eq!(texture.download_floats(2, 1, 2).unwrap(), [0., 1., 2., 3.]);
        // Source readback uses 3553 even when the bound texture has another target.
        assert_eq!(
            *log.lock().unwrap(),
            ["bind:35866:61", "read:3553:0:33319:5126:4", "bind:35866:0"]
        );
        assert_eq!(
            (0..=5).map(component_format).collect::<Vec<_>>(),
            [0, 6403, 33319, 6407, 6408, 0]
        );
    }
    #[test]
    fn dimensional_uploads_keep_null_pixels_mipmap_timing_and_configuration() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let config_log = log.clone();
        let conf: Configure = Arc::new(move |t| {
            config_log.lock().unwrap().push("configure".into());
            t.set_parameter(10241, 9728);
        });
        let pixels = Arc::new(Mutex::new(vec![1, 2, 3, 4]));
        let one = Texture1D::new(
            pixels.clone(),
            4,
            6403,
            6403,
            5121,
            backend.clone(),
            context.clone(),
            &runtime,
        );
        assert!(Arc::ptr_eq(&one.img, &pixels));
        assert_eq!(
            *log.lock().unwrap(),
            [
                "create",
                "bind:3552:61",
                "image1:3552:6403:4:6403:5121:4",
                "bind:3552:0"
            ]
        );
        log.lock().unwrap().clear();
        let two = SourceTexture2D::new(
            None,
            [2, 3],
            6408,
            32856,
            5125,
            true,
            conf.clone(),
            backend.clone(),
            context.clone(),
            &runtime,
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                "create",
                "bind:3553:61",
                "image2:3553:32856:[2, 3]:6408:5125:None",
                "mips:3553",
                "bind:3553:0",
                "configure",
                "bind:3553:61",
                "parameter:3553:10241:9728",
                "bind:3553:0"
            ]
        );
        assert!(two.img.is_none());
        log.lock().unwrap().clear();
        let array = Texture2DArray::new(
            Some(pixels.clone()),
            [2, 3, 4],
            6408,
            32856,
            5121,
            true,
            conf,
            backend,
            context,
            &runtime,
        );
        assert!(Arc::ptr_eq(array.img.as_ref().unwrap(), &pixels));
        assert_eq!(
            *log.lock().unwrap(),
            [
                "create",
                "bind:35866:61",
                "image3:3553:32856:[2, 3, 4]:6408:5121:Some(4)",
                "mips:35866",
                "bind:35866:0",
                "configure",
                "bind:35866:61",
                "parameter:35866:10241:9728",
                "bind:35866:0"
            ]
        );
    }
    #[test]
    fn context_deletion_is_deferred_and_failed_readback_leaves_source_binding() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let texture = Texture::new(
            3553,
            Texture::empty_configuration(),
            backend,
            context.clone(),
            &runtime,
        );
        log.lock().unwrap().clear();
        assert!(texture.download_floats(-1, 1, 1).is_err());
        assert_eq!(*log.lock().unwrap(), ["bind:3553:61"]);
        log.lock().unwrap().clear();
        context.close();
        assert!(texture.freed());
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(*log.lock().unwrap(), ["delete:61"]);
    }
}

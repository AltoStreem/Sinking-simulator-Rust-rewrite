//! TexturedFBO.java draw scopes and attachment construction; source has no resize method.
#![allow(dead_code)]
use crate::{fbo::Fbo, framebuffer_target::FramebufferTarget};
use std::sync::{Arc, Mutex};
pub(crate) type Targets = Arc<Mutex<Box<[Arc<dyn FramebufferTarget>]>>>;
pub(crate) struct TexturedFbo {
    pub fbo: Fbo,
    pub width: i32,
    pub height: i32,
    pub targets: Targets,
    pub depth: Option<Arc<dyn FramebufferTarget>>,
    pub on_unbind: Option<Arc<dyn Fn(&Targets) + Send + Sync>>,
}
impl TexturedFbo {
    pub fn new(
        width: i32,
        height: i32,
        targets: Targets,
        depth: Option<Arc<dyn FramebufferTarget>>,
        fbo: Fbo,
    ) -> Result<Self, String> {
        let result = Self {
            width,
            height,
            targets,
            depth,
            fbo,
            on_unbind: None,
        };
        result.bind();
        let count = result.targets.lock().unwrap().len();
        if count > 24 {
            return Err("cannot bind more than 24 update targets to FBO".into());
        }
        // Constructor retains the array; attachment bindings are a construction-time snapshot.
        let snapshot = result.targets.lock().unwrap().clone();
        for (index, target) in snapshot.iter().enumerate() {
            result.bind_texture(target.as_ref(), 36064 + index as i32);
        }
        if let Some(depth) = &result.depth {
            result.bind_texture(depth.as_ref(), 33306);
        }
        result.unbind();
        Ok(result)
    }
    pub fn bind(&self) {
        self.fbo.bind();
    }
    pub fn unbind(&self) {
        self.fbo.unbind_with(|| {
            if let Some(hook) = &self.on_unbind {
                hook(&self.targets);
            }
        });
    }
    pub fn bind_texture(&self, target: &dyn FramebufferTarget, attachment: i32) {
        self.bind();
        target.bind_to_framebuffer(attachment);
        self.unbind();
    }
    pub fn draw<R>(&self, draw: impl FnOnce() -> R) -> R {
        let viewport = self.fbo.backend.lock().unwrap().viewport();
        self.bind();
        let attachments: Vec<_> = (0..self.targets.lock().unwrap().len())
            .map(|index| 36064 + index as i32)
            .collect();
        {
            let mut backend = self.fbo.backend.lock().unwrap();
            backend.draw_buffers(&attachments);
            backend.set_viewport([0, 0, self.width, self.height]);
        }
        let result = draw(); // Original has no finally: failure leaves FBO/viewport bound.
        self.fbo
            .backend
            .lock()
            .unwrap()
            .check_error("After FB Draw");
        self.unbind();
        let mut backend = self.fbo.backend.lock().unwrap();
        backend.check_error("FB Unbind");
        backend.set_viewport(viewport);
        backend.check_error("FB reset wiewport");
        result
    }
}
/// Connects the translated framebuffer scope to the existing TargetPass contract.
impl crate::passes::target_pass::TargetBinding for TexturedFbo {
    fn check_error(&mut self, label: &str) {
        self.fbo.backend.lock().unwrap().check_error(label);
    }
    fn viewport(&mut self) -> [i32; 4] {
        self.fbo.backend.lock().unwrap().viewport()
    }
    fn bind(&mut self) {
        TexturedFbo::bind(self)
    }
    fn draw_buffers(&mut self, attachments: &[u32]) {
        self.fbo.backend.lock().unwrap().draw_buffers(
            &attachments
                .iter()
                .map(|value| *value as i32)
                .collect::<Vec<_>>(),
        );
    }
    fn set_viewport(&mut self, viewport: [i32; 4]) {
        self.fbo.backend.lock().unwrap().set_viewport(viewport);
    }
    fn unbind(&mut self) {
        TexturedFbo::unbind(self)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        fbo::FramebufferBackend,
        render_buffer::{RenderBuffer, RenderBufferBackend},
        resource::ResourceRuntime,
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct Backend(Log);
    pub(crate) fn backend(log: Arc<Mutex<Vec<String>>>) -> Arc<Mutex<dyn FramebufferBackend>> {
        Arc::new(Mutex::new(Backend(log)))
    }
    impl Backend {
        fn log(&self, value: impl Into<String>) {
            self.0.lock().unwrap().push(value.into());
        }
    }
    impl FramebufferBackend for Backend {
        fn create_framebuffer(&mut self) -> i32 {
            self.log("create_fbo");
            41
        }
        fn bind_framebuffer(&mut self, target: i32, id: i32) {
            self.log(format!("fbo:{target}:{id}"));
        }
        fn check_error(&mut self, label: &str) {
            self.log(format!("check:{label}"));
        }
        fn viewport(&mut self) -> [i32; 4] {
            self.log("get_viewport");
            [2, 3, 800, 600]
        }
        fn draw_buffers(&mut self, attachments: &[i32]) {
            self.log(format!("draw_buffers:{attachments:?}"));
        }
        fn set_viewport(&mut self, viewport: [i32; 4]) {
            self.log(format!("viewport:{viewport:?}"));
        }
        fn delete_framebuffer(&mut self, id: i32) {
            self.log(format!("delete_fbo:{id}"));
        }
    }
    impl RenderBufferBackend for Backend {
        fn create_renderbuffer(&mut self) -> i32 {
            self.log("create_rb");
            51
        }
        fn bind_renderbuffer(&mut self, target: i32, id: i32) {
            self.log(format!("rb:{target}:{id}"));
        }
        fn storage(&mut self, target: i32, format: i32, w: i32, h: i32) {
            self.log(format!("storage:{target}:{format}:{w}:{h}"));
        }
        fn attach_renderbuffer(&mut self, target: i32, attachment: i32, kind: i32, id: i32) {
            self.log(format!("attach:{target}:{attachment}:{kind}:{id}"));
        }
        fn delete_renderbuffer(&mut self, id: i32) {
            self.log(format!("delete_rb:{id}"));
        }
    }
    struct Target(Log);
    impl FramebufferTarget for Target {
        fn bind_to_framebuffer(&self, attachment: i32) {
            self.0.lock().unwrap().push(format!("target:{attachment}"));
        }
    }
    #[test]
    fn constructor_and_draw_preserve_attachment_unbind_and_viewport_order() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let fbo = Fbo::new(backend, context, &runtime);
        let targets: Targets = Arc::new(Mutex::new(
            vec![
                Arc::new(Target(log.clone())) as Arc<dyn FramebufferTarget>,
                Arc::new(Target(log.clone())) as Arc<dyn FramebufferTarget>,
            ]
            .into_boxed_slice(),
        ));
        let depth: Arc<dyn FramebufferTarget> = Arc::new(Target(log.clone()));
        let mut textured = TexturedFbo::new(100, 200, targets.clone(), Some(depth), fbo).unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "create_fbo",
                "fbo:36160:41",
                "check:FBO Bind",
                "fbo:36160:41",
                "check:FBO Bind",
                "target:36064",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "fbo:36160:41",
                "check:FBO Bind",
                "target:36065",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "fbo:36160:41",
                "check:FBO Bind",
                "target:33306",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind"
            ]
        );
        log.lock().unwrap().clear();
        let hook_log = log.clone();
        textured.on_unbind = Some(Arc::new(move |_| {
            hook_log.lock().unwrap().push("hook".into())
        }));
        let result = textured.draw(|| {
            log.lock().unwrap().push("draw".into());
            17
        });
        assert_eq!(result, 17);
        assert_eq!(
            *log.lock().unwrap(),
            [
                "get_viewport",
                "fbo:36160:41",
                "check:FBO Bind",
                "draw_buffers:[36064, 36065]",
                "viewport:[0, 0, 100, 200]",
                "draw",
                "check:After FB Draw",
                "fbo:36160:0",
                "check:FBO Unbind",
                "hook",
                "check:FBO onUnbind",
                "check:FB Unbind",
                "viewport:[2, 3, 800, 600]",
                "check:FB reset wiewport"
            ]
        );
        // Source subclasses can replace targets through protected setTargets; draw uses the current array.
        textured.targets = Arc::new(Mutex::new(vec![].into_boxed_slice()));
        log.lock().unwrap().clear();
        textured.draw(|| {});
        assert!(log.lock().unwrap().contains(&"draw_buffers:[]".into()));
        log.lock().unwrap().clear();
        textured.fbo.close();
        assert!(textured.fbo.freed());
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(*log.lock().unwrap(), ["delete_fbo:41"]);
    }
    #[test]
    fn attachment_limit_and_failed_callback_keep_source_bound_state() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let target: Arc<dyn FramebufferTarget> = Arc::new(Target(log.clone()));
        let targets = Arc::new(Mutex::new(vec![target; 25].into_boxed_slice()));
        let fbo = Fbo::new(backend.clone(), context.clone(), &runtime);
        assert_eq!(
            TexturedFbo::new(1, 1, targets, None, fbo).err().as_deref(),
            Some("cannot bind more than 24 update targets to FBO")
        );
        assert_eq!(
            *log.lock().unwrap(),
            ["create_fbo", "fbo:36160:41", "check:FBO Bind"]
        );
        let textured = TexturedFbo::new(
            1,
            1,
            Arc::new(Mutex::new(vec![].into_boxed_slice())),
            None,
            Fbo::new(backend, context, &runtime),
        )
        .unwrap();
        log.lock().unwrap().clear();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            textured.draw(|| panic!("source callback failure"))
        }));
        assert!(result.is_err());
        assert_eq!(
            *log.lock().unwrap(),
            [
                "get_viewport",
                "fbo:36160:41",
                "check:FBO Bind",
                "draw_buffers:[]",
                "viewport:[0, 0, 1, 1]"
            ]
        );
    }
    #[test]
    fn render_buffer_storage_attachment_and_cleanup_match_source_constants() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let buffer = RenderBuffer::new(64, 32, 36013, backend, context.clone(), &runtime);
        buffer.bind_to_framebuffer(33306);
        assert_eq!(
            *log.lock().unwrap(),
            [
                "create_rb",
                "rb:36161:51",
                "storage:36161:36013:64:32",
                "rb:36161:0",
                "attach:36160:33306:36161:51"
            ]
        );
        assert_eq!(
            [buffer.width, buffer.height, buffer.format],
            [64, 32, 36013]
        );
        log.lock().unwrap().clear();
        context.close();
        assert!(buffer.freed());
        assert!(log.lock().unwrap().is_empty());
        runtime.run_main();
        assert_eq!(*log.lock().unwrap(), ["delete_rb:51"]);
    }
}

//! ShipResource.getTexture/getTextureNow/setTexture: retained native GPU object.
use super::image_cache::SourceImage;
use crate::{
    resource::{ResourceHandle, ResourceRuntime},
    texture::{Configure, TextureBackend},
    texture_2d::SourceTexture2D,
};
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub(crate) struct SourceTextureEnvironment {
    pub backend: Arc<Mutex<dyn TextureBackend>>,
    pub context: ResourceHandle,
    pub runtime: ResourceRuntime,
}
impl SourceTextureEnvironment {
    fn create(&self, image: SourceImage, configure: Configure) -> Arc<SourceTexture2D> {
        Arc::new(SourceTexture2D::from_image(
            &image.lock().unwrap(),
            32856,
            true,
            configure,
            self.backend.clone(),
            self.context.clone(),
            &self.runtime,
        ))
    }
}
#[derive(Clone, Default)]
pub(crate) struct SourceTextureCache(Arc<Mutex<Option<Arc<SourceTexture2D>>>>);
impl std::fmt::Debug for SourceTextureCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextureCache")
            .field(
                "texture",
                &self
                    .0
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map(|texture| texture.texture.id()),
            )
            .finish()
    }
}
impl SourceTextureCache {
    fn poll_locked(
        texture: &mut Option<Arc<SourceTexture2D>>,
        environment: &SourceTextureEnvironment,
        image: impl FnOnce() -> Result<Option<SourceImage>, String>,
        accessed: impl FnOnce(),
    ) -> Result<Option<Arc<SourceTexture2D>>, String> {
        if texture.is_none() {
            if let Some(image) = image()? {
                *texture =
                    Some(environment.create(image, Arc::new(super::texture_config::configure)));
            }
        }
        // Source getTexture assigns lastUsed even when the texture was cached.
        accessed();
        Ok(texture.clone())
    }
    pub fn poll(
        &self,
        environment: &SourceTextureEnvironment,
        image: impl FnOnce() -> Result<Option<SourceImage>, String>,
        accessed: impl FnOnce(),
    ) -> Result<Option<Arc<SourceTexture2D>>, String> {
        Self::poll_locked(&mut self.0.lock().unwrap(), environment, image, accessed)
    }
    pub fn now(
        &self,
        environment: &SourceTextureEnvironment,
        poll_image: impl FnOnce() -> Result<Option<SourceImage>, String>,
        image_now: impl FnOnce() -> Result<SourceImage, String>,
        accessed: impl FnOnce(),
    ) -> Result<Arc<SourceTexture2D>, String> {
        let mut texture = self.0.lock().unwrap();
        if let Some(texture) = Self::poll_locked(&mut texture, environment, poll_image, accessed)? {
            return Ok(texture);
        }
        let replacement =
            environment.create(image_now()?, Arc::new(super::texture_now_config::configure));
        // setTexture closes the previous resource before assigning the new one.
        Self::set_locked(&mut texture, Some(replacement.clone()));
        Ok(replacement)
    }
    fn set_locked(
        texture: &mut Option<Arc<SourceTexture2D>>,
        replacement: Option<Arc<SourceTexture2D>>,
    ) {
        if let Some(previous) = texture.as_ref() {
            previous.texture.close();
        }
        *texture = replacement;
    }
    pub fn set(&self, texture: Option<Arc<SourceTexture2D>>) {
        Self::set_locked(&mut self.0.lock().unwrap(), texture);
    }
    pub fn release(&self) {
        self.set(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    fn environment(log: Arc<Mutex<Vec<String>>>) -> SourceTextureEnvironment {
        let runtime = ResourceRuntime::default();
        SourceTextureEnvironment {
            backend: crate::render_fbo::source_tests::backend(log),
            context: runtime.allocate(&[], || {}),
            runtime,
        }
    }
    fn image() -> SourceImage {
        Arc::new(Mutex::new(crate::image_data::ImageData::new(
            vec![1, 2, 3, 4],
            1,
            1,
            6408,
        )))
    }
    #[test]
    fn native_cache_preserves_getters_configuration_identity_and_deferred_close() {
        let log = Arc::new(Mutex::new(vec![]));
        let environment = environment(log.clone());
        let cache = SourceTextureCache::default();
        let touches = Cell::new(0);
        let accessed = || touches.set(touches.get() + 1);
        assert!(
            cache
                .poll(&environment, || Ok(None), accessed)
                .unwrap()
                .is_none()
        );
        let first = cache
            .now(&environment, || Ok(None), || Ok(image()), accessed)
            .unwrap();
        assert_eq!(touches.get(), 2);
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"image2:1:3553:32856:[1, 1]:6408:5121:false".into()));
        assert_eq!(
            events
                .iter()
                .filter(|s| s.starts_with("parameter:"))
                .cloned()
                .collect::<Vec<_>>(),
            [
                "parameter:3553:10240:9728",
                "parameter:3553:10241:9986",
                "parameter:3553:10242:33069",
                "parameter:3553:10243:33069"
            ]
        );
        assert_eq!(events.iter().filter(|s| s.as_str() == "mips:1").count(), 1);
        let again = cache
            .clone()
            .now(
                &environment,
                || panic!("cached texture skips image"),
                || panic!("cached texture skips blocking image"),
                accessed,
            )
            .unwrap();
        assert!(Arc::ptr_eq(&first, &again));
        assert_eq!(touches.get(), 3);
        cache.release();
        assert!(first.texture.freed());
        assert!(!log.lock().unwrap().iter().any(|s| s == "delete_tex:1"));
        environment.runtime.run_main();
        assert!(log.lock().unwrap().contains(&"delete_tex:1".into()));
        let second = cache
            .poll(&environment, || Ok(Some(image())), accessed)
            .unwrap()
            .unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        cache.set(Some(second.clone())); // Source setter closes even when passed the same object.
        assert!(second.texture.freed());
        assert!(Arc::ptr_eq(
            &second,
            &cache
                .poll(&environment, || panic!("cached"), accessed)
                .unwrap()
                .unwrap()
        ));
    }
    #[test]
    fn native_image_failure_preserves_source_touch_order_and_retry_state() {
        let log = Arc::new(Mutex::new(vec![]));
        let environment = environment(log);
        let cache = SourceTextureCache::default();
        let touches = Cell::new(0);
        let accessed = || touches.set(touches.get() + 1);
        assert!(
            cache
                .poll(&environment, || Err("image poll".into()), accessed)
                .is_err()
        );
        assert_eq!(touches.get(), 0);
        assert!(
            cache
                .now(
                    &environment,
                    || Ok(None),
                    || Err("blocking image".into()),
                    accessed
                )
                .is_err()
        );
        assert_eq!(touches.get(), 1);
        assert!(
            cache
                .now(
                    &environment,
                    || Ok(Some(image())),
                    || panic!("poll made texture"),
                    accessed
                )
                .is_ok()
        );
        assert_eq!(touches.get(), 2);
    }
}

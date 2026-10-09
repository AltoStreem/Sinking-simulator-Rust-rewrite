//! Bevy asset adapter for ShipResource.texture's retained identity and close.
use bevy::prelude::{Assets, Handle, Image};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub(crate) struct TextureCache(Arc<Mutex<Option<Handle<Image>>>>);

impl TextureCache {
    pub(crate) fn poll(
        &self,
        assets: &mut Assets<Image>,
        image: impl FnOnce() -> Result<Option<image::RgbaImage>, String>,
    ) -> Result<Option<Handle<Image>>, String> {
        let mut texture = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if texture.is_none() {
            if let Some(image) = image()? {
                *texture = Some(assets.add(crate::texture_2d::ship_texture(image)));
            }
        }
        Ok(texture.clone())
    }

    pub(crate) fn now(
        &self,
        assets: &mut Assets<Image>,
        poll_image: impl FnOnce() -> Result<Option<image::RgbaImage>, String>,
        image_now: impl FnOnce() -> Result<image::RgbaImage, String>,
        accessed: impl FnOnce(),
    ) -> Result<Handle<Image>, String> {
        let cached = self.poll(assets, poll_image)?;
        accessed();
        if let Some(texture) = cached {
            return Ok(texture);
        }
        let image = image_now()?;
        let mut texture = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let replacement = assets.add(crate::texture_2d::ship_texture(image));
        if let Some(previous) = texture.replace(replacement.clone()) {
            assets.remove(previous.id());
        }
        Ok(replacement)
    }

    pub(crate) fn release(&self) {
        // Bevy releases unused strong handles during its asset maintenance.
        *self.0.lock().unwrap_or_else(|error| error.into_inner()) = None;
    }

    pub(crate) fn close(&self, assets: &mut Assets<Image>) {
        if let Some(texture) = self
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        {
            assets.remove(texture.id());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_texture_identity_skips_image_lookup_and_closes_live_handles() {
        let cache = TextureCache::default();
        let mut assets = Assets::<Image>::default();
        assert!(cache.poll(&mut assets, || Ok(None)).unwrap().is_none());
        let texture = cache
            .now(
                &mut assets,
                || Ok(None),
                || Ok(image::RgbaImage::new(2, 1)),
                || {},
            )
            .unwrap();
        let cloned = cache.clone();
        assert_eq!(
            cloned
                .poll(&mut assets, || panic!("cached texture skips image lookup"))
                .unwrap()
                .unwrap(),
            texture
        );
        cloned.close(&mut assets);
        assert!(assets.get(texture.id()).is_none());
        let new = cache
            .now(
                &mut assets,
                || Ok(Some(image::RgbaImage::new(1, 1))),
                || panic!("poll produced texture"),
                || {},
            )
            .unwrap();
        assert_ne!(new.id(), texture.id());
    }
}

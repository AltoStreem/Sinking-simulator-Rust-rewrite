//! Recovered MusicPlayer$loadIcon$1.class texture initializer.
pub(crate) fn configure(texture: &crate::texture::Texture) {
    for (parameter, value) in [(10240, 9729), (10241, 9987), (10242, 33069), (10243, 33069)] {
        texture.set_parameter(parameter, value);
    }
}
pub(crate) fn configuration() -> crate::texture::Configure {
    std::sync::Arc::new(configure)
}

#[allow(dead_code)]
pub(crate) fn load_icon(
    name: &str,
    backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: &crate::resource::ResourceRuntime,
) -> Result<crate::texture_2d::SourceTexture2D, String> {
    let path = std::path::PathBuf::from(format!("icons/music/{name}.png"));
    let image = crate::file_reader::FileReader::game().read_image(&path, 0)?;
    Ok(crate::texture_2d::SourceTexture2D::from_image(
        &image,
        32856,
        true,
        configuration(),
        backend,
        context,
        runtime,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_configuration_preserves_parameter_and_bind_order() {
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let backend = crate::render_fbo::source_tests::backend(events.clone());
        let texture = crate::texture::Texture::new(
            3553,
            crate::texture::Texture::empty_configuration(),
            backend,
            context,
            &runtime,
        );
        events.lock().unwrap().clear();
        super::configure(&texture);
        let actual = events.lock().unwrap().clone();
        let mut expected = Vec::new();
        for (parameter, value) in [(10240, 9729), (10241, 9987), (10242, 33069), (10243, 33069)] {
            expected.push(format!("texture:3553:{}", texture.id()));
            expected.push(format!("parameter:3553:{parameter}:{value}"));
            expected.push("texture:3553:0".to_owned());
        }
        assert_eq!(actual, expected);
    }
}

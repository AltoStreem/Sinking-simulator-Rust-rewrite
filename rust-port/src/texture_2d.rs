//! Texture2D configuration used by ShipResource.texture and textureNow.
//! Source GL constants: MAG NEAREST (9728), MIN NEAREST_MIPMAP_LINEAR
//! (9986), and CLAMP_TO_BORDER (33069). Border sampling is handled in
//! ship_texture.wgsl so this does not require optional GPU sampler features.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use image::RgbaImage;

pub(crate) fn ship_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::nearest()
    })
}

/// Keep mip colors in original RGBA code space, matching the source RGBA8
/// texture rather than averaging Bevy's decoded linear-light colors.
pub(crate) fn ship_texture(rgba: RgbaImage) -> Image {
    let width = rgba.width();
    let height = rgba.height();
    assert!(width > 0 && height > 0);
    let mut texture = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.as_raw().clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    #[cfg(windows)]
    let (data,levels)=crate::native_gl_mips::generate(&rgba)
        .expect("Source RGBA8 OpenGL mip generation failed");
    // Portable approximation; native-driver pixel parity is unproven here.
    #[cfg(not(windows))]
    let (data,levels)={
        let mut levels=1;let mut data=rgba.as_raw().clone();let mut previous=rgba;
        while previous.width()>1 || previous.height()>1 {
            let next=encoded_mip_level(&previous);data.extend_from_slice(next.as_raw());
            previous=next;levels+=1;
        }
        (data,levels)
    };
    texture.data = Some(data);
    texture.texture_descriptor.mip_level_count = levels;
    texture.sampler = ship_sampler();
    texture
}

/// glGenerateMipmap's encoded bilinear center reduction, checked against the
/// source OpenGL operation on the native Radeon driver. Odd-size interpolation
/// may differ by one byte between CPU float math and driver filter precision.
pub(crate) fn encoded_mip_level(previous:&RgbaImage)->RgbaImage {
    let width=(previous.width()/2).max(1);let height=(previous.height()/2).max(1);
    RgbaImage::from_fn(width,height,|x,y| {
        let sx=(x as f32+0.5)*previous.width() as f32/width as f32-0.5;
        let sy=(y as f32+0.5)*previous.height() as f32/height as f32-0.5;
        let ix=sx.floor() as i32;let iy=sy.floor() as i32;
        let fx=sx-sx.floor();let fy=sy-sy.floor();
        let at=|x:i32,y:i32|previous.get_pixel(x.clamp(0,previous.width() as i32-1) as u32,
            y.clamp(0,previous.height() as i32-1) as u32);
        image::Rgba(std::array::from_fn(|channel| {
            let top=at(ix,iy)[channel] as f32*(1.0-fx)+at(ix+1,iy)[channel] as f32*fx;
            let bottom=at(ix,iy+1)[channel] as f32*(1.0-fx)+at(ix+1,iy+1)[channel] as f32*fx;
            (top*(1.0-fy)+bottom*fy).round().clamp(0.0,255.0) as u8
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_encoded_mips_follow_original_gl_driver_readbacks() {
        if let Ok(destination)=std::env::var("SS2_SOURCE_ICON_PROBE_INPUT") {
            let mut inputs=Vec::new();
            for folder in ["assets/icons","assets/icons/music"] {
                for entry in std::fs::read_dir(folder).unwrap() {
                    let path=entry.unwrap().path();
                    if path.extension().is_none_or(|extension|extension!="png") {continue;}
                    let rgba=image::open(&path).unwrap().to_rgba8();
                    let texture=ship_texture(rgba.clone());let mut width=rgba.width();let mut height=rgba.height();
                    let mut levels=Vec::new();let mut offset=0;
                    loop {
                        let length=(width*height*4) as usize;
                        levels.push(serde_json::json!({"Width":width,"Height":height,
                            "Rgba":&texture.data.as_ref().unwrap()[offset..offset+length]}));
                        if width==1 && height==1 {break;}
                        offset+=length;width=(width/2).max(1);height=(height/2).max(1);
                    }
                    inputs.push(serde_json::json!({"Name":path.to_string_lossy(),"Width":rgba.width(),
                        "Height":rgba.height(),"Input":rgba.as_raw(),"ExpectedLevels":levels}));
                }
            }
            std::fs::write(destination,serde_json::to_vec(&inputs).unwrap()).unwrap();
            println!("Exported {} original icon inputs and native mip expectations for GL comparison",inputs.len());
        }
        let cases:serde_json::Value=serde_json::from_str(include_str!("../tools/fixtures/source-mipmap-gl-reference.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let levels=case["Levels"].as_array().unwrap();
            let pixels=|level:&serde_json::Value|level["Rgba"].as_array().unwrap().iter()
                .map(|value|value.as_u64().unwrap() as u8).collect::<Vec<_>>();
            let base=RgbaImage::from_raw(case["Width"].as_u64().unwrap() as u32,
                case["Height"].as_u64().unwrap() as u32,pixels(&levels[0])).unwrap();
            let mip_texture=ship_texture(base.clone());
            assert_eq!(mip_texture.texture_descriptor.mip_level_count,levels.len() as u32);
            let mut offset=0;let mut max_difference=0;let mut previous_triangle=base;
            let mut old_difference=0;
            for (index,level) in levels.iter().enumerate() {
                let expected=pixels(level);
                let actual=&mip_texture.data.as_ref().unwrap()[offset..offset+expected.len()];
                for (&actual,&expected) in actual.iter().zip(&expected) {
                    max_difference=max_difference.max(actual.abs_diff(expected));
                    let tolerance=if cfg!(windows) {0}else {1};
                    assert!(actual.abs_diff(expected)<=tolerance,"GL mip {index} expected {expected}, got {actual}");
                }
                if index>0 {
                    previous_triangle=image::imageops::resize(&previous_triangle,
                        level["Width"].as_u64().unwrap() as u32,level["Height"].as_u64().unwrap() as u32,
                        image::imageops::FilterType::Triangle);
                    for (&old,&expected) in previous_triangle.as_raw().iter().zip(&expected) {
                        old_difference=old_difference.max(old.abs_diff(expected));
                    }
                }
                offset+=expected.len();
            }
            println!("Original GL mip {}x{}: max byte difference {max_difference}, prior triangle-resize difference {old_difference}",case["Width"],case["Height"]);
        }
    }
    #[test]
    fn source_texture_has_complete_mips_and_pixel_filtering() {
        let texture = ship_texture(RgbaImage::from_pixel(8, 4, image::Rgba([20, 40, 80, 255])));
        assert_eq!(texture.texture_descriptor.mip_level_count, 4);
        assert_eq!(texture.data.as_ref().unwrap().len(), (32 + 8 + 2 + 1) * 4);
        for pixel in texture.data.unwrap().chunks_exact(4) {
            assert_eq!(pixel, [20, 40, 80, 255]);
        }
        let ImageSampler::Descriptor(sampler) = texture.sampler else {
            panic!("missing source sampler")
        };
        assert_eq!(sampler.mag_filter, ImageFilterMode::Nearest);
        assert_eq!(sampler.min_filter, ImageFilterMode::Nearest);
        assert_eq!(sampler.mipmap_filter, ImageFilterMode::Linear);
    }
    #[test]
    fn source_texture_non_square_mips_reach_one_pixel() {
        let texture = ship_texture(RgbaImage::from_pixel(1, 7, image::Rgba([255, 0, 0, 255])));
        assert_eq!(texture.texture_descriptor.mip_level_count, 3);
        assert_eq!(texture.data.unwrap().len(), (7 + 3 + 1) * 4);
    }
}

/// Texture2D.java upload/resource path; existing Bevy image conversion remains above.
#[allow(dead_code)]
pub(crate) struct SourceTexture2D {
    pub texture: crate::texture::Texture,
    pub img: Option<crate::texture::PixelBuffer>,
    pub width: i32,
    pub height: i32,
    pub format: i32,
    pub internal_format: i32,
}
#[allow(dead_code)]
impl SourceTexture2D {
    pub fn new(
        img: Option<crate::texture::PixelBuffer>,
        size: [i32; 2],
        format: i32,
        internal_format: i32,
        data_format: i32,
        mipmap: bool,
        conf: crate::texture::Configure,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let texture = crate::texture::Texture::new(3553, conf, backend, context, runtime);
        texture.bind();
        {
            let pixels = img.as_ref().map(|buffer| buffer.lock().unwrap());
            texture.backend.lock().unwrap().image_2d(
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
            format,
            internal_format,
        }
    }
    pub fn from_image(
        dat: &crate::image_data::ImageData,
        internal_format: i32,
        mipmap: bool,
        conf: crate::texture::Configure,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let pixels = std::sync::Arc::new(std::sync::Mutex::new(dat.buffer.clone()));
        Self::new(
            Some(pixels),
            [dat.width, dat.height],
            dat.image_format,
            internal_format,
            dat.format,
            mipmap,
            conf,
            backend,
            context,
            runtime,
        )
    }
}
impl crate::framebuffer_target::FramebufferTarget for SourceTexture2D {
    fn texture(&self) -> Option<&crate::texture::Texture> {
        Some(&self.texture)
    }
    fn bind_to_framebuffer(&self, attachment: i32) {
        self.texture.bind_to_framebuffer(attachment);
    }
}

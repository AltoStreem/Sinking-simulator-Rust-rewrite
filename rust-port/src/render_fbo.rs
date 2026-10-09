//! RenderFBO.java GPU framebuffer-unbind mipmap regeneration adapter.
//! Multiple attachments, source stencil targets and dependency lifetime remain pending.
use crate::screen_fbo::ScreenFbo;
use bevy::prelude::*;
use bevy::{
    asset::AssetServer,
    core_pipeline::schedule::Core2d,
    render::{
        RenderApp, RenderStartup,
        camera::ExtractedCamera,
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderContext, ViewQuery},
        texture::GpuImage,
    },
};
pub(crate) struct RenderFboPlugin;
pub(crate) struct MipmapPipelinePlugin;
impl Plugin for MipmapPipelinePlugin {
    fn build(&self,app:&mut App) {
        if let Some(render)=app.get_sub_app_mut(RenderApp) {
            render.add_systems(RenderStartup,initialize_mipmaps);
        }
    }
}
impl Plugin for RenderFboPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ScreenFbo>::default());
        if !app.is_plugin_added::<MipmapPipelinePlugin>() {app.add_plugins(MipmapPipelinePlugin);}
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(
                Core2d,
                regenerate_mipmaps.after(bevy::core_pipeline::upscaling::upscaling),
            );
    }
}
#[derive(Resource)]
pub(crate) struct MipmapPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
    sampler: Sampler,
}
impl MipmapPipeline {
    pub(crate) fn is_ready(&self,cache:&PipelineCache)->bool {
        cache.get_compute_pipeline(self.pipeline).is_some()
    }
}
fn initialize_mipmaps(
    mut commands: Commands,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
    device: Res<bevy::render::renderer::RenderDevice>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "SS2 framebuffer mipmap layout",
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba8Unorm,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    );
    let pipeline = cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 framebuffer mipmap regeneration".into()),
        layout: vec![layout.clone()],
        shader: assets.load("shaders/framebuffer_mipmaps.wgsl"),
        entry_point: Some("generate_mipmap".into()),
        ..default()
    });
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });
    commands.insert_resource(MipmapPipeline {
        layout,
        pipeline,
        sampler,
    });
}
fn regenerate_mipmaps(
    view: ViewQuery<&ExtractedCamera>,
    target: Option<Res<ScreenFbo>>,
    images: Res<RenderAssets<GpuImage>>,
    pipeline: Option<Res<MipmapPipeline>>,
    cache: Res<PipelineCache>,
    mut context: RenderContext,
) {
    let Some(target) = target else {
        return;
    };
    let camera = view.into_inner();
    let Some(bevy::camera::NormalizedRenderTarget::Image(image_target)) = &camera.target else {
        return;
    };
    if image_target.handle != target.texture {
        return;
    }
    let (Some(source), Some(destination)) =
        (images.get(&target.texture), images.get(&target.filtered))
    else {
        return;
    };
    let size = source.texture_descriptor.size;
    if size != destination.texture_descriptor.size {
        return;
    }
    // Base-level copy also occurs while the mip pipeline is compiling.
    context.command_encoder().copy_texture_to_texture(
        TexelCopyTextureInfo {
            texture: &source.texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyTextureInfo {
            texture: &destination.texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        size,
    );
    let Some(pipeline) = pipeline else {
        return;
    };
    encode_texture_mips(destination,&pipeline,&cache,&mut context);
}

pub(crate) fn encode_texture_mips(destination:&GpuImage,pipeline:&MipmapPipeline,
    cache:&PipelineCache,context:&mut RenderContext)->bool {
    let Some(compute)=cache.get_compute_pipeline(pipeline.pipeline) else {return false;};
    let size=destination.texture_descriptor.size;
    let layout = cache.get_bind_group_layout(&pipeline.layout);
    for level in 1..destination.texture_descriptor.mip_level_count {
        let previous = destination.texture.create_view(&TextureViewDescriptor {
            base_mip_level: level - 1,
            mip_level_count: Some(1),
            ..default()
        });
        let next = destination.texture.create_view(&TextureViewDescriptor {
            base_mip_level: level,
            mip_level_count: Some(1),
            ..default()
        });
        let bind_group = context.render_device().create_bind_group(
            Some("SS2 framebuffer mip level"),
            &layout,
            &BindGroupEntries::sequential((&previous, &pipeline.sampler, &next)),
        );
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("SS2 generate framebuffer mip level"),
                ..default()
            });
        pass.set_pipeline(compute);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            (size.width >> level).max(1).div_ceil(8),
            (size.height >> level).max(1).div_ceil(8),
            1,
        );
    }
    true
}

/// Source RenderFBO allocation/resize path; the active Bevy mipmap plugin remains above.
#[allow(dead_code)]
pub(crate) struct SourceRenderFbo {
    pub framebuffer: crate::textured_fbo::TexturedFbo,
    pub number: i32,
    pub format: i32,
    pub stencil_depth: bool,
    pub mipmap: bool,
    pub conf: crate::texture::Configure,
    texture_backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
    renderbuffer_backend:
        std::sync::Arc<std::sync::Mutex<dyn crate::render_buffer::RenderBufferBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: crate::resource::ResourceRuntime,
}
#[allow(dead_code)]
impl SourceRenderFbo {
    pub fn new(
        size: [i32; 2],
        number: i32,
        format: i32,
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
        use crate::framebuffer_target::FramebufferTarget;
        use std::sync::{Arc, Mutex};
        // Source checks this before constructing textures, depth or framebuffer.
        if number > 24 {
            return Err("cannot bind more than 24 update targets to FBO".into());
        }
        let targets: Vec<Arc<dyn FramebufferTarget>> = (0..number)
            .map(|_| {
                Arc::new(crate::texture_2d::SourceTexture2D::new(
                    None,
                    size,
                    6408,
                    format,
                    5125,
                    mipmap,
                    conf.clone(),
                    texture_backend.clone(),
                    context.clone(),
                    runtime,
                )) as Arc<dyn FramebufferTarget>
            })
            .collect();
        let depth = if stencil_depth {
            Some(Arc::new(crate::render_buffer::RenderBuffer::new(
                size[0],
                size[1],
                36013,
                renderbuffer_backend.clone(),
                context.clone(),
                runtime,
            )) as Arc<dyn FramebufferTarget>)
        } else {
            None
        };
        let fbo = crate::fbo::Fbo::new(framebuffer_backend, context.clone(), runtime);
        let mut framebuffer = crate::textured_fbo::TexturedFbo::new(
            size[0],
            size[1],
            Arc::new(Mutex::new(targets.into_boxed_slice())),
            depth,
            fbo,
        )?;
        // During the Java superclass constructor the final mipmap field is still false.
        // Install this hook only after that construction finishes.
        framebuffer.on_unbind = Some(Arc::new(move |targets| {
            if mipmap {
                let snapshot = targets.lock().unwrap().clone();
                for target in snapshot.iter() {
                    if let Some(texture) = target.texture() {
                        texture.gen_mipmaps();
                    }
                }
            }
        }));
        Ok(Self {
            framebuffer,
            number,
            format,
            stencil_depth,
            mipmap,
            conf,
            texture_backend,
            renderbuffer_backend,
            context,
            runtime: runtime.clone(),
        })
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        use crate::framebuffer_target::FramebufferTarget;
        use std::sync::{Arc, Mutex};
        self.framebuffer.bind();
        self.framebuffer.width = width;
        self.framebuffer.height = height;
        // Source iterates the current array length, not the constructor's number field.
        let count = self.framebuffer.targets.lock().unwrap().len();
        let mut targets: Vec<Arc<dyn FramebufferTarget>> = Vec::with_capacity(count);
        for index in 0..count {
            let texture = Arc::new(crate::texture_2d::SourceTexture2D::new(
                None,
                [width, height],
                6408,
                self.format,
                5125,
                self.mipmap,
                self.conf.clone(),
                self.texture_backend.clone(),
                self.context.clone(),
                &self.runtime,
            ));
            // Each attachment unbind still regenerates the OLD array's mipmaps.
            self.framebuffer
                .bind_texture(texture.as_ref(), 36064 + index as i32);
            targets.push(texture);
        }
        self.framebuffer.targets = Arc::new(Mutex::new(targets.into_boxed_slice()));
        if self.framebuffer.depth.is_some() {
            let depth = Arc::new(crate::render_buffer::RenderBuffer::new(
                width,
                height,
                36013,
                self.renderbuffer_backend.clone(),
                self.context.clone(),
                &self.runtime,
            ));
            self.framebuffer.bind_texture(depth.as_ref(), 33306);
            self.framebuffer.depth = Some(depth);
        }
        self.framebuffer.unbind();
    }
    pub fn draw<R>(&self, draw: impl FnOnce() -> R) -> R {
        self.framebuffer.draw(draw)
    }
}

#[cfg(test)]
pub(crate) mod source_tests {
    use super::SourceRenderFbo;
    use crate::{
        fbo::FramebufferBackend,
        render_buffer::RenderBufferBackend,
        resource::ResourceRuntime,
        texture::{Texture, TextureBackend},
    };
    use std::sync::{Arc, Mutex};
    type Log = Arc<Mutex<Vec<String>>>;
    pub(crate) struct Backend {
        log: Log,
        next_texture: i32,
        bound: i32,
    }
    impl Backend {
        fn log(&self, s: impl Into<String>) {
            self.log.lock().unwrap().push(s.into());
        }
    }
    impl TextureBackend for Backend {
        fn create_texture(&mut self) -> i32 {
            self.next_texture += 1;
            self.log(format!("create_tex:{}", self.next_texture));
            self.next_texture
        }
        fn bind_texture(&mut self, target: i32, id: i32) {
            self.bound = id;
            self.log(format!("texture:{target}:{id}"));
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
                "image2:{}:{target}:{internal}:{size:?}:{format}:{kind}:{}",
                self.bound,
                data.is_none()
            ));
        }
        fn image_3d(
            &mut self,
            target: i32,
            internal: i32,
            size: [i32; 3],
            format: i32,
            kind: i32,
            _: Option<&[u8]>,
        ) {
            self.log(format!(
                "image3:{target}:{internal}:{size:?}:{format}:{kind}"
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
        }
        fn framebuffer_texture(&mut self, target: i32, attachment: i32, id: i32, level: i32) {
            self.log(format!("attach_tex:{target}:{attachment}:{id}:{level}"));
        }
        fn check_error(&mut self, label: &str) {
            self.log(format!("check:{label}"));
        }
        fn generate_mipmaps(&mut self, _: i32) {
            self.log(format!("mips:{}", self.bound));
        }
        fn delete_texture(&mut self, id: i32) {
            self.log(format!("delete_tex:{id}"));
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
            [0, 0, 800, 600]
        }
        fn draw_buffers(&mut self, a: &[i32]) {
            self.log(format!("draw_buffers:{a:?}"));
        }
        fn set_viewport(&mut self, v: [i32; 4]) {
            self.log(format!("viewport:{v:?}"));
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
        fn attach_renderbuffer(&mut self, target: i32, a: i32, kind: i32, id: i32) {
            self.log(format!("attach_rb:{target}:{a}:{kind}:{id}"));
        }
        fn delete_renderbuffer(&mut self, id: i32) {
            self.log(format!("delete_rb:{id}"));
        }
    }
    pub(crate) fn backend(log: Log) -> Arc<Mutex<Backend>> {
        Arc::new(Mutex::new(Backend {
            log,
            next_texture: 0,
            bound: 0,
        }))
    }
    fn mip_events(log: &Log) -> Vec<String> {
        log.lock()
            .unwrap()
            .iter()
            .filter(|s| s.starts_with("mips:"))
            .cloned()
            .collect()
    }
    #[test]
    fn resize_uses_old_target_mips_then_replaces_array_and_regenerates_new_targets() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = backend(log.clone());
        let mut fbo = SourceRenderFbo::new(
            [100, 200],
            2,
            32856,
            true,
            true,
            Texture::empty_configuration(),
            backend.clone(),
            backend.clone(),
            backend,
            context,
            &runtime,
        )
        .unwrap();
        assert_eq!(mip_events(&log), ["mips:1", "mips:2"]); // No superclass unbind mipmaps before final field initialization.
        let old = fbo.framebuffer.targets.clone();
        let old_depth = fbo.framebuffer.depth.clone().unwrap();
        log.lock().unwrap().clear();
        fbo.resize(300, 400);
        assert_eq!(
            mip_events(&log),
            [
                "mips:3", "mips:1", "mips:2", "mips:4", "mips:1", "mips:2", "mips:3", "mips:4",
                "mips:3", "mips:4"
            ]
        );
        assert_eq!([fbo.framebuffer.width, fbo.framebuffer.height], [300, 400]);
        assert!(!Arc::ptr_eq(&old, &fbo.framebuffer.targets));
        assert_eq!(old.lock().unwrap()[0].texture().unwrap().id(), 1);
        assert!(!Arc::ptr_eq(
            &old_depth,
            fbo.framebuffer.depth.as_ref().unwrap()
        ));
        assert!(
            log.lock()
                .unwrap()
                .contains(&"storage:36161:36013:300:400".into())
        );
        assert!(
            log.lock()
                .unwrap()
                .contains(&"image2:3:3553:32856:[300, 400]:6408:5125:true".into())
        );
        log.lock().unwrap().clear();
        fbo.draw(|| {});
        assert_eq!(mip_events(&log), ["mips:3", "mips:4"]);
    }
    #[test]
    fn resize_uses_current_array_length_optional_depth_and_source_target_limit() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let backend = backend(log.clone());
        assert!(
            SourceRenderFbo::new(
                [1, 1],
                25,
                32856,
                false,
                false,
                Texture::empty_configuration(),
                backend.clone(),
                backend.clone(),
                backend.clone(),
                context.clone(),
                &runtime
            )
            .is_err()
        );
        assert!(log.lock().unwrap().is_empty());
        let mut fbo = SourceRenderFbo::new(
            [1, 1],
            -1,
            32856,
            false,
            false,
            Texture::empty_configuration(),
            backend.clone(),
            backend.clone(),
            backend.clone(),
            context.clone(),
            &runtime,
        )
        .unwrap();
        assert_eq!(fbo.framebuffer.targets.lock().unwrap().len(), 0);
        let texture = Arc::new(crate::texture_2d::SourceTexture2D::new(
            None,
            [2, 2],
            6408,
            32856,
            5125,
            false,
            Texture::empty_configuration(),
            backend,
            context,
            &runtime,
        ));
        fbo.framebuffer.targets = Arc::new(Mutex::new(
            vec![texture as Arc<dyn crate::framebuffer_target::FramebufferTarget>]
                .into_boxed_slice(),
        ));
        log.lock().unwrap().clear();
        fbo.resize(4, 5);
        assert_eq!(fbo.number, -1);
        assert_eq!(fbo.framebuffer.targets.lock().unwrap().len(), 1);
        assert!(fbo.framebuffer.depth.is_none());
        assert!(mip_events(&log).is_empty());
        assert!(!log.lock().unwrap().iter().any(|s| s == "create_rb"));
    }
}

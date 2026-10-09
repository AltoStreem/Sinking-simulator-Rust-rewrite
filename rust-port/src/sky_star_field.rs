//! Sky.Companion.bakeStars: bake RGBA8 on construction and positive resize callbacks.
use bevy::{
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderContext, RenderGraph},
        texture::GpuImage,
    },
};
pub(crate) const SOURCE: &str = "\n    #version 150 core\n\n    in vec2 vTexCoord;\n    out vec4 FragColor;\n\n    float noise( in vec2 x )\n    {\n        float xhash = cos( x.x * 37.0 );\n        float yhash = cos( x.y * 57.0 );\n        return fract( 415.92653 * ( xhash + yhash ) );\n    }\n\n    float stars( in vec2 samplePos, float threshold )\n    {\n        float StarVal = noise( samplePos );\n        if ( StarVal >= threshold )\n            StarVal = pow( (StarVal - threshold)/(1.0 - threshold), 6.0 );\n        else\n            StarVal = 0.0;\n        return StarVal;\n    }\n\n    float field( in vec2 samplePos, float threshold )\n    {\n        float fractX = fract( samplePos.x );\n        float fractY = fract( samplePos.y );\n        vec2 floorSample = floor( samplePos );\n        float v1 = stars( floorSample, threshold );\n        float v2 = stars( floorSample + vec2( 0.0, 1.0 ), threshold );\n        float v3 = stars( floorSample + vec2( 1.0, 0.0 ), threshold );\n        float v4 = stars( floorSample + vec2( 1.0, 1.0 ), threshold );\n\n        float StarVal =   v1 * ( 1.0 - fractX ) * ( 1.0 - fractY )\n                        + v2 * ( 1.0 - fractX ) * fractY\n                        + v3 * fractX * ( 1.0 - fractY )\n                        + v4 * fractX * fractY;\n        return StarVal;\n    }\n\n    void main()\n    {\n        float StarFieldThreshhold = 0.99;\n        vec2 samplePos = gl_FragCoord.xy * 0.8;\n        FragColor = vec4(vec3(field( samplePos, StarFieldThreshhold)), 1.0);\n    }\n";

pub(crate) use crate::sky_companion::SourceStarBaker;

#[derive(Resource, Clone, ExtractResource)]
pub(crate) struct StarField {
    pub texture: Handle<Image>,
    pub size: UVec2,
    revision: u64,
}
impl StarField {
    pub fn new(size: UVec2, images: &mut Assets<Image>) -> Self {
        let size = size.max(UVec2::ONE);
        Self {
            texture: images.add(star_image(size)),
            size,
            revision: 0,
        }
    }
    pub fn resize(&mut self, size: UVec2, images: &mut Assets<Image>) {
        // Source updates the resolution uniform even for zero dimensions, but
        // only replaces the baked texture when both dimensions are positive.
        if size.x == 0 || size.y == 0 {
            return;
        }
        self.texture = images.add(star_image(size));
        self.size = size;
        self.revision = self.revision.wrapping_add(1);
    }
    /// Polling the frame's window state is only a fallback for size changes.
    /// Actual resize callbacks also rebake when dimensions are unchanged.
    pub fn sync_size(&mut self, size: UVec2, images: &mut Assets<Image>) {
        if size != self.size {
            self.resize(size, images);
        }
    }
}
fn resize_stars(
    mut events: MessageReader<bevy::window::WindowResized>,
    windows: Query<&Window>,
    mut stars: Option<ResMut<StarField>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        let Ok(window) = windows.get(event.window) else {
            continue;
        };
        let Some(stars) = stars.as_mut() else {
            continue;
        };
        let scale = window.resolution.scale_factor() as f64;
        let size = UVec2::new(
            (event.width as f64 * scale).round() as u32,
            (event.height as f64 * scale).round() as u32,
        );
        stars.resize(size, &mut images);
    }
}
fn star_image(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8Unorm, None);
    image.data = None;
    image.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING;
    image.sampler = crate::sky_stars_texture::sampler();
    image
}
pub(crate) struct StarFieldPlugin;
impl Plugin for StarFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<StarField>::default());
        app.add_systems(Update, resize_stars.before(crate::sky::animate_sky));
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(RenderStartup, initialize)
                .add_systems(
                    RenderGraph,
                    bake.before(bevy::core_pipeline::schedule::camera_driver),
                );
        }
    }
}
#[derive(Resource)]
struct StarPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}
fn initialize(mut commands: Commands, assets: Res<AssetServer>, cache: Res<PipelineCache>) {
    let layout = BindGroupLayoutDescriptor::new(
        "SS2 baked stars",
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba8Unorm,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }],
    );
    let pipeline = cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 bake star texture".into()),
        layout: vec![layout.clone()],
        shader: assets.load("shaders/sky_star_field.wgsl"),
        entry_point: Some("bake_stars".into()),
        ..default()
    });
    commands.insert_resource(StarPipeline { layout, pipeline });
}
fn bake(
    target: Option<Res<StarField>>,
    images: Res<RenderAssets<GpuImage>>,
    pipeline: Option<Res<StarPipeline>>,
    cache: Res<PipelineCache>,
    mut context: RenderContext,
    mut baked: Local<Option<(AssetId<Image>, u64)>>,
) {
    let (Some(target), Some(pipeline)) = (target, pipeline) else {
        return;
    };
    let identity = (target.texture.id(), target.revision);
    if *baked == Some(identity) {
        return;
    }
    let (Some(image), Some(compute)) = (
        images.get(&target.texture),
        cache.get_compute_pipeline(pipeline.pipeline),
    ) else {
        return;
    };
    if image.texture_descriptor.size.width != target.size.x
        || image.texture_descriptor.size.height != target.size.y
    {
        return;
    }
    let layout = cache.get_bind_group_layout(&pipeline.layout);
    let group = context.render_device().create_bind_group(
        Some("SS2 star bake target"),
        &layout,
        &BindGroupEntries::single(&image.texture_view),
    );
    {
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("SS2 bake stars on creation/resize"),
                ..default()
            });
        pass.set_pipeline(compute);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(target.size.x.div_ceil(8), target.size.y.div_ceil(8), 1);
    }
    *baked = Some(identity);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_sky_static_shader_strings_match_the_decompiled_initializer() {
        let original =
            std::fs::read_to_string("../SS2/decompiled/com/wicpar/sinkingsimulator/Sky.java")
                .unwrap();
        for source in [crate::sky::SourceSkyShaders::SKY_SOURCE, SOURCE] {
            assert!(
                original.contains(&format!("new Shader({source:?}, 35632")),
                "native shader literal differs from the original initializer"
            );
        }
    }
    #[test]
    fn source_star_bake_uses_rgba8_byte_texture_and_position_only_draw_scope() {
        use std::sync::{Arc, Mutex};
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let texture_backend = crate::render_fbo::source_tests::backend(log.clone());
        let model_backend = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(log.clone())));
        let shaders_backend = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            log.clone(),
        )));
        let vertices = crate::vertex_shaders::SourceVertexShaders::new(
            shaders_backend.clone(),
            context.clone(),
            &runtime,
        );
        let sky_shaders = crate::sky::SourceSkyShaders::new(
            &vertices,
            shaders_backend.clone(),
            shaders_backend,
            context.clone(),
            &runtime,
        )
        .unwrap();
        assert!(sky_shaders.sky.bindings.is_empty());
        assert_eq!(sky_shaders.star_field.shaders[0].bindings, ["Position"]);
        assert!(sky_shaders.star_field.shaders[1].bindings.is_empty());
        let program = sky_shaders.star_field;
        let fullscreen = crate::physics_full_screen::SourcePhysicsFullscreen::new(
            model_backend.clone(),
            model_backend.clone(),
            model_backend,
            context.clone(),
            &runtime,
        );
        let blend_log = log.clone();
        let baker = SourceStarBaker::new(
            program,
            fullscreen.model,
            texture_backend.clone(),
            texture_backend,
            context,
            &runtime,
            move |enabled| blend_log.lock().unwrap().push(format!("blend:{enabled}")),
        );
        log.lock().unwrap().clear();
        let stars = baker.make_stars([20, 10]);
        assert_eq!((stars.width, stars.height), (20, 10));
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"image2:1:3553:32856:[20, 10]:6408:5121:true".into()));
        assert!(!events.iter().any(|e| e.starts_with("mips:")));
        let draw_start = events.iter().position(|e| e == "get_viewport").unwrap();
        assert_eq!(
            &events[draw_start..],
            [
                "get_viewport",
                "fbo:36160:41",
                "check:FBO Bind",
                "draw_buffers:[36064]",
                "viewport:[0, 0, 20, 10]",
                "blend:false",
                "use:9",
                "vao:21",
                "enable:0",
                "draw:4:6:5125:0",
                "disable:0",
                "vao:0",
                "use:0",
                "check:After FB Draw",
                "fbo:36160:0",
                "check:FBO Unbind",
                "check:FBO onUnbind",
                "check:FB Unbind",
                "viewport:[0, 0, 800, 600]",
                "check:FB reset wiewport"
            ]
        );
        assert!(
            !events
                .iter()
                .any(|e| e == "enable:1" || e == "blend:true" || e.starts_with("delete_fbo:"))
        );
        log.lock().unwrap().clear();
        runtime.run_main();
        assert!(log.lock().unwrap().iter().any(|e| e == "delete_fbo:41"));
    }
    #[test]
    fn active_resize_events_rebake_equal_sizes_and_preserve_zero_size_texture() {
        use bevy::window::WindowResized;
        let mut app = App::new();
        app.add_message::<WindowResized>();
        let mut images = Assets::<Image>::default();
        let stars = StarField::new(UVec2::new(20, 10), &mut images);
        let original = stars.texture.clone();
        app.insert_resource(images)
            .insert_resource(stars)
            .add_systems(Update, resize_stars);
        let mut window = Window::default();
        window.resolution.set_scale_factor_override(Some(2.0));
        let entity = app.world_mut().spawn(window).id();
        for _ in 0..2 {
            app.world_mut().write_message(WindowResized {
                window: entity,
                width: 10.0,
                height: 5.0,
            });
        }
        app.update();
        let stars = app.world().resource::<StarField>();
        assert_eq!(stars.size, UVec2::new(20, 10));
        assert_eq!(stars.revision, 2);
        assert_ne!(stars.texture, original);
        let current = stars.texture.clone();
        app.world_mut().write_message(WindowResized {
            window: entity,
            width: 0.0,
            height: 5.0,
        });
        app.update();
        assert_eq!(app.world().resource::<StarField>().texture, current);
        app.update();
        assert_eq!(app.world().resource::<StarField>().revision, 2);
        assert_eq!(app.world().resource::<StarField>().texture, current);
    }
    #[test]
    fn star_texture_is_rgba8_nonmipmapped_and_only_replaced_on_positive_resize() {
        let mut images = Assets::default();
        let mut stars = StarField::new(UVec2::new(20, 10), &mut images);
        let first = stars.texture.clone();
        let image = images.get(&first).unwrap();
        assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm);
        assert_eq!(image.texture_descriptor.mip_level_count, 1);
        assert!(
            image
                .texture_descriptor
                .usage
                .contains(TextureUsages::STORAGE_BINDING)
        );
        stars.sync_size(UVec2::new(20, 10), &mut images);
        for size in [UVec2::new(0, 10), UVec2::new(20, 0)] {
            stars.resize(size, &mut images);
            assert_eq!(stars.texture, first);
            assert_eq!(stars.revision, 0);
        }
        stars.resize(UVec2::new(21, 10), &mut images);
        assert_ne!(stars.texture, first);
        assert_eq!(stars.revision, 1);
        let resized = stars.texture.clone();
        stars.resize(UVec2::new(21, 10), &mut images);
        assert_ne!(stars.texture, resized);
        assert_eq!(stars.revision, 2);
    }
    #[test]
    fn recovered_star_bake_shader_validates() {
        let shader =
            naga::front::wgsl::parse_str(include_str!("../assets/shaders/sky_star_field.wgsl"))
                .unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&shader)
        .unwrap();
    }
}

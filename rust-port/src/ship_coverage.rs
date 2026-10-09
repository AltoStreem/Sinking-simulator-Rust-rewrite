//! Active adaptation of Ship.render's set1/ifnot1 stencil coverage.
use crate::{fragment_shaders::ShipMaterial, ship_physics::GpuShipPhysicsAssets, ShipMesh, WorldCamera};
use bevy::{prelude::*, camera::{RenderTarget, visibility::{NoFrustumCulling, RenderLayers}},
    reflect::TypePath, render::{render_resource::AsBindGroup, storage::ShaderBuffer},
    shader::ShaderRef, sprite_render::Material2d};

const LAYER: usize = 8;
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct CoverageMaterial {
    #[uniform(0)] pub(crate) params: Vec4,
    #[storage(9, read_only)] pub(crate) masks: Handle<ShaderBuffer>,
    #[storage(10, read_only)] pub(crate) positions: Handle<ShaderBuffer>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Resource, Default)]
    struct Pixels { coverage: Vec<u8>, scene: Vec<u8>, coverage_revision: u64, scene_revision: u64 }
    #[derive(Component)]
    struct Capture(bool);
    fn capture(event: On<bevy::render::gpu_readback::ReadbackComplete>,
        labels: Query<&Capture>, mut pixels: ResMut<Pixels>) {
        let Ok(label) = labels.get(event.entity) else { return; };
        if label.0 { pixels.coverage = event.data.clone(); pixels.coverage_revision += 1; }
        else { pixels.scene = event.data.clone(); pixels.scene_revision += 1; }
    }
    fn red_inside(bytes: &[u8]) -> usize {
        // Bevy texture readbacks pad each 32-pixel RGBA row to 256 bytes.
        if bytes.len() != 32 * 256 { return 0; }
        (10..22).flat_map(|y| (10..22).map(move |x| y * 256 + x * 4))
            .filter(|&i| bytes[i] > 200 && bytes[i + 1] < 30).count()
    }
    fn covered(bytes: &[u8]) -> usize {
        covered_size(bytes, 32)
    }
    fn covered_size(bytes: &[u8], size: usize) -> usize {
        if bytes.len() != size * 256 { return 0; }
        (0..size).flat_map(|y| (0..size).map(move |x| y * 256 + x * 4))
            .filter(|&i| bytes[i] > 200).count()
    }
    fn is_covered(bytes: &[u8], x: usize, y: usize) -> bool {
        bytes.len() == 8192 && bytes[y * 256 + x * 4] > 200
    }
    fn wait_pixels(app: &mut App, description: &str, predicate: impl Fn(&Pixels) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let pixels = app.world().resource::<Pixels>();
        let mut observed = (pixels.coverage_revision, pixels.scene_revision);
        let mut matching_pairs = 0;
        loop {
            app.update();
            let pixels = app.world().resource::<Pixels>();
            if pixels.coverage_revision != observed.0 && pixels.scene_revision != observed.1 {
                observed = (pixels.coverage_revision, pixels.scene_revision);
                matching_pairs = if predicate(pixels) { matching_pairs + 1 } else { 0 };
                if matching_pairs >= 3 { return; }
            }
            assert!(std::time::Instant::now() < deadline, "GPU pixel check timed out: {description}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    #[test]
    #[ignore = "Requires an actual GPU; run explicitly with --ignored"]
    fn gpu_pixels_clip_struts_under_transparent_hull_and_reveal_broken_hull() {
        use bevy::{app::PluginsState, render::gpu_readback::Readback,
            sprite_render::Material2dPlugin, window::{ExitCondition, WindowPlugin}};
        let mut app = App::new();
        app.add_plugins(DefaultPlugins
            .set(AssetPlugin { file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")), ..default() })
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
            .add_plugins((Material2dPlugin::<ShipMaterial>::default(), Material2dPlugin::<CoverageMaterial>::default()))
            .init_resource::<Pixels>().add_systems(Startup, setup);
        while app.plugins_state() != PluginsState::Ready {
            bevy::tasks::tick_global_task_pools_on_main_thread();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        app.finish(); app.cleanup();
        let mut structure = crate::ShipStructure::empty_fallback();
        structure.texel_width = 2; structure.texel_height = 2;
        structure.texel_solid = vec![true; 4]; structure.texel_strut_masks = vec![255; 4];
        structure.texel_materials = vec![Some(crate::MaterialProperties::from(&crate::materials::Material::default())); 4];
        structure.texel_rest_positions = vec![Vec2::new(-1.0,1.0), Vec2::new(1.0,1.0), Vec2::new(-1.0,-1.0), Vec2::new(1.0,-1.0)];
        structure.texel_positions = structure.texel_rest_positions.clone();
        let physics = crate::ship_physics::make_gpu_ship_physics_assets(&structure,
            &mut app.world_mut().resource_mut::<Assets<ShaderBuffer>>());
        let masks_handle = physics.masks.clone();
        let positions_handle = physics.positions.clone();
        let mut scene = target(UVec2::splat(32));
        scene.data = None;
        let scene_handle = app.world_mut().resource_mut::<Assets<Image>>().add(scene);
        let transparent = crate::texture_2d::ship_texture(image::RgbaImage::from_pixel(2,2,image::Rgba([255,255,255,0])));
        let red = crate::texture_2d::ship_texture(image::RgbaImage::from_pixel(2,2,image::Rgba([255,0,0,255])));
        let transparent = app.world_mut().resource_mut::<Assets<Image>>().add(transparent);
        let red = app.world_mut().resource_mut::<Assets<Image>>().add(red);
        let material = app.world_mut().resource_mut::<Assets<ShipMaterial>>().add(ShipMaterial {
            params: Vec4::new(1.0,0.0,2.0,2.0), sea_color: Vec4::ZERO,
            texture: transparent.clone(), internal_lights: transparent.clone(), external_lights: transparent,
            water: physics.water.clone(), masks: physics.masks.clone(), positions: physics.positions.clone(),
            coverage_mode: Vec4::ZERO, coverage: Handle::default(),
        });
        app.insert_resource(physics);
        let window = app.world_mut().spawn(Window { resolution: (32,32).into(), ..default() }).id();
        app.world_mut().spawn((Camera2d, WorldCamera, Camera { order: -1,
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.0,1.0,0.0)), ..default() },
            RenderTarget::Image(scene_handle.clone().into()),
            Projection::Orthographic(OrthographicProjection { scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: 4.0 }, ..OrthographicProjection::default_2d() }),
            RenderLayers::layer(0)));
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(crate::ship_gpu_geometry::build_mesh(&structure));
        let hull = app.world_mut().spawn((ShipMesh(mesh.clone(), material.clone()), Mesh2d(mesh),
            MeshMaterial2d(material.clone()), Transform::default(), NoFrustumCulling)).id();
        let strut_mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(crate::ship_gpu_geometry::build_struts_mesh(&structure));
        let strut = app.world_mut().spawn((crate::ship_struts::ShipStruts {
            mesh: strut_mesh.clone(), dimensions: (2,2), masks: vec![], occupied: vec![true;4],
        }, Mesh2d(strut_mesh), MeshMaterial2d(material), Transform::from_xyz(0.0,0.0,0.001), NoFrustumCulling)).id();
        app.update();
        let coverage = app.world().resource::<HullCoverage>().texture.clone();
        app.world_mut().spawn((Readback::texture(coverage), Capture(true))).observe(capture);
        app.world_mut().spawn((Readback::texture(scene_handle.clone()), Capture(false))).observe(capture);
        let strut_material = app.world().get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0.clone();
        {
            let mut materials = app.world_mut().resource_mut::<Assets<ShipMaterial>>();
            let mut strut = materials.get_mut(&strut_material).unwrap();
            // Deliberately opaque diagnostic struts make coverage rejection
            // observable beneath a fully transparent display texture.
            strut.texture = red; strut.coverage_mode = Vec4::ZERO;
        }
        wait_pixels(&mut app, "unmasked opaque struts", |pixels| covered(&pixels.coverage) == 256 && red_inside(&pixels.scene) > 0);
        println!("GPU control: {} covered pixels, {} interior strut pixels", covered(&app.world().resource::<Pixels>().coverage), red_inside(&app.world().resource::<Pixels>().scene));
        app.world_mut().resource_mut::<Assets<ShipMaterial>>().get_mut(&strut_material).unwrap().coverage_mode = Vec4::X;
        wait_pixels(&mut app, "transparent hull suppresses struts", |pixels| covered(&pixels.coverage) == 256 && pixels.scene.len() == 8192 && red_inside(&pixels.scene) == 0);
        println!("GPU transparent hull: 256 covered pixels, zero interior strut pixels");
        let mut masks = crate::mask_struts_data::gpu_mask_storage(crate::gpu_mask_data(&structure));
        for mask in &mut masks { mask[2] = 0; }
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&masks_handle).unwrap() = ShaderBuffer::from(masks);
        wait_pixels(&mut app, "live broken hull exposes struts", |pixels| covered(&pixels.coverage) == 0 && red_inside(&pixels.scene) > 0);
        println!("GPU broken hull: zero covered pixels, {} interior strut pixels", red_inside(&app.world().resource::<Pixels>().scene));
        // Emit only source fallback kind 2: SW,NW,NE. Representative points
        // are away from raster edges, whose native GL equality is separate.
        let mut masks = crate::mask_struts_data::gpu_mask_storage(crate::gpu_mask_data(&structure));
        for mask in &mut masks { mask[2] = 0; }
        masks[0][2] = 5; masks[1][2] = 8;
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&masks_handle).unwrap() = ShaderBuffer::from(masks);
        wait_pixels(&mut app, "source fallback triangle", |pixels|
            is_covered(&pixels.coverage, 12, 12) && !is_covered(&pixels.coverage, 20, 20)
                && covered(&pixels.coverage) < 256);
        println!("GPU fallback triangle: {} covered pixels", covered(&app.world().resource::<Pixels>().coverage));
        // Update only GPU storage, leaving CPU rest positions and mesh bounds
        // unchanged. Coverage must move with the GPU hull.
        let positions: Vec<[f32;4]> = structure.texel_rest_positions.iter()
            .map(|point| [point.x + 0.5, point.y, 0.0, 0.0]).collect();
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&positions_handle).unwrap() = ShaderBuffer::from(positions);
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&masks_handle).unwrap() =
            ShaderBuffer::from(crate::mask_struts_data::gpu_mask_storage(crate::gpu_mask_data(&structure)));
        wait_pixels(&mut app, "live GPU position displacement", |pixels|
            covered(&pixels.coverage) == 256 && !is_covered(&pixels.coverage, 10, 16)
                && is_covered(&pixels.coverage, 26, 16));
        println!("GPU displacement: 256 covered pixels, moved four physical pixels to the right");
        app.add_systems(PostUpdate, sync
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate));
        app.world_mut().get_mut::<Window>(window).unwrap().resolution.set(64.0,64.0);
        *app.world_mut().resource_mut::<Assets<Image>>().get_mut(&scene_handle).unwrap() = target(UVec2::splat(64));
        wait_pixels(&mut app, "physical framebuffer resize", |pixels| covered_size(&pixels.coverage, 64) == 1024);
        println!("GPU resize: 64x64 target, 1024 covered pixels");
        assert_eq!(app.world().resource::<HullCoverage>().size, UVec2::splat(64));
        for point in &mut structure.texel_rest_positions { *point *= 0.5; }
        structure.texel_positions.clone_from(&structure.texel_rest_positions);
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(crate::ship_gpu_geometry::build_mesh(&structure));
        app.world_mut().get_mut::<ShipMesh>(hull).unwrap().0 = mesh.clone();
        app.world_mut().get_mut::<Mesh2d>(hull).unwrap().0 = mesh;
        let mut physics = app.world_mut().remove_resource::<GpuShipPhysicsAssets>().unwrap();
        crate::ship_physics::reset_gpu_ship_physics(&structure, &mut physics,
            &mut app.world_mut().resource_mut::<Assets<ShaderBuffer>>());
        app.insert_resource(physics);
        wait_pixels(&mut app, "replacement hull mesh and reset positions", |pixels| covered_size(&pixels.coverage,64) == 256);
        println!("GPU replacement: new mesh/reset position storage, 256 covered pixels");
        // Retire requests and let already mapped results drain before dropping
        // the app, avoiding shutdown callbacks to closed readback channels.
        let requests: Vec<_> = app.world_mut().query_filtered::<Entity, With<Readback>>()
            .iter(app.world()).collect();
        for entity in requests { app.world_mut().despawn(entity); }
        for _ in 0..12 { app.update(); std::thread::sleep(std::time::Duration::from_millis(10)); }
    }
    #[test]
    fn coverage_fragment_is_opaque_geometry_without_texture_alpha() {
        let shader = include_str!("../assets/shaders/ship_coverage.wgsl");
        assert!(!shader.contains("textureSample("));
        assert!(shader.contains("return vec4<f32>(1.0);"));
        let source = shader.replace("#import bevy_sprite::mesh2d_vertex_output::VertexOutput",
            "struct VertexOutput { @builtin(position) position: vec4<f32>, }");
        let module = naga::front::wgsl::parse_str(&source).unwrap();
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module).unwrap();
    }
    #[test]
    fn coverage_setup_and_sync_follow_live_hull_camera_and_resize() {
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = crate::ship_physics::make_gpu_ship_physics_assets(
            &crate::ShipStructure::empty_fallback(), &mut buffers);
        let mut materials = Assets::<ShipMaterial>::default();
        let material = materials.add(ShipMaterial {
            params: Vec4::ONE, sea_color: Vec4::ONE,
            texture: Handle::default(), internal_lights: Handle::default(), external_lights: Handle::default(),
            water: physics.water.clone(), masks: physics.masks.clone(), positions: physics.positions.clone(),
            coverage_mode: Vec4::ZERO, coverage: Handle::default(),
        });
        let mut app = App::new();
        app.insert_resource(physics).insert_resource(materials)
            .init_resource::<Assets<Image>>().init_resource::<Assets<CoverageMaterial>>()
            .add_systems(Startup, setup).add_systems(PostUpdate, sync);
        let window = app.world_mut().spawn(Window { resolution: (1280, 720).into(), ..default() }).id();
        let camera = app.world_mut().spawn((WorldCamera, Camera::default(),
            Transform::from_xyz(11.0, 12.0, 1000.0),
            Projection::Orthographic(OrthographicProjection::default_2d()))).id();
        let hull = app.world_mut().spawn((ShipMesh(Handle::default(), material.clone()),
            Mesh2d(Handle::default()), Transform::from_xyz(3.0, 4.0, 0.0))).id();
        let strut = app.world_mut().spawn((crate::ship_struts::ShipStruts {
            mesh: Handle::default(), dimensions: (1,1), masks: vec![], occupied: vec![false],
        }, MeshMaterial2d(material.clone()))).id();
        app.update();
        let world = app.world_mut();
        assert_eq!(*world.get::<Msaa>(camera).unwrap(), Msaa::Off);
        let coverage_camera = world.query_filtered::<Entity, With<CoverageCamera>>().single(world).unwrap();
        assert_eq!(world.get::<Camera>(coverage_camera).unwrap().order, -4);
        assert_eq!(world.get::<Transform>(coverage_camera).unwrap().translation, Vec3::new(11.0,12.0,1000.0));
        let coverage_hull = world.query_filtered::<Entity, With<CoverageHull>>().single(world).unwrap();
        assert_eq!(world.get::<Transform>(coverage_hull).unwrap().translation, Vec3::new(3.0,4.0,0.0));
        let texture = world.resource::<HullCoverage>().texture.clone();
        let strut_material = world.get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0.clone();
        assert_ne!(strut_material, material);
        assert_eq!(world.resource::<Assets<ShipMaterial>>().get(&strut_material).unwrap().coverage_mode, Vec4::X);
        assert_eq!(world.resource::<Assets<ShipMaterial>>().get(&material).unwrap().coverage_mode, Vec4::ZERO);
        assert_eq!(world.resource::<Assets<ShipMaterial>>().get(&strut_material).unwrap().coverage, texture);
        world.get_mut::<Window>(window).unwrap().resolution.set(1400.0, 800.0);
        world.get_mut::<Transform>(camera).unwrap().translation.x = 99.0;
        world.get_mut::<Transform>(hull).unwrap().translation.y = 77.0;
        world.resource_mut::<GpuShipPhysicsAssets>().width = 2;
        world.resource_mut::<Assets<ShipMaterial>>().get_mut(&material).unwrap().sea_color = Vec4::splat(0.25);
        app.update();
        let world = app.world_mut();
        assert_eq!(world.resource::<HullCoverage>().size, UVec2::new(1400,800));
        assert_eq!(world.resource::<HullCoverage>().texture, texture);
        assert_eq!(world.get::<Transform>(coverage_camera).unwrap().translation.x, 99.0);
        assert_eq!(world.get::<Transform>(coverage_hull).unwrap().translation.y, 77.0);
        assert_eq!(world.resource::<Assets<ShipMaterial>>().get(&strut_material).unwrap().sea_color, Vec4::splat(0.25));
        assert_eq!(world.resource::<Assets<CoverageMaterial>>().iter().next().unwrap().1.params.z, 2.0);
    }
}
impl Material2d for CoverageMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/ship_gpu_vertex.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/ship_coverage.wgsl".into() }
}
#[derive(Resource)]
pub(crate) struct HullCoverage { texture: Handle<Image>, size: UVec2 }
#[derive(Component)] pub(crate) struct CoverageCamera;
#[derive(Component)] pub(crate) struct CoverageHull;

fn target(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(size.x.max(1), size.y.max(1),
        bevy::render::render_resource::TextureFormat::Rgba8Unorm, None);
    image.data = None;
    image.copy_on_resize = false;
    image.texture_descriptor.usage |= bevy::render::render_resource::TextureUsages::COPY_SRC;
    image
}

pub(crate) fn setup(
    mut commands: Commands, windows: Query<&Window>,
    world: Query<(Entity, &Transform, &Projection), With<WorldCamera>>,
    hulls: Query<(&ShipMesh, &Mesh2d, &Transform), With<ShipMesh>>,
    mut struts: Query<&mut MeshMaterial2d<ShipMaterial>, With<crate::ship_struts::ShipStruts>>,
    physics: Res<GpuShipPhysicsAssets>, mut images: ResMut<Assets<Image>>,
    mut coverage_materials: ResMut<Assets<CoverageMaterial>>,
    mut materials: ResMut<Assets<ShipMaterial>>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((world_entity, transform, projection)) = world.single() else { return; };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    let texture = images.add(target(size));
    for (hull, _, _) in &hulls {
        if let Some(mut material) = materials.get_mut(&hull.1) { material.coverage = texture.clone(); }
    }
    for mut strut in &mut struts {
        let Some(mut material) = materials.get(&strut.0).cloned() else { continue; };
        material.coverage_mode = Vec4::X;
        strut.0 = materials.add(material);
    }
    let material = coverage_materials.add(CoverageMaterial {
        params: Vec4::new(0.0, 0.0, physics.width as f32, physics.height as f32),
        masks: physics.masks.clone(), positions: physics.positions.clone(),
    });
    for (_, mesh, transform) in &hulls {
        commands.spawn((CoverageHull, mesh.clone(), MeshMaterial2d(material.clone()),
            *transform, NoFrustumCulling, RenderLayers::layer(LAYER)));
    }
    // Both passes use one physical sample per pixel, matching source's default
    // non-multisampled framebuffer and preventing coverage edge disagreements.
    commands.entity(world_entity).insert(Msaa::Off);
    commands.spawn((Camera2d, CoverageCamera, Camera { order: -4,
        clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
        RenderTarget::Image(texture.clone().into()), *transform, projection.clone(),
        Msaa::Off, RenderLayers::layer(LAYER)));
    commands.insert_resource(HullCoverage { texture, size });
}

pub(crate) fn sync(
    windows: Query<&Window>,
    world: Query<(&Transform, &Projection), (With<WorldCamera>, Without<CoverageCamera>, Without<CoverageHull>)>,
    hulls: Query<(&ShipMesh, &Transform), (Without<CoverageCamera>, Without<CoverageHull>)>,
    mut cameras: Query<(&mut Transform, &mut Projection), (With<CoverageCamera>, Without<CoverageHull>)>,
    mut coverage_hulls: Query<(&mut Mesh2d, &mut Transform), (With<CoverageHull>, Without<CoverageCamera>)>,
    struts: Query<&MeshMaterial2d<ShipMaterial>, With<crate::ship_struts::ShipStruts>>,
    mut target_state: ResMut<HullCoverage>, mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ShipMaterial>>, mut coverage_materials: ResMut<Assets<CoverageMaterial>>,
    physics: Res<GpuShipPhysicsAssets>,
    coverage_handles: Query<&MeshMaterial2d<CoverageMaterial>, With<CoverageHull>>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((world_transform, projection)) = world.single() else { return; };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    if target_state.size != size {
        if let Some(mut image) = images.get_mut(&target_state.texture) { *image = target(size); }
        target_state.size = size;
    }
    for (mut transform, mut current_projection) in &mut cameras {
        *transform = *world_transform; *current_projection = projection.clone();
    }
    let Ok((hull, transform)) = hulls.single() else { return; };
    for (mut mesh, mut current_transform) in &mut coverage_hulls {
        mesh.0 = hull.0.clone(); *current_transform = *transform;
    }
    for handle in &coverage_handles {
        if let Some(mut material) = coverage_materials.get_mut(&handle.0) {
            material.params.z = physics.width as f32; material.params.w = physics.height as f32;
            material.positions = physics.positions.clone(); material.masks = physics.masks.clone();
        }
    }
    if let Some(source) = materials.get(&hull.1).cloned() {
        for strut in &struts {
            if let Some(mut material) = materials.get_mut(&strut.0) {
                *material = source.clone(); material.coverage_mode = Vec4::X;
            }
        }
    }
}

//! Actual GPU gate for the live Test Ship replacement handoff.
//! Diagnostic geometry isolates fresh storage bindings from physics accuracy.
use crate::*;
use bevy::{app::PluginsState, camera::{RenderTarget, visibility::{NoFrustumCulling, RenderLayers}},
    render::{gpu_readback::{Readback, ReadbackComplete}, render_resource::{TextureFormat, TextureUsages}},
    sprite_render::Material2dPlugin, window::{ExitCondition, WindowPlugin}};
use crate::ship_coverage::{CoverageCamera, CoverageHull, CoverageMaterial};

#[derive(Resource, Default)]
struct Captured {
    scene: Vec<u8>, coverage: Vec<u8>, old_positions: Vec<u8>, old_masks: Vec<u8>,
    revisions: [u64; 4],
}
#[derive(Component)]
struct Capture(usize);
fn capture(event: On<ReadbackComplete>, labels: Query<&Capture>, mut captured: ResMut<Captured>) {
    let Ok(label) = labels.get(event.entity) else { return; };
    match label.0 {
        0 => captured.scene = event.data.clone(),
        1 => captured.coverage = event.data.clone(),
        2 => captured.old_positions = event.data.clone(),
        3 => captured.old_masks = event.data.clone(),
        _ => unreachable!(),
    }
    captured.revisions[label.0] += 1;
}
fn target() -> Image {
    let mut image = Image::new_target_texture(32, 32, TextureFormat::Rgba8Unorm, None);
    image.data = None;
    image.copy_on_resize = false;
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    image
}
fn square(half: f32, x: f32) -> ShipStructure {
    let mut structure = ShipStructure::empty_fallback();
    structure.texel_width = 2; structure.texel_height = 2;
    structure.texel_solid = vec![true; 4];
    structure.texel_strut_masks = vec![255; 4];
    structure.texel_materials = vec![Some(MaterialProperties::from(&materials::Material::default())); 4];
    structure.texel_rest_positions = vec![Vec2::new(x-half,half), Vec2::new(x+half,half),
        Vec2::new(x-half,-half), Vec2::new(x+half,-half)];
    structure.texel_positions.clone_from(&structure.texel_rest_positions);
    structure
}
fn pixel(bytes: &[u8], x: usize, y: usize) -> &[u8] {
    &bytes[y*256+x*4..y*256+x*4+4]
}
fn covered(bytes: &[u8]) -> usize {
    if bytes.len() != 8192 { return 0; }
    (0..32).flat_map(|y| (0..32).map(move |x| (x,y)))
        .filter(|&(x,y)| pixel(bytes,x,y)[0] > 200).count()
}
fn blue(bytes: &[u8]) -> usize {
    if bytes.len() != 8192 { return 0; }
    (0..32).flat_map(|y| (0..32).map(move |x| (x,y)))
        .filter(|&(x,y)| { let p=pixel(bytes,x,y); p[2]>200 && p[1]<30 && p[0]<30 }).count()
}
fn blue_right(bytes: &[u8]) -> usize {
    if bytes.len() != 8192 { return 0; }
    (0..32).flat_map(|y| (20..32).map(move |x| (x,y)))
        .filter(|&(x,y)| { let p=pixel(bytes,x,y); p[2]>200 && p[1]<30 && p[0]<30 }).count()
}
fn wait_pixels(app: &mut App, description: &str, predicate: impl Fn(&Captured) -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut revisions = app.world().resource::<Captured>().revisions;
    let mut matches = 0;
    loop {
        app.update();
        let captured = app.world().resource::<Captured>();
        if captured.revisions.iter().zip(revisions).all(|(current,old)| *current != old) {
            revisions = captured.revisions;
            matches = if predicate(captured) { matches+1 } else { 0 };
            if matches == 3 { return; }
        }
        assert!(std::time::Instant::now()<deadline,
            "GPU replacement check timed out: {description}; coverage={}, blue={}, revisions={:?}",
            covered(&captured.coverage), blue(&captured.scene), captured.revisions);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
#[ignore = "Requires an actual GPU; run explicitly with --ignored"]
fn actual_gpu_live_preview_replacement_rebinds_hull_struts_and_coverage() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
        .set(AssetPlugin { file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")), ..default() })
        .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
        .add_plugins((Material2dPlugin::<ShipMaterial>::default(), Material2dPlugin::<CoverageMaterial>::default()))
        .init_resource::<Captured>()
        .init_resource::<GpuShipPhysicsSnapshot>()
        .init_resource::<tools::move_tool::MoveDragState>()
        .init_resource::<ship_upload_preview::ActivePreview>()
        .insert_resource(ShipCatalog(vec![],vec![]))
        .add_systems(Startup, ship_coverage::setup)
        .add_systems(Update, (load_selected_ship, fragment_shaders::update_ship_lighting).chain())
        .add_systems(PostUpdate, ship_coverage::sync
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate));
    while app.plugins_state() != PluginsState::Ready {
        bevy::tasks::tick_global_task_pools_on_main_thread();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    app.finish(); app.cleanup();
    let structure = square(1.0,0.0);
    let physics = ship_physics::make_gpu_ship_physics_assets(&structure,
        &mut app.world_mut().resource_mut::<Assets<ShaderBuffer>>());
    let old_physics = physics.clone();
    let mut simulation = Simulation::default(); simulation.day=1.0; simulation.show_internal_water=false;
    app.insert_resource(simulation).insert_resource(structure).insert_resource(physics);
    let scene = app.world_mut().resource_mut::<Assets<Image>>().add(target());
    let transparent = app.world_mut().resource_mut::<Assets<Image>>().add(texture_2d::ship_texture(
        image::RgbaImage::from_pixel(2,2,image::Rgba([0,0,0,0]))));
    let blue_texture = app.world_mut().resource_mut::<Assets<Image>>().add(texture_2d::ship_texture(
        image::RgbaImage::from_pixel(2,2,image::Rgba([0,0,255,255]))));
    let material = app.world_mut().resource_mut::<Assets<ShipMaterial>>().add(ShipMaterial {
        params: Vec4::new(1.0,0.0,2.0,2.0), sea_color: Vec4::ZERO,
        texture:blue_texture, internal_lights:transparent.clone(), external_lights:transparent,
        water:old_physics.water.clone(), masks:old_physics.masks.clone(), positions:old_physics.positions.clone(),
        coverage_mode:Vec4::ZERO, coverage:Handle::default(),
    });
    app.world_mut().spawn(Window { resolution:(32,32).into(), ..default() });
    app.world_mut().spawn((Camera2d, WorldCamera, Camera { order:-1,
        clear_color:ClearColorConfig::Custom(Color::linear_rgb(0.0,1.0,0.0)), ..default() },
        RenderTarget::Image(scene.clone().into()), RenderLayers::layer(0),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode:bevy::camera::ScalingMode::FixedVertical { viewport_height:4.0 },
            ..OrthographicProjection::default_2d() })));
    let mesh = { let structure=app.world().resource::<ShipStructure>(); ship_gpu_geometry::build_mesh(structure) };
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
    let hull = app.world_mut().spawn((ShipMesh(mesh.clone(),material.clone()), Mesh2d(mesh),
        MeshMaterial2d(material.clone()), MeshSyncState(vec![],vec![]), Transform::default(), NoFrustumCulling)).id();
    let mesh = { let structure=app.world().resource::<ShipStructure>(); ship_gpu_geometry::build_struts_mesh(structure) };
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
    let strut = app.world_mut().spawn((ship_struts::ShipStruts { mesh:mesh.clone(), dimensions:(2,2),
        masks:vec![],occupied:vec![true;4] }, Mesh2d(mesh), MeshMaterial2d(material.clone()),
        // Strut line thickness extends outside hull coverage. Isolate the exact
        // filled hull pixel count before testing the independent edge pass.
        Transform::from_xyz(0.0,0.0,0.001), Visibility::Hidden, NoFrustumCulling)).id();
    app.update();
    let coverage_camera = app.world_mut().query_filtered::<Entity,With<CoverageCamera>>().single(app.world()).unwrap();
    let RenderTarget::Image(coverage) = app.world().get::<RenderTarget>(coverage_camera).unwrap() else { panic!("coverage target") };
    let coverage = coverage.handle.clone();
    app.world_mut().spawn((Readback::texture(scene),Capture(0))).observe(capture);
    app.world_mut().spawn((Readback::texture(coverage),Capture(1))).observe(capture);
    app.world_mut().spawn((Readback::buffer(old_physics.positions.clone()),Capture(2))).observe(capture);
    app.world_mut().spawn((Readback::buffer(old_physics.masks.clone()),Capture(3))).observe(capture);
    wait_pixels(&mut app,"original square",|p| covered(&p.coverage)==256 && blue(&p.scene)==256);
    let old_positions=app.world().resource::<Captured>().old_positions.clone();
    let old_masks=app.world().resource::<Captured>().old_masks.clone();
    let old_strut=app.world().get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0.clone();
    let old_hull_material=app.world().resource::<Assets<ShipMaterial>>().get(&material).unwrap().clone();
    let coverage_hull=app.world_mut().query_filtered::<Entity,With<CoverageHull>>().single(app.world()).unwrap();
    let old_coverage=app.world().get::<MeshMaterial2d<CoverageMaterial>>(coverage_hull).unwrap().0.clone();
    {
        let mut preview=app.world_mut().resource_mut::<ship_upload_preview::ActivePreview>();
        // This is the live editor's pending-structure handoff, not the legacy
        // in-place reset helper used by the older coverage fixture.
        preview.pending=Some(square(0.5,0.5)); preview.revision+=1;
    }
    wait_pixels(&mut app,"fresh preview square",|p| covered(&p.coverage)==64 && blue(&p.scene)==64
        && pixel(&p.coverage,20,16)[0]>200 && pixel(&p.coverage,10,16)[0]<30
        && p.old_positions==old_positions && p.old_masks==old_masks);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    assert_ne!(physics.positions,old_physics.positions);
    assert_ne!(physics.masks,old_physics.masks); assert_ne!(physics.water,old_physics.water);
    assert_eq!(physics.generation,old_physics.generation+1);
    let new_hull=app.world().get::<MeshMaterial2d<ShipMaterial>>(hull).unwrap().0.clone();
    let new_strut=app.world().get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0.clone();
    assert_ne!(new_hull,material); assert_ne!(new_strut,old_strut);
    assert_ne!(app.world().get::<MeshMaterial2d<CoverageMaterial>>(coverage_hull).unwrap().0,old_coverage);
    for handle in [&new_hull,&new_strut] {
        let current=app.world().resource::<Assets<ShipMaterial>>().get(handle).unwrap();
        assert_eq!(current.positions,physics.positions); assert_eq!(current.masks,physics.masks);
        assert_eq!(current.water,physics.water);
    }
    let positions:Vec<[f32;4]>=square(0.5,-0.5).texel_rest_positions.iter().map(|p| [p.x,p.y,0.0,0.0]).collect();
    *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.positions).unwrap()=ShaderBuffer::from(positions);
    wait_pixels(&mut app,"replacement GPU positions move both passes",|p| covered(&p.coverage)==64 && blue(&p.scene)==64
        && pixel(&p.coverage,12,16)[0]>200 && pixel(&p.coverage,20,16)[0]<30
        && p.old_positions==old_positions && p.old_masks==old_masks);
    let mut masks=mask_struts_data::gpu_mask_storage(gpu_mask_data(app.world().resource::<ShipStructure>()));
    for mask in &mut masks { mask[2]=0; }
    *app.world_mut().get_mut::<Visibility>(strut).unwrap()=Visibility::Visible;
    *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.masks).unwrap()=ShaderBuffer::from(masks.clone());
    wait_pixels(&mut app,"replacement masks expose independently rebound struts",|p|
        covered(&p.coverage)==0 && blue(&p.scene)>0 && blue_right(&p.scene)==0
        && p.old_positions==old_positions && p.old_masks==old_masks);
    for mask in &mut masks { mask[1]=0; }
    *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.masks).unwrap()=ShaderBuffer::from(masks);
    wait_pixels(&mut app,"replacement strut masks remove visible edges",|p|
        covered(&p.coverage)==0 && blue(&p.scene)==0 && p.old_positions==old_positions && p.old_masks==old_masks);
    let old=app.world().resource::<Assets<ShipMaterial>>().get(&material).unwrap();
    assert_eq!(old.positions,old_hull_material.positions); assert_eq!(old.masks,old_hull_material.masks);
    assert_eq!(old.water,old_hull_material.water); assert_eq!(old.params,old_hull_material.params);
    println!("GPU live Test Ship: coverage/scene 256→64; new GPU positions moved both passes; new masks exposed and removed struts; retained old GPU bytes unchanged");
    let requests:Vec<_>=app.world_mut().query_filtered::<Entity,With<Readback>>().iter(app.world()).collect();
    for request in requests { app.world_mut().despawn(request); }
    for _ in 0..12 { app.update(); std::thread::sleep(std::time::Duration::from_millis(10)); }
}

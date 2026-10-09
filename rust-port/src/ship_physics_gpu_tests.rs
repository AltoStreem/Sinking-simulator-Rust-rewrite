//! Explicit actual-GPU checks of the production physics dispatcher and kernels.
use super::*;

#[derive(Resource, Default)]
pub(super) struct DispatchControl {
    pub(super) armed: bool,
    pub(super) executed: usize,
    pub(super) remaining: usize,
    pub(super) physics_steps: usize,
    pub(super) water_updates: usize,
}

fn app() -> App {
    use bevy::{app::PluginsState, window::{ExitCondition, WindowPlugin}};
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
        .set(AssetPlugin { file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")), ..default() })
        .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
        .add_plugins(GpuShipPhysicsPlugin);
    app.sub_app_mut(RenderApp).world_mut().init_resource::<DispatchControl>();
    while app.plugins_state() != PluginsState::Ready {
        bevy::tasks::tick_global_task_pools_on_main_thread();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    app.finish(); app.cleanup();
    let adapter=app.sub_app(RenderApp).world().resource::<bevy::render::renderer::RenderAdapterInfo>();
    println!("Bevy physics GPU: {}, backend={:?}, driver={} {}",adapter.name,adapter.backend,adapter.driver,adapter.driver_info);
    app
}

fn case(app: &mut App, positions: &[[f32;4]], masks: &[[u32;4]], gravity: f32, buoyancy: f32,
    displacement: [f32;4], brush: [f32;4], cut: [f32;4], water: f32, force: [f32;4]) -> usize {
    let count = positions.len();
    let mut structure = ShipStructure::empty_fallback();
    // Match the coarse occupancy backing when a fixture spans more than one
    // proxy node; production initialization builds both representations.
    structure.breached = vec![false; count];
    structure.texel_width = count; structure.texel_height = 1;
    structure.texel_solid = vec![true; count]; structure.texel_strut_masks = vec![0; count];
    structure.texel_materials = vec![Some(MaterialProperties::from(&materials::Material::default())); count];
    structure.texel_rest_positions = positions.iter().map(|p| Vec2::new(p[0],p[1])).collect();
    structure.texel_positions = structure.texel_rest_positions.clone();
    let mut buffers = app.world_mut().remove_resource::<Assets<ShaderBuffer>>().unwrap();
    let mut physics = if let Some(mut previous) = app.world_mut().remove_resource::<GpuShipPhysicsAssets>() {
        replace_gpu_ship_physics(&structure, &mut previous, &mut buffers); previous
    } else { make_gpu_ship_physics_assets(&structure, &mut buffers) };
    physics.iterations = 1; physics.water_steps = 1;
    let mut settings = gpu_settings(&structure, 1,1,1.0/60.0,0.0,400.0,
        gravity,0.0,0.0,1.0,0.0,buoyancy,40.0,1.0,0.0,0.0,0.0,1.0,0.0);
    settings[5] = displacement; settings[6] = brush; settings[7] = cut;
    for (handle, data) in [
        (&physics.positions, ShaderBuffer::from(positions.to_vec())),
        (&physics.materials, ShaderBuffer::from(vec![[10.0f32,1e10,1e10,10.0];count])),
        (&physics.masks, ShaderBuffer::from(mask_struts_data::gpu_mask_storage(masks.to_vec()))),
        (&physics.water, ShaderBuffer::from(vec![[water,0.0,0.0,0.0];count*3])),
        (&physics.forces, ShaderBuffer::from(vec![force;count])),
        (&physics.settings, ShaderBuffer::from(settings)),
    ] { *buffers.get_mut(handle).unwrap() = data; }
    app.insert_resource(buffers).insert_resource(physics).insert_resource(GpuShipPhysicsSnapshot::default());
    let mut control = app.sub_app_mut(RenderApp).world_mut().resource_mut::<DispatchControl>();
    control.armed = true;
    control.executed + 1
}

fn wait(app: &mut App, expected_dispatch: usize, label: &str,
    predicate: impl Fn(&GpuShipPhysicsSnapshot) -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut matches = 0;
    loop {
        app.update();
        let executed = app.sub_app_mut(RenderApp).world().resource::<DispatchControl>().executed;
        assert!(executed <= expected_dispatch, "verification must not dispatch an extra physics frame");
        if app.world().resource::<GpuShipPhysicsSnapshot>().positions.iter().any(|p| !p.is_finite()) {
            let positions = app.world().resource::<GpuShipPhysicsSnapshot>().positions.clone();
            close(app);
            panic!("GPU physics produced nonfinite positions for {label}: {positions:?}");
        }
        if executed == expected_dispatch && predicate(app.world().resource::<GpuShipPhysicsSnapshot>()) {
            matches += 1;
            if matches >= 6 { println!("GPU physics: {label} passed at dispatcher frame {expected_dispatch}"); return; }
        } else { matches = 0; }
        if std::time::Instant::now() >= deadline {
            let snapshot = app.world().resource::<GpuShipPhysicsSnapshot>();
            let detail = format!("positions={:?}, masks={:?}, water={:?}", snapshot.positions, snapshot.masks, snapshot.water);
            close(app);
            panic!("GPU physics timed out: {label}; {detail}");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
fn close(app: &mut App) {
    // Keep the generation tags so the synchronizer does not recreate requests;
    // remove only Readback and let pending mapped results drain.
    let entities: Vec<_> = app.world_mut().query_filtered::<Entity, With<Readback>>().iter(app.world()).collect();
    for entity in entities { app.world_mut().entity_mut(entity).remove::<Readback>(); }
    for _ in 0..12 { app.update(); std::thread::sleep(std::time::Duration::from_millis(10)); }
}
fn near(actual: f32, expected: f32) -> bool { (actual - expected).abs() < 0.00002 }

#[cfg(windows)]
#[test]
#[ignore = "Requires actual Bevy GPU plus source GLSL OpenGL driver execution"]
fn actual_gpu_physics_matches_original_glsl_driver_force_integration_and_mask_repair() {
    let mut app=app();
    for (position,gravity,buoyancy,drag,label) in [
        ([0.0,10.0,0.0,0.0],9.81,0.0,0.0,"native GLSL gravity"),
        ([0.0,10.0,0.0,0.0],9.81,1.0,0.0,"native GLSL air buoyancy"),
        ([0.0,-2.0,0.0,0.0],9.81,1.0,0.0,"native GLSL submerged buoyancy"),
        ([0.0,0.5,0.0,0.0],9.81,1.0,0.0,"native GLSL inclusive wave boundary"),
        ([0.0,10.0,2.0,-1.0],0.0,0.0,1.0,"native GLSL air drag"),
        ([0.0,-2.0,0.01,-0.02],0.0,0.0,1.0,"native GLSL water drag"),
        ([0.0,-399.99,3.0,-4.0],0.0,0.0,0.0,"native GLSL floor crossing"),
        ([0.0,-401.0,3.0,0.0],0.0,0.0,0.0,"native GLSL below-floor compression"),
        ([0.0,-400.0,0.0,0.0],0.0,0.0,0.0,"native GLSL stationary floor NaN"),
    ] {
        let positions=[position]; let masks=[[8,0,0,0]];
        let materials=[[10.0f32,1e10,1e10,10.0]]; let forces=[[0.0f32;4]];
        let frame=case(&mut app,&positions,&masks,gravity,buoyancy,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        let settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,0.0,400.0,
            gravity,0.0,0.0,1.0,drag,buoyancy,40.0,1.0,0.0,0.0,0.0,1.0,0.0);
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
        let expected=crate::native_gl_physics_reference::step(1,1,&positions,&masks,&materials,&forces,&settings).unwrap();
        assert!(expected.positions.iter().flatten().all(|lane|lane.is_finite()),"source driver nonfinite: {label}");
        wait(&mut app,frame,label,|snapshot|snapshot.positions.len()==1 && snapshot.masks==expected.masks
            && snapshot.positions[0].to_array().iter().zip(expected.positions[0]).all(|(&a,e)|(a-e).abs()<=2.0e-5+e.abs()*2.0e-6));
    }
    for (distance,strength,label) in [(1.2,1e10,"native GLSL retained tensile springs"),
        (2.0,1.0,"native GLSL broken tensile springs"),(0.5,1.0,"native GLSL broken compressive springs")] {
        let mut positions:Vec<_>=(0..9).map(|i|[(i%3) as f32,10.0+(i/3) as f32,0.0,0.0]).collect();
        positions[5][0]=positions[4][0]+distance;
        let mut masks=vec![[12,0,0,0];9];masks[4]=[8,1,1,0];masks[5]=[8,16,16,0];
        let materials=vec![[10.0f32,strength,strength,10.0];9];let forces=vec![[0.0f32;4];9];
        let frame=case(&mut app,&positions,&masks,0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        {let mut active=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();active.width=3;active.height=3;}
        let mut settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,0.0,400.0,
            0.0,1.0,0.0,1.0,0.0,0.0,40.0,1.0,0.0,0.0,0.0,1.0,0.0);
        settings[0][0]=3.0;settings[0][1]=3.0;
        {let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
            *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(materials.clone());}
        let expected=crate::native_gl_physics_reference::step(3,3,&positions,&masks,&materials,&forces,&settings).unwrap();
        wait(&mut app,frame,label,|snapshot|snapshot.positions.len()==9 && snapshot.masks==expected.masks
            && snapshot.positions.iter().zip(&expected.positions).all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|(a-e).abs()<=2.0e-5+e.abs()*2.0e-6)));
    }
    close(&mut app);crate::native_gl_mips::release_current_thread();
}

#[cfg(windows)]
#[test]
#[ignore = "Requires actual Bevy GPU and retained GLSL native water MRT passes"]
fn actual_gpu_water_matches_original_glsl_driver_flow_transport_and_mass() {
    let mut app=app();
    let positions:Vec<_>=(0..12).map(|i|[(i%4) as f32,10.0+(i/4) as f32,0.0,0.0]).collect();
    for (links,funk,flow,velocity,label) in [(true,0.0,60.0,[0.0,0.0],"native GLSL connected water"),
        (false,0.0,60.0,[0.0,0.0],"native GLSL closed water links"),
        (true,0.8,30.0,[0.4,-0.2],"native GLSL funk and advected velocity"),
        (true,0.0,0.0,[0.4,-0.2],"native GLSL zero flow and retained velocity")] {
        let mut masks=vec![[14,0,0,0];12];masks[5]=[8,if links {1}else {0},if links {1}else {0},0];masks[6]=[8,if links {16}else {0},if links {16}else {0},0];
        let mut water=vec![[0.0f32;4];12];water[5]=[2.0,0.0,velocity[0],velocity[1]];water[6]=[0.5,0.0,0.0,0.0];
        let materials=vec![[1.225f32,1e10,1e10,1.225];12];let forces=vec![[0.0f32;4];12];
        let frame=case(&mut app,&positions,&masks,9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        {let mut active=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();active.width=4;active.height=3;}
        let mut settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,0.0,400.0,
            9.81,0.0,0.0,1.0,0.0,1.0,40.0,1.0,0.0,flow,funk,1.0,0.0);
        settings[0][0]=4.0;settings[0][1]=3.0;
        {let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
            *buffers.get_mut(&physics.water).unwrap()=ShaderBuffer::from(water.repeat(3));
            *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(vec![[1.225f32,1e10,1e10,1.225];12]);}
        let expected=crate::native_gl_physics_reference::step_with_water(4,3,&positions,&masks,&materials,&forces,&settings,Some(&water)).unwrap();
        assert!(expected.water.iter().flatten().all(|lane|lane.is_finite()),"source driver nonfinite water: {label}");
        wait(&mut app,frame,label,|snapshot|snapshot.water.len()==12 && snapshot.masks==expected.masks
            && snapshot.positions.iter().zip(&expected.positions).all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|(a-e).abs()<=2e-5+e.abs()*2e-6))
            && snapshot.water.iter().zip(&expected.water).all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|(a-e).abs()<=2e-5+e.abs()*2e-6)));
        let total=expected.water.iter().map(|value|value[0]).sum::<f32>();assert!((total-2.5).abs()<2e-5);
    }
    close(&mut app);crate::native_gl_mips::release_current_thread();
}

#[cfg(windows)]
#[test]
#[ignore = "Requires persistent original OpenGL and Bevy GPU default-schedule execution"]
fn actual_gpu_repeated_default_schedule_matches_persistent_original_glsl() {
    let mut app=app(); app.init_resource::<NativeMaterialReadback>(); app.init_resource::<NativeStageReadback>();
    let mut mismatches=vec![];
    let iterations=std::env::var("SS2_REFERENCE_ITERATIONS").ok().map(|value|value.parse::<u32>().unwrap()).unwrap_or(50);
    let water_steps=std::env::var("SS2_REFERENCE_WATER_STEPS").ok().map(|value|value.parse::<u32>().unwrap()).unwrap_or(5.min(iterations));
    assert!(iterations>0 && water_steps>0 && water_steps<=iterations);
    for (links,funk,flow,velocity,base_y,base_mass,inflow,thickness,label) in [
        (true,0.0,60.0,[0.0,0.0],10.0,1.225,0.0,0.0,"original 120-frame connected water"),
        (false,0.0,60.0,[0.0,0.0],10.0,1.225,0.0,0.0,"original 120-frame closed water"),
        (true,0.8,30.0,[0.4,-0.2],10.0,1.225,0.0,0.0,"original 120-frame funk and velocity"),
        (true,0.0,0.0,[0.4,-0.2],10.0,1.225,0.0,0.0,"original 120-frame zero flow"),
        (false,0.0,60.0,[0.0,0.0],-4.0,10.0,1.0,0.915,"original 120-frame submerged motion water and mass"),
        (false,0.0,60.0,[0.4,-0.2],10.0,10.0,1.0,0.915,"original 120-frame exposed motion water and mass"),
    ] {
        // Optional diagnostic checkpoints; the normal fidelity gate remains
        // all six workloads at 120 frames with unchanged strict tolerances.
        if let Ok(filter)=std::env::var("SS2_REFERENCE_CASE") {
            if !label.contains(&filter) {continue;}
        }
        let positions:Vec<_>=(0..12).map(|i|[(i%4) as f32,base_y+(i/4) as f32,0.0,0.0]).collect();
        let mut masks=vec![[14,0,0,0];12];
        masks[5]=[8,if links {1}else {0},if links {1}else {0},0];
        masks[6]=[8,if links {16}else {0},if links {16}else {0},0];
        let materials=vec![[base_mass,1e10,1e10,base_mass];12];
        let forces=vec![[0.0;4];12];
        let mut water=vec![[0.0;4];12];water[5]=[2.0,0.25,velocity[0],velocity[1]];
        water[6]=[0.5,0.75,-velocity[0],velocity[1]];
        let frames=std::env::var("SS2_REFERENCE_FRAMES").ok()
            .map(|value|value.parse::<usize>().expect("positive diagnostic frame count"))
            .unwrap_or(120);
        assert!(frames>0);
        let expected_frame=case(&mut app,&positions,&masks,9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        let mut settings=gpu_settings(&ShipStructure::empty_fallback(),iterations,water_steps,1.0/60.0,0.0,400.0,
            9.81,0.0,0.0,1.0,0.0,1.0,40.0,1.0,inflow,flow,funk,1.0,thickness);
        settings[0][0]=4.0;settings[0][1]=3.0;
        {
            let mut active=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
            active.width=4;active.height=3;active.iterations=iterations;active.water_steps=water_steps;
            let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
            *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(materials.clone());
            *buffers.get_mut(&physics.water).unwrap()=ShaderBuffer::from(water.repeat(3));
        }
        app.world_mut().resource_mut::<NativeMaterialReadback>().0.clear();
        app.world_mut().spawn(Readback::buffer(physics.materials.clone())).observe(native_material_readback);
        let trace=std::env::var_os("SS2_REFERENCE_TRACE").is_some();
        if trace {
            app.world_mut().resource_mut::<NativeStageReadback>().values=std::array::from_fn(|_|vec![]);
            for (slot,buffer) in [&physics.forces,&physics.water_outflow_1,&physics.water_outflow_2,&physics.water_velocity_1,&physics.water_velocity_2].into_iter().enumerate() {
                app.world_mut().spawn((Readback::buffer(buffer.clone()),NativeStageSlot(slot))).observe(native_stage_readback);
            }
        }
        {
            let mut control=app.sub_app_mut(RenderApp).world_mut().resource_mut::<DispatchControl>();
            control.armed=false;control.remaining=frames;
        }
        let expected=crate::native_gl_physics_reference::run_frames(4,3,&positions,&masks,
            &materials,&forces,&settings,Some(&water),frames,water_steps as usize).unwrap();
        assert_eq!(expected.physics_steps,frames*iterations as usize);assert_eq!(expected.water_updates,frames*(iterations/(iterations/water_steps)) as usize);
        assert!(expected.positions.iter().chain(&expected.water).chain(&expected.materials)
            .flatten().all(|value|value.is_finite()),"nonfinite original driver: {label}");
        println!("{label}: original active positions {:?}, water {:?}, mass {:?}", &expected.positions[5..7], &expected.water[5..7], &expected.materials[5..7]);
        let within=|a:f32,e:f32| (a-e).abs()<=2e-4+e.abs()*2e-5;
        wait(&mut app,expected_frame+frames-1,&format!("readback completed: {label}"),|snapshot|
            snapshot.positions.len()==12 && snapshot.water.len()==12);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {
            let actual=&app.world().resource::<NativeMaterialReadback>().0;
            if actual.len()==12 && (!trace || app.world().resource::<NativeStageReadback>().values.iter().all(|values|values.len()==12)) {break;}
            assert!(std::time::Instant::now()<deadline,"material readback mismatch {label}: {actual:?}, expected {:?}",expected.materials);
            app.update();std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if trace {
            for (slot,original) in std::iter::once(&expected.forces).chain(expected.water_scratch.iter()).enumerate() {
                println!("STAGE {label} checkpoint {frames} slot {slot}: original {:?}, Bevy {:?}", &original[5..7], &app.world().resource::<NativeStageReadback>().values[slot][5..7]);
            }
        }
        let actual=app.world().resource::<GpuShipPhysicsSnapshot>();
        let max_pos=actual.positions.iter().zip(&expected.positions).flat_map(|(a,e)|a.to_array().into_iter().zip(e).map(|(a,e)|(a-e).abs())).fold(0.0f32,f32::max);
        let max_water=actual.water.iter().zip(&expected.water).flat_map(|(a,e)|a.to_array().into_iter().zip(e).map(|(a,e)|(a-e).abs())).fold(0.0f32,f32::max);
        println!("{label}: original native {} physics / {} water steps at checkpoint {frames}; max position error {max_pos}, max water error {max_water}; source active masses {:?}", expected.physics_steps, expected.water_updates, [expected.materials[5][3],expected.materials[6][3]]);
        let materials_match=app.world().resource::<NativeMaterialReadback>().0.iter().zip(&expected.materials)
            .all(|(a,e)|a.iter().zip(e).all(|(&a,&e)|within(a,e)));
        let positions_match=actual.positions.iter().zip(&expected.positions)
            .all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|within(a,e)));
        let water_matches=actual.water.iter().zip(&expected.water)
            .all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|within(a,e)));
        if !positions_match || !water_matches || !materials_match || actual.masks!=expected.masks {
            println!("MISMATCH {label}: production active positions {:?}, water {:?}, materials {:?}", &actual.positions[5..7],&actual.water[5..7],&app.world().resource::<NativeMaterialReadback>().0[5..7]);
            mismatches.push(format!("{label}: positions={positions_match},water={water_matches},materials={materials_match},masks={}",actual.masks==expected.masks));
        }
        let control=app.sub_app(RenderApp).world().resource::<DispatchControl>();
        assert_eq!(control.physics_steps,control.executed*iterations as usize);assert_eq!(control.water_updates,control.executed*(iterations/(iterations/water_steps)) as usize);
        // Drain this case's extra material observer without rearming dispatch.
        let entities:Vec<_>=app.world_mut().query_filtered::<Entity,With<Readback>>().iter(app.world()).collect();
        for entity in entities {app.world_mut().entity_mut(entity).remove::<Readback>();}
        for _ in 0..12 {app.update();std::thread::sleep(std::time::Duration::from_millis(10));}
    }
    close(&mut app);crate::native_gl_mips::release_current_thread();
    assert!(mismatches.is_empty(),"Repeated native source mismatches: {mismatches:?}");
}

#[cfg(windows)]
#[derive(Resource,Default)]
struct NativeStageReadback { values: [Vec<[f32;4]>;5] }
#[cfg(windows)]
#[derive(Component)]
struct NativeStageSlot(usize);
#[cfg(windows)]
fn native_stage_readback(event: On<ReadbackComplete>, slots: Query<&NativeStageSlot>, mut stages: ResMut<NativeStageReadback>) {
    if let Ok(slot)=slots.get(event.entity) { stages.values[slot.0]=event.to_shader_type(); }
}

#[cfg(windows)]
#[derive(Resource,Default)]
struct NativeMaterialReadback(Vec<[f32;4]>);

#[derive(Resource)]
pub(super) struct DiagnosticShader(pub(super) Handle<bevy::shader::Shader>);

#[cfg(windows)]
#[test]
#[ignore = "Requires paired original/production integration intermediates"]
fn actual_gpu_integration_intermediates_match_original_bits() {
    assert_eq!(integration_intermediate_differences(None), [0;4]);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires original OpenGL and Vulkan reflection arithmetic comparison"]
fn actual_gpu_reflection_dot_arithmetic_candidates() {
    // Diagnostic substitutions only: the production shader and exact gate above
    // remain unchanged until a candidate is supported by direct measurements.
    for expression in [
        "velocity.x * velocity.x + velocity.y * velocity.y + 1.0",
        "fma(velocity.x, velocity.x, velocity.y * velocity.y) + 1.0",
        "fma(velocity.y, velocity.y, velocity.x * velocity.x) + 1.0",
        "fma(velocity.x, velocity.x, fma(velocity.y, velocity.y, 1.0))",
    ] {
        let differences = integration_intermediate_differences(Some(expression));
        println!("Reflection denominator {expression}: {differences:?}");
        assert_eq!(&differences[..2], &[0,0], "candidate must preserve source velocity");
    }
}

#[cfg(windows)]
fn integration_intermediate_differences(denominator: Option<&str>) -> [usize;4] {
    let mut app=app();
    let source=include_str!("../assets/shaders/ship_physics.wgsl");
    let source = match denominator {
        Some(expression) => {
            let candidate = source.replace("fma(velocity.y, velocity.y, velocity.x * velocity.x) + 1.0", expression);
            assert!(source.contains("fma(velocity.y, velocity.y, velocity.x * velocity.x) + 1.0"));
            candidate
        }
        None => source.to_owned(),
    };
    let source=source.replace(
        "positions[index] = vec4<f32>(position, next_velocity);",
        "positions[index] = vec4<f32>(velocity, reflected_velocity);");
    assert_ne!(source,include_str!("../assets/shaders/ship_physics.wgsl"));
    let shader=app.world_mut().resource_mut::<Assets<bevy::shader::Shader>>()
        .add(bevy::shader::Shader::from_wgsl(source,"diagnostic-integration.wgsl"));
    app.sub_app_mut(RenderApp).world_mut().insert_resource(DiagnosticShader(shader));
    let positions:Vec<_>=(0..128).map(|i|[i as f32,10000.0,(i as f32-64.0)*0.03713,(i as f32-64.0)*0.2137]).collect();
    let masks=vec![[8,0,0,0];128];
    let materials=vec![[1.0f32,1e10,1e10,1.0];128];
    let forces:Vec<_>=(0..128).map(|i|[0.0,-7.5367827+i as f32*0.01711,0.0,0.0]).collect();
    let first=case(&mut app,&positions,&masks,0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let mut settings=gpu_settings(&ShipStructure::empty_fallback(),50,5,1.0/60.0,0.0,400.0,
        0.0,0.0,0.0,1.0,0.0,0.0,40.0,1.0,0.0,0.0,0.0,1.0,0.0);
    settings[0][0]=128.0;settings[0][1]=1.0;
    {
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings);
        *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(materials);
        *buffers.get_mut(&physics.forces).unwrap()=ShaderBuffer::from(forces);
    }
    let expected=crate::native_gl_physics_reference::integration_intermediates();
    wait(&mut app,first,"integration intermediate readback",|snapshot|snapshot.positions.len()==128);
    let actual=&app.world().resource::<GpuShipPhysicsSnapshot>().positions;let mut differences=[0usize;4];
    for i in 0..128 {for lane in 0..4 {
        if actual[i][lane].to_bits()!=expected[1][i][lane].to_bits() {
            println!("INTERMEDIATE {i}/{lane}: source {:?} Bevy {:?}",expected[1][i],actual[i]);differences[lane]+=1;
        }
    }}
    close(&mut app);crate::native_gl_mips::release_current_thread();
    println!("Paired integration velocity/reflection: {differences:?} differing lanes across 128 cells");
    differences
}

#[cfg(windows)]
#[test]
#[ignore = "Requires exact original/production GPU integration comparison"]
fn actual_gpu_integration_common_inputs_match_original_bits() {
    assert_eq!(integration_output_differences(None, false), [0;4]);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires original OpenGL and Vulkan returned integration arithmetic"]
fn actual_gpu_integration_mix_arithmetic_candidates() {
    for expression in [
        "fma(vec2<f32>(-1.0), reflected_velocity, velocity)",
        "velocity - reflected_velocity",
        "velocity - bitcast<vec2<f32>>(bitcast<vec2<u32>>(reflected_velocity) ^ vec2<u32>(bitcast<u32>(settings[8].z)))",
    ] {
        let differences = integration_output_differences(Some(expression), false);
        println!("Integration subtraction {expression}: {differences:?}");
    }
    let differences=integration_output_differences(None, true);
    println!("Live reflection and returned velocity: {differences:?}");
}

#[cfg(windows)]
fn integration_output_differences(difference: Option<&str>, expose_reflection: bool) -> [usize;4] {
    let mut app=app();
    if difference.is_some() || expose_reflection {
        let source=include_str!("../assets/shaders/ship_physics.wgsl");
        let original_difference="velocity - bitcast<vec2<f32>>(bitcast<vec2<u32>>(reflected_velocity) ^ vec2<u32>(bitcast<u32>(settings[8].z)))";
        assert!(source.contains(original_difference));
        let source=source.replace(original_difference, difference.unwrap_or(original_difference));
        let source=if expose_reflection {
            source.replace("positions[index] = vec4<f32>(position, next_velocity);",
                "positions[index] = vec4<f32>(reflected_velocity, next_velocity);")
        } else { source };
        let shader=app.world_mut().resource_mut::<Assets<bevy::shader::Shader>>()
            .add(bevy::shader::Shader::from_wgsl(source,"diagnostic-integration-mix.wgsl"));
        app.sub_app_mut(RenderApp).world_mut().insert_resource(DiagnosticShader(shader));
    }
    let positions:Vec<_>=(0..128).map(|i|[i as f32,10000.0,(i as f32-64.0)*0.03713,(i as f32-64.0)*0.2137]).collect();
    let masks=vec![[8,0,0,0];128];
    let materials=vec![[1.0f32,1e10,1e10,1.0];128];
    let forces:Vec<_>=(0..128).map(|i|[0.0,-7.5367827+i as f32*0.01711,0.0,0.0]).collect();
    let first=case(&mut app,&positions,&masks,0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let mut settings=gpu_settings(&ShipStructure::empty_fallback(),50,5,1.0/60.0,0.0,400.0,
        0.0,0.0,0.0,1.0,0.0,0.0,40.0,1.0,0.0,0.0,0.0,1.0,0.0);
    settings[0][0]=128.0;settings[0][1]=1.0;
    {
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
        *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(materials.clone());
        *buffers.get_mut(&physics.forces).unwrap()=ShaderBuffer::from(forces.clone());
    }
    let expected=crate::native_gl_physics_reference::step(128,1,&positions,&masks,&materials,&forces,&settings).unwrap();
    let original_standalone=crate::native_gl_physics_reference::integration_intermediates();
    let standalone_differences: usize = expected.positions.iter().zip(&original_standalone[0])
        .map(|(a,b)| a.iter().zip(b).filter(|(a,b)| a.to_bits()!=b.to_bits()).count()).sum();
    println!("Original complete cycle versus standalone integration: {standalone_differences} differing lanes");
    wait(&mut app,first,"integration position readback",|snapshot|snapshot.positions.len()==128);
    let actual=&app.world().resource::<GpuShipPhysicsSnapshot>().positions;let mut differences=[0usize;4];
    for i in 0..128 {for lane in 0..4 {
        let expected_lane=if expose_reflection && lane<2 { original_standalone[1][i][lane+2] } else { expected.positions[i][lane] };
        if actual[i][lane].to_bits()!=expected_lane.to_bits() {
            println!("INTEGRATE {i}/{lane}: source lane {expected_lane:?} Bevy {:?}",actual[i]);differences[lane]+=1;
        }
    }}
    close(&mut app);crate::native_gl_mips::release_current_thread();
    println!("Common-input integration: {differences:?} differing XY/velocity lanes across 128 cells");
    differences
}

#[cfg(windows)]
#[test]
#[ignore = "Requires exact original/production GPU force feedback comparison"]
fn actual_gpu_force_feedback_common_inputs_match_original_bits() {
    let mut app=app();app.init_resource::<NativeStageReadback>();
    let positions:Vec<_>=(0..128).map(|i|[i as f32,10000.0,0.0,0.0]).collect();
    let masks=vec![[8,0,0,0];128];
    let materials:Vec<_>=(0..128).map(|i|[10.0,1e10,1e10,1.9708751+i as f32*0.03137]).collect();
    let forces:Vec<_>=(0..128).map(|i|[0.0,-7.5367827+i as f32*0.01711,0.0,0.0]).collect();
    let first=case(&mut app,&positions,&masks,9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let mut settings=gpu_settings(&ShipStructure::empty_fallback(),50,5,1.0/60.0,0.0,400.0,
        9.81,0.0,0.0,1.0,0.0,1.0,40.0,1.0,0.0,0.0,0.0,1.0,0.915);
    // The diagnostic executes exactly one force/integration cycle. The packed
    // source iteration value still sets the same dt/fps as the default game.
    settings[0][0]=128.0;settings[0][1]=1.0;
    {
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
        *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(materials.clone());
        *buffers.get_mut(&physics.forces).unwrap()=ShaderBuffer::from(forces.clone());
    }
    app.world_mut().spawn((Readback::buffer(physics.forces.clone()),NativeStageSlot(0))).observe(native_stage_readback);
    let expected=crate::native_gl_physics_reference::step(128,1,&positions,&masks,&materials,&forces,&settings).unwrap();
    wait(&mut app,first,"force feedback position readback",|snapshot|snapshot.positions.len()==128);
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
    while app.world().resource::<NativeStageReadback>().values[0].len()!=128 {
        assert!(std::time::Instant::now()<deadline);app.update();std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let actual=&app.world().resource::<NativeStageReadback>().values[0];let mut differences=0;
    for i in 0..128 {for lane in 0..2 {
        if actual[i][lane].to_bits()!=expected.forces[i][lane].to_bits() {
            println!("FORCE {i}/{lane}: source {:?} Bevy {:?}",expected.forces[i],actual[i]);differences+=1;
        }
    }}
    close(&mut app);crate::native_gl_mips::release_current_thread();
    println!("Common-input force feedback: {differences} differing XY lanes across 128 cells");
    assert_eq!(differences,0);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires original and Bevy fill intermediate GPU diagnostics"]
fn actual_gpu_fill_arithmetic_intermediates() {
    for (inflow,flow) in [(1.0,60.0),(1.37,23.5)] {
        println!("Fill intermediates inflow={inflow},flow={flow}: {:?}",fill_arithmetic_intermediate_differences(inflow,flow,false,10000.0));
    }
    println!("Submerged intermediates: {:?}",fill_arithmetic_intermediate_differences(0.31,92.0,false,-2.0));
    println!("Product stages: {:?}",fill_arithmetic_intermediate_differences(1.37,23.5,true,10000.0));
}
#[cfg(windows)]
fn fill_arithmetic_intermediate_differences(inflow:f32,flow:f32,products:bool,y:f32) -> [usize;4] {
    let mut app=app();
    let source=std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/shaders/ship_physics.wgsl")).unwrap().replace("\r\n","\n");
    let source=source.replace("output_water.x += bitcast<f32>(bitcast<u32>(new_velocity) ^ bitcast<u32>(settings[8].z));", "output_water.x += bitcast<f32>(bitcast<u32>(new_velocity) ^ bitcast<u32>(settings[8].z)); output_water = vec4<f32>(sqrt(2.0 * gravity * abs(difference)), new_velocity, output_water.x, difference);")
        .replace("let input_water = water_buffer[index + count];", "let input_water = water_buffer[index + count]; water_buffer[index + count * 2u] = input_water; return;");
    let source=if products {source.replace("vec4<f32>(sqrt(2.0 * gravity * abs(difference)), new_velocity, output_water.x, difference)","vec4<f32>(speed, influx_velocity, scaled_velocity, new_velocity)")}else {source};
    let shader=app.world_mut().resource_mut::<Assets<bevy::shader::Shader>>().add(bevy::shader::Shader::from_wgsl(source,"diagnostic-fill.wgsl"));
    app.sub_app_mut(RenderApp).world_mut().insert_resource(DiagnosticShader(shader));
    let positions:Vec<_>=(0..128).map(|i|[i as f32,y,0.0,0.0]).collect();
    let masks=vec![[8,0,0,0];128];
    let water:Vec<_>=(0..128).map(|i|[0.05+i as f32*0.03137,0.0,0.0,0.0]).collect();
    let frame=case(&mut app,&positions,&masks,9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let mut settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,0.0,400.0,9.81,0.0,0.0,1.0,0.0,0.0,40.0,1.0,inflow,flow,0.0,1.0,0.915);
    settings[0][0]=128.0;settings[0][1]=1.0;
    {let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings);
        *buffers.get_mut(&physics.water).unwrap()=ShaderBuffer::from(water.repeat(3));}
    let mut expected=crate::native_gl_physics_reference::water_fill_intermediates(inflow,flow,(-9.81f32*(1.0/60.0)).mul_add(1.0/60.0,y));
    if products {for i in 0..128 {let root=expected[1][i][0];let inward=-root*inflow;let scaled=inward*flow;expected[1][i]=[root,inward,scaled,-(-(scaled*((1.0f32/60.0)/60.0))).min(water[i][0])];}}
    wait(&mut app,frame,"fill intermediate diagnostic",|snapshot|snapshot.water.len()==128);
    let actual=&app.world().resource::<GpuShipPhysicsSnapshot>().water;let mut differences=[0;4];let mut oracle_differences=0;
    for i in 0..128 {
        if !products && expected[0][i][0].to_bits()!=expected[1][i][2].to_bits() {oracle_differences+=1;}
        for lane in 0..4 {if actual[i][lane].to_bits()!=expected[1][i][lane].to_bits() {
            differences[lane]+=1;println!("FILL {i}/{lane}: original={:?}, Bevy={:?}, original amount={:?}",expected[1][i],actual[i],expected[0][i]);
        }}
    }
    let mut vel_matches=[0;4];let mut amount_matches=[0;4];
    for i in 0..128 {
        let root=expected[1][i][0];let q=water[i][0];let step=(1.0f32/60.0)/60.0;
        let variants=[((-root*inflow)*flow)*step,(-root)*(inflow*flow*step),(-root*inflow)*(flow*step),(-root*flow)*(inflow*step)];
        for (slot,v) in variants.into_iter().enumerate() {let v=-(-v).min(q);
            if v.to_bits()==expected[1][i][1].to_bits() {vel_matches[slot]+=1;}
            if (q+v).to_bits()==expected[0][i][0].to_bits() {amount_matches[slot]+=1;}
        }
    }
    println!("Original velocity CPU left/precombined/flow-times-step/inflow-times-step matches={vel_matches:?}; amount matches={amount_matches:?}");
    println!("Fill root/velocity/amount/difference mismatches={differences:?}; diagnostic source amount vs retained source differences={oracle_differences}");
    close(&mut app);crate::native_gl_mips::release_current_thread();
    assert_eq!(oracle_differences,0,"Diagnostic copy changes the source result");
    assert_eq!(differences,[0;4],"Original and Bevy fill intermediates differ");
    differences
}

#[cfg(windows)]
#[test]
#[ignore = "Requires exact common-input native and Bevy water-fill/mass comparison"]
fn actual_gpu_fill_and_mass_common_inputs_match_original_bits() {
    common_input_fill_and_mass(false,1.0,60.0,10000.0,0.0);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires isolated common-input GPU material-mass comparison"]
fn actual_gpu_mass_common_inputs_match_original_bits() {
    common_input_fill_and_mass(true,1.0,60.0,10000.0,0.0);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires exact original GPU fill with non-default inflow and flow"]
fn actual_gpu_nondefault_fill_and_mass_match_original_bits() {
    for (inflow,flow) in [(1.37,23.5),(0.31,92.0),(2.7,14.2),(0.0,60.0),(1.0,0.0)] {
        for time in [0.0,1.375,30.031] {for y in [10000.0,-2.0] {
            println!("Common fill parameters: inflow={inflow}, flow={flow},y={y},time={time}");
            common_input_fill_and_mass(false,inflow,flow,y,time);
        }}
    }
}

#[cfg(windows)]
fn common_input_fill_and_mass(sealed: bool,inflow:f32,flow:f32,y:f32,time:f32) {
    let mut app=app();app.init_resource::<NativeMaterialReadback>();
    let positions:Vec<_>=(0..128).map(|i|[i as f32,y,0.0,0.0]).collect();
    let masks=vec![[8,0,if sealed {255}else {0},0];128];
    let materials=vec![[10.0,1e10,1e10,10.0];128];
    let forces=vec![[0.0;4];128];
    let water:Vec<_>=(0..128).map(|i|[0.05+i as f32*0.03137,0.0,0.0,0.0]).collect();
    let first=case(&mut app,&positions,&masks,9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let mut settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,time,400.0,
        9.81,0.0,0.0,1.0,0.0,0.0,40.0,1.0,inflow,flow,0.0,1.0,0.915);
    settings[0][0]=128.0;settings[0][1]=1.0;
    assert_eq!(settings[3][3],inflow);assert_eq!(settings[4][0],flow);
    if sealed {settings[4][0]=0.0;}
    {
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        *buffers.get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
        *buffers.get_mut(&physics.water).unwrap()=ShaderBuffer::from(water.repeat(3));
    }
    app.world_mut().spawn(Readback::buffer(physics.materials.clone())).observe(native_material_readback);
    let expected=crate::native_gl_physics_reference::step_with_water(128,1,&positions,&masks,&materials,&forces,&settings,Some(&water)).unwrap();
    wait(&mut app,first,"common-input fill readback ready",|snapshot|snapshot.water.len()==128);
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
    while app.world().resource::<NativeMaterialReadback>().0.len()!=128 {
        assert!(std::time::Instant::now()<deadline);app.update();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut water_differences=0;let mut mass_differences=0;
    for index in 0..128 {
        let actual_water=app.world().resource::<GpuShipPhysicsSnapshot>().water[index].x;
        let actual_mass=app.world().resource::<NativeMaterialReadback>().0[index][3];
        let source_water=expected.water[index][0];let source_mass=expected.materials[index][3];
        if actual_water.to_bits()!=source_water.to_bits() {water_differences+=1;}
        if actual_mass.to_bits()!=source_mass.to_bits() {mass_differences+=1;}
        if actual_water.to_bits()!=source_water.to_bits() || actual_mass.to_bits()!=source_mass.to_bits() {
            println!("COMMON index {index} input {}: source water {source_water} ({:08x}), Bevy {actual_water} ({:08x}); source mass {source_mass} ({:08x}), Bevy {actual_mass} ({:08x})",water[index][0],source_water.to_bits(),actual_water.to_bits(),source_mass.to_bits(),actual_mass.to_bits());
        }
    }
    close(&mut app);crate::native_gl_mips::release_current_thread();
    println!("Common-input 128-cell fill/mass: water differences {water_differences}; mass differences {mass_differences}");
    assert_eq!((water_differences,mass_differences),(0,0));
}

#[cfg(windows)]
#[test]
#[ignore = "Requires production GPU and original GLSL ground stencil execution"]
fn actual_gpu_ground_water_is_excluded_by_original_dynamic_stencil() {
    let mut app = app();
    for flag in [4, 12] {
        let positions = [[0.0, -2.0, 0.4, -0.2]];
        let masks = [[flag, 0, 0, 0]];
        let water = [[2.0, 0.25, 0.4, -0.2]];
        let materials = [[10.0, 1e10, 1e10, 10.0]];
        let forces = [[0.0; 4]];
        let frame = case(&mut app, &positions, &masks, 9.81, 0.0,
            [0.0;4], [0.0;4], [0.0;4], 0.0, [0.0;4]);
        let physics = app.world().resource::<GpuShipPhysicsAssets>().clone();
        let settings = gpu_settings(&ShipStructure::empty_fallback(), 1, 1,
            1.0/60.0, 0.0, 400.0, 9.81, 0.0, 0.0, 1.0, 0.0, 0.0,
            40.0, 1.0, 1.0, 60.0, 0.8, 1.0, 0.915);
        {
            let mut buffers = app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            *buffers.get_mut(&physics.settings).unwrap() = ShaderBuffer::from(settings.clone());
            *buffers.get_mut(&physics.water).unwrap() = ShaderBuffer::from(water.repeat(3));
        }
        let expected = crate::native_gl_physics_reference::step_with_water(
            1, 1, &positions, &masks, &materials, &forces, &settings, Some(&water)).unwrap();
        assert_eq!(expected.water, water);
        assert_eq!(expected.positions, positions);
        assert_eq!(expected.materials, materials);
        wait(&mut app, frame, "ground retains water amount and velocity", |snapshot|
            snapshot.positions.len() == 1 && snapshot.water.len() == 1
            && snapshot.positions[0].to_array() == positions[0]
            && snapshot.water[0].to_array() == water[0]);
    }
    close(&mut app);
    crate::native_gl_mips::release_current_thread();
}
#[cfg(windows)]
fn native_material_readback(event:On<ReadbackComplete>,mut material:ResMut<NativeMaterialReadback>) {
    material.0=event.to_shader_type();
}
#[cfg(windows)]
#[test]
#[ignore = "Requires native original GLSL water fill/mass execution and Bevy material buffer readback"]
fn actual_gpu_water_fill_and_mass_match_original_glsl_driver() {
    let mut app=app();app.init_resource::<NativeMaterialReadback>();
    let mut readback=None;
    for (flags,y,initial_water,inflow,flow,weight,label) in [
        (10u32,-2.0,0.0,1.0,60.0,1.0,"native GLSL hull depth and mass"),
        (8,-2.0,0.2,1.0,60.0,1.0,"native GLSL exposed interior influx"),
        (10,-2.0,0.0,1.0,60.0,0.3,"native GLSL hull ignores interior weight multiplier"),
        (8,10.0,2.0,0.0,0.0,0.3,"native GLSL saturated weighted interior mass"),
    ] {
        let positions=[[0.0f32,y,0.0,0.0]];let masks=[[flags,0,0,0]];
        let materials=[[10.0f32,1e10,1e10,10.0]];let forces=[[0.0f32;4]];
        let water=[[initial_water,0.0,0.0,0.0]];
        let frame=case(&mut app,&positions,&masks,9.81,0.0,[0.0;4],[0.0;4],[0.0;4],initial_water,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        if let Some(entity)=readback.take(){app.world_mut().despawn(entity);}
        app.world_mut().resource_mut::<NativeMaterialReadback>().0.clear();
        readback=Some(app.world_mut().spawn(Readback::buffer(physics.materials.clone())).observe(native_material_readback).id());
        let settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,0.0,400.0,
            9.81,0.0,0.0,1.0,0.0,0.0,40.0,1.0,inflow,flow,0.0,weight,0.085);
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings.clone());
        let expected=crate::native_gl_physics_reference::step_with_water(1,1,&positions,&masks,&materials,&forces,&settings,Some(&water)).unwrap();
        wait(&mut app,frame,label,|snapshot|snapshot.positions.len()==1 && snapshot.water.len()==1 && snapshot.masks==expected.masks
            && snapshot.positions[0].to_array().iter().zip(expected.positions[0]).all(|(&a,e)|(a-e).abs()<=2e-5+e.abs()*2e-6)
            && snapshot.water[0].to_array().iter().zip(expected.water[0]).all(|(&a,e)|(a-e).abs()<=2e-5+e.abs()*2e-6));
        let actual=&app.world().resource::<NativeMaterialReadback>().0;
        assert_eq!(actual.len(),1,"{label}: native material readback missing");
        for (&a,e) in actual[0].iter().zip(expected.materials[0]){assert!((a-e).abs()<=2e-5+e.abs()*2e-6,"{label}: material {a} differs from original driver {e}");}
        println!("{label}: actual water={:?}, mass={}, source driver mass={}",app.world().resource::<GpuShipPhysicsSnapshot>().water[0],actual[0][3],expected.materials[0][3]);
    }
    close(&mut app);crate::native_gl_mips::release_current_thread();
}

#[test]
#[ignore = "Requires repeated actual GPU physics dispatch; run explicitly with --ignored"]
fn actual_gpu_repeated_default_step_integration_matches_source() {
    let mut app=app();
    let first=case(&mut app,&[[0.0,10.0,0.0,0.0]],&[[8,0,0,0]],
        9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
    {
        let mut active=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
        active.iterations=50;active.water_steps=5;
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        let mut settings=buffers.get_mut(&physics.settings).unwrap();
        let bytes=settings.data.as_mut().unwrap();
        for (offset,value) in [(8,50.0f32),(56,(1.0f32/60.0)/5.0),(128,(1.0f32/60.0)/50.0),(132,50.0/(1.0f32/60.0))] {
            bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes());
        }
    }
    let frames=120;
    {
        let mut control=app.sub_app_mut(RenderApp).world_mut().resource_mut::<DispatchControl>();
        control.armed=false;control.remaining=frames;
    }
    // Retained source FORCE feedback divides the previous force by mass before
    // adding gravity. Evaluate 6,000 source substeps without uploading snapshots
    // between native GPU frames. No spring, drag or buoyancy terms in this case.
    let dt=(1.0f32/60.0)/50.0;
    let mut acceleration=0.0f32;let mut velocity=0.0f32;let mut y=10.0f32;
    for _ in 0..frames*50 {
        acceleration=acceleration/10.0-9.81;
        velocity=acceleration*dt+velocity;y=velocity*dt+y;
    }
    let expected=Vec4::new(0.0,y,0.0,velocity);
    wait(&mut app,first+frames-1,"120 frames / 6000 default physics substeps",|s|
        s.positions.len()==1 && s.positions[0].to_array().iter().zip(expected.to_array())
            .all(|(&a,e)|(a-e).abs()<=2.0e-4+e.abs()*1.0e-5)
        && s.water.len()==1 && s.water[0]==Vec4::ZERO
        && s.masks==vec![[8,0,0,0]]);
    println!("Repeated source integration expected {expected:?}, GPU {:?}",app.world().resource::<GpuShipPhysicsSnapshot>().positions);
    let control=app.sub_app(RenderApp).world().resource::<DispatchControl>();
    assert_eq!((control.executed,control.physics_steps,control.water_updates),(120,6000,600));
    close(&mut app);
}
fn source_floor_step(p: [f32;4], dt: f32) -> Vec4 {
    // Independently evaluate the retained INTEGRATE GLSL literal.
    let velocity = Vec2::new(p[2],p[3]);
    let inverse_speed = 1.0/(velocity.length_squared()+1.0);
    let reflected = Vec2::new(inverse_speed,-inverse_speed)*velocity;
    let mut collision = (-400.0-p[1])/(velocity.y*dt);
    if collision.is_nan() || collision < 0.0 || collision > 1.0 { collision = 1.0; }
    let mut position = Vec2::new(p[0],p[1])+reflected.lerp(velocity,collision)*dt;
    let mut next = if collision==1.0 {velocity} else {reflected};
    if position.y < -400.0 { position.y=(position.y+400.0)*0.01-400.0; next.x*=0.5; }
    Vec4::new(position.x,position.y,next.x,next.y)
}

// Evaluate source FLOW_WATER/TRANSPORT_WATER independently on an enclosed
// fixture. Every flow cell has eight in-bounds neighbors, so this reference
// makes no claim about source out-of-range texelFetch behavior.
fn source_enclosed_water(positions: &[[f32;4]], masks: &[[u32;4]], water: &[[f32;4]],
    width: usize, gravity: f32, funk: f32, flow_dt: f32) -> Vec<Vec4> {
    let offsets = [(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1),(0,-1),(1,-1)];
    let neighbor = |index: usize, direction: usize| {
        let (dx,dy) = offsets[direction];
        ((index/width) as isize+dy) as usize*width+((index%width) as isize+dx) as usize
    };
    let active = |index: usize| masks[index][0]!=0 && masks[index][0]&7==0;
    let mut weights = vec![[0.0f32;8];water.len()];
    let mut velocities = weights.clone();
    for index in 0..water.len() {
        if !active(index) { continue; }
        let mut sum = 0.0;
        for direction in 0..8 {
            let other = neighbor(index,direction);
            let delta = Vec2::new(positions[other][0]-positions[index][0],positions[other][1]-positions[index][1]);
            let normal = if delta==Vec2::ZERO {Vec2::ZERO} else {delta.normalize()};
            let head = water[index][0]-water[other][0]+positions[index][1]-positions[other][1];
            let sign = if head>0.0 {1.0} else if head<0.0 {-1.0} else {0.0};
            let bernoulli = sign*(2.0*gravity*head.abs()).sqrt();
            // Native source GLSL contracts funk * bernoulli + projected speed.
            let velocity = (1.0+funk*(water[index][0]-1.0))
                .mul_add(bernoulli,normal.dot(Vec2::new(water[index][2],water[index][3]))).max(0.0);
            velocities[index][direction]=velocity;
            let diagonal = if direction%2==0 {1.0} else {std::f32::consts::FRAC_1_SQRT_2};
            weights[index][direction]=velocity*diagonal;
            sum+=weights[index][direction];
        }
        let factor = if sum==0.0 {0.0} else {water[index][0]*flow_dt/sum};
        for weight in &mut weights[index] { *weight*=factor; }
    }
    water.iter().enumerate().map(|(index,&input)| {
        if !active(index) { return Vec4::from_array(input); }
        let mut amount=input[0];
        let input_velocity=Vec2::new(input[2],input[3]);
        let mut momentum=input_velocity*amount;
        for direction in 0..8 {
            let other=neighbor(index,direction);
            let delta=Vec2::new(positions[other][0]-positions[index][0],positions[other][1]-positions[index][1]);
            let normal=if delta==Vec2::ZERO {Vec2::ZERO} else {delta.normalize()};
            let out=weights[index][direction];
            if masks[index][0]&2==0 && masks[index][2]&(1<<direction)!=0 && masks[other][0]&2==0 {
                let opposite=(direction+4)%8;
                let incoming=weights[other][opposite];
                amount-=out; momentum-=input_velocity*out;
                amount+=incoming; momentum-=normal*velocities[other][opposite]*incoming;
            } else { momentum-=normal*velocities[index][direction]*out; }
        }
        momentum=if amount==0.0 {Vec2::ZERO} else {momentum/amount};
        Vec4::new(amount,input[1],momentum.x,momentum.y)
    }).collect()
}

#[test]
#[ignore = "Requires actual GPU water dispatch; run explicitly with --ignored"]
fn actual_gpu_enclosed_water_transport_matches_source() {
    let mut app=app();
    let positions: Vec<_>=(0..12).map(|i| [(i%4) as f32,10.0+(i/4) as f32,0.0,0.0]).collect();
    for (links,funk,flow,velocity,label) in [
        (true,0.0,60.0,[0.0,0.0],"connected pressure and wall reflection"),
        (false,0.0,60.0,[0.0,0.0],"closed links retain quantity and reflect momentum"),
        (true,0.8,30.0,[0.4,-0.2],"funk and advected momentum"),
        (true,0.0,0.0,[0.4,-0.2],"zero flow preserves water state"),
    ] {
        let mut masks=vec![[14,0,0,0];12];
        masks[5]=[8,if links {1} else {0},if links {1} else {0},0];
        masks[6]=[8,if links {16} else {0},if links {16} else {0},0];
        let mut water=vec![[0.0f32;4];12];
        water[5]=[2.0,0.25,velocity[0],velocity[1]];
        water[6]=[0.5,0.75,-velocity[0],velocity[1]];
        let expected=source_enclosed_water(&positions,&masks,&water,4,9.81,funk,flow/60.0);
        let frame=case(&mut app,&positions,&masks,9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let mut physics=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
        physics.width=4; physics.height=3;
        let material_handle=physics.materials.clone(); let settings_handle=physics.settings.clone(); let water_handle=physics.water.clone();
        let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
        {
            let mut settings_buffer=buffers.get_mut(&settings_handle).unwrap();
            let bytes=settings_buffer.data.as_mut().unwrap();
            for (offset,value) in [(0,4.0f32),(4,3.0),(64,flow),(68,funk)] {
                bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes());
            }
        }
        *buffers.get_mut(&water_handle).unwrap()=ShaderBuffer::from(water.repeat(3));
        *buffers.get_mut(&material_handle).unwrap()=ShaderBuffer::from(vec![[1.225f32,1e10,1e10,1.225];12]);
        drop(buffers);
        wait(&mut app,frame,label,|s| s.water.len()==12 && s.water.iter().zip(&expected)
            .all(|(actual,expected)| actual.to_array().iter().zip(expected.to_array()).all(|(&a,e)|near(a,e))));
    }
    close(&mut app);
}

#[test]
#[ignore = "Requires repeated actual GPU water dispatch; run explicitly with --ignored"]
fn actual_gpu_repeated_default_step_water_matches_source() {
    let mut app=app();
    let positions:Vec<_>=(0..12).map(|i|[(i%4) as f32,10.0+(i/4) as f32,0.0,0.0]).collect();
    for (links,funk,flow,velocity,label) in [
        (true,0.0,60.0,[0.0,0.0],"default repeated connected pressure / wall reflection"),
        (false,0.0,60.0,[0.0,0.0],"repeated closed water links"),
        (true,0.8,30.0,[0.4,-0.2],"repeated funk / advected momentum"),
        (true,0.0,0.0,[0.4,-0.2],"repeated zero flow"),
    ] {
        let mut masks=vec![[14,0,0,0];12];
        masks[5]=[8,if links {1}else {0},if links {1}else {0},0];
        masks[6]=[8,if links {16}else {0},if links {16}else {0},0];
        let mut water=vec![[0.0f32;4];12];
        water[5]=[2.0,0.25,velocity[0],velocity[1]];
        water[6]=[0.5,0.75,-velocity[0],velocity[1]];
        let frames=120;
        let mut expected=water.clone();
        let mut expected_positions=positions.clone();
        let mut source_force=vec![0.0f32;12];
        let dt=(1.0f32/60.0)/50.0;
        // Source mix's rounded air endpoint is not exactly the AIR constant.
        let density=1025.0f32+(1.225f32-1025.0);
        for step in 0..frames*50 {
            for index in [5,6] {
                source_force[index]=(source_force[index]+density*9.81).mul_add(1.0/1.225,-9.81);
                expected_positions[index][3]=source_force[index].mul_add(dt,expected_positions[index][3]);
                expected_positions[index][1]=expected_positions[index][3].mul_add(dt,expected_positions[index][1]);
            }
            if step%10==9 {
                expected=source_enclosed_water(&expected_positions,&masks,&expected,4,9.81,funk,
                    flow*((1.0f32/60.0)/5.0)).into_iter().map(|water|water.to_array()).collect();
                assert!(expected.iter().flatten().all(|value|value.is_finite()),"Source reference became nonfinite: {label}");
            }
        }
        let first=case(&mut app,&positions,&masks,9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        {
            let mut active=app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
            active.width=4;active.height=3;active.iterations=50;active.water_steps=5;
            let mut buffers=app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            {
                let mut settings=buffers.get_mut(&physics.settings).unwrap();let bytes=settings.data.as_mut().unwrap();
                for (offset,value) in [(0,4.0f32),(4,3.0),(8,50.0),(56,(1.0f32/60.0)/5.0),(64,flow),(68,funk),(128,(1.0f32/60.0)/50.0),(132,50.0/(1.0f32/60.0))] {
                    bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes());
                }
            }
            *buffers.get_mut(&physics.water).unwrap()=ShaderBuffer::from(water.repeat(3));
            *buffers.get_mut(&physics.materials).unwrap()=ShaderBuffer::from(vec![[1.225f32,1e10,1e10,1.225];12]);
        }
        {
            let mut control=app.sub_app_mut(RenderApp).world_mut().resource_mut::<DispatchControl>();
            control.armed=false;control.remaining=frames;
        }
        // CPU arithmetic is useful diagnostically, but long pressure sign
        // transitions are sensitive to GPU contraction and sqrt/normalization.
        // On Windows retain all four 120-frame workloads and compare against
        // the stronger original-driver reference, not a different CPU phase.
        #[cfg(windows)]
        let (expected_positions,expected) = {
            let settings_data=app.world().resource::<Assets<ShaderBuffer>>().get(&physics.settings).unwrap().data.as_ref().unwrap();
            let settings:Vec<[f32;4]>=settings_data.chunks_exact(16).map(|row|std::array::from_fn(|i|f32::from_le_bytes(row[i*4..i*4+4].try_into().unwrap()))).collect();
            let original=crate::native_gl_physics_reference::run_frames(4,3,&positions,&masks,
                &vec![[1.225f32,1e10,1e10,1.225];12],&vec![[0.0;4];12],&settings,Some(&water),frames,5).unwrap();
            assert_eq!(original.physics_steps,frames*50);assert_eq!(original.water_updates,frames*5);
            let cpu_error=expected.iter().zip(&original.water).flat_map(|(a,e)|a.iter().zip(e).map(|(a,e)|(a-e).abs())).fold(0.0f32,f32::max);
            println!("{label}: CPU/native diagnostic maximum water error {cpu_error}; fidelity expected values come from original GLSL");
            (original.positions,original.water)
        };
        wait(&mut app,first+frames-1,label,|s| s.water.len()==12 && s.masks==masks
            && s.positions.iter().zip(&expected_positions).all(|(a,e)|a.to_array().iter().zip(e).all(|(&a,&e)|near(a,e)))
            && s.water.iter().zip(&expected).all(|(a,e)|a.to_array().iter().zip(e)
                .all(|(&a,&e)|(a-e).abs()<=2.0e-4+e.abs()*2.0e-5)));
        let snapshot=app.world().resource::<GpuShipPhysicsSnapshot>();
        let maximum=snapshot.water.iter().zip(&expected).flat_map(|(a,e)|
            a.to_array().into_iter().zip(e).map(|(a,&e)|(a-e).abs())).fold(0.0f32,f32::max);
        let quantity=snapshot.water.iter().map(|water|water.x).sum::<f32>();
        assert!((quantity-2.5).abs()<2.0e-4,"Repeated transport did not conserve water: {quantity}");
        println!("{label}: 120 frames / 600 water updates, max absolute lane error {maximum}, total water {quantity}");
        let control=app.sub_app(RenderApp).world().resource::<DispatchControl>();
        assert_eq!(control.physics_steps,control.executed*50);
        assert_eq!(control.water_updates,control.executed*5);
    }
    close(&mut app);
    #[cfg(windows)] crate::native_gl_mips::release_current_thread();
}

#[test]
#[ignore = "Requires actual GPU physics dispatch; run explicitly with --ignored"]
fn actual_gpu_physics_source_gravity_move_break_and_water_commands() {
    let mut app = app();
    let dt = 1.0f32/60.0;
    let frame = case(&mut app, &[[0.0,10.0,0.0,0.0]], &[[8,0,0,0]],
        9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    wait(&mut app, frame, "gravity/integration", |s| s.positions.len()==1
        && near(s.positions[0].y,10.0-9.81*dt*dt) && near(s.positions[0].w,-9.81*dt));
    for (y, density, label) in [(10.0,1.225,"air buoyancy"),(-2.0,1025.0,"water buoyancy"),(0.5,1.225,"inclusive air/water boundary")] {
        let frame = case(&mut app, &[[0.0,y,0.0,0.0]], &[[8,0,0,0]],
            9.81,1.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let acceleration = 9.81*(density/10.0-1.0);
        wait(&mut app, frame, label, |s| s.positions.len()==1
            && near(s.positions[0].y,y+acceleration*dt*dt) && near(s.positions[0].w,acceleration*dt));
    }
    let frame = case(&mut app, &[[0.0,10.0,0.0,0.0]], &[[8,0,0,0]],
        0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[3.0,-2.0,0.0,0.0]);
    wait(&mut app, frame, "retained force feedback", |s| s.positions.len()==1
        && near(s.positions[0].z,0.3*dt) && near(s.positions[0].w,-0.2*dt));
    let frame = case(&mut app, &[[0.0,10.0,3.0,4.0]], &[[12,0,0,0]],
        9.81,0.0,[2.0,-3.0,0.0,0.0],[0.0;4],[0.0;4],0.0,[0.0;4]);
    wait(&mut app, frame, "ground exclusion and velocity-preserving move", |s|
        s.positions == vec![Vec4::new(2.0,7.0,3.0,4.0)]);
    let frame = case(&mut app, &[[0.0,10.0,0.0,0.0],[1.0,10.0,0.0,0.0],[2.0,10.0,0.0,0.0]],
        &[[12,1,1,23],[12,17,17,23],[12,16,16,23]],0.0,0.0,[0.0;4],[0.0;4],[1.0,10.0,0.5,1.0],0.0,[0.0;4]);
    wait(&mut app, frame, "both ends of destroy links retain material", |s| s.masks.len()==3
        && s.masks.iter().all(|m| m[0]==12 && m[1]==0 && m[2]==0));
    for (mode, expected, label) in [(1.0,3.0,"flood brush"),(-1.0,0.0,"dry brush clamps water to zero")] {
        let frame = case(&mut app, &[[0.0,10.0,0.0,0.0]], &[[12,0,0,0]],
            0.0,0.0,[0.0;4],[0.0,10.0,2.0,mode],[0.0;4],1.0,[0.0;4]);
        wait(&mut app, frame, label, |s| s.water.len()==1 && near(s.water[0].x,expected));
    }
    for (position,label) in [([0.0,-399.99,3.0,-4.0],"floor crossing reflection"),
        ([0.0,-401.0,3.0,0.0],"below-floor compression and horizontal damping"),
        ([0.0,-400.0,0.0,0.0],"floor zero-displacement NaN collision branch")] {
        let frame = case(&mut app, &[position], &[[8,0,0,0]],
            0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let expected = source_floor_step(position,dt);
        wait(&mut app, frame, label, |s| s.positions.len()==1
            && s.positions[0].to_array().iter().zip(expected.to_array()).all(|(&actual,expected)| near(actual,expected)));
    }
    for (distance,label) in [(2.0,"tensile stress cuts links after applying load"),(0.5,"compressive stress cuts links after applying load")] {
        let frame = case(&mut app, &[[0.0,10.0,0.0,0.0],[distance,10.0,0.0,0.0]],
            &[[8,1,1,0],[8,16,16,0]],0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics = app.world().resource::<GpuShipPhysicsAssets>().clone();
        {
            let mut buffers = app.world_mut().resource_mut::<Assets<ShaderBuffer>>();
            // row 1.y = source rigidity. Threshold uses material strength * fps.
            buffers.get_mut(&physics.settings).unwrap().data.as_mut().unwrap()[20..24]
                .copy_from_slice(&1.0f32.to_le_bytes());
            *buffers.get_mut(&physics.materials).unwrap() = ShaderBuffer::from(vec![[10.0f32,1.0,1.0,10.0];2]);
        }
        let velocity = (distance-1.0)*750.0*10.0*(0.03/dt)/10.0*dt;
        wait(&mut app, frame, label, |s| s.masks.len()==2 && s.positions.len()==2
            && s.masks.iter().all(|m| m[0]==8 && m[1]==0 && m[2]==0)
            && near(s.positions[0].z,velocity) && near(s.positions[1].z,-velocity));
    }
    let frame = case(&mut app, &[[0.0,10.0,0.0,0.0],[1.0,10.0,0.0,0.0]],
        &[[8,0,1,0],[8,16,16,0]],0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    wait(&mut app, frame, "final reciprocal link repair uses stable mask plane", |s| s.masks.len()==2
        // Both outputs sample the pre-pass neighbor Y: the left water link
        // survives this frame even though the right structural link is cut.
        && s.masks[0][1..3]==[0,1] && s.masks[1][1..3]==[0,0]);
    close(&mut app);
}

#[cfg(windows)]
#[test]
#[ignore = "Requires original and Bevy wave intermediate GPU diagnostics"]
fn actual_gpu_wave_intermediates() {
    for time in [0.0,1.375] {
        let mut app=app();
        let source=std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/shaders/ship_physics.wgsl")).unwrap().replace("\r\n","\n");
        let source=source.replace("output_water.x += bitcast<f32>(bitcast<u32>(new_velocity) ^ bitcast<u32>(settings[8].z));", "output_water.x += bitcast<f32>(bitcast<u32>(new_velocity) ^ bitcast<u32>(settings[8].z)); let x=positions[index].x; let time=settings[3].x; let inv=3.141592/settings[2].z; output_water=vec4<f32>(wave_height(x,time),sin(x*inv+time*0.3),sin(inv*3.0*x-time),inv);")
            .replace("let input_water = water_buffer[index + count];", "let input_water = water_buffer[index + count]; water_buffer[index + count * 2u] = input_water; return;");
        assert!(source.contains("output_water=vec4<f32>(wave_height"));
        let shader=app.world_mut().resource_mut::<Assets<bevy::shader::Shader>>().add(bevy::shader::Shader::from_wgsl(source,"diagnostic-wave.wgsl"));
        app.sub_app_mut(RenderApp).world_mut().insert_resource(DiagnosticShader(shader));
        let positions:Vec<_>=(0..128).map(|i|[i as f32,-2.0,0.0,0.0]).collect();
        let frame=case(&mut app,&positions,&vec![[8,0,0,0];128],9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
        let physics=app.world().resource::<GpuShipPhysicsAssets>().clone();
        let mut settings=gpu_settings(&ShipStructure::empty_fallback(),1,1,1.0/60.0,time,400.0,9.81,0.0,0.0,1.0,0.0,0.0,40.0,1.0,0.31,92.0,0.0,1.0,0.915);
        settings[0][0]=128.0;settings[0][1]=1.0;
        *app.world_mut().resource_mut::<Assets<ShaderBuffer>>().get_mut(&physics.settings).unwrap()=ShaderBuffer::from(settings);
        let expected=crate::native_gl_physics_reference::wave_intermediates(time);
        wait(&mut app,frame,"wave intermediate readback",|snapshot|snapshot.water.len()==128);
        let mut differences=[0;4];let actual=&app.world().resource::<GpuShipPhysicsSnapshot>().water;
        for i in 0..128 {for lane in 0..4 {if expected[i][lane].to_bits()!=actual[i][lane].to_bits(){differences[lane]+=1;println!("WAVE {time} {i}/{lane}: original={:?},Bevy={:?}",expected[i],actual[i]);}}}
        println!("Wave/sine1/sine2/inverse differences t={time}: {differences:?}");
        close(&mut app);crate::native_gl_mips::release_current_thread();
    }
}


#[test]
#[ignore = "Requires native GPU allocations/readback; verifies live new-Ship storage independence"]
fn actual_gpu_new_ship_keeps_retained_old_positions_unchanged() {
    let mut app=app();
    let first=[0.0,10.0,0.0,0.0];
    let frame=case(&mut app,&[first],&[[8,0,0,0]],9.81,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    wait(&mut app,frame,"old ship gravity before replacement",|snapshot|snapshot.positions.len()==1&&snapshot.positions[0].y<first[1]);
    let old=app.world().resource::<GpuShipPhysicsAssets>().clone();
    let expected:Vec<_>=app.world().resource::<GpuShipPhysicsSnapshot>().positions[0].to_array()
        .into_iter().flat_map(f32::to_le_bytes).collect();
    #[derive(Resource,Default)] struct RetainedPositions(Option<Vec<u8>>);
    app.init_resource::<RetainedPositions>();
    app.world_mut().spawn(Readback::buffer(old.positions.clone())).observe(
        |event:On<ReadbackComplete>,mut retained:ResMut<RetainedPositions>|{retained.0=Some(event.data.clone());});
    let next=[[80.0,22.0,0.0,0.0],[90.0,25.0,0.0,0.0]];
    let frame=case(&mut app,&next,&[[8,0,0,0];2],0.0,0.0,[0.0;4],[0.0;4],[0.0;4],0.0,[0.0;4]);
    let active=app.world().resource::<GpuShipPhysicsAssets>();
    assert_ne!(active.positions,old.positions);assert_ne!(active.settings,old.settings);
    assert_eq!(active.generation,old.generation.wrapping_add(1));
    wait(&mut app,frame,"fresh two-cell ship after replacement",|snapshot|snapshot.positions.len()==2
        &&snapshot.positions.iter().map(|p|p.to_array()).eq(next));
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
    while app.world().resource::<RetainedPositions>().0.is_none() {
        app.update();std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(std::time::Instant::now()<deadline,"Retained old GPU storage did not read back");
    }
    assert_eq!(app.world().resource::<RetainedPositions>().0.as_ref().unwrap(),&expected,
        "New Ship changed the retired but externally retained original GPU positions");
    println!("Native ship storage: old one-cell GPU position bytes preserved; new two-cell handles/generation dispatched independently");
    close(&mut app);
}

//! Bevy/WGSL translation of the original SS2 `ShipPhysics` pass graph.
//!
//! The implementation owns the SS2 per-pixel state buffers, source pass
//! sequencing, readback, reset path, and simulation parameters.
use super::*;
use crate::{force_data::ForceDataHolder, water_data::WaterDataHolder};

#[derive(Resource, Clone, ExtractResource)]
pub(super) struct GpuShipPhysicsAssets {
    pub(super) positions: Handle<ShaderBuffer>,
    pub(super) materials: Handle<ShaderBuffer>,
    pub(super) masks: Handle<ShaderBuffer>,
    pub(super) forces: Handle<ShaderBuffer>,
    pub(super) settings: Handle<ShaderBuffer>,
    pub(super) water: Handle<ShaderBuffer>,
    pub(super) water_outflow_1: Handle<ShaderBuffer>,
    pub(super) water_outflow_2: Handle<ShaderBuffer>,
    pub(super) water_velocity_1: Handle<ShaderBuffer>,
    pub(super) water_velocity_2: Handle<ShaderBuffer>,
    pub(super) water_brush: Option<[f32; 4]>,
    pub(super) break_brush: Option<[f32; 4]>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) iterations: u32,
    pub(super) water_steps: u32,
    pub(super) generation: u64,
}

#[derive(Component)]
pub(super) struct PhysicsReadbackGeneration(pub u64);

pub(super) fn spawn_readbacks(commands: &mut Commands, physics: &GpuShipPhysicsAssets) {
    commands.spawn((Readback::buffer(physics.positions.clone()), PhysicsReadbackGeneration(physics.generation)))
        .observe(capture_gpu_ship_physics_readback);
    commands.spawn((Readback::buffer(physics.water.clone()), PhysicsReadbackGeneration(physics.generation)))
        .observe(capture_gpu_water_readback);
    commands.spawn((Readback::buffer(physics.masks.clone()), PhysicsReadbackGeneration(physics.generation)))
        .observe(capture_gpu_mask_readback);
}

fn sync_readback_generation(
    mut commands: Commands,
    physics: Res<GpuShipPhysicsAssets>,
    readbacks: Query<(Entity, &PhysicsReadbackGeneration)>,
) {
    if readbacks.iter().count() == 3 && readbacks.iter().all(|(_, tag)| tag.0 == physics.generation) {
        return;
    }
    for (entity, _) in &readbacks { commands.entity(entity).despawn(); }
    spawn_readbacks(&mut commands, &physics);
}

fn current_readback(
    event: &ReadbackComplete,
    physics: &GpuShipPhysicsAssets,
    tags: &Query<&PhysicsReadbackGeneration>,
    planes: usize,
) -> bool {
    tags.get(event.entity).is_ok_and(|tag| tag.0 == physics.generation)
        && event.data.len() == physics.width as usize * physics.height as usize * planes * 16
}

#[derive(Resource, Default)]
pub(super) struct GpuShipPhysicsSnapshot {
    pub(super) positions: Vec<Vec4>,
    pub(super) water: Vec<Vec4>,
    pub(super) masks: Vec<[u32; 4]>,
}

pub(super) struct GpuShipPhysicsPlugin;

impl Plugin for GpuShipPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuShipPhysicsSnapshot>()
            .add_systems(PostUpdate, sync_readback_generation)
            .add_plugins(ExtractResourcePlugin::<GpuShipPhysicsAssets>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, initialize_gpu_ship_physics)
            .add_systems(
                Render,
                prepare_gpu_ship_physics_bind_group
                    .in_set(bevy::render::RenderSystems::PrepareBindGroups),
            )
            .add_systems(
                RenderGraph,
                dispatch_gpu_ship_physics.before(bevy::core_pipeline::schedule::camera_driver),
            );
    }
}

#[derive(Resource)]
struct GpuShipPhysicsPipeline {
    layout: BindGroupLayoutDescriptor,
    forces: CachedComputePipelineId,
    move_positions: CachedComputePipelineId,
    brush_water: CachedComputePipelineId,
    break_links: CachedComputePipelineId,
    integrate: CachedComputePipelineId,
    water_fill: CachedComputePipelineId,
    water_flow: CachedComputePipelineId,
    water_transport: CachedComputePipelineId,
    update_mass: CachedComputePipelineId,
    commit_water: CachedComputePipelineId,
    snapshot_masks: CachedComputePipelineId,
    repair_masks: CachedComputePipelineId,
}

#[derive(Resource)]
struct GpuShipPhysicsBindGroup {
    group: BindGroup,
    identity: PhysicsBindingIdentity,
}

// Source Ship.render/update use that instance's physics textures. A render
// preparation delay must never run new-Ship work through the previous group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PhysicsBindingIdentity {
    generation: u64,
    buffers: [bevy::asset::AssetId<ShaderBuffer>; 10],
}
impl PhysicsBindingIdentity {
    fn for_ship(physics: &GpuShipPhysicsAssets) -> Self {
        Self {
            generation: physics.generation,
            buffers: [physics.positions.id(), physics.materials.id(), physics.masks.id(),
                physics.forces.id(), physics.settings.id(), physics.water.id(),
                physics.water_outflow_1.id(), physics.water_outflow_2.id(),
                physics.water_velocity_1.id(), physics.water_velocity_2.id()],
        }
    }
    fn matches(self, physics: &GpuShipPhysicsAssets) -> bool {
        self == Self::for_ship(physics)
    }
}

fn initialize_gpu_ship_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    #[cfg(test)] diagnostic_shader: Option<Res<gpu_tests::DiagnosticShader>>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "SS2 per-texel ship physics",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[u32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer_read_only::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
                storage_buffer::<Vec<[f32; 4]>>(false),
            ),
        ),
    );
    #[cfg(not(test))]
    let shader = asset_server.load(GPU_PHYSICS_SHADER);
    #[cfg(test)]
    let shader = diagnostic_shader.map(|value|value.0.clone())
        .unwrap_or_else(||asset_server.load(GPU_PHYSICS_SHADER));
    let move_positions = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 position displacement pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("move_positions")),
        ..default()
    });
    let brush_water = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 live water tool pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("brush_water")),
        ..default()
    });
    let break_links = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 source destroy tool pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("break_links")),
        ..default()
    });
    let forces = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 per-texel spring force pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("forces")),
        ..default()
    });
    let integrate = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 per-texel position pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("integrate")),
        ..default()
    });
    let water_fill = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 water ingress pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("water_fill")),
        ..default()
    });
    let water_flow = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 water outflow pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("water_flow")),
        ..default()
    });
    let water_transport = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 water transport pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("water_transport")),
        ..default()
    });
    let update_mass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 water-weighted material mass pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("update_mass")),
        ..default()
    });
    let snapshot_masks = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 mask snapshot pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("snapshot_masks")),
        ..default()
    });
    let repair_masks = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 reciprocal strut pass".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("repair_masks")),
        ..default()
    });
    let commit_water = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("SS2 water state commit pass".into()),
        layout: vec![layout.clone()],
        shader,
        entry_point: Some(Cow::from("commit_water")),
        ..default()
    });
    commands.insert_resource(GpuShipPhysicsPipeline {
        layout,
        forces,
        move_positions,
        brush_water,
        break_links,
        integrate,
        water_fill,
        water_flow,
        water_transport,
        update_mass,
        commit_water,
        snapshot_masks,
        repair_masks,
    });
}

fn prepare_gpu_ship_physics_bind_group(
    mut commands: Commands,
    physics: Option<Res<GpuShipPhysicsAssets>>,
    pipeline: Option<Res<GpuShipPhysicsPipeline>>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let (Some(physics), Some(pipeline)) = (physics, pipeline) else {
        commands.remove_resource::<GpuShipPhysicsBindGroup>();
        return;
    };
    let (
        Some(positions),
        Some(materials),
        Some(masks),
        Some(forces),
        Some(settings),
        Some(water),
        Some(water_outflow_1),
        Some(water_outflow_2),
        Some(water_velocity_1),
        Some(water_velocity_2),
    ) = (
        gpu_buffers.get(&physics.positions),
        gpu_buffers.get(&physics.materials),
        gpu_buffers.get(&physics.masks),
        gpu_buffers.get(&physics.forces),
        gpu_buffers.get(&physics.settings),
        gpu_buffers.get(&physics.water),
        gpu_buffers.get(&physics.water_outflow_1),
        gpu_buffers.get(&physics.water_outflow_2),
        gpu_buffers.get(&physics.water_velocity_1),
        gpu_buffers.get(&physics.water_velocity_2),
    )
    else {
        // Any unavailable buffer invalidates the entire prepared Ship group.
        commands.remove_resource::<GpuShipPhysicsBindGroup>();
        return;
    };
    let bind_group = render_device.create_bind_group(
        Some("SS2 per-texel ship physics buffers"),
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            materials.buffer.as_entire_buffer_binding(),
            masks.buffer.as_entire_buffer_binding(),
            forces.buffer.as_entire_buffer_binding(),
            settings.buffer.as_entire_buffer_binding(),
            water.buffer.as_entire_buffer_binding(),
            water_outflow_1.buffer.as_entire_buffer_binding(),
            water_outflow_2.buffer.as_entire_buffer_binding(),
            water_velocity_1.buffer.as_entire_buffer_binding(),
            water_velocity_2.buffer.as_entire_buffer_binding(),
        )),
    );
    commands.insert_resource(GpuShipPhysicsBindGroup {
        group: bind_group,
        identity: PhysicsBindingIdentity::for_ship(&physics),
    });
}

fn dispatch_gpu_ship_physics(
    mut render_context: RenderContext,
    physics: Option<Res<GpuShipPhysicsAssets>>,
    bind_group: Option<Res<GpuShipPhysicsBindGroup>>,
    pipeline: Option<Res<GpuShipPhysicsPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    #[cfg(test)] mut verification: Option<ResMut<gpu_tests::DispatchControl>>,
) {
    #[cfg(test)]
    if verification.as_ref().is_some_and(|control| !control.armed && control.remaining==0) { return; }
    let (Some(physics), Some(bind_group), Some(pipeline)) = (physics, bind_group, pipeline) else {
        return;
    };
    // Commands retiring an unavailable group are deferred. The identity check
    // also blocks dispatch before that retirement has been applied.
    if !bind_group.identity.matches(&physics) { return; }
    let (
        Some(break_pipeline),
        Some(brush_pipeline),
        Some(move_pipeline),
        Some(force_pipeline),
        Some(integrate_pipeline),
        Some(water_fill_pipeline),
        Some(water_flow_pipeline),
        Some(water_transport_pipeline),
        Some(update_mass_pipeline),
        Some(commit_water_pipeline),
        Some(snapshot_masks_pipeline),
        Some(repair_masks_pipeline),
    ) = (
        pipeline_cache.get_compute_pipeline(pipeline.break_links),
        pipeline_cache.get_compute_pipeline(pipeline.brush_water),
        pipeline_cache.get_compute_pipeline(pipeline.move_positions),
        pipeline_cache.get_compute_pipeline(pipeline.forces),
        pipeline_cache.get_compute_pipeline(pipeline.integrate),
        pipeline_cache.get_compute_pipeline(pipeline.water_fill),
        pipeline_cache.get_compute_pipeline(pipeline.water_flow),
        pipeline_cache.get_compute_pipeline(pipeline.water_transport),
        pipeline_cache.get_compute_pipeline(pipeline.update_mass),
        pipeline_cache.get_compute_pipeline(pipeline.commit_water),
        pipeline_cache.get_compute_pipeline(pipeline.snapshot_masks),
        pipeline_cache.get_compute_pipeline(pipeline.repair_masks),
    )
    else {
        return;
    };
    if physics.iterations == 0 {
        return;
    }
    #[cfg(test)]
    if let Some(control) = verification.as_mut() {
        control.armed = false;
        control.remaining=control.remaining.saturating_sub(1);
        control.executed += 1;
    }
    let workgroups = physics
        .width
        .saturating_mul(physics.height)
        .div_ceil(GPU_PHYSICS_WORKGROUP_SIZE);
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("SS2 per-texel ship physics"),
            ..default()
        });
    pass.set_bind_group(0, &bind_group.group, &[]);
    pass.set_pipeline(move_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    pass.set_pipeline(break_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    pass.set_pipeline(brush_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    let water_steps = physics.water_steps.max(1).min(physics.iterations);
    let water_interval = source_water_interval(physics.iterations, water_steps);
    for step in 0..physics.iterations {
        #[cfg(test)]
        if let Some(control)=verification.as_mut() {control.physics_steps+=1;}
        pass.set_pipeline(force_pipeline);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(integrate_pipeline);
        pass.dispatch_workgroups(workgroups, 1, 1);
        if (step + 1) % water_interval == 0 {
            #[cfg(test)]
            if let Some(control)=verification.as_mut() {control.water_updates+=1;}
            pass.set_pipeline(water_fill_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(water_flow_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(water_transport_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(update_mass_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(commit_water_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(snapshot_masks_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(repair_masks_pipeline);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
    }
}

#[cfg(test)]
#[path = "ship_physics_gpu_tests.rs"]
mod gpu_tests;

/// ShipPhysics.update uses integer division, not a rounded distribution.
/// Active Bevy settings guarantee 1 <= water_steps <= iterations; source
/// invalid-setting exception behavior is not reproduced by that UI adapter.
fn source_water_interval(iterations: u32, water_steps: u32) -> u32 {
    iterations / water_steps
}

pub(super) fn capture_gpu_ship_physics_readback(
    event: On<ReadbackComplete>,
    physics: Res<GpuShipPhysicsAssets>,
    tags: Query<&PhysicsReadbackGeneration>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
    if !current_readback(&event, &physics, &tags, 1) { return; }
    let positions: Vec<[f32; 4]> = event.to_shader_type();
    snapshot.positions = positions.into_iter().map(Vec4::from_array).collect();
}

pub(super) fn capture_gpu_water_readback(
    event: On<ReadbackComplete>,
    physics: Res<GpuShipPhysicsAssets>,
    tags: Query<&PhysicsReadbackGeneration>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
    if !current_readback(&event, &physics, &tags, 3) { return; }
    let water: Vec<[f32; 4]> = event.to_shader_type();
    let count = water.len() / 3;
    snapshot.water = water
        .into_iter()
        .take(count)
        .map(Vec4::from_array)
        .collect();
}

pub(super) fn capture_gpu_mask_readback(
    event: On<ReadbackComplete>,
    physics: Res<GpuShipPhysicsAssets>,
    tags: Query<&PhysicsReadbackGeneration>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
    if !current_readback(&event, &physics, &tags, 2) { return; }
    let planes: Vec<[u32; 4]> = event.to_shader_type();
    snapshot.masks = planes[..planes.len() / 2].to_vec();
}

pub(super) fn make_gpu_ship_physics_assets(
    structure: &ShipStructure,
    buffers: &mut Assets<ShaderBuffer>,
) -> GpuShipPhysicsAssets {
    let positions: Vec<[f32; 4]> = structure
        .texel_rest_positions
        .iter()
        .map(|position| [position.x, position.y, 0.0, 0.0])
        .collect();
    let materials = gpu_material_data(structure);
    let masks = crate::mask_struts_data::gpu_mask_storage(gpu_mask_data(structure));
    let forces = ForceDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water = WaterDataHolder::new(structure.texel_width, structure.texel_height);
    let water_state = water.into_state_planes(3);
    let water_outflow_1 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_outflow_2 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_velocity_1 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_velocity_2 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    GpuShipPhysicsAssets {
        positions: buffers.add(ShaderBuffer::from(positions)),
        materials: buffers.add(ShaderBuffer::from(materials)),
        masks: buffers.add(ShaderBuffer::from(masks)),
        forces: buffers.add(ShaderBuffer::from(forces)),
        settings: buffers.add(ShaderBuffer::from(gpu_settings(
            structure,
            50,
            5,
            1.0 / 60.0,
            0.0,
            400.0,
            9.81,
            1.0,
            1.0,
            1.0,
            1.0,
            1.0,
            40.0,
            1.0,
            1.0,
            60.0,
            0.0,
            1.0,
            0.915,
        ))),
        water: buffers.add(ShaderBuffer::from(water_state)),
        water_outflow_1: buffers.add(ShaderBuffer::from(water_outflow_1)),
        water_outflow_2: buffers.add(ShaderBuffer::from(water_outflow_2)),
        water_velocity_1: buffers.add(ShaderBuffer::from(water_velocity_1)),
        water_velocity_2: buffers.add(ShaderBuffer::from(water_velocity_2)),
        water_brush: None,
        break_brush: None,
        width: structure.texel_width as u32,
        height: structure.texel_height as u32,
        iterations: 50,
        water_steps: 5,
        generation: 0,
    }
}

/// Ship selection/reset constructs a new source ShipPhysics; every per-ship
/// state allocation has a fresh identity. Retained old handles are untouched.
/// Native asset retirement follows Bevy ownership, not original JVM GC timing.
pub(super) fn replace_gpu_ship_physics(
    structure: &ShipStructure,
    physics: &mut GpuShipPhysicsAssets,
    buffers: &mut Assets<ShaderBuffer>,
) {
    let generation=physics.generation.wrapping_add(1);
    let mut replacement=make_gpu_ship_physics_assets(structure,buffers);
    // New Ship begins in placement mode and cannot run physics until release.
    replacement.iterations=0;
    replacement.generation=generation;
    *physics=replacement;
}

/// In-place storage restoration for isolated fixtures/legacy callers.
/// The live new-Ship path uses replace_gpu_ship_physics instead.
pub(super) fn reset_gpu_ship_physics(
    structure: &ShipStructure,
    physics: &mut GpuShipPhysicsAssets,
    buffers: &mut Assets<ShaderBuffer>,
) {
    // A new source ShipPhysics has no command queued by the previous ship.
    // Its owning Ship begins in placement mode until the first release.
    physics.water_brush = None;
    physics.break_brush = None;
    physics.iterations = 0;
    physics.generation = physics.generation.wrapping_add(1);
    let positions: Vec<[f32; 4]> = structure
        .texel_rest_positions
        .iter()
        .map(|position| [position.x, position.y, 0.0, 0.0])
        .collect();
    let materials = gpu_material_data(structure);
    let masks = crate::mask_struts_data::gpu_mask_storage(gpu_mask_data(structure));
    let forces = ForceDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water = WaterDataHolder::new(structure.texel_width, structure.texel_height);
    let water_state = water.into_state_planes(3);
    let water_outflow_1 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_outflow_2 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_velocity_1 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    let water_velocity_2 =
        WaterDataHolder::new(structure.texel_width, structure.texel_height).into_storage();
    for (handle, data) in [
        (&physics.positions, ShaderBuffer::from(positions)),
        (&physics.materials, ShaderBuffer::from(materials)),
        (&physics.masks, ShaderBuffer::from(masks)),
        (&physics.forces, ShaderBuffer::from(forces)),
        (&physics.water, ShaderBuffer::from(water_state)),
        (
            &physics.water_outflow_1,
            ShaderBuffer::from(water_outflow_1),
        ),
        (
            &physics.water_outflow_2,
            ShaderBuffer::from(water_outflow_2),
        ),
        (
            &physics.water_velocity_1,
            ShaderBuffer::from(water_velocity_1),
        ),
        (
            &physics.water_velocity_2,
            ShaderBuffer::from(water_velocity_2),
        ),
        (
            &physics.settings,
            ShaderBuffer::from(gpu_settings(
                structure,
                physics.iterations,
                physics.water_steps,
                1.0 / 60.0,
                0.0,
                400.0,
                9.81,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                40.0,
                25.0,
                1.0,
                60.0,
                0.0,
                1.0,
                0.915,
            )),
        ),
    ] {
        if let Some(mut buffer) = buffers.get_mut(handle) {
            *buffer = data;
        }
    }
    physics.width = structure.texel_width as u32;
    physics.height = structure.texel_height as u32;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn gpu_settings(
    structure: &ShipStructure,
    iterations: u32,
    water_steps: u32,
    frame_delta: f32,
    elapsed: f32,
    sea_depth: f32,
    gravity: f32,
    rigidity: f32,
    damping: f32,
    strength: f32,
    drag: f32,
    buoyancy: f32,
    wave_width: f32,
    wave_height: f32,
    water_inflow: f32,
    water_flow: f32,
    water_funk: f32,
    water_weight: f32,
    thickness: f32,
) -> Vec<[f32; 4]> {
    let steps = iterations.max(1);
    let water_steps = water_steps.max(1);
    vec![
        [
            structure.texel_width as f32,
            structure.texel_height as f32,
            steps as f32,
            frame_delta,
        ],
        [gravity, rigidity, damping, strength],
        [drag, buoyancy, wave_width, wave_height],
        [
            elapsed,
            -sea_depth,
            frame_delta / water_steps as f32,
            water_inflow,
        ],
        [water_flow, water_funk, water_weight, thickness],
        [0.0; 4], // Source posChangePass displacement; populated on release.
        [0.0; 4], // Source water brush: xy cursor, z radius, w flood/dry mode.
        [0.0; 4], // Source destroy brush: xy cursor, z radius, w enabled.
        // Source ShipPhysics.update computes both uniforms on the CPU. GPU
        // division can lower to reciprocal multiplication with different bits.
        // Z must retain positive-zero bits: the source-rounded integration
        // subtraction uses it as an opaque XOR identity precision barrier.
        [frame_delta / steps as f32, steps as f32 / frame_delta, 0.0, 0.0],
    ]
}

#[cfg(test)]
mod displacement_tests {
    #[test]
    fn source_final_pass_uses_truncated_interval_including_nondivisible_counts() {
        for (physics, water, expected) in [
            (50, 5, vec![10, 20, 30, 40, 50]),
            (11, 3, vec![3, 6, 9]),
            (7, 4, vec![1, 2, 3, 4, 5, 6, 7]),
            (6, 4, vec![1, 2, 3, 4, 5, 6]),
        ] {
            let interval = super::source_water_interval(physics, water);
            let actual: Vec<_> = (0..physics)
                .filter(|step| step % interval == interval - 1)
                .map(|step| step + 1)
                .collect();
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn source_physics_timing_uniforms_are_cpu_rounded_before_upload() {
        let settings=super::gpu_settings(&super::ShipStructure::empty_fallback(),
            50,5,0.016666668,0.0,400.0,9.81,1.0,1.0,1.0,1.0,1.0,
            40.0,1.0,1.0,60.0,0.0,1.0,0.915);
        assert_eq!(settings[8][0].to_bits(),(0.016666668f32/50.0).to_bits());
        assert_eq!(settings[8][1].to_bits(),(50.0f32/0.016666668).to_bits());
        assert_eq!(settings[8][2].to_bits(), 0, "precision barrier must preserve every reflection bit");
        let shader=include_str!("../assets/shaders/ship_physics.wgsl");
        assert!(shader.contains("let dt = settings[8].x;"));
        assert!(shader.contains("let fps = settings[8].y;"));
        assert!(shader.contains("let physics_delta = settings[8].x;"));
    }

    #[test]
    fn physics_shader_including_source_move_pass_is_valid() {
        let source = include_str!("../assets/shaders/ship_physics.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("physics WGSL parse");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("physics WGSL validation");
        assert!(
            module
                .entry_points
                .iter()
                .any(|p| p.name == "move_positions")
        );
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use crate::tools::move_tool::MoveDragState;

    #[test]
    fn compute_binding_identity_rejects_replaced_ship_before_group_retirement() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let mut physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let original = PhysicsBindingIdentity::for_ship(&physics);
        assert!(original.matches(&physics));
        // Changes to execution settings do not change which Ship owns storage.
        physics.iterations = 9;
        physics.water_steps = 3;
        assert!(original.matches(&physics));
        replace_gpu_ship_physics(&structure, &mut physics, &mut buffers);
        assert!(!original.matches(&physics));
        assert!(PhysicsBindingIdentity::for_ship(&physics).matches(&physics));
        // Fresh allocation remains distinguishable if generation wraps/repeats.
        physics.generation = original.generation;
        assert!(!original.matches(&physics));
    }

    #[test]
    fn compute_binding_identity_rejects_each_individually_replaced_buffer() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let original = PhysicsBindingIdentity::for_ship(&physics);
        let replacement = buffers.add(ShaderBuffer::from(vec![[0.0f32; 4]]));
        for index in 0..10 {
            let mut changed = physics.clone();
            let slots = [&mut changed.positions, &mut changed.materials, &mut changed.masks,
                &mut changed.forces, &mut changed.settings, &mut changed.water,
                &mut changed.water_outflow_1, &mut changed.water_outflow_2,
                &mut changed.water_velocity_1, &mut changed.water_velocity_2];
            *slots.into_iter().nth(index).unwrap() = replacement.clone();
            assert_eq!(changed.generation, physics.generation);
            assert!(!original.matches(&changed), "stale buffer slot {index} accepted");
            assert!(PhysicsBindingIdentity::for_ship(&changed).matches(&changed));
        }
    }

    #[test]
    fn compute_binding_identity_rejects_in_place_reset_generation() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let mut physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let original = PhysicsBindingIdentity::for_ship(&physics);
        reset_gpu_ship_physics(&structure, &mut physics, &mut buffers);
        assert_eq!(original.buffers[0], physics.positions.id());
        assert!(!original.matches(&physics));
        assert!(PhysicsBindingIdentity::for_ship(&physics).matches(&physics));
    }
    #[test]
    fn replaced_ship_rejects_old_readbacks_even_with_identical_dimensions() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let mut app = App::new();
        app.insert_resource(physics).insert_resource(buffers)
            .init_resource::<GpuShipPhysicsSnapshot>()
            .add_systems(PostUpdate, sync_readback_generation);
        app.update();
        let old: Vec<_> = app.world_mut().query::<(Entity, &Readback)>()
            .iter(app.world()).map(|(entity, readback)| (entity, readback.clone())).collect();
        assert_eq!(old.len(), 3);
        let position = ShaderBuffer::from(vec![[3.0f32, 4.0, 5.0, 6.0]]).data.unwrap();
        let water = ShaderBuffer::from(vec![[7.0f32; 4], [8.0; 4], [9.0; 4]]).data.unwrap();
        let masks = ShaderBuffer::from(vec![[11u32; 4], [13u32; 4]]).data.unwrap();
        let payload = |readback: &Readback, physics: &GpuShipPhysicsAssets| {
            let Readback::Buffer { buffer, .. } = readback else { panic!() };
            if *buffer == physics.positions { position.clone() }
            else if *buffer == physics.water { water.clone() }
            else { masks.clone() }
        };
        for (entity, readback) in &old {
            let data = payload(readback, app.world().resource::<GpuShipPhysicsAssets>());
            app.world_mut().trigger(ReadbackComplete { entity: *entity, data });
        }
        assert_eq!(app.world().resource::<GpuShipPhysicsSnapshot>().positions, vec![Vec4::new(3.0, 4.0, 5.0, 6.0)]);
        assert_eq!(app.world().resource::<GpuShipPhysicsSnapshot>().water, vec![Vec4::splat(7.0)]);
        assert_eq!(app.world().resource::<GpuShipPhysicsSnapshot>().masks, vec![[11; 4]]);
        let mut physics = app.world_mut().remove_resource::<GpuShipPhysicsAssets>().unwrap();
        replace_gpu_ship_physics(&structure, &mut physics,
            &mut app.world_mut().resource_mut::<Assets<ShaderBuffer>>());
        app.insert_resource(physics).insert_resource(GpuShipPhysicsSnapshot::default());
        // Old observers still exist here: generation validation must reject
        // their completions even before the PostUpdate retirement runs.
        for (entity, readback) in &old {
            let data = payload(readback, app.world().resource::<GpuShipPhysicsAssets>());
            app.world_mut().trigger(ReadbackComplete { entity: *entity, data });
        }
        let snapshot = app.world().resource::<GpuShipPhysicsSnapshot>();
        assert!(snapshot.positions.is_empty() && snapshot.water.is_empty() && snapshot.masks.is_empty());
        app.update();
        for (entity, _) in &old { assert!(app.world().get_entity(*entity).is_err()); }
        // Bevy can also deliver mapped completions after the requesting
        // entity was retired. Its old entity observers must stay retired.
        for (entity, readback) in &old {
            let data = payload(readback, app.world().resource::<GpuShipPhysicsAssets>());
            app.world_mut().trigger(ReadbackComplete { entity: *entity, data });
        }
        assert!(app.world().resource::<GpuShipPhysicsSnapshot>().positions.is_empty());
        let fresh: Vec<_> = app.world_mut().query::<(Entity, &Readback, &PhysicsReadbackGeneration)>()
            .iter(app.world()).map(|(entity, readback, tag)| {
                assert_eq!(tag.0, 1); (entity, readback.clone())
            }).collect();
        assert_eq!(fresh.len(), 3);
        for (entity, readback) in fresh {
            // Reject invalid/truncated payloads before shader decoding.
            app.world_mut().trigger(ReadbackComplete { entity, data: vec![0; 1] });
            let data = payload(&readback, app.world().resource::<GpuShipPhysicsAssets>());
            app.world_mut().trigger(ReadbackComplete { entity, data });
        }
        let snapshot = app.world().resource::<GpuShipPhysicsSnapshot>();
        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.water, vec![Vec4::splat(7.0)]);
        assert_eq!(snapshot.masks, vec![[11; 4]]);
    }

    #[test]
    fn settings_and_damage_commands_never_replace_live_gpu_topology() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let masks = physics.masks.clone();
        // Stand in for a newer GPU topology than either the rest state or
        // readback. Both current and scratch planes must remain untouched.
        let live = vec![[8u32, 0, 0, 23], [8u32, 0, 0, 41]];
        *buffers.get_mut(&masks).unwrap() = ShaderBuffer::from(live);
        let expected = buffers.get(&masks).unwrap().data.clone();
        let mut app = App::new();
        app.insert_resource(structure)
            .insert_resource(buffers)
            .insert_resource(physics)
            .init_resource::<Simulation>()
            .init_resource::<GpuShipPhysicsSnapshot>()
            .init_resource::<MoveDragState>()
            .add_systems(Update, crate::update_gpu_physics_settings);
        // A stale readback still contains intact links.
        app.world_mut().resource_mut::<GpuShipPhysicsSnapshot>().masks =
            vec![[8, 255, 255, 0]];
        app.update();
        assert_eq!(app.world().resource::<Assets<ShaderBuffer>>()
            .get(&masks).unwrap().data, expected);
        app.world_mut().resource_mut::<ShipStructure>().breached[0] = true;
        crate::tools::break_tool::apply(
            &mut app.world_mut().resource_mut::<GpuShipPhysicsAssets>(),
            Vec2::new(3.0, 4.0), 2.0,
        );
        app.update();
        assert_eq!(app.world().resource::<Assets<ShaderBuffer>>()
            .get(&masks).unwrap().data, expected);
        assert!(app.world().resource::<GpuShipPhysicsAssets>().break_brush.is_none());
        let physics = app.world().resource::<GpuShipPhysicsAssets>();
        let settings = app.world().resource::<Assets<ShaderBuffer>>()
            .get(&physics.settings).unwrap();
        let expected_settings = ShaderBuffer::from(vec![[3.0f32, 4.0, 2.0, 1.0]]);
        assert_eq!(&settings.data.as_ref().unwrap()[7 * 16..8 * 16],
            expected_settings.data.as_ref().unwrap());
        // Ship replacement still explicitly initializes every GPU mask plane.
        let mut world_buffers = app.world_mut().remove_resource::<Assets<ShaderBuffer>>().unwrap();
        let mut world_physics = app.world_mut().remove_resource::<GpuShipPhysicsAssets>().unwrap();
        reset_gpu_ship_physics(
            app.world().resource::<ShipStructure>(), &mut world_physics, &mut world_buffers,
        );
        assert_ne!(world_buffers.get(&masks).unwrap().data, expected);
    }

    #[test]
    fn active_drag_freezes_solver_and_release_commands_are_consumed_once() {
        let structure = ShipStructure::empty_fallback();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let mut app = App::new();
        app.insert_resource(structure)
            .insert_resource(buffers)
            .insert_resource(physics)
            .init_resource::<Simulation>()
            .init_resource::<GpuShipPhysicsSnapshot>()
            .init_resource::<MoveDragState>()
            .add_systems(Update, crate::update_gpu_physics_settings);
        assert!(app.world().resource::<MoveDragState>().dragging);
        app.update();
        assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().iterations, 0);
        app.world_mut()
            .resource_mut::<MoveDragState>()
            .release(false);
        app.world_mut()
            .resource_mut::<MoveDragState>()
            .pending_translation = Vec2::ONE;
        app.world_mut()
            .resource_mut::<GpuShipPhysicsAssets>()
            .water_brush = Some([0.0, 0.0, 1.0, 1.0]);
        app.update();
        assert_eq!(
            app.world().resource::<MoveDragState>().pending_translation,
            Vec2::ZERO
        );
        assert!(
            app.world()
                .resource::<GpuShipPhysicsAssets>()
                .water_brush
                .is_none()
        );
        assert_eq!(
            app.world().resource::<GpuShipPhysicsAssets>().iterations,
            50
        );
        app.update();
        assert_eq!(
            app.world().resource::<MoveDragState>().pending_translation,
            Vec2::ZERO
        );
    }
}


#[cfg(test)]
mod source_allocation_tests {
    use super::*;
    fn handles(p:&GpuShipPhysicsAssets)->[Handle<ShaderBuffer>;10] {
        [p.positions.clone(),p.materials.clone(),p.masks.clone(),p.forces.clone(),p.settings.clone(),
         p.water.clone(),p.water_outflow_1.clone(),p.water_outflow_2.clone(),p.water_velocity_1.clone(),p.water_velocity_2.clone()]
    }
    #[test]
    fn new_source_ship_has_fresh_buffers_and_preserves_retained_previous_state() {
        let structure=ShipStructure::empty_fallback();let mut buffers=Assets::<ShaderBuffer>::default();
        let mut physics=make_gpu_ship_physics_assets(&structure,&mut buffers);
        let old=handles(&physics);
        for handle in &old {*buffers.get_mut(handle).unwrap()=ShaderBuffer::from(vec![[99.0f32;4]]);}
        let old_data:Vec<_>=old.iter().map(|h|buffers.get(h).unwrap().data.clone()).collect();
        physics.water_brush=Some([1.0;4]);physics.break_brush=Some([2.0;4]);physics.generation=u64::MAX;
        let mut reference=Assets::<ShaderBuffer>::default();let expected=make_gpu_ship_physics_assets(&structure,&mut reference);
        replace_gpu_ship_physics(&structure,&mut physics,&mut buffers);
        assert_eq!(physics.generation,0);assert_eq!(physics.iterations,0);
        assert!(physics.water_brush.is_none()&&physics.break_brush.is_none());
        let fresh=handles(&physics);
        for (index,(handle,expected_handle)) in fresh.iter().zip(handles(&expected)).enumerate() {
            assert!(!old.contains(handle),"Previous source buffer reused at {index}");
            assert_eq!(buffers.get(handle).unwrap().data,reference.get(&expected_handle).unwrap().data);
        }
        for (handle,data) in old.iter().zip(old_data) {assert_eq!(buffers.get(handle).unwrap().data,data);}
        replace_gpu_ship_physics(&structure,&mut physics,&mut buffers);
        assert_eq!(physics.generation,1);
        assert!(handles(&physics).iter().all(|h|!old.contains(h)&&!fresh.contains(h)));
    }
}

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
struct GpuShipPhysicsBindGroup(BindGroup);

fn initialize_gpu_ship_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
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
    let shader = asset_server.load(GPU_PHYSICS_SHADER);
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
    commands.insert_resource(GpuShipPhysicsBindGroup(bind_group));
}

fn dispatch_gpu_ship_physics(
    mut render_context: RenderContext,
    physics: Option<Res<GpuShipPhysicsAssets>>,
    bind_group: Option<Res<GpuShipPhysicsBindGroup>>,
    pipeline: Option<Res<GpuShipPhysicsPipeline>>,
    pipeline_cache: Res<PipelineCache>,
) {
    let (Some(physics), Some(bind_group), Some(pipeline)) = (physics, bind_group, pipeline) else {
        return;
    };
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
    pass.set_bind_group(0, &bind_group.0, &[]);
    pass.set_pipeline(move_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    pass.set_pipeline(break_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    pass.set_pipeline(brush_pipeline);
    pass.dispatch_workgroups(workgroups, 1, 1);
    if physics.iterations == 0 {
        return;
    }
    let water_steps = physics.water_steps.max(1).min(physics.iterations);
    let water_interval = physics.iterations.div_ceil(water_steps).max(1);
    for step in 0..physics.iterations {
        pass.set_pipeline(force_pipeline);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(integrate_pipeline);
        pass.dispatch_workgroups(workgroups, 1, 1);
        if (step + 1) % water_interval == 0 {
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

pub(super) fn capture_gpu_ship_physics_readback(
    event: On<ReadbackComplete>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
    let positions: Vec<[f32; 4]> = event.to_shader_type();
    snapshot.positions = positions.into_iter().map(Vec4::from_array).collect();
}

pub(super) fn capture_gpu_water_readback(
    event: On<ReadbackComplete>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
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
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
) {
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
    }
}

pub(super) fn reset_gpu_ship_physics(
    structure: &ShipStructure,
    physics: &mut GpuShipPhysicsAssets,
    buffers: &mut Assets<ShaderBuffer>,
) {
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
    ]
}

#[cfg(test)]
mod displacement_tests {
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
        app.world_mut().resource_mut::<MoveDragState>().dragging = true;
        app.update();
        assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().iterations, 0);
        app.world_mut().resource_mut::<MoveDragState>().dragging = false;
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

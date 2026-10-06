mod al_buffer;
mod al_context;
mod al_context_start_reference;
mod al_device;
mod al_resource;
mod al_source;
mod al_util_kt;
mod backed_property;
mod camera_2d;
mod camera_control;
mod dslfix;
mod enums;
mod executor_kt;
mod fbo;
mod file_reader;
mod file_reader_kt;
mod float_data_holder;
mod float_property;
mod floor;
mod force_data;
mod fragment_shaders;
mod framebuffer_target;
mod fullscreen;
mod game_parameter_provider_kt;
mod game_parameters;
mod gl_builder;
mod gl_context;
mod gl_data_holder;
mod gl_resource;
mod gl_state;
mod glfw;
mod glfw_monitors;
mod gui;
mod gui_kt;
mod gui_tool_factory;
mod i_drawable;
mod image_data;
mod input_handler;
mod input_handler_delegate;
mod int_property;
mod jvm_character;
mod kotlin_helpers;
mod mask_struts_data;
mod mass_strength_data;
mod materials;
mod mem_util;
mod model;
mod monitor;
mod monitor_scale;
mod monitor_video_mode;
mod music_player;
mod music_player_progress_reference;
mod music_player_volume_reference;
mod passes;
mod pos_vel_data;
mod render_buffer;
mod render_fbo;
mod resource;
mod screen_fbo;
mod sea;
mod shaded_model;
mod shader;
mod shader_program;
mod ship;
mod ship_data;
mod ship_physics;
mod ship_resources;
mod ship_resource_when_mappings;
mod ship_upload;
mod ship_struts;
mod ship_thumbnail;
mod sky;
mod tee_output_stream;
mod texture;
mod texture_1d;
mod texture_2d;
mod texture_2d_array;
mod textured_fbo;
mod time_sync;
mod time_sync_reporter;
mod toolbox;
mod toolbox_references;
mod toolbox_reload;
mod toolbox_reload_file_predicate;
mod toolbox_reload_filesystem;
mod toolbox_reload_name_comparator;
mod toolbox_render_3;
mod toolbox_settings;
mod toolbox_ship_browser;
mod tools;
mod typed_data_holder;
mod uint8_data_holder;
mod uv_model;
mod vao;
mod vbo;
mod vector2_property;
mod vertex_shaders;
mod water_data;
mod window;
mod window_framebuffer_callback;

use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::{ClearColorConfig, ScalingMode, visibility::RenderLayers},
    input::mouse::{MouseScrollUnit, MouseWheel},
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    reflect::TypePath,
    render::{
        Render, RenderApp, RenderStartup,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePassDescriptor,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
            binding_types::{storage_buffer, storage_buffer_read_only},
        },
        renderer::{RenderContext, RenderDevice, RenderGraph},
        storage::{GpuShaderBuffer, ShaderBuffer},
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};
use camera_control::{CameraControlState, handle_camera_control};
use fragment_shaders::ShipMaterial;
use mask_struts_data::{build_strut_masks, gpu_mask_data};
use mass_strength_data::gpu_material_data;
use ship_physics::{
    GpuShipPhysicsAssets, GpuShipPhysicsPlugin, GpuShipPhysicsSnapshot, capture_gpu_mask_readback,
    capture_gpu_ship_physics_readback, capture_gpu_water_readback, gpu_settings,
    make_gpu_ship_physics_assets, reset_gpu_ship_physics,
};
use ship_resources::{ShipLayer, ShipResourceFile, ShipResourceType, parse_resource_path};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use tools::tool::Tool;

const SEA_LEVEL: f32 = -120.0;
const WORLD_WIDTH: f32 = 1280.0;
const SHIP_HALF_WIDTH: f32 = 180.0;
const DEFAULT_TOOL_SIZE: f32 = 1.0;
const EIGHT_NEIGHBORS: [(isize, isize); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
// The CPU lattice is retained for cavity display and direct user tools only.
const PHYSICS_NODE_PIXELS: usize = 4;
const DRAG_SEA_COLOR: usize = usize::MAX;
const DRAG_SEA_HUE: usize = usize::MAX - 1;
const DRAG_SEA_ALPHA: usize = usize::MAX - 2;
const GPU_PHYSICS_SHADER: &str = "shaders/ship_physics.wgsl";
const GPU_PHYSICS_WORKGROUP_SIZE: u32 = 64;

fn main() {
    let ship_catalog = ShipCatalog::discover();
    let initial_structure = ShipStructure::load_for_choice(&ship_catalog.0[0]);
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::asset::AssetPlugin {
                    file_path: std::env::current_dir()
                        .expect("working directory")
                        .join("assets")
                        .to_string_lossy()
                        .into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sinking Simulator — Bevy port".into(),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: (2554, 1378).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            Material2dPlugin::<InternalWaterMaterial>::default(),
            Material2dPlugin::<tools::brush_preview::BrushMaterial>::default(),
            Material2dPlugin::<ShipMaterial>::default(),
            Material2dPlugin::<sky::SkyMaterial>::default(),
            Material2dPlugin::<ReflectionMaterial>::default(),
            Material2dPlugin::<OceanSurfaceMaterial>::default(),
            Material2dPlugin::<sea::SeaMaterial>::default(),
            Material2dPlugin::<OceanDepthMaterial>::default(),
            Material2dPlugin::<UnderwaterEffectMaterial>::default(),
            GpuShipPhysicsPlugin,
            resource::ResourcePlugin,
            render_fbo::RenderFboPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.40, 0.68, 0.82)))
        .init_resource::<Simulation>()
        .init_resource::<time_sync::TimeSync>()
        .add_systems(Startup, time_sync::register_lifecycle)
        .add_systems(Last, time_sync::sync_frame)
        .insert_resource(music_player::MusicPlayer::discover())
        .init_resource::<CameraControlState>()
        .init_resource::<tools::move_tool::MoveDragState>()
        .insert_resource(ship_catalog)
        .insert_resource(initial_structure)
        .add_systems(Startup, (setup, floor::setup, sky::setup))
        .add_systems(Startup, sea::setup.after(setup))
        .add_systems(
            PostUpdate,
            sea::sync
                .before(bevy::camera::CameraUpdateSystems)
                .before(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(
            Update,
            music_player::sync.after(select_toolbox_tab_and_settings),
        )
        .add_systems(
            Update,
            (update_gpu_physics_settings, apply_gpu_physics_readback)
                .chain()
                .after(tools::tool::handle_ship_tool),
        )
        .add_systems(
            Update,
            (
                (
                    handle_controls,
                    handle_camera_control,
                    select_toolbox_tab_and_settings,
                    select_ship_from_panel,
                    select_ship_layer,
                    import_dropped_ship,
                    load_selected_ship,
                )
                    .chain(),
                (
                    select_tool_from_panel,
                    tools::move_tool::handle_ship_move,
                    tools::brush_preview::update_damage_brush_preview,
                    tools::tool::handle_ship_tool,
                    animate_water,
                    animate_sea_depth,
                    sky::animate_sky.after(handle_camera_control),
                    sync_deformed_mesh.after(apply_gpu_physics_readback),
                    sync_internal_water_mesh,
                    animate_ship,
                    ship_struts::sync_ship_struts,
                    animate_reflection,
                    animate_leaks,
                    sync_toolbox_visibility,
                    sync_ship_assets,
                    sync_ship_cards,
                    sync_tool_panel_visibility,
                    sync_settings_ui,
                    update_ship_layer_label,
                    update_hud,
                )
                    .chain(),
            )
                .chain(),
        )
        .add_systems(
            Update,
            fragment_shaders::update_ship_lighting.after(sync_ship_assets),
        )
        .add_systems(Update, floor::update.after(handle_camera_control))
        .add_systems(Update, music_player::handle_seek.before(music_player::sync))
        .run();
}

#[derive(Resource)]
struct Simulation {
    elapsed: f32,
    flooding: f32,
    paused: bool,
    tool: Tool,
    ship_index: usize,
    selected_layer: usize,
    ship_scroll: usize,
    ship_search: String,
    ship_search_active: bool,
    pump_enabled: bool,
    wave_amplitude: f32,
    water_flow: f32,
    buoyancy: f32,
    wave_width: f32,
    sea_depth: f32,
    drag: f32,
    water_influx: f32,
    water_funk: f32,
    gravity: f32,
    strength: f32,
    rigidity: f32,
    damping: f32,
    water_weight: f32,
    thickness: f32,
    water_darkness: f32,
    sea_color: Vec3,
    sea_alpha: f32,
    sea_hue: f32,
    cycle_length: f32,
    day: f32,
    cycle_enabled: bool,
    show_tools: bool,
    toolbox_collapsed: bool,
    physics_iterations: f32,
    water_steps: f32,
    tool_size: f32,
    music_playing: bool,
    music_volume: f32,
    show_internal_water: bool,
    active_tab: ToolboxTab,
}

/// Material-map image represented as a connected, clumped soft-body lattice.
#[derive(Resource)]
struct ShipStructure {
    width: usize,
    height: usize,
    texel_width: usize,
    texel_height: usize,
    half_height: f32,
    texel_solid: Vec<bool>,
    texel_strut_masks: Vec<u8>,
    texel_materials: Vec<Option<MaterialProperties>>,
    texel_rest_positions: Vec<Vec2>,
    texel_positions: Vec<Vec2>,
    solid: Vec<bool>,
    interior: Vec<bool>,
    strut_masks: Vec<u8>,
    materials: Vec<Option<MaterialProperties>>,
    breached: Vec<bool>,
    leaking: Vec<bool>,
    rest_positions: Vec<Vec2>,
    positions: Vec<Vec2>,
    last_positions: Vec<Vec2>,
    springs: Vec<Spring>,
    water: Vec<f32>,
    flooding: f32,
    motion_position: Vec2,
    motion_velocity: Vec2,
    manual_offset: Vec2,
    angle: f32,
    angular_velocity: f32,
}

struct Spring {
    a: usize,
    b: usize,
    direction: u8,
    rest_length: f32,
    tensile_strength: f32,
    compressive_strength: f32,
    broken: bool,
}

#[derive(Clone, Copy)]
struct MaterialProperties {
    strength: f32,
    tensile_strength: f32,
    compressive_strength: f32,
    mass: f32,
    hull: bool,
    ground: bool,
    rope: bool,
    invisible: bool,
}

impl From<&materials::Material> for MaterialProperties {
    fn from(material: &materials::Material) -> Self {
        Self {
            strength: material.strength,
            tensile_strength: material.tensile_strength.unwrap_or(material.strength),
            compressive_strength: material
                .compressive_strength
                .unwrap_or(material.strength * 4.0),
            mass: material.mass,
            hull: material.is_hull,
            ground: material.is_ground,
            rope: material.is_rope,
            invisible: material.invisible,
        }
    }
}

#[derive(Default)]
struct MaterialSampleAccumulator {
    samples: usize,
    strength: f32,
    tensile_strength: f32,
    compressive_strength: f32,
    mass: f32,
    hull_samples: usize,
    ground_samples: usize,
    rope_samples: usize,
    invisible_samples: usize,
}

impl MaterialSampleAccumulator {
    fn add(&mut self, material: MaterialProperties) {
        self.samples += 1;
        self.strength += material.strength;
        self.tensile_strength += material.tensile_strength;
        self.compressive_strength += material.compressive_strength;
        self.mass += material.mass;
        self.hull_samples += usize::from(material.hull);
        self.ground_samples += usize::from(material.ground);
        self.rope_samples += usize::from(material.rope);
        self.invisible_samples += usize::from(material.invisible);
    }

    fn into_properties(self) -> Option<MaterialProperties> {
        (self.samples > 0).then(|| MaterialProperties {
            strength: self.strength / self.samples as f32,
            tensile_strength: self.tensile_strength / self.samples as f32,
            compressive_strength: self.compressive_strength / self.samples as f32,
            mass: self.mass / self.samples as f32,
            hull: self.hull_samples * 2 > self.samples,
            ground: self.ground_samples * 2 > self.samples,
            rope: self.rope_samples * 2 > self.samples,
            invisible: self.invisible_samples * 2 > self.samples,
        })
    }
}

fn classify_interior_air(solid: &[bool], width: usize, height: usize) -> Vec<bool> {
    if width == 0 || height == 0 || solid.len() != width * height {
        return vec![false; solid.len()];
    }
    let mut exterior = vec![false; solid.len()];
    let mut pending = VecDeque::new();
    for x in 0..width {
        enqueue_exterior_air(x, solid, &mut exterior, &mut pending);
        enqueue_exterior_air((height - 1) * width + x, solid, &mut exterior, &mut pending);
    }
    for y in 0..height {
        enqueue_exterior_air(y * width, solid, &mut exterior, &mut pending);
        enqueue_exterior_air(y * width + width - 1, solid, &mut exterior, &mut pending);
    }
    while let Some(index) = pending.pop_front() {
        let x = index % width;
        let y = index / width;
        for (dx, dy) in EIGHT_NEIGHBORS {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 && nx < width as isize && ny < height as isize {
                enqueue_exterior_air(
                    ny as usize * width + nx as usize,
                    solid,
                    &mut exterior,
                    &mut pending,
                );
            }
        }
    }
    solid
        .iter()
        .zip(exterior)
        .map(|(&is_solid, is_exterior)| !is_solid && !is_exterior)
        .collect()
}

fn enqueue_exterior_air(
    index: usize,
    solid: &[bool],
    exterior: &mut [bool],
    pending: &mut VecDeque<usize>,
) {
    if !solid[index] && !exterior[index] {
        exterior[index] = true;
        pending.push_back(index);
    }
}

fn effective_density(
    mass: f32,
    water_fill: f32,
    hull: bool,
    ground: bool,
    thickness: f32,
    water_weight: f32,
) -> f32 {
    let fill = water_fill.clamp(0.0, 1.0);
    let water_density = if hull || ground {
        1025.0
    } else {
        1025.0 * water_weight.max(0.0)
    };
    let fluid_density = 1.225 + (water_density - 1.225) * fill;
    mass.max(0.001) + (fluid_density - mass.max(0.001)) * (1.0 - thickness).clamp(0.0, 1.0)
}

fn buoyancy_acceleration(
    gravity: f32,
    buoyancy: f32,
    fluid_density: f32,
    body_density: f32,
) -> f32 {
    -gravity + gravity * buoyancy.max(0.0) * fluid_density.max(0.0) / body_density.max(0.001)
}

fn resolve_floor_collision(
    previous_position: Vec2,
    predicted_position: Vec2,
    floor_height: f32,
    delta: f32,
) -> (Vec2, Vec2) {
    if delta <= 0.0 {
        return (predicted_position, Vec2::ZERO);
    }
    let velocity = (predicted_position - previous_position) / delta;
    let displacement_y = predicted_position.y - previous_position.y;
    let mut collision = if displacement_y < 0.0 {
        (floor_height - previous_position.y) / displacement_y
    } else {
        1.0
    };
    if !collision.is_finite() || !(0.0..=1.0).contains(&collision) {
        collision = 1.0;
    }
    let speed_scale = 1.0 / (velocity.length_squared() + 1.0);
    let reflected = Vec2::new(velocity.x * speed_scale, -velocity.y * speed_scale);
    let position_velocity = reflected.lerp(velocity, collision);
    let resolved_position = previous_position + position_velocity * delta;
    let mut resolved_velocity = if collision == 1.0 {
        velocity
    } else {
        reflected
    };
    let mut resolved_position = resolved_position;
    if resolved_position.y < floor_height {
        resolved_position.y = (resolved_position.y - floor_height) * 0.01 + floor_height;
        resolved_velocity.x *= 0.5;
    }
    (resolved_position, resolved_velocity)
}

fn clamp_hull_deformation(position: Vec2, rest_position: Vec2, maximum: f32) -> Vec2 {
    rest_position + (position - rest_position).clamp_length_max(maximum.max(0.0))
}

fn flow_interior_water(
    water: &mut [f32],
    interior: &[bool],
    width: usize,
    height: usize,
    cell_height: f32,
    gravity: f32,
    delta: f32,
    flow_scale: f32,
    funk: f32,
) {
    if width == 0 || height == 0 || water.len() != width * height || interior.len() != water.len() {
        return;
    }
    let cell_height = cell_height.max(0.001);
    let mut proposals = Vec::<(usize, usize, f32)>::new();
    let mut outgoing = vec![0.0; water.len()];
    let mut incoming = vec![0.0; water.len()];
    for y in 0..height {
        for x in 0..width {
            let from = y * width + x;
            if !interior[from] || water[from] <= 0.0 {
                continue;
            }
            for (to, edge_weight) in [
                (x + 1 < width).then_some((from + 1, 1.0)),
                (y + 1 < height).then_some((from + width, 1.0)),
                (x + 1 < width && y + 1 < height)
                    .then_some((from + width + 1, std::f32::consts::FRAC_1_SQRT_2)),
                (x > 0 && y + 1 < height)
                    .then_some((from + width - 1, std::f32::consts::FRAC_1_SQRT_2)),
            ]
            .into_iter()
            .flatten()
            {
                if !interior[to] {
                    continue;
                }
                let to_y = to / width;
                let head_from = (height as f32 - y as f32 - 0.5 + water[from]) * cell_height;
                let head_to = (height as f32 - to_y as f32 - 0.5 + water[to]) * cell_height;
                let difference = head_from - head_to;
                if difference.abs() < 0.001 {
                    continue;
                }
                let (source, target, head) = if difference > 0.0 {
                    (from, to, difference)
                } else {
                    (to, from, -difference)
                };
                let amount = (2.0 * gravity.max(0.0) * head).sqrt() * delta.max(0.0) / cell_height
                    * flow_scale.max(0.0)
                    * 0.03
                    * edge_weight
                    * (1.0 + funk.max(0.0) * (water[source] - 1.0)).max(0.0);
                if amount > 0.0 {
                    proposals.push((source, target, amount));
                    outgoing[source] += amount;
                    incoming[target] += amount;
                }
            }
        }
    }
    let mut transfers = vec![0.0; water.len()];
    for (source, target, amount) in proposals {
        let source_scale = (water[source] / outgoing[source].max(0.001)).min(1.0);
        let target_capacity = (1.0 - water[target]).max(0.0);
        let target_scale = (target_capacity / incoming[target].max(0.001)).min(1.0);
        let moved = amount * source_scale.min(target_scale);
        transfers[source] -= moved;
        transfers[target] += moved;
    }
    for (amount, change) in water.iter_mut().zip(transfers) {
        if *amount > 0.0 || change > 0.0 {
            *amount = (*amount + change).clamp(0.0, 1.0);
        }
    }
}

fn is_exposed_node(solid: &[bool], width: usize, height: usize, index: usize) -> bool {
    let x = index % width;
    let y = index / width;
    EIGHT_NEIGHBORS.iter().any(|(dx, dy)| {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        nx < 0
            || ny < 0
            || nx >= width as isize
            || ny >= height as isize
            || !solid[ny as usize * width + nx as usize]
    })
}

fn normalized_node_mass(material: Option<MaterialProperties>) -> f32 {
    material
        .map(|material| (material.mass / 2409.0).clamp(0.25, 1000.0))
        .unwrap_or(1.0)
}

// SS2 compares elastic spring load against strength * iterations * fps.
fn source_spring_break_scale(iterations: usize, frame_delta: f32) -> f32 {
    let iterations = iterations.max(1) as f32;
    let fps = iterations / frame_delta.max(1.0e-5);
    fps * iterations
}

fn spring_stiffness(mass_a: f32, mass_b: f32, rigidity: f32) -> f32 {
    let material_density = mass_a.min(mass_b).clamp(0.25, 4.0);
    36.0 * rigidity.clamp(0.0, 10.0) * material_density
}

fn spring_extension_force(
    delta_position: Vec2,
    rest_length: f32,
    mass_a: f32,
    mass_b: f32,
    rigidity: f32,
) -> f32 {
    (delta_position.length() - rest_length) * spring_stiffness(mass_a, mass_b, rigidity)
}

fn spring_force(
    delta_position: Vec2,
    relative_velocity: Vec2,
    rest_length: f32,
    mass_a: f32,
    mass_b: f32,
    rigidity: f32,
    damping: f32,
) -> Vec2 {
    let length = delta_position.length();
    if length <= 1.0e-5 || rest_length <= 1.0e-5 {
        return Vec2::ZERO;
    }
    let direction = delta_position / length;
    let reduced_mass = mass_a * mass_b / (mass_a + mass_b).max(1.0e-5);
    let stiffness = spring_stiffness(mass_a, mass_b, rigidity);
    let damping_force = 2.0 * (stiffness * reduced_mass).sqrt() * damping.clamp(0.0, 10.0);
    let extension = length - rest_length;
    direction * (extension * stiffness) + relative_velocity * damping_force
}

// Original ShipPhysics force shader: frame/substep scaling, effective mass and rope softness.
fn source_cpu_spring_force(
    delta_position: Vec2,
    relative_velocity: Vec2,
    rest_length: f32,
    mass_a: f32,
    mass_b: f32,
    rigidity: f32,
    damping: f32,
    rope: bool,
    iterations: usize,
    frame_delta: f32,
) -> (Vec2, f32) {
    let b = 0.03 * source_spring_break_scale(iterations, frame_delta);
    let stiffness = 750.0 * mass_a.min(mass_b) * b * rigidity;
    let elastic_load =
        (delta_position.length() - rest_length) * stiffness * if rope { 0.001 } else { 1.0 };
    let force = if delta_position != Vec2::ZERO {
        delta_position.normalize() * elastic_load + b * damping * relative_velocity
    } else {
        Vec2::ZERO
    };
    (force, elastic_load)
}
fn should_break_spring(
    spring_force: f32,
    tensile_strength: f32,
    compressive_strength: f32,
    strength: f32,
) -> bool {
    let limit = if spring_force < 0.0 {
        compressive_strength
    } else {
        tensile_strength
    };
    spring_force.abs() > limit.max(0.0) * strength.max(0.0)
}

fn break_spring(spring: &mut Spring, strut_masks: &mut [u8]) {
    spring.broken = true;
    strut_masks[spring.a] &= !(1 << spring.direction);
    strut_masks[spring.b] &= !(1 << ((spring.direction + 4) % 8));
}

fn restore_spring(spring: &mut Spring, strut_masks: &mut [u8]) {
    spring.broken = false;
    strut_masks[spring.a] |= 1 << spring.direction;
    strut_masks[spring.b] |= 1 << ((spring.direction + 4) % 8);
}

fn parse_material_palette(json: &str) -> Result<HashMap<u32, MaterialProperties>, String> {
    let materials = materials::Materials::from_json(json)?;
    Ok(materials
        .materials
        .into_iter()
        .map(|(rgb, material)| {
            let properties = MaterialProperties::from(&material);
            (rgb, properties)
        })
        .collect())
}

fn find_connected_white_background(image: &image::RgbaImage) -> Vec<bool> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let is_white = |index: usize| {
        let pixel = image
            .get_pixel((index % width) as u32, (index / width) as u32)
            .0;
        pixel[3] > 8 && pixel[0] >= 245 && pixel[1] >= 245 && pixel[2] >= 245
    };
    let mut background = vec![false; width * height];
    let mut pending = VecDeque::new();
    for x in 0..width {
        for y in [0, height - 1] {
            let index = y * width + x;
            if !background[index] && is_white(index) {
                background[index] = true;
                pending.push_back(index);
            }
        }
    }
    for y in 0..height {
        for x in [0, width - 1] {
            let index = y * width + x;
            if !background[index] && is_white(index) {
                background[index] = true;
                pending.push_back(index);
            }
        }
    }
    while let Some(index) = pending.pop_front() {
        let x = index % width;
        let y = index / width;
        let neighbors = [
            x.checked_sub(1).map(|nx| y * width + nx),
            (x + 1 < width).then_some(y * width + x + 1),
            y.checked_sub(1).map(|ny| ny * width + x),
            (y + 1 < height).then_some((y + 1) * width + x),
        ];
        for neighbor in neighbors.into_iter().flatten() {
            if !background[neighbor] && is_white(neighbor) {
                background[neighbor] = true;
                pending.push_back(neighbor);
            }
        }
    }
    background
}

impl ShipStructure {
    fn validate_texel_data(&self) {
        let texel_count = self.texel_width * self.texel_height;
        assert_eq!(self.texel_solid.len(), texel_count);
        assert_eq!(self.texel_strut_masks.len(), texel_count);
        assert_eq!(self.texel_materials.len(), texel_count);
        assert!(
            self.texel_solid
                .iter()
                .zip(&self.texel_materials)
                .all(|(solid, material)| *solid == material.is_some()),
            "source texel occupancy must match material assignments"
        );
    }

    fn load_for_choice(choice: &ShipChoice) -> Self {
        let image_path = format!("assets/{}", choice.physics_asset);
        let Ok(image) = image::open(&image_path) else {
            return Self::empty_fallback();
        };
        let image = image.to_rgba8();
        // SS2 evaluates this map per texel on the GPU. A CPU-side per-texel
        // port created hundreds of thousands of points and over a million
        // constraints, causing stalls and self-tearing. The art remains at its
        // original resolution and is UV-mapped across these material clusters.
        let source_width = image.width() as usize;
        let source_height = image.height() as usize;
        let half_height = (SHIP_HALF_WIDTH * source_height as f32 / source_width.max(1) as f32)
            .clamp(40.0, 180.0);
        let width = source_width.div_ceil(PHYSICS_NODE_PIXELS);
        let height = source_height.div_ceil(PHYSICS_NODE_PIXELS);
        let white_background = if choice.material_map {
            vec![false; source_width * source_height]
        } else {
            find_connected_white_background(&image)
        };
        let global_materials = match std::fs::read_to_string("assets/config/materials.json") {
            Ok(json) => materials::Materials::from_json(&json).unwrap_or_else(|error| {
                bevy::log::error!("Could not parse global material palette: {error}");
                materials::Materials::default()
            }),
            Err(error) => {
                bevy::log::error!("Could not read global material palette: {error}");
                materials::Materials::default()
            }
        };
        let palette = if choice.material_map {
            ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(&image_path))
                .map(|thumbnail| thumbnail.materials(&global_materials))
                .unwrap_or_else(|_| global_materials.clone())
        } else {
            global_materials
        };
        // ShipData uses the original BASE pixels, including invisible materials
        // and RGB values with zero alpha. Visibility filtering applies solely
        // to BaseDerivedTextureShipResource, never to the physics material map.
        let source_data = ship_data::ShipData::new(image, palette);
        let image = &source_data.img;
        let mut solid = vec![false; width * height];
        let mut cell_materials = vec![None; width * height];
        let mut texel_solid = vec![false; source_width * source_height];
        let mut texel_materials = vec![None; source_width * source_height];
        for source_y in 0..source_height {
            for source_x in 0..source_width {
                let pixel = image.get_pixel(source_x as u32, source_y as u32).0;
                if !choice.material_map && pixel[3] <= 8 {
                    continue;
                }
                let material = source_data
                    .material_at(source_x as u32, source_y as u32)
                    .map(MaterialProperties::from)
                    .or_else(|| {
                        (!choice.material_map
                            && !white_background[source_y * source_width + source_x])
                            .then_some(MaterialProperties {
                                strength: 65.0,
                                tensile_strength: 65.0,
                                compressive_strength: 405.0,
                                mass: 2409.0,
                                hull: true,
                                ground: false,
                                rope: false,
                                invisible: false,
                            })
                    });
                if let Some(material) = material {
                    let index = source_y * source_width + source_x;
                    texel_solid[index] = true;
                    texel_materials[index] = Some(material);
                }
            }
        }
        for node_y in 0..height {
            for node_x in 0..width {
                let mut samples = MaterialSampleAccumulator::default();
                let y_end = ((node_y + 1) * PHYSICS_NODE_PIXELS).min(source_height);
                let x_end = ((node_x + 1) * PHYSICS_NODE_PIXELS).min(source_width);
                for source_y in node_y * PHYSICS_NODE_PIXELS..y_end {
                    for source_x in node_x * PHYSICS_NODE_PIXELS..x_end {
                        if let Some(material) = texel_materials[source_y * source_width + source_x]
                        {
                            samples.add(material);
                        }
                    }
                }
                if let Some(material) = samples.into_properties() {
                    let index = node_y * width + node_x;
                    solid[index] = true;
                    cell_materials[index] = Some(material);
                }
            }
        }
        let interior = classify_interior_air(&solid, width, height);
        let strut_masks = build_strut_masks(&solid, width, height);
        let texel_strut_masks = build_strut_masks(&texel_solid, source_width, source_height);
        let texel_rest_positions =
            pos_vel_data::PosVelDataHolder::new(source_width, source_height).positions;
        let count = width * height;
        let rest_positions: Vec<Vec2> = (0..count)
            .map(|i| {
                let x = i % width;
                let y = i / width;
                Vec2::new(
                    -SHIP_HALF_WIDTH + (x as f32 + 0.5) / width as f32 * SHIP_HALF_WIDTH * 2.0,
                    half_height - (y as f32 + 0.5) / height as f32 * half_height * 2.0,
                )
            })
            .collect();
        let mut springs = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let a = y * width + x;
                if !solid[a] {
                    continue;
                }
                for (direction, (dx, dy)) in [(1isize, 0isize), (1, 1), (0, 1), (-1, 1)]
                    .into_iter()
                    .enumerate()
                {
                    if strut_masks[a] & (1 << direction) == 0 {
                        continue;
                    }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                        continue;
                    }
                    let b = ny as usize * width + nx as usize;
                    if solid[b] {
                        let fallback = MaterialProperties {
                            strength: 1.0,
                            tensile_strength: 1.0,
                            compressive_strength: 1.0,
                            mass: 1.0,
                            hull: false,
                            ground: false,
                            rope: false,
                            invisible: false,
                        };
                        let material_a = cell_materials[a].unwrap_or(fallback);
                        let material_b = cell_materials[b].unwrap_or(material_a);
                        springs.push(Spring {
                            a,
                            b,
                            direction: direction as u8,
                            rest_length: rest_positions[a].distance(rest_positions[b]),
                            tensile_strength: material_a
                                .tensile_strength
                                .min(material_b.tensile_strength),
                            compressive_strength: material_a
                                .compressive_strength
                                .min(material_b.compressive_strength),
                            broken: false,
                        });
                    }
                }
            }
        }
        Self {
            width,
            height,
            texel_width: source_width,
            texel_height: source_height,
            half_height,
            texel_solid,
            texel_strut_masks,
            texel_materials,
            texel_positions: texel_rest_positions.clone(),
            texel_rest_positions,
            solid,
            interior,
            strut_masks,
            materials: cell_materials,
            breached: vec![false; count],
            leaking: vec![false; count],
            positions: rest_positions.clone(),
            last_positions: rest_positions.clone(),
            rest_positions,
            springs,
            water: vec![0.0; count],
            flooding: 0.0,
            motion_position: Vec2::new(0.0, SEA_LEVEL),
            motion_velocity: Vec2::ZERO,
            manual_offset: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
        }
    }

    fn empty_fallback() -> Self {
        Self {
            width: 1,
            height: 1,
            texel_width: 1,
            texel_height: 1,
            half_height: 52.5,
            texel_solid: vec![false],
            texel_strut_masks: vec![0],
            texel_materials: vec![None],
            texel_rest_positions: vec![Vec2::ZERO],
            texel_positions: vec![Vec2::ZERO],
            solid: vec![true],
            interior: vec![false],
            strut_masks: vec![0],
            materials: vec![None],
            breached: vec![false],
            leaking: vec![false],
            rest_positions: vec![Vec2::ZERO],
            positions: vec![Vec2::ZERO],
            last_positions: vec![Vec2::ZERO],
            springs: Vec::new(),
            water: vec![0.0],
            flooding: 0.0,
            motion_position: Vec2::new(0.0, SEA_LEVEL),
            motion_velocity: Vec2::ZERO,
            manual_offset: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
        }
    }

    fn breach_near(&mut self, local: Vec2, radius: f32) -> bool {
        let ship_height = self.half_height * 2.0;
        let cell_w = SHIP_HALF_WIDTH * 2.0 / self.width as f32;
        let cell_h = ship_height / self.height as f32;
        let min_x = (((local.x - radius) / (SHIP_HALF_WIDTH * 2.0) + 0.5) * self.width as f32)
            .floor()
            .clamp(0.0, (self.width - 1) as f32) as usize;
        let max_x = (((local.x + radius) / (SHIP_HALF_WIDTH * 2.0) + 0.5) * self.width as f32)
            .ceil()
            .clamp(0.0, (self.width - 1) as f32) as usize;
        let min_y = (((self.half_height - (local.y + radius)) / ship_height) * self.height as f32)
            .floor()
            .clamp(0.0, (self.height - 1) as f32) as usize;
        let max_y = (((self.half_height - (local.y - radius)) / ship_height) * self.height as f32)
            .ceil()
            .clamp(0.0, (self.height - 1) as f32) as usize;
        let mut changed = false;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = (x as f32 + 0.5) * cell_w - SHIP_HALF_WIDTH - local.x;
                let dy = self.half_height - (y as f32 + 0.5) * cell_h - local.y;
                let i = y * self.width + x;
                if dx * dx + dy * dy <= radius * radius && self.solid[i] && !self.breached[i] {
                    self.breached[i] = true;
                    self.leaking[i] = true;
                    // Expose the eight surrounding material nodes to incoming
                    // water, matching the original's 8-neighbor mask topology.
                    for (dx, dy) in EIGHT_NEIGHBORS {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        if nx >= 0
                            && ny >= 0
                            && nx < self.width as isize
                            && ny < self.height as isize
                        {
                            let n = ny as usize * self.width + nx as usize;
                            if self.solid[n] && !self.breached[n] {
                                self.leaking[n] = true;
                            }
                        }
                    }
                    changed = true;
                }
            }
        }
        if changed {
            let breached = &self.breached;
            for spring in &mut self.springs {
                if breached[spring.a] || breached[spring.b] {
                    spring.broken = true;
                    self.strut_masks[spring.a] &= !(1 << spring.direction);
                    self.strut_masks[spring.b] &= !(1 << ((spring.direction + 4) % 8));
                }
            }
        }
        changed
    }

    fn repair_near(&mut self, local: Vec2, radius: f32) {
        let cell_w = SHIP_HALF_WIDTH * 2.0 / self.width as f32;
        let cell_h = self.half_height * 2.0 / self.height as f32;
        for i in 0..self.breached.len() {
            if !self.breached[i] {
                continue;
            }
            let x = i % self.width;
            let y = i / self.width;
            let dx = -SHIP_HALF_WIDTH + (x as f32 + 0.5) * cell_w - local.x;
            let dy = self.half_height - (y as f32 + 0.5) * cell_h - local.y;
            if dx * dx + dy * dy <= radius * radius {
                self.breached[i] = false;
            }
        }
        for i in 0..self.leaking.len() {
            if !self.leaking[i] {
                continue;
            }
            let x = i % self.width;
            let y = i / self.width;
            let still_open = EIGHT_NEIGHBORS.iter().any(|(dx, dy)| {
                let nx = x as isize + *dx;
                let ny = y as isize + *dy;
                nx >= 0
                    && ny >= 0
                    && nx < self.width as isize
                    && ny < self.height as isize
                    && self.breached[ny as usize * self.width + nx as usize]
            });
            self.leaking[i] = still_open;
        }
        let restorable: Vec<bool> = (0..self.breached.len())
            .map(|i| {
                let x = i % self.width;
                let y = i / self.width;
                let dx = -SHIP_HALF_WIDTH + (x as f32 + 0.5) * cell_w - local.x;
                let dy = self.half_height - (y as f32 + 0.5) * cell_h - local.y;
                dx * dx + dy * dy <= radius * radius && !self.breached[i]
            })
            .collect();
        for spring in &mut self.springs {
            if restorable[spring.a] && restorable[spring.b] {
                restore_spring(spring, &mut self.strut_masks);
            }
        }
    }

    fn flood_near(&mut self, local: Vec2, radius: f32) {
        let x = (((local.x / (SHIP_HALF_WIDTH * 2.0)) + 0.5) * self.width as f32)
            .floor()
            .clamp(0.0, (self.width - 1) as f32) as usize;
        let y = (((self.half_height - local.y) / (self.half_height * 2.0)) * self.height as f32)
            .floor()
            .clamp(0.0, (self.height - 1) as f32) as usize;
        let cell_w = SHIP_HALF_WIDTH * 2.0 / self.width as f32;
        let cell_h = self.half_height * 2.0 / self.height as f32;
        let min_x = x.saturating_sub((radius / cell_w).ceil() as usize);
        let max_x = (x + (radius / cell_w).ceil() as usize).min(self.width - 1);
        let min_y = y.saturating_sub((radius / cell_h).ceil() as usize);
        let max_y = (y + (radius / cell_h).ceil() as usize).min(self.height - 1);
        for ny in min_y..=max_y {
            for nx in min_x..=max_x {
                let i = ny * self.width + nx;
                if !self.interior[i] {
                    continue;
                }
                let dx = -SHIP_HALF_WIDTH + (nx as f32 + 0.5) * cell_w - local.x;
                let dy = self.half_height - (ny as f32 + 0.5) * cell_h - local.y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < radius {
                    self.water[i] = (self.water[i] + (1.0 - distance / radius) * 0.25).min(1.0);
                }
            }
        }
    }

    fn pump_near(&mut self, local: Vec2, radius: f32) {
        let x = (((local.x / (SHIP_HALF_WIDTH * 2.0)) + 0.5) * self.width as f32)
            .floor()
            .clamp(0.0, (self.width - 1) as f32) as usize;
        let y = (((self.half_height - local.y) / (self.half_height * 2.0)) * self.height as f32)
            .floor()
            .clamp(0.0, (self.height - 1) as f32) as usize;
        let rx = (radius / (SHIP_HALF_WIDTH * 2.0) * self.width as f32).ceil() as usize;
        let ry = (radius / (self.half_height * 2.0) * self.height as f32).ceil() as usize;
        for ny in y.saturating_sub(ry)..=(y + ry).min(self.height - 1) {
            for nx in x.saturating_sub(rx)..=(x + rx).min(self.width - 1) {
                let i = ny * self.width + nx;
                if self.interior[i] && self.water[i] > 0.0 {
                    let dx = -SHIP_HALF_WIDTH
                        + (nx as f32 + 0.5) * (SHIP_HALF_WIDTH * 2.0 / self.width as f32)
                        - local.x;
                    let dy = self.half_height
                        - (ny as f32 + 0.5) * (self.half_height * 2.0 / self.height as f32)
                        - local.y;
                    if dx * dx + dy * dy <= radius * radius {
                        self.water[i] = (self.water[i] - 0.35).max(0.0);
                    }
                }
            }
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            flooding: 0.0,
            paused: false,
            tool: Tool::Break,
            // The only ship with a complete source material map and soft-body
            // setup in this port is Titanic, so start on the matching asset.
            ship_index: 0,
            selected_layer: 0,
            ship_scroll: 0,
            ship_search: String::new(),
            ship_search_active: false,
            pump_enabled: false,
            wave_amplitude: 1.0,
            water_flow: 60.0,
            buoyancy: 1.0,
            wave_width: 40.0,
            sea_depth: 400.0,
            drag: 1.0,
            water_influx: 1.0,
            water_funk: 0.0,
            gravity: 9.81,
            strength: 1.0,
            rigidity: 1.0,
            damping: 1.0,
            water_weight: 1.0,
            thickness: 0.085,
            water_darkness: 1.0,
            sea_color: Vec3::new(0.0, 71.0 / 255.0, 159.0 / 255.0),
            sea_alpha: game_parameters::GameParameterProvider::default()
                .water_color
                .w,
            sea_hue: rgb_to_hue(Vec3::new(0.0, 71.0 / 255.0, 159.0 / 255.0)),
            cycle_length: 120.0,
            day: 1.0,
            cycle_enabled: true,
            show_tools: true,
            toolbox_collapsed: false,
            physics_iterations: 50.0,
            water_steps: 5.0,
            tool_size: DEFAULT_TOOL_SIZE,
            music_playing: true,
            music_volume: 0.25,
            show_internal_water: true,
            active_tab: ToolboxTab::Ships,
        }
    }
}

impl Simulation {
    fn water_color(&self) -> Vec4 {
        self.sea_color.extend(self.sea_alpha)
    }
    fn setting(&self, index: usize) -> f32 {
        match index {
            0 => self.wave_width,
            1 => self.wave_amplitude,
            2 => self.sea_depth,
            3 => self.buoyancy,
            4 => self.drag,
            5 => self.water_flow,
            6 => self.water_influx,
            7 => self.water_funk,
            8 => self.gravity,
            9 => self.strength,
            10 => self.rigidity,
            11 => self.damping,
            12 => self.water_weight,
            13 => self.thickness,
            14 => self.cycle_length,
            15 => self.water_darkness,
            16 => self.physics_iterations,
            17 => self.water_steps,
            18 => self.day,
            19 => self.tool_size,
            20 => self.music_volume,
            21 => self.sea_color.x,
            22 => self.sea_color.y,
            23 => self.sea_color.z,
            24 => self.sea_alpha,
            _ => 0.0,
        }
    }
    fn adjust(&mut self, index: usize, delta: f32) {
        let target = match index {
            0 => &mut self.wave_width,
            1 => &mut self.wave_amplitude,
            2 => &mut self.sea_depth,
            3 => &mut self.buoyancy,
            4 => &mut self.drag,
            5 => &mut self.water_flow,
            6 => &mut self.water_influx,
            7 => &mut self.water_funk,
            8 => &mut self.gravity,
            9 => &mut self.strength,
            10 => &mut self.rigidity,
            11 => &mut self.damping,
            12 => &mut self.water_weight,
            13 => &mut self.thickness,
            14 => &mut self.cycle_length,
            15 => &mut self.water_darkness,
            16 => &mut self.physics_iterations,
            17 => &mut self.water_steps,
            18 => {
                self.day = (self.day + delta).clamp(0.0, 1.0);
                return;
            }
            19 => {
                self.tool_size = (self.tool_size + delta * 0.1).clamp(0.1, 4.0);
                return;
            }
            20 => {
                self.music_volume = (self.music_volume + delta).clamp(0.0, 1.0);
                return;
            }
            21 => {
                self.sea_color.x = (self.sea_color.x + delta).clamp(0.0, 1.0);
                let (hue, saturation, _) = rgb_to_hsv(self.sea_color);
                if saturation > 0.0 {
                    self.sea_hue = hue;
                }
                return;
            }
            22 => {
                self.sea_color.y = (self.sea_color.y + delta).clamp(0.0, 1.0);
                let (hue, saturation, _) = rgb_to_hsv(self.sea_color);
                if saturation > 0.0 {
                    self.sea_hue = hue;
                }
                return;
            }
            23 => {
                self.sea_color.z = (self.sea_color.z + delta).clamp(0.0, 1.0);
                let (hue, saturation, _) = rgb_to_hsv(self.sea_color);
                if saturation > 0.0 {
                    self.sea_hue = hue;
                }
                return;
            }
            24 => {
                self.sea_alpha = (self.sea_alpha + delta).clamp(0.0, 1.0);
                return;
            }
            _ => return,
        };
        *target = (*target + delta).max(if index == 13 {
            0.005
        } else if index == 15 {
            0.1
        } else {
            0.0
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolboxTab {
    Ships,
    Physics,
    Graphics,
    Music,
    Performance,
    Advanced,
}

#[derive(Clone, Copy)]
enum SettingAction {
    Adjust(usize, f32),
    ToggleCycle,
    ToggleTools,
    ToggleWater,
    ToggleMusic,
    ToggleShuffle,
    ToggleRepeat,
    NextTrack,
}

#[derive(Clone, Copy)]
enum SettingValue {
    Number(usize),
}

#[derive(Clone)]
struct ShipChoice {
    name: String,
    asset: String,
    physics_asset: String,
    material_map: bool,
    scale: f32,
}

#[derive(Clone)]
struct ShipLayerChoice {
    name: ShipLayer,
    asset: String,
}

#[derive(Resource)]
struct ShipCatalog(Vec<ShipChoice>, Vec<Vec<ShipLayerChoice>>);

impl ShipCatalog {
    fn discover() -> Self {
        use std::path::{Path, PathBuf};

        let root = Path::new("assets/ss2_ships");
        let mut choices = Vec::new();
        for (name, asset, physics_asset, mapped) in [
            (
                "RMS Titanic",
                "assets/ships/Titanic.png",
                "assets/ships/Titanic_base.png",
                true,
            ),
            (
                "SS Normandie",
                "assets/ships/SS_Normandie_CPM_keyed.png",
                "assets/ships/SS_Normandie_CPM_keyed.png",
                false,
            ),
            (
                "Queen Mary",
                "assets/ships/RMS_Queen_Mary_keyed.png",
                "assets/ships/RMS_Queen_Mary_keyed.png",
                false,
            ),
            (
                "Leviathan",
                "assets/ships/Leviathan_keyed.png",
                "assets/ships/Leviathan_keyed.png",
                false,
            ),
        ] {
            // Earlier prototype names may not exist in the extracted asset set.
            // Never add a selectable ship whose appearance or BASE is missing.
            if !Path::new(asset).is_file() || !Path::new(physics_asset).is_file() {
                continue;
            }
            push_ship_choice(
                &mut choices,
                Path::new(asset),
                Path::new(physics_asset),
                name.to_owned(),
                mapped,
            );
        }
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || !is_png(&path) {
                    continue;
                }
                let name = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Ship")
                    .replace('_', " ");
                push_ship_choice(&mut choices, &path, &path, name, false);
            }
        }

        fn collect_material_maps(folder: &Path, root: &Path, choices: &mut Vec<ShipChoice>) {
            let Ok(entries) = std::fs::read_dir(folder) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().and_then(|n| n.to_str()) == Some("Structure Pack") {
                        continue;
                    }
                    collect_material_maps(&path, root, choices);
                    continue;
                }
                if !is_png(&path) {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                let Some(ship_stem) = stem.strip_suffix("_base") else {
                    continue;
                };
                if ship_stem.eq_ignore_ascii_case("base") || ship_stem.is_empty() {
                    continue;
                }
                let parent = path.parent().unwrap_or(root);
                let candidates = [
                    format!("{ship_stem}_texture.png"),
                    format!("{ship_stem}_exterior_texture.png"),
                    format!("{ship_stem}_Outside_texture.png"),
                ];
                let texture = candidates
                    .iter()
                    .map(|candidate| parent.join(candidate))
                    .find(|candidate| candidate.is_file())
                    .unwrap_or_else(|| path.clone());
                let name = ship_stem.replace('_', " ");
                push_ship_choice(choices, &texture, &path, name, true);
            }
        }
        collect_material_maps(root, root, &mut choices);

        // ShipResource.fromFile in the original game groups *_BASE, *_TEXTURE,
        // *_INLIGHTS and *_EXLIGHTS files by ship and optional Layer.
        let mut source_resources = HashMap::new();
        collect_source_ship_resources(Path::new("assets/source_ships"), &mut source_resources);
        for resources in source_resources.values() {
            let Ok(thumbnail) = ship_thumbnail::ShipThumbnail::new(resources.clone()) else {
                continue;
            };
            let Some(ship_thumbnail::ThumbnailResource::File(base)) =
                thumbnail.get_resource(ShipResourceType::Base, &ShipLayer::default())
            else {
                continue;
            };
            let appearance =
                match thumbnail.get_resource(ShipResourceType::Texture, &ShipLayer::default()) {
                    Some(ship_thumbnail::ThumbnailResource::File(resource)) => &resource.path,
                    _ => &base.path,
                };
            push_ship_choice(
                &mut choices,
                appearance,
                &base.path,
                thumbnail.name().unwrap_or(&base.ship).replace('_', " "),
                true,
            );
        }

        choices.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        choices.dedup_by(|a, b| a.asset == b.asset && a.physics_asset == b.physics_asset);
        if let Some(initial_ship) = choices
            .iter()
            .position(|choice| choice.physics_asset.contains("/pacmaster/"))
        {
            choices.swap(0, initial_ship);
        }
        if choices.is_empty() {
            let fallback = PathBuf::from("assets/ships/Titanic.png");
            push_ship_choice(
                &mut choices,
                &fallback,
                &fallback,
                "RMS Titanic".to_owned(),
                false,
            );
        }

        let mut layers: Vec<Vec<ShipLayerChoice>> = choices
            .iter()
            .map(|choice| {
                vec![ShipLayerChoice {
                    name: ShipLayer::default(),
                    asset: choice.asset.clone(),
                }]
            })
            .collect();
        for (index, choice) in choices.iter().enumerate() {
            let key = normalized_ship_key(&choice.name);
            for resources in source_resources.values() {
                for resource in resources {
                    if normalized_ship_key(&resource.ship) != key
                        || resource.resource_type != ShipResourceType::Texture
                        || !resource.resource_type.is_layered()
                        || resource.layer.is_default()
                    {
                        continue;
                    }
                    let layer_assets = &mut layers[index];
                    if layer_assets
                        .iter()
                        .any(|existing| existing.name == resource.layer)
                    {
                        continue;
                    }
                    layer_assets.push(ShipLayerChoice {
                        name: resource.layer.clone(),
                        asset: asset_path(&resource.path),
                    });
                }
            }
            layers[index][1..].sort_by(|a, b| {
                a.name
                    .name()
                    .to_ascii_lowercase()
                    .cmp(&b.name.name().to_ascii_lowercase())
            });
        }
        Self(choices, layers)
    }

    fn layer_asset(&self, ship_index: usize, layer_index: usize) -> Option<&str> {
        self.1
            .get(ship_index)
            .and_then(|layers| layers.get(layer_index).or_else(|| layers.first()))
            .map(|layer| layer.asset.as_str())
    }

    fn layer_name(&self, ship_index: usize, layer_index: usize) -> Option<&str> {
        self.1
            .get(ship_index)
            .and_then(|layers| layers.get(layer_index).or_else(|| layers.first()))
            .map(|layer| layer.name.name())
    }

    fn add_imported(&mut self, choice: ShipChoice) -> usize {
        if let Some(index) = self
            .0
            .iter()
            .position(|existing| existing.asset == choice.asset)
        {
            return index;
        }
        let default_layer = ShipLayerChoice {
            name: ShipLayer::default(),
            asset: choice.asset.clone(),
        };
        self.0.push(choice);
        self.1.push(vec![default_layer]);
        self.0.len() - 1
    }
}

fn normalized_ship_key(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn asset_path(path: &std::path::Path) -> String {
    path.strip_prefix("assets")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_owned()
}

fn collect_source_ship_resources(
    folder: &std::path::Path,
    resources: &mut HashMap<String, Vec<ShipResourceFile>>,
) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) != Some("Structure Pack") {
                collect_source_ship_resources(&path, resources);
            }
            continue;
        }
        let Ok(resource) = parse_resource_path(&path) else {
            continue;
        };
        resources
            .entry(resource.ship.clone())
            .or_default()
            .push(resource);
    }
}

fn is_png(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}

fn push_ship_choice(
    choices: &mut Vec<ShipChoice>,
    appearance: &std::path::Path,
    material_map: &std::path::Path,
    name: String,
    has_material_map: bool,
) {
    let to_asset_path = |path: &std::path::Path| {
        path.strip_prefix("assets")
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_owned()
    };
    let asset = to_asset_path(appearance);
    let physics_asset = to_asset_path(material_map);
    let (width, _) = image::image_dimensions(appearance).unwrap_or((1000, 250));
    let scale = (360.0 / width.max(1) as f32).clamp(0.01, 1.0);
    choices.push(ShipChoice {
        name,
        asset,
        physics_asset,
        material_map: has_material_map,
        scale,
    });
}

fn spawn_ship_catalog_card(
    commands: &mut Commands,
    assets: &AssetServer,
    index: usize,
    choice: &ShipChoice,
    visible: bool,
) {
    let y = 122.0 - index as f32 * 57.0;
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        ToolboxContent,
        ShipCard(index),
        Sprite::from_color(Color::srgb(0.23, 0.27, 0.40), Vec2::new(326.0, 48.0)),
        Transform::from_xyz(-465.0, y, 21.0),
        RenderLayers::layer(1),
        visibility,
    ));
    let mut thumbnail = Sprite::from_image(assets.load(choice.asset.clone()));
    thumbnail.custom_size = Some(Vec2::new(100.0, 26.0));
    commands.spawn((
        ToolboxContent,
        ShipThumbnail(index),
        thumbnail,
        Transform::from_xyz(-575.0, y, 22.0),
        RenderLayers::layer(1),
        visibility,
    ));
    commands.spawn((
        ToolboxContent,
        ShipNameLabel(index),
        Text2d::new(choice.name.clone()),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-480.0, y, 22.0),
        RenderLayers::layer(1),
        visibility,
    ));
}

#[derive(Component)]
struct OceanDepth {
    mesh: Handle<Mesh>,
    material: Handle<OceanDepthMaterial>,
    width: f32,
    depth: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct OceanDepthMaterial {
    #[uniform(0)]
    surface_color: LinearRgba,
    #[uniform(1)]
    deep_color: LinearRgba,
    #[uniform(2)]
    params: Vec4,
}

#[derive(Component)]
struct UnderwaterEffect {
    mesh: Handle<Mesh>,
    material: Handle<UnderwaterEffectMaterial>,
    width: f32,
    depth: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct UnderwaterEffectMaterial {
    #[uniform(0)]
    params: Vec4,
    #[uniform(1)]
    color: LinearRgba,
}

impl Material2d for UnderwaterEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/underwater_effect.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl Material2d for OceanDepthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ocean_depth.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

#[derive(Component)]
struct OceanSurface {
    mesh: Handle<Mesh>,
    material: Handle<OceanSurfaceMaterial>,
    width: f32,
    height: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct OceanSurfaceMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(1)]
    params: Vec4,
}

impl Material2d for OceanSurfaceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ocean_surface.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

#[derive(Component)]
struct ShipSprite;

#[derive(Component)]
struct WorldCamera;

#[derive(Component)]
struct ShipMesh(Handle<Mesh>, Handle<ShipMaterial>);

#[derive(Component)]
struct InternalWaterMesh {
    mesh: Handle<Mesh>,
    material: Handle<InternalWaterMaterial>,
    cells: Vec<usize>,
    width: usize,
    height: usize,
    interior: Vec<bool>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct InternalWaterMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(1)]
    params: Vec4,
}

impl Material2d for InternalWaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/internal_water.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
#[derive(Component)]
struct MusicStatus;

#[derive(Component)]
struct MeshSyncState(Vec<bool>, Vec<[u32; 4]>);

#[derive(Component)]
struct ReflectionMesh {
    mesh: Handle<Mesh>,
    material: Handle<ReflectionMaterial>,
    half_height: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ReflectionMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    tint: LinearRgba,
    #[uniform(3)]
    params: Vec4,
}

impl Material2d for ReflectionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ship_reflection.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct LeakMarker(Vec2);

#[derive(Component)]
struct DamageBrushPreview;

#[derive(Component)]
struct ShipCard(usize);

#[derive(Component)]
struct ShipThumbnail(usize);

#[derive(Component)]
struct ShipNameLabel(usize);

#[derive(Component)]
struct ToolCard(Tool);

#[derive(Component)]
struct ToolGlyph;

#[derive(Component)]
struct ToolPanelUi;

#[derive(Component)]
struct ShipLayerLabel;

#[derive(Component)]
struct TabButton(ToolboxTab);

#[derive(Component)]
struct TabPage(ToolboxTab);

#[derive(Component)]
struct SettingButton(SettingAction);

#[derive(Component)]
struct SettingToggleMark(SettingAction);

#[derive(Component)]
struct SettingRow(usize);

#[derive(Component)]
struct SettingReadout(SettingValue);

#[derive(Component)]
struct ShipSearchText;

#[derive(Component)]
struct SeaColorPreview(bool);

#[derive(Component)]
struct SeaAlphaPicker(Handle<Mesh>);

#[derive(Component)]
struct SeaAlphaSelector;

#[derive(Component)]
struct SeaColorPicker(Handle<Mesh>);

#[derive(Component)]
struct SeaColorSelector;

#[derive(Component)]
struct SeaColorReadout(usize);

#[derive(Component)]
struct ToolboxContent;

#[derive(Component)]
struct ToolboxHeader;

#[derive(Component)]
struct ToolboxTitle;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ship_materials: ResMut<Assets<ShipMaterial>>,
    mut water_materials: ResMut<Assets<InternalWaterMaterial>>,
    mut brush_materials: ResMut<Assets<tools::brush_preview::BrushMaterial>>,
    mut reflection_materials: ResMut<Assets<ReflectionMaterial>>,
    mut ocean_materials: ResMut<Assets<OceanSurfaceMaterial>>,
    mut depth_materials: ResMut<Assets<OceanDepthMaterial>>,
    mut underwater_materials: ResMut<Assets<UnderwaterEffectMaterial>>,
    structure: Res<ShipStructure>,
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
) {
    let gpu_physics = make_gpu_ship_physics_assets(&structure, &mut shader_buffers);
    commands.insert_resource(gpu_physics.clone());
    commands
        .spawn(Readback::buffer(gpu_physics.positions))
        .observe(capture_gpu_ship_physics_readback);
    commands
        .spawn(Readback::buffer(gpu_physics.water.clone()))
        .observe(capture_gpu_water_readback);
    commands
        .spawn(Readback::buffer(gpu_physics.masks.clone()))
        .observe(capture_gpu_mask_readback);
    let scaled_projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 720.0,
        },
        scale: 1.0,
        ..OrthographicProjection::default_2d()
    });
    commands.spawn((
        Camera2d,
        scaled_projection.clone(),
        WorldCamera,
        RenderLayers::layer(0),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        scaled_projection,
        RenderLayers::layer(1),
    ));

    // Layered sea bands retain the original blue depth gradient.
    let depth_mesh = meshes.add(ocean_depth_mesh(WORLD_WIDTH, simulation.sea_depth));
    let depth_material = depth_materials.add(OceanDepthMaterial {
        surface_color: LinearRgba::new(
            simulation.sea_color.x,
            simulation.sea_color.y,
            simulation.sea_color.z,
            1.0,
        ),
        deep_color: LinearRgba::new(
            simulation.sea_color.x * 0.20,
            simulation.sea_color.y * 0.48,
            simulation.sea_color.z * 0.78,
            1.0,
        ),
        params: Vec4::new(
            simulation.sea_depth,
            (1.0 / simulation.water_darkness.max(0.1)).clamp(0.2, 1.5),
            SEA_LEVEL,
            0.0,
        ),
    });
    commands.spawn((
        OceanDepth {
            mesh: depth_mesh.clone(),
            material: depth_material.clone(),
            width: WORLD_WIDTH,
            depth: simulation.sea_depth,
        },
        Mesh2d(depth_mesh),
        MeshMaterial2d(depth_material),
        Transform::from_xyz(0.0, SEA_LEVEL, -4.0),
    ));
    let underwater_mesh = meshes.add(ocean_depth_mesh(WORLD_WIDTH, simulation.sea_depth));
    let underwater_material = underwater_materials.add(UnderwaterEffectMaterial {
        params: Vec4::new(simulation.elapsed, simulation.wave_width, SEA_LEVEL, 0.0),
        color: LinearRgba::new(
            simulation.sea_color.x * 0.25,
            simulation.sea_color.y * 0.50,
            simulation.sea_color.z * 0.75,
            1.0,
        ),
    });
    commands.spawn((
        UnderwaterEffect {
            mesh: underwater_mesh.clone(),
            material: underwater_material.clone(),
            width: WORLD_WIDTH,
            depth: simulation.sea_depth,
        },
        Mesh2d(underwater_mesh),
        MeshMaterial2d(underwater_material),
        Transform::from_xyz(0.0, SEA_LEVEL, 1.0),
    ));
    let ocean_mesh = meshes.add(ocean_surface_mesh(
        WORLD_WIDTH,
        (simulation.wave_amplitude).max(1.0) + 2.0,
    ));
    let ocean_material = ocean_materials.add(OceanSurfaceMaterial {
        color: LinearRgba::new(
            simulation.sea_color.x * 1.15,
            simulation.sea_color.y * 1.15,
            simulation.sea_color.z * 1.15,
            1.0,
        ),
        params: Vec4::new(
            std::f32::consts::PI / simulation.wave_width.max(0.001),
            simulation.elapsed,
            simulation.wave_amplitude,
            0.0,
        ),
    });
    commands.spawn((
        OceanSurface {
            mesh: ocean_mesh.clone(),
            material: ocean_material.clone(),
            width: WORLD_WIDTH,
            height: simulation.wave_amplitude + 2.0,
        },
        Mesh2d(ocean_mesh),
        MeshMaterial2d(ocean_material),
        Transform::from_xyz(0.0, SEA_LEVEL, -3.0),
    ));

    let initial_choice = &catalog.0[simulation.ship_index.min(catalog.0.len() - 1)];
    let ship = load_ship_visual_asset(initial_choice, &initial_choice.asset, &assets, &mut images);
    commands.spawn((
        ShipSprite,
        Sprite::from_image(ship.clone()),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0).with_scale(Vec3::ONE),
        Visibility::Hidden,
    ));
    let mesh = build_deformable_ship_mesh(&structure);
    let mesh_handle = meshes.add(mesh);
    let texture = ship.clone();
    let (internal_lights, external_lights) =
        load_ship_light_assets(initial_choice, &ShipLayer::default(), &assets, &mut images);
    let ship_material = ship_materials.add(ShipMaterial {
        texture,
        internal_lights,
        external_lights,
        params: Vec4::new(
            simulation.day,
            u8::from(simulation.show_internal_water) as f32,
            structure.texel_width as f32,
            structure.texel_height as f32,
        ),
        sea_color: simulation.water_color(),
        water: gpu_physics.water.clone(),
        masks: gpu_physics.masks.clone(),
    });
    commands.spawn((
        ShipMesh(mesh_handle.clone(), ship_material.clone()),
        MeshSyncState(structure.breached.clone(), gpu_mask_data(&structure)),
        Mesh2d(mesh_handle),
        MeshMaterial2d(ship_material.clone()),
        Visibility::Inherited,
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
    ));
    let strut_masks = gpu_mask_data(&structure);
    let strut_mesh = meshes.add(ship_struts::build_mesh(&structure, &strut_masks));
    commands.spawn((
        ship_struts::ShipStruts {
            mesh: strut_mesh.clone(),
            dimensions: (structure.texel_width, structure.texel_height),
            masks: strut_masks,
        },
        Mesh2d(strut_mesh),
        MeshMaterial2d(ship_material),
        Transform::from_xyz(0.0, SEA_LEVEL, -0.001),
    ));
    let (water_mesh_data, water_cells) = build_internal_water_mesh(&structure);
    let water_mesh = meshes.add(water_mesh_data);
    let water_material = water_materials.add(InternalWaterMaterial {
        color: LinearRgba::new(0.02, 0.24, 0.68, 0.78),
        params: Vec4::new(
            std::f32::consts::PI / simulation.wave_width.max(0.001),
            simulation.elapsed,
            simulation.wave_amplitude,
            0.0,
        ),
    });
    commands.spawn((
        InternalWaterMesh {
            mesh: water_mesh.clone(),
            material: water_material.clone(),
            cells: water_cells,
            width: structure.width,
            height: structure.height,
            interior: structure.interior.clone(),
        },
        Mesh2d(water_mesh),
        MeshMaterial2d(water_material),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.5),
    ));
    // Source shader extends the half-opacity disk by distance/10 at its edge.
    // A 2.3-unit quad covers the complete falloff when scaled by tool radius.
    let brush_mesh = meshes.add(Rectangle::new(2.3, 2.3));
    let brush_material = brush_materials.add(tools::brush_preview::BrushMaterial {
        cursor_radius: Vec4::new(0.0, 0.0, 1.0, 0.0),
        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
    });
    commands.spawn((
        DamageBrushPreview,
        Mesh2d(brush_mesh),
        MeshMaterial2d(brush_material),
        Transform::from_xyz(0.0, 0.0, 8.0),
        Visibility::Hidden,
    ));
    let reflection_mesh = meshes.add(reflection_mesh(&structure));
    let reflection_material = reflection_materials.add(ReflectionMaterial {
        texture: ship,
        tint: LinearRgba::new(0.45, 0.65, 0.90, 0.30),
        params: Vec4::new(
            std::f32::consts::PI / simulation.wave_width.max(0.001),
            simulation.elapsed,
            simulation.wave_amplitude,
            SEA_LEVEL,
        ),
    });
    commands.spawn((
        ReflectionMesh {
            mesh: reflection_mesh.clone(),
            material: reflection_material.clone(),
            half_height: structure.half_height,
        },
        Mesh2d(reflection_mesh),
        MeshMaterial2d(reflection_material),
        Transform::from_xyz(0.0, SEA_LEVEL, -2.0).with_scale(Vec3::splat(initial_choice.scale)),
    ));

    // Toolbox panel and tabs inspired by the original game layout.
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.32, 0.35, 0.52), Vec2::new(356.0, 706.0)),
        Transform::from_xyz(-465.0, 0.0, 19.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(
            Color::srgba(0.08, 0.10, 0.16, 0.96),
            Vec2::new(350.0, 700.0),
        ),
        Transform::from_xyz(-465.0, 0.0, 20.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxHeader,
        Sprite::from_color(Color::srgb(0.25, 0.29, 0.48), Vec2::new(350.0, 34.0)),
        Transform::from_xyz(-465.0, 333.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxTitle,
        Text2d::new("▼  Toolbox"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-585.0, 333.0, 22.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.12, 0.15, 0.23), Vec2::new(330.0, 34.0)),
        Transform::from_xyz(-465.0, 294.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.12, 0.14, 0.20), Vec2::new(8.0, 632.0)),
        Transform::from_xyz(-297.0, -14.0, 21.5),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.23, 0.28, 0.43), Vec2::new(7.0, 48.0)),
        Transform::from_xyz(-297.0, 214.0, 22.0),
        RenderLayers::layer(1),
    ));
    let tabs = [
        (ToolboxTab::Ships, "Ships", 42.0),
        (ToolboxTab::Physics, "Physics", 52.0),
        (ToolboxTab::Graphics, "Graphics", 58.0),
        (ToolboxTab::Music, "Music", 43.0),
        (ToolboxTab::Performance, "Performance", 69.0),
        (ToolboxTab::Advanced, "Advanced", 60.0),
    ];
    let mut tab_left = -628.0;
    for (tab, label, width) in tabs {
        let x = tab_left + width * 0.5;
        commands.spawn((
            ToolboxContent,
            TabButton(tab),
            Sprite::from_color(Color::srgb(0.24, 0.29, 0.46), Vec2::new(width - 2.0, 26.0)),
            Transform::from_xyz(x, 294.0, 22.0),
            RenderLayers::layer(1),
        ));
        spawn_label(
            &mut commands,
            label,
            Vec3::new(x, 294.0, 23.0),
            10.0,
            Color::WHITE,
        );
        tab_left += width;
    }
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(225.0, 30.0)),
        Transform::from_xyz(-514.0, 253.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        ShipSearchText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-514.0, 253.0, 22.0),
        RenderLayers::layer(1),
    ));
    spawn_label(
        &mut commands,
        "Search",
        Vec3::new(-391.0, 253.0, 22.0),
        16.0,
        Color::WHITE,
    );
    for (text, y) in [
        ("Edit current ship", 214.0),
        ("Import PNG (drop onto window)", 174.0),
    ] {
        commands.spawn((
            ToolboxContent,
            Sprite::from_color(Color::srgb(0.24, 0.28, 0.43), Vec2::new(326.0, 34.0)),
            Transform::from_xyz(-465.0, y, 21.0),
            RenderLayers::layer(1),
        ));
        spawn_label(
            &mut commands,
            text,
            Vec3::new(-465.0, y, 22.0),
            16.0,
            Color::WHITE,
        );
    }
    for (index, choice) in catalog.0.iter().enumerate() {
        spawn_ship_catalog_card(&mut commands, &assets, index, choice, index < 4);
    }

    // Clickable settings pages replace the original inert tab labels.
    for tab in [
        ToolboxTab::Physics,
        ToolboxTab::Graphics,
        ToolboxTab::Music,
        ToolboxTab::Performance,
        ToolboxTab::Advanced,
    ] {
        commands.spawn((
            TabPage(tab),
            ToolboxContent,
            Sprite::from_color(Color::srgb(0.10, 0.13, 0.20), Vec2::new(326.0, 600.0)),
            Transform::from_xyz(-465.0, -44.0, 24.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    let settings = [
        ("Wave Width", 0usize),
        ("Wave Height", 1),
        ("Sea Depth", 2),
        ("Buoyancy", 3),
        ("Drag", 4),
        ("Water Flow", 5),
        ("Water Influx", 6),
        ("Water funk", 7),
        ("Gravity", 8),
        ("Strength", 9),
        ("Rigidity", 10),
        ("Dampening", 11),
        ("Water Weight", 12),
        ("Thickness", 13),
    ];
    for (i, (label, id)) in settings.into_iter().enumerate() {
        let y = 248.0 - i as f32 * 44.0;
        spawn_setting_row(&mut commands, ToolboxTab::Physics, id, y);
        spawn_page_label(
            &mut commands,
            ToolboxTab::Physics,
            label,
            Vec3::new(-391.0, y, 25.0),
            15.0,
        );
        spawn_setting_readout(
            &mut commands,
            SettingValue::Number(id),
            Vec3::new(-514.0, y, 25.0),
        );
    }
    spawn_page_label(
        &mut commands,
        ToolboxTab::Graphics,
        "Show Tools",
        Vec3::new(-535.0, 220.0, 25.0),
        15.0,
    );
    spawn_setting_button(
        &mut commands,
        "✓",
        SettingAction::ToggleTools,
        Vec3::new(-618.0, 220.0, 25.0),
    );
    spawn_page_label(
        &mut commands,
        ToolboxTab::Graphics,
        "Cycle",
        Vec3::new(-535.0, 184.0, 25.0),
        15.0,
    );
    spawn_setting_button(
        &mut commands,
        "✓",
        SettingAction::ToggleCycle,
        Vec3::new(-618.0, 184.0, 25.0),
    );
    for (label, id, y) in [
        ("Cycle length", 14usize, 144.0),
        ("Day", 18, 108.0),
        ("Water Darkness", 15, 72.0),
    ] {
        spawn_setting_row(&mut commands, ToolboxTab::Graphics, id, y);
        spawn_page_label(
            &mut commands,
            ToolboxTab::Graphics,
            label,
            Vec3::new(-391.0, y, 25.0),
            15.0,
        );
        spawn_setting_readout(
            &mut commands,
            SettingValue::Number(id),
            Vec3::new(-514.0, y, 25.0),
        );
    }
    let color_picker_mesh = meshes.add(color_picker_mesh(simulation.sea_hue));
    let color_picker_material = materials.add(ColorMaterial::default());
    commands.spawn((
        SeaColorPicker(color_picker_mesh.clone()),
        TabPage(ToolboxTab::Graphics),
        ToolboxContent,
        Mesh2d(color_picker_mesh),
        MeshMaterial2d(color_picker_material),
        Transform::from_xyz(-533.0, -130.0, 25.5),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    let hue_bar = meshes.add(hue_bar_mesh());
    commands.spawn((
        TabPage(ToolboxTab::Graphics),
        ToolboxContent,
        Mesh2d(hue_bar),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::from_xyz(-432.0, -130.0, 25.5),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    let alpha_bar = meshes.add(alpha_bar_mesh(simulation.sea_color));
    commands.spawn((
        TabPage(ToolboxTab::Graphics),
        ToolboxContent,
        SeaAlphaPicker(alpha_bar.clone()),
        Mesh2d(alpha_bar),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::from_xyz(-412.0, -130.0, 25.4),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    // Source AlphaPreviewHalf: opaque colour on the left, alpha over checks on the right.
    for row in 0..8 {
        for column in 0..4 {
            let shade = if (row + column) % 2 == 0 { 0.72 } else { 0.46 };
            commands.spawn((
                TabPage(ToolboxTab::Graphics),
                ToolboxContent,
                Sprite::from_color(Color::srgb(shade, shade, shade), Vec2::splat(7.0)),
                Transform::from_xyz(-336.5 + column as f32 * 7.0, -89.5 + row as f32 * 7.0, 25.3),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ));
        }
    }
    for (alpha, x) in [(false, -354.0), (true, -326.0)] {
        commands.spawn((
            SeaColorPreview(alpha),
            TabPage(ToolboxTab::Graphics),
            ToolboxContent,
            Sprite::from_color(Color::WHITE, Vec2::new(28.0, 56.0)),
            Transform::from_xyz(x, -65.0, 25.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        SeaColorSelector,
        SeaAlphaSelector,
        TabPage(ToolboxTab::Graphics),
        ToolboxContent,
        Sprite::from_color(Color::WHITE, Vec2::new(18.0, 2.0)),
        Transform::from_xyz(-412.0, toolbox::alpha_marker_y(simulation.sea_alpha), 26.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    let color_selector_mesh = meshes.add(Annulus::new(5.0, 6.0));
    let color_selector_material = materials.add(ColorMaterial::from(Color::WHITE));
    commands.spawn((
        SeaColorSelector,
        TabPage(ToolboxTab::Graphics),
        ToolboxContent,
        Mesh2d(color_selector_mesh),
        MeshMaterial2d(color_selector_material),
        Transform::from_xyz(-440.0, -50.0, 26.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    spawn_page_label(
        &mut commands,
        ToolboxTab::Graphics,
        "Sea color",
        Vec3::new(-363.0, 52.0, 25.0),
        15.0,
    );
    for (index, label) in ["R", "G", "B", "A"].into_iter().enumerate() {
        let x = -602.0 + index as f32 * 62.0;
        commands.spawn((
            TabPage(ToolboxTab::Graphics),
            ToolboxContent,
            Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(58.0, 30.0)),
            Transform::from_xyz(x, -333.0, 24.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        commands.spawn((
            TabPage(ToolboxTab::Graphics),
            ToolboxContent,
            SeaColorReadout(index),
            Text2d::new(format!("{label}: 0")),
            TextFont {
                font_size: FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(x, -333.0, 25.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    spawn_page_label(
        &mut commands,
        ToolboxTab::Music,
        "MUSIC",
        Vec3::new(-465.0, 155.0, 25.0),
        15.0,
    );
    music_player::spawn_controls(&mut commands, &assets);
    commands.spawn((
        Text2d::new("Loading soundtrack…"),
        TextFont {
            font_size: FontSize::Px(11.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-445.0, 70.0, 25.0),
        RenderLayers::layer(1),
        TabPage(ToolboxTab::Music),
        MusicStatus,
        Visibility::Hidden,
    ));
    spawn_page_label(
        &mut commands,
        ToolboxTab::Music,
        "Volume",
        Vec3::new(-535.0, 35.0, 25.0),
        12.0,
    );
    spawn_setting_readout(
        &mut commands,
        SettingValue::Number(20),
        Vec3::new(-405.0, 35.0, 25.0),
    );
    spawn_setting_button(
        &mut commands,
        "−",
        SettingAction::Adjust(20, -0.05),
        Vec3::new(-322.0, 35.0, 25.0),
    );
    spawn_setting_button(
        &mut commands,
        "+",
        SettingAction::Adjust(20, 0.05),
        Vec3::new(-292.0, 35.0, 25.0),
    );
    for (label, id, y) in [("Physics", 16usize, 220.0), ("Water flow", 17, 184.0)] {
        spawn_setting_row(&mut commands, ToolboxTab::Performance, id, y);
        spawn_page_label(
            &mut commands,
            ToolboxTab::Performance,
            label,
            Vec3::new(-391.0, y, 25.0),
            15.0,
        );
        spawn_setting_readout(
            &mut commands,
            SettingValue::Number(id),
            Vec3::new(-514.0, y, 25.0),
        );
    }
    spawn_page_label(
        &mut commands,
        ToolboxTab::Advanced,
        "ADVANCED",
        Vec3::new(-465.0, 155.0, 25.0),
        15.0,
    );
    spawn_page_label(
        &mut commands,
        ToolboxTab::Advanced,
        "Flood, cut, repair, and pump tools",
        Vec3::new(-465.0, 95.0, 25.0),
        11.0,
    );
    spawn_page_label(
        &mut commands,
        ToolboxTab::Advanced,
        "Show flood water",
        Vec3::new(-535.0, 20.0, 25.0),
        12.0,
    );
    spawn_setting_button(
        &mut commands,
        "✓",
        SettingAction::ToggleWater,
        Vec3::new(-618.0, 20.0, 25.0),
    );
    spawn_page_label(
        &mut commands,
        ToolboxTab::Advanced,
        "Hole / tool size",
        Vec3::new(-530.0, 55.0, 25.0),
        12.0,
    );
    spawn_setting_readout(
        &mut commands,
        SettingValue::Number(19),
        Vec3::new(-405.0, 55.0, 25.0),
    );
    spawn_setting_button(
        &mut commands,
        "−",
        SettingAction::Adjust(19, -1.0),
        Vec3::new(-322.0, 55.0, 25.0),
    );
    spawn_setting_button(
        &mut commands,
        "+",
        SettingAction::Adjust(19, 1.0),
        Vec3::new(-292.0, 55.0, 25.0),
    );

    commands.spawn((
        ToolPanelUi,
        Sprite::from_color(
            Color::srgba(0.12, 0.30, 0.52, 0.91),
            Vec2::new(370.0, 190.0),
        ),
        Transform::from_xyz(-85.0, 255.0, 19.0),
        RenderLayers::layer(1),
    ));
    for (index, (tool, icon)) in [
        (Tool::Break, "icons/Break.png"),
        (Tool::Dry, "icons/Dry.png"),
        (Tool::Flood, "icons/Flood.png"),
        (Tool::Move, "icons/Move.png"),
    ]
    .into_iter()
    .enumerate()
    {
        let x = -220.0 + index as f32 * 78.0;
        commands.spawn((
            ToolCard(tool),
            Sprite::from_color(Color::srgb(0.78, 0.48, 0.02), Vec2::splat(56.0)),
            Transform::from_xyz(x, 250.0, 21.0),
            RenderLayers::layer(1),
        ));
        let mut glyph = Sprite::from_image(assets.load(icon));
        glyph.custom_size = Some(Vec2::splat(46.0));
        commands.spawn((
            ToolGlyph,
            glyph,
            Transform::from_xyz(x, 250.0, 22.0),
            RenderLayers::layer(1),
        ));
    }
    commands.spawn((
        ToolPanelUi,
        Sprite::from_color(Color::srgb(0.30, 0.40, 0.54), Vec2::new(230.0, 28.0)),
        Transform::from_xyz(-145.0, 310.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolPanelUi,
        ShipLayerLabel,
        Text2d::new("Default"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-145.0, 310.0, 22.0),
        RenderLayers::layer(1),
    ));
    spawn_tool_panel_label(
        &mut commands,
        "Show Layer",
        Vec3::new(22.0, 310.0, 22.0),
        15.0,
    );
    commands.spawn((
        ToolPanelUi,
        Sprite::from_color(Color::srgb(0.24, 0.29, 0.43), Vec2::new(22.0, 24.0)),
        Transform::from_xyz(-40.0, 310.0, 21.0),
        RenderLayers::layer(1),
    ));
    spawn_tool_panel_label(&mut commands, "▼", Vec3::new(-40.0, 310.0, 22.0), 14.0);
    commands.spawn((
        ToolPanelUi,
        SettingRow(19),
        Sprite::from_color(Color::srgb(0.30, 0.40, 0.54), Vec2::new(230.0, 28.0)),
        Transform::from_xyz(-145.0, 188.0, 21.0),
        RenderLayers::layer(1),
        Visibility::Inherited,
    ));
    spawn_tool_panel_label(
        &mut commands,
        "Tool Size",
        Vec3::new(20.0, 188.0, 22.0),
        15.0,
    );
    commands.spawn((
        ToolPanelUi,
        SettingReadout(SettingValue::Number(19)),
        Text2d::new("1.000"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-145.0, 188.0, 22.0),
        RenderLayers::layer(1),
    ));
    spawn_tool_panel_label(
        &mut commands,
        "You can hide the tools in the graphics settings",
        Vec3::new(-85.0, 163.0, 22.0),
        10.0,
    );

    commands.spawn((
        Hud,
        Text2d::new("SINKING SIMULATOR  /  BEVY PORT"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(175.0, 320.0, 10.0),
        RenderLayers::layer(1),
    ));
}

/// Combine current damage with GPU breakage. New tool holes must not restore
/// links previously broken by stress, even when a readback is one frame old.
fn current_render_masks(
    structure: &ShipStructure,
    snapshot: &GpuShipPhysicsSnapshot,
) -> Vec<[u32; 4]> {
    let mut masks = gpu_mask_data(structure);
    if snapshot.masks.len() == masks.len() {
        for (mask, live) in masks.iter_mut().zip(&snapshot.masks) {
            mask[1] &= live[1];
            mask[2] &= live[2];
        }
    }
    masks
}

fn build_deformable_ship_mesh(structure: &ShipStructure) -> Mesh {
    structure.validate_texel_data();
    let width = structure.texel_width;
    let height = structure.texel_height;
    let positions: Vec<[f32; 3]> = structure
        .texel_positions
        .iter()
        .map(|p| [p.x, p.y, 0.0])
        .collect();
    let uvs: Vec<[f32; 2]> = (0..width * height)
        .map(|i| {
            [
                (i % width) as f32 / width as f32 + 0.5 / width as f32,
                (i / width) as f32 / height as f32 + 0.5 / height as f32,
            ]
        })
        .collect();
    let indices = ship::Ship::triangle_indices(width, height, &gpu_mask_data(structure));
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0; 4]; width * height]);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn sync_deformed_mesh(
    structure: Res<ShipStructure>,
    snapshot: Res<GpuShipPhysicsSnapshot>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ship_meshes: Query<(&ShipMesh, &mut MeshSyncState)>,
) {
    let masks = current_render_masks(&structure, &snapshot);
    for (ship_mesh, mut sync_state) in &mut ship_meshes {
        let Some(mut mesh) = meshes.get_mut(&ship_mesh.0) else {
            continue;
        };
        if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            // Source geometry fetches each texel's own position. Averaging
            // neighboring displacement would keep severed pieces connected.
            for (vertex, position) in vertices.iter_mut().zip(&structure.texel_positions) {
                *vertex = [position.x, position.y, 0.0];
            }
        }
        if sync_state.1 != masks {
            mesh.insert_indices(Indices::U32(ship::Ship::triangle_indices(
                structure.texel_width,
                structure.texel_height,
                &masks,
            )));
            sync_state.1.clone_from(&masks);
        }
        sync_state.0.clone_from(&structure.breached);
    }
}

fn deformed_grid_vertex(structure: &ShipStructure, x: usize, y: usize) -> Vec2 {
    let cell_w = SHIP_HALF_WIDTH * 2.0 / structure.width as f32;
    let cell_h = structure.half_height * 2.0 / structure.height as f32;
    let mut displacement = Vec2::ZERO;
    let mut weights = 0.0;
    for ny in y.saturating_sub(2)..=(y + 1).min(structure.height.saturating_sub(1)) {
        for nx in x.saturating_sub(2)..=(x + 1).min(structure.width.saturating_sub(1)) {
            let index = ny * structure.width + nx;
            if !structure.solid[index] {
                continue;
            }
            let dx = nx as f32 + 0.5 - x as f32;
            let dy = ny as f32 + 0.5 - y as f32;
            let weight = 1.0 / (dx * dx + dy * dy + 0.25);
            displacement += (structure.positions[index] - structure.rest_positions[index]) * weight;
            weights += weight;
        }
    }
    if weights > 0.0 {
        displacement /= weights;
    }
    Vec2::new(
        -SHIP_HALF_WIDTH + x as f32 * cell_w,
        structure.half_height - y as f32 * cell_h,
    ) + displacement
}

fn build_internal_water_mesh(structure: &ShipStructure) -> (Mesh, Vec<usize>) {
    let cells = structure
        .interior
        .iter()
        .enumerate()
        .filter_map(|(index, is_interior)| is_interior.then_some(index))
        .collect::<Vec<_>>();
    let mut positions = Vec::with_capacity(cells.len() * 4);
    let mut uvs = Vec::with_capacity(cells.len() * 4);
    let mut indices = Vec::with_capacity(cells.len() * 6);
    if cells.is_empty() {
        // Bevy's mesh allocator cannot extract a zero-vertex mesh. A collapsed
        // triangle keeps the optional water renderer valid but invisible.
        positions.extend_from_slice(&[[0.0; 3]; 3]);
        uvs.extend_from_slice(&[[0.0; 2]; 3]);
        indices.extend_from_slice(&[0, 1, 2]);
    }
    for (cell_index, &index) in cells.iter().enumerate() {
        let x = index % structure.width;
        let y = index / structure.width;
        let bottom_left = deformed_grid_vertex(structure, x, y + 1);
        let bottom_right = deformed_grid_vertex(structure, x + 1, y + 1);
        let vertex_start = (cell_index * 4) as u32;
        positions.extend_from_slice(&[
            [bottom_left.x, bottom_left.y, 0.0],
            [bottom_right.x, bottom_right.y, 0.0],
            [bottom_right.x, bottom_right.y, 0.0],
            [bottom_left.x, bottom_left.y, 0.0],
        ]);
        uvs.extend_from_slice(&[[0.0, 1.0], [1.0, 1.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[
            vertex_start,
            vertex_start + 1,
            vertex_start + 2,
            vertex_start,
            vertex_start + 2,
            vertex_start + 3,
        ]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    (mesh, cells)
}

fn ocean_surface_mesh(width: f32, height: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-width * 0.5, 0.0, 0.0],
            [width * 0.5, 0.0, 0.0],
            [width * 0.5, height, 0.0],
            [-width * 0.5, height, 0.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn ocean_depth_mesh(width: f32, depth: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-width * 0.5, -depth, 0.0],
            [width * 0.5, -depth, 0.0],
            [width * 0.5, 0.0, 0.0],
            [-width * 0.5, 0.0, 0.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn reflection_mesh(structure: &ShipStructure) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-SHIP_HALF_WIDTH, 0.0, 0.0],
            [SHIP_HALF_WIDTH, 0.0, 0.0],
            [SHIP_HALF_WIDTH, -structure.half_height * 2.0, 0.0],
            [-SHIP_HALF_WIDTH, -structure.half_height * 2.0, 0.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn sync_internal_water_mesh(
    structure: Res<ShipStructure>,
    simulation: Res<Simulation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<InternalWaterMaterial>>,
    mut water_meshes: Query<&mut InternalWaterMesh>,
) {
    if !structure.is_changed() && !simulation.is_changed() {
        return;
    }
    let mut positions = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    for mut water_mesh in &mut water_meshes {
        if water_mesh.width != structure.width
            || water_mesh.height != structure.height
            || water_mesh.interior != structure.interior
        {
            let (mesh_data, cells) = build_internal_water_mesh(&structure);
            let Some(mut mesh) = meshes.get_mut(&water_mesh.mesh) else {
                continue;
            };
            *mesh = mesh_data;
            water_mesh.cells = cells;
            water_mesh.width = structure.width;
            water_mesh.height = structure.height;
            water_mesh.interior.clone_from(&structure.interior);
        }
        if water_mesh.cells.is_empty() {
            continue;
        }
        positions.clear();
        positions.reserve(water_mesh.cells.len() * 4);
        uvs.clear();
        uvs.reserve(water_mesh.cells.len() * 4);
        for &index in &water_mesh.cells {
            let x = index % structure.width;
            let y = index / structure.width;
            let fill = if simulation.show_internal_water {
                structure.water[index].clamp(0.0, 1.0)
            } else {
                0.0
            };
            let bottom_left = deformed_grid_vertex(&structure, x, y + 1);
            let bottom_right = deformed_grid_vertex(&structure, x + 1, y + 1);
            let top_left = deformed_grid_vertex(&structure, x, y);
            let top_right = deformed_grid_vertex(&structure, x + 1, y);
            let surface_fraction = fill;
            let surface_left = bottom_left.lerp(top_left, surface_fraction);
            let surface_right = bottom_right.lerp(top_right, surface_fraction);
            let surface_uv = 1.0 - fill;
            for point in [bottom_left, bottom_right, surface_right, surface_left] {
                positions.push([point.x, point.y, 0.0]);
            }
            uvs.extend_from_slice(&[[0.0, 1.0], [1.0, 1.0], [1.0, surface_uv], [0.0, surface_uv]]);
        }
        if let Some(mut material) = materials.get_mut(&water_mesh.material) {
            material.params = Vec4::new(
                std::f32::consts::PI / simulation.wave_width.max(0.001),
                simulation.elapsed,
                simulation.wave_amplitude,
                0.0,
            );
        }
        if let Some(mut mesh) = meshes.get_mut(&water_mesh.mesh) {
            if let Some(VertexAttributeValues::Float32x3(vertices)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                vertices.copy_from_slice(&positions);
            }
            if let Some(VertexAttributeValues::Float32x2(mesh_uvs)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                mesh_uvs.copy_from_slice(&uvs);
            }
        }
    }
}

fn spawn_label(commands: &mut Commands, text: &str, position: Vec3, size: f32, color: Color) {
    commands.spawn((
        ToolboxContent,
        Text2d::new(text.to_string()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        RenderLayers::layer(1),
    ));
}

fn spawn_tool_panel_label(commands: &mut Commands, text: &str, position: Vec3, size: f32) {
    commands.spawn((
        ToolPanelUi,
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(position),
        RenderLayers::layer(1),
        Visibility::Inherited,
    ));
}

fn spawn_page_label(
    commands: &mut Commands,
    tab: ToolboxTab,
    text: &str,
    position: Vec3,
    size: f32,
) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(position),
        RenderLayers::layer(1),
        ToolboxContent,
        TabPage(tab),
        Visibility::Hidden,
    ));
}

fn spawn_setting_row(commands: &mut Commands, tab: ToolboxTab, index: usize, y: f32) {
    commands.spawn((
        SettingRow(index),
        TabPage(tab),
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(225.0, 34.0)),
        Transform::from_xyz(-514.0, y, 24.5),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
}

fn spawn_setting_button(
    commands: &mut Commands,
    label: &str,
    action: SettingAction,
    position: Vec3,
) {
    let toggle = matches!(
        action,
        SettingAction::ToggleCycle | SettingAction::ToggleTools | SettingAction::ToggleWater
    );
    let page = match action {
        SettingAction::ToggleCycle | SettingAction::ToggleTools => ToolboxTab::Graphics,
        SettingAction::ToggleWater => ToolboxTab::Advanced,
        SettingAction::Adjust(14 | 15 | 18 | 21 | 22 | 23 | 24, _) => ToolboxTab::Graphics,
        SettingAction::Adjust(16 | 17, _) => ToolboxTab::Performance,
        SettingAction::Adjust(19, _) => ToolboxTab::Advanced,
        SettingAction::ToggleMusic
        | SettingAction::ToggleShuffle
        | SettingAction::ToggleRepeat
        | SettingAction::NextTrack
        | SettingAction::Adjust(20, _) => ToolboxTab::Music,
        _ => ToolboxTab::Physics,
    };
    commands.spawn((
        ToolboxContent,
        SettingButton(action),
        TabPage(page),
        Sprite::from_color(
            Color::srgb(0.32, 0.34, 0.36),
            if toggle {
                Vec2::splat(20.0)
            } else {
                Vec2::new(30.0, 26.0)
            },
        ),
        Transform::from_xyz(position.x, position.y, position.z - 0.5),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    if toggle {
        commands.spawn((
            Text2d::new(label),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(position + Vec3::new(0.0, -1.0, 0.5)),
            RenderLayers::layer(1),
            ToolboxContent,
            TabPage(page),
            SettingToggleMark(action),
            Visibility::Hidden,
        ));
    } else {
        spawn_page_label(commands, page, label, position, 13.0);
    }
}

fn spawn_setting_readout(commands: &mut Commands, value: SettingValue, position: Vec3) {
    let page = match value {
        SettingValue::Number(14 | 15 | 18 | 21 | 22 | 23 | 24) => ToolboxTab::Graphics,
        SettingValue::Number(16 | 17) => ToolboxTab::Performance,
        SettingValue::Number(19) => ToolboxTab::Advanced,
        SettingValue::Number(20) => ToolboxTab::Music,
        _ => ToolboxTab::Physics,
    };
    let initial = "1.0";
    commands.spawn((
        Text2d::new(initial),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.87, 0.92)),
        Transform::from_translation(position),
        RenderLayers::layer(1),
        ToolboxContent,
        TabPage(page),
        SettingReadout(value),
        Visibility::Hidden,
    ));
}

fn handle_controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<Simulation>,
    mut structure: ResMut<ShipStructure>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
    mut gpu_physics: ResMut<GpuShipPhysicsAssets>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    markers: Query<Entity, With<LeakMarker>>,
) {
    if keys.just_pressed(KeyCode::Space) && !simulation.ship_search_active {
        simulation.paused = !simulation.paused;
    }
    if !simulation.ship_search_active && keys.just_pressed(KeyCode::Digit1) {
        simulation.tool = Tool::Break;
    }
    if !simulation.ship_search_active && keys.just_pressed(KeyCode::Digit2) {
        simulation.tool = Tool::Dry;
    }
    if !simulation.ship_search_active && keys.just_pressed(KeyCode::KeyR) {
        for entity in &markers {
            commands.entity(entity).despawn();
        }
        *simulation = Simulation::default();
        structure.breached.fill(false);
        structure.leaking.fill(false);
        structure.water.fill(0.0);
        structure.positions = structure.rest_positions.clone();
        structure.last_positions = structure.rest_positions.clone();
        structure.strut_masks =
            build_strut_masks(&structure.solid, structure.width, structure.height);
        for spring in &mut structure.springs {
            spring.broken = false;
        }
        structure.flooding = 0.0;
        structure.motion_position = Vec2::new(0.0, SEA_LEVEL);
        structure.motion_velocity = Vec2::ZERO;
        structure.manual_offset = Vec2::ZERO;
        structure.angle = 0.0;
        structure.angular_velocity = 0.0;
        let texel_rest_positions = structure.texel_rest_positions.clone();
        structure.texel_positions.clone_from(&texel_rest_positions);
        reset_gpu_ship_physics(&structure, &mut gpu_physics, &mut shader_buffers);
        *snapshot = GpuShipPhysicsSnapshot::default();
        return;
    }

    if !simulation.paused && simulation.cycle_enabled {
        simulation.day = time_sync::daylight(simulation.elapsed, simulation.cycle_length);
    }
}
fn select_ship_layer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    catalog: Res<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
) {
    if !mouse.just_pressed(MouseButton::Left)
        || simulation.toolbox_collapsed
        || !simulation.show_tools
    {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );
    if !point_in_tool_panel(point)
        || !(-260.0..=-30.0).contains(&point.x)
        || !(296.0..=324.0).contains(&point.y)
    {
        return;
    }
    let count = catalog
        .1
        .get(simulation.ship_index)
        .map_or(1, |layers| layers.len().max(1));
    simulation.selected_layer = (simulation.selected_layer + 1) % count;
}

fn update_ship_layer_label(
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
    mut labels: Query<&mut Text2d, With<ShipLayerLabel>>,
) {
    let layer = catalog
        .layer_name(simulation.ship_index, simulation.selected_layer)
        .unwrap_or("Default");
    for mut label in &mut labels {
        label.0 = layer.to_owned();
    }
}

fn select_ship_from_panel(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    catalog: Res<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
    mut search_text: Query<&mut Text2d, With<ShipSearchText>>,
) {
    if simulation.toolbox_collapsed || simulation.active_tab != ToolboxTab::Ships {
        simulation.ship_search_active = false;
        return;
    }
    if simulation.ship_search_active {
        let mut changed = false;
        if keys.just_pressed(KeyCode::Backspace) {
            changed = simulation.ship_search.pop().is_some();
        }
        if keys.just_pressed(KeyCode::Escape) {
            simulation.ship_search_active = false;
        }
        for (key, character) in [
            (KeyCode::KeyA, 'a'),
            (KeyCode::KeyB, 'b'),
            (KeyCode::KeyC, 'c'),
            (KeyCode::KeyD, 'd'),
            (KeyCode::KeyE, 'e'),
            (KeyCode::KeyF, 'f'),
            (KeyCode::KeyG, 'g'),
            (KeyCode::KeyH, 'h'),
            (KeyCode::KeyI, 'i'),
            (KeyCode::KeyJ, 'j'),
            (KeyCode::KeyK, 'k'),
            (KeyCode::KeyL, 'l'),
            (KeyCode::KeyM, 'm'),
            (KeyCode::KeyN, 'n'),
            (KeyCode::KeyO, 'o'),
            (KeyCode::KeyP, 'p'),
            (KeyCode::KeyQ, 'q'),
            (KeyCode::KeyR, 'r'),
            (KeyCode::KeyS, 's'),
            (KeyCode::KeyT, 't'),
            (KeyCode::KeyU, 'u'),
            (KeyCode::KeyV, 'v'),
            (KeyCode::KeyW, 'w'),
            (KeyCode::KeyX, 'x'),
            (KeyCode::KeyY, 'y'),
            (KeyCode::KeyZ, 'z'),
            (KeyCode::Space, ' '),
        ] {
            if keys.just_pressed(key) {
                simulation.ship_search.push(character);
                changed = true;
            }
        }
        if changed {
            simulation.ship_scroll = 0;
        }
    }
    if let Ok(mut label) = search_text.single_mut() {
        label.0.clone_from(&simulation.ship_search);
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let ui_scale = 720.0 / window.height().max(1.0);
    let x = (cursor.x - window.width() * 0.5) * ui_scale;
    let y = (window.height() * 0.5 - cursor.y) * ui_scale;
    if mouse.just_pressed(MouseButton::Left) {
        simulation.ship_search_active =
            (-628.0..=-401.5).contains(&x) && (238.0..=268.0).contains(&y);
    }
    let visible_choices = filtered_ship_indices(&catalog, &simulation.ship_search);
    if (-628.0..=-302.0).contains(&x) && (-75.0..=146.0).contains(&y) {
        for event in wheel.read() {
            let movement = match event.unit {
                MouseScrollUnit::Line => event.y.round() as isize,
                MouseScrollUnit::Pixel => (event.y / 40.0).round() as isize,
            };
            let max_scroll = visible_choices.len().saturating_sub(4);
            simulation.ship_scroll = if movement < 0 {
                simulation
                    .ship_scroll
                    .saturating_add(movement.unsigned_abs())
                    .min(max_scroll)
            } else {
                simulation.ship_scroll.saturating_sub(movement as usize)
            };
        }
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if x < -628.0 || x > -302.0 || y > 146.0 || y < -75.0 {
        return;
    }
    let row = ((122.0 - y + 28.0) / 57.0).floor().max(0.0) as usize;
    if let Some(&index) = visible_choices.get(simulation.ship_scroll + row) {
        simulation.ship_index = index;
    }
}

fn filtered_ship_indices(catalog: &ShipCatalog, search: &str) -> Vec<usize> {
    let query = search.trim().to_lowercase();
    catalog
        .0
        .iter()
        .enumerate()
        .filter(|(_, choice)| query.is_empty() || choice.name.to_lowercase().contains(&query))
        .map(|(index, _)| index)
        .collect()
}

fn import_dropped_ship(
    mut events: MessageReader<FileDragAndDrop>,
    assets: Res<AssetServer>,
    mut catalog: ResMut<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
    mut commands: Commands,
) {
    for event in events.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = event else {
            continue;
        };
        if !is_png(path_buf) {
            bevy::log::warn!(
                "Only PNG ship images can be imported: {}",
                path_buf.display()
            );
            continue;
        }
        let (width, height) = match image::image_dimensions(path_buf) {
            Ok(dimensions) => dimensions,
            Err(error) => {
                bevy::log::warn!(
                    "Could not read dropped ship image {}: {error}",
                    path_buf.display()
                );
                continue;
            }
        };
        if u64::from(width) * u64::from(height) > 16_777_216 {
            bevy::log::warn!(
                "Dropped ship image is too large ({}x{}): {}",
                width,
                height,
                path_buf.display()
            );
            continue;
        }
        if let Err(error) = image::open(path_buf) {
            bevy::log::warn!(
                "Could not decode dropped ship image {}: {error}",
                path_buf.display()
            );
            continue;
        }

        let name = path_buf
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Imported ship")
            .trim()
            .to_owned();
        let safe_stem: String = name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let safe_stem = safe_stem.trim_matches('_');
        let safe_stem = if safe_stem.is_empty() {
            "imported_ship"
        } else {
            safe_stem
        };
        let directory = std::path::Path::new("assets/user_ships");
        if let Err(error) = std::fs::create_dir_all(directory) {
            bevy::log::error!(
                "Could not create ship import directory {}: {error}",
                directory.display()
            );
            continue;
        }
        let mut destination = directory.join(format!("{safe_stem}.png"));
        let mut suffix = 2;
        while destination.exists() {
            destination = directory.join(format!("{safe_stem}_{suffix}.png"));
            suffix += 1;
        }
        if let Err(error) = std::fs::copy(path_buf, &destination) {
            bevy::log::error!(
                "Could not copy imported ship to {}: {error}",
                destination.display()
            );
            continue;
        }

        let mut choices = Vec::with_capacity(1);
        push_ship_choice(&mut choices, &destination, &destination, name, false);
        let choice = choices.pop().expect("one imported ship choice was created");
        let index = catalog.add_imported(choice.clone());
        spawn_ship_catalog_card(&mut commands, &assets, index, &choice, false);
        simulation.ship_index = index;
    }
}

fn load_selected_ship(
    catalog: Res<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
    mut structure: ResMut<ShipStructure>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
    mut gpu_physics: ResMut<GpuShipPhysicsAssets>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ship_meshes: Query<(&ShipMesh, &mut MeshSyncState)>,
    mut last_ship: Local<Option<usize>>,
) {
    if last_ship.is_none() {
        *last_ship = Some(simulation.ship_index);
        return;
    }
    if *last_ship == Some(simulation.ship_index) {
        return;
    }
    let Some(choice) = catalog.0.get(simulation.ship_index) else {
        return;
    };
    *last_ship = Some(simulation.ship_index);
    simulation.selected_layer = 0;
    *structure = ShipStructure::load_for_choice(choice);
    reset_gpu_ship_physics(&structure, &mut gpu_physics, &mut shader_buffers);
    *snapshot = GpuShipPhysicsSnapshot::default();
    let replacement = build_deformable_ship_mesh(&structure);
    for (ship_mesh, mut sync_state) in &mut ship_meshes {
        if let Some(mut mesh) = meshes.get_mut(&ship_mesh.0) {
            *mesh = replacement.clone();
        }
        sync_state.0.clone_from(&structure.breached);
        sync_state.1.clear();
    }
}

fn select_toolbox_tab_and_settings(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    tabs: Query<(&TabButton, &Transform, &Sprite)>,
    buttons: Query<(&SettingButton, &Transform, &Visibility)>,
    rows: Query<(&SettingRow, &Transform, &Visibility)>,
    mut simulation: ResMut<Simulation>,
    mut dragging: Local<Option<(usize, Vec2)>>,
    mut music: ResMut<music_player::MusicPlayer>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        *dragging = None;
        return;
    };
    let ui_scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * ui_scale,
        (window.height() * 0.5 - cursor.y) * ui_scale,
    );
    if mouse.just_released(MouseButton::Left) || !mouse.pressed(MouseButton::Left) {
        *dragging = None;
    }
    if !mouse.just_pressed(MouseButton::Left) && mouse.pressed(MouseButton::Left) {
        if let Some((index, previous)) = *dragging {
            match index {
                DRAG_SEA_COLOR => apply_sea_color_point(point, &mut simulation),
                DRAG_SEA_HUE => apply_sea_hue_point(point, &mut simulation),
                DRAG_SEA_ALPHA => simulation.sea_alpha = toolbox::alpha_at(point.y),
                _ => simulation.adjust(index, (point.x - previous.x) * setting_drag_speed(index)),
            }
            *dragging = Some((index, point));
        }
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if point_in_toolbox_header(point) {
        simulation.toolbox_collapsed = !simulation.toolbox_collapsed;
        return;
    }
    if mouse.just_pressed(MouseButton::Left)
        && (-260.0..=-30.0).contains(&point.x)
        && (174.0..=202.0).contains(&point.y)
    {
        *dragging = Some((19, point));
        return;
    }
    if simulation.toolbox_collapsed {
        return;
    }
    if point.x < -640.0 || point.x > -290.0 {
        return;
    }
    if simulation.active_tab == ToolboxTab::Graphics {
        if (-419.0..=-405.0).contains(&point.x) && (-310.0..=50.0).contains(&point.y) {
            *dragging = Some((DRAG_SEA_ALPHA, point));
            simulation.sea_alpha = toolbox::alpha_at(point.y);
            return;
        }
        if let Some(channel) = toolbox::rgba_readout_hit(point) {
            *dragging = Some((21 + channel, point));
            return;
        }
        if (-627.0..=-439.0).contains(&point.x) && (-310.0..=50.0).contains(&point.y) {
            *dragging = Some((DRAG_SEA_COLOR, point));
            apply_sea_color_point(point, &mut simulation);
            return;
        }
        if (-441.0..=-423.0).contains(&point.x) && (-310.0..=50.0).contains(&point.y) {
            *dragging = Some((DRAG_SEA_HUE, point));
            apply_sea_hue_point(point, &mut simulation);
            return;
        }
    }
    if let Some((tab, _, _)) = tabs.iter().find(|(_, transform, sprite)| {
        (point.x - transform.translation.x).abs()
            < sprite.custom_size.unwrap_or(Vec2::splat(0.0)).x * 0.5
            && (point.y - transform.translation.y).abs() < 14.0
    }) {
        simulation.active_tab = tab.0;
        return;
    }
    if let Some((row, _, _)) = rows.iter().find(|(_, transform, visibility)| {
        **visibility != Visibility::Hidden
            && (point.x - transform.translation.x).abs() < 112.5
            && (point.y - transform.translation.y).abs() < 17.0
    }) {
        *dragging = Some((row.0, point));
        return;
    }
    let Some((button, _, _)) = buttons.iter().find(|(_, transform, visibility)| {
        **visibility != Visibility::Hidden
            && (point.x - transform.translation.x).abs() < 17.0
            && (point.y - transform.translation.y).abs() < 15.0
    }) else {
        return;
    };
    match button.0 {
        SettingAction::Adjust(index, delta) => simulation.adjust(index, delta),
        SettingAction::ToggleCycle => simulation.cycle_enabled = !simulation.cycle_enabled,
        SettingAction::ToggleTools => simulation.show_tools = !simulation.show_tools,
        SettingAction::ToggleWater => {
            simulation.show_internal_water = !simulation.show_internal_water
        }
        SettingAction::ToggleMusic => simulation.music_playing = !simulation.music_playing,
        SettingAction::ToggleShuffle => music.shuffle = !music.shuffle,
        SettingAction::ToggleRepeat => music.repeat = !music.repeat,
        SettingAction::NextTrack => {
            music.paused = !simulation.music_playing;
            music.next_track();
            simulation.music_playing = !music.paused;
        }
    }
}

fn setting_drag_speed(index: usize) -> f32 {
    match index {
        0 => 0.25,
        1 | 3 | 4 | 6 | 7 | 9 | 10 | 11 | 12 | 15 => 0.01,
        2 => 2.0,
        5 => 0.5,
        8 => 0.05,
        13 => 0.001,
        14 => 0.5,
        16 => 1.0,
        17 => 0.1,
        18 => 0.005,
        21..=24 => 1.0 / 255.0,
        _ => 0.01,
    }
}

fn apply_sea_color_point(point: Vec2, simulation: &mut Simulation) {
    let saturation = ((point.x + 627.0) / 188.0).clamp(0.0, 1.0);
    let value = ((point.y + 310.0) / 360.0).clamp(0.0, 1.0);
    simulation.sea_color = hsv_to_rgb(simulation.sea_hue, saturation, value);
}

fn apply_sea_hue_point(point: Vec2, simulation: &mut Simulation) {
    simulation.sea_hue = (360.0 * (1.0 - (point.y + 310.0) / 360.0)).clamp(0.0, 360.0);
    let (_, saturation, value) = rgb_to_hsv(simulation.sea_color);
    simulation.sea_color = hsv_to_rgb(simulation.sea_hue, saturation, value);
}

fn rgb_to_hue(rgb: Vec3) -> f32 {
    rgb_to_hsv(rgb).0
}

fn rgb_to_hsv(rgb: Vec3) -> (f32, f32, f32) {
    let max = rgb.max_element();
    let min = rgb.min_element();
    let delta = max - min;
    if delta <= f32::EPSILON {
        return (0.0, 0.0, max);
    }
    let hue = if max == rgb.x {
        60.0 * ((rgb.y - rgb.z) / delta).rem_euclid(6.0)
    } else if max == rgb.y {
        60.0 * ((rgb.z - rgb.x) / delta + 2.0)
    } else {
        60.0 * ((rgb.x - rgb.y) / delta + 4.0)
    };
    (hue.rem_euclid(360.0), delta / max.max(f32::EPSILON), max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let chroma = value * saturation;
    let sector = (hue.rem_euclid(360.0) / 60.0).rem_euclid(6.0);
    let secondary = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match sector as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let offset = value - chroma;
    Vec3::new(red + offset, green + offset, blue + offset)
}

fn color_picker_mesh(hue: f32) -> Mesh {
    const STEPS: usize = 32;
    let mut positions = Vec::with_capacity((STEPS + 1) * (STEPS + 1));
    let mut colors = Vec::with_capacity((STEPS + 1) * (STEPS + 1));
    let mut indices = Vec::with_capacity(STEPS * STEPS * 6);
    for row in 0..=STEPS {
        let value = 1.0 - row as f32 / STEPS as f32;
        for column in 0..=STEPS {
            let saturation = column as f32 / STEPS as f32;
            let color = hsv_to_rgb(hue, saturation, value);
            positions.push([
                -94.0 + column as f32 * (188.0 / STEPS as f32),
                180.0 - row as f32 * (360.0 / STEPS as f32),
                0.0,
            ]);
            colors.push([color.x, color.y, color.z, 1.0]);
        }
    }
    for row in 0..STEPS {
        for column in 0..STEPS {
            let top_left = (row * (STEPS + 1) + column) as u32;
            let top_right = top_left + 1;
            let bottom_left = top_left + (STEPS + 1) as u32;
            let bottom_right = bottom_left + 1;
            indices.extend_from_slice(&[
                top_left,
                bottom_left,
                top_right,
                top_right,
                bottom_left,
                bottom_right,
            ]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn hue_bar_mesh() -> Mesh {
    const STEPS: usize = 24;
    let mut positions = Vec::with_capacity(STEPS * 4);
    let mut colors = Vec::with_capacity(STEPS * 4);
    let mut indices = Vec::with_capacity(STEPS * 6);
    for row in 0..STEPS {
        let top = 180.0 - row as f32 * 15.0;
        let bottom = top - 15.0;
        let color = hsv_to_rgb(360.0 * (1.0 - (row as f32 + 0.5) / STEPS as f32), 1.0, 1.0);
        let base = (row * 4) as u32;
        positions.extend_from_slice(&[
            [-9.0, top, 0.0],
            [-9.0, bottom, 0.0],
            [9.0, top, 0.0],
            [9.0, bottom, 0.0],
        ]);
        colors.extend_from_slice(&[[color.x, color.y, color.z, 1.0]; 4]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn alpha_bar_mesh(color: Vec3) -> Mesh {
    const COLUMNS: usize = 2;
    const ROWS: usize = 16;
    let mut positions = Vec::with_capacity(COLUMNS * ROWS * 4);
    let mut colors = Vec::with_capacity(COLUMNS * ROWS * 4);
    let mut indices = Vec::with_capacity(COLUMNS * ROWS * 6);
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let left = -7.0 + column as f32 * 7.0;
            let right = left + 7.0;
            let top = 180.0 - row as f32 * 22.5;
            let bottom = top - 22.5;
            let shade = if (row + column) % 2 == 0 { 0.72 } else { 0.46 };
            let base = ((row * COLUMNS + column) * 4) as u32;
            positions.extend_from_slice(&[
                [left, top, 0.0],
                [left, bottom, 0.0],
                [right, top, 0.0],
                [right, bottom, 0.0],
            ]);
            for alpha in [
                1.0 - row as f32 / ROWS as f32,
                1.0 - (row + 1) as f32 / ROWS as f32,
                1.0 - row as f32 / ROWS as f32,
                1.0 - (row + 1) as f32 / ROWS as f32,
            ] {
                let rgb = Vec3::splat(shade).lerp(color, alpha);
                colors.push([rgb.x, rgb.y, rgb.z, 1.0]);
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn point_in_toolbox_header(point: Vec2) -> bool {
    (-640.0..=-290.0).contains(&point.x) && (316.0..=350.0).contains(&point.y)
}

fn point_in_tool_panel(point: Vec2) -> bool {
    (-270.0..=100.0).contains(&point.x) && (160.0..=350.0).contains(&point.y)
}

fn sync_settings_ui(
    simulation: Res<Simulation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pages: Query<(&TabPage, &mut Visibility), Without<TabButton>>,
    mut tabs: Query<
        (&TabButton, &mut Sprite, &mut Visibility),
        (Without<SeaColorPreview>, Without<TabPage>),
    >,
    mut readouts: Query<
        (&SettingReadout, &mut Text2d),
        (Without<SettingToggleMark>, Without<SeaColorReadout>),
    >,
    mut toggle_marks: Query<
        (&SettingToggleMark, &mut Text2d),
        (Without<SettingReadout>, Without<SeaColorReadout>),
    >,
    mut sea_color_preview: Query<(&SeaColorPreview, &mut Sprite), Without<TabButton>>,
    mut color_selector: Query<(&mut Transform, Option<&SeaAlphaSelector>), With<SeaColorSelector>>,
    mut color_readouts: Query<
        (&SeaColorReadout, &mut Text2d),
        (Without<SettingReadout>, Without<SettingToggleMark>),
    >,
    color_picker: Query<&SeaColorPicker>,
    alpha_picker: Query<&SeaAlphaPicker>,
    mut last_alpha_color: Local<Option<[u32; 3]>>,
    mut last_picker_hue: Local<Option<u32>>,
) {
    for (page, mut visibility) in &mut pages {
        *visibility = if !simulation.toolbox_collapsed && page.0 == simulation.active_tab {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (tab, mut sprite, mut visibility) in &mut tabs {
        sprite.color = if tab.0 == simulation.active_tab {
            Color::srgb(0.34, 0.40, 0.62)
        } else {
            Color::srgb(0.20, 0.24, 0.36)
        };
        *visibility = if simulation.toolbox_collapsed {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (readout, mut label) in &mut readouts {
        label.0 = match readout.0 {
            SettingValue::Number(index) => format!("{:.3}", simulation.setting(index)),
        };
    }
    for (mark, mut label) in &mut toggle_marks {
        let checked = match mark.0 {
            SettingAction::ToggleTools => simulation.show_tools,
            SettingAction::ToggleCycle => simulation.cycle_enabled,
            SettingAction::ToggleWater => simulation.show_internal_water,
            _ => false,
        };
        label.0 = if checked { "✓" } else { "" }.to_owned();
    }
    for (half, mut preview) in &mut sea_color_preview {
        preview.color = Color::srgba(
            simulation.sea_color.x,
            simulation.sea_color.y,
            simulation.sea_color.z,
            if half.0 { simulation.sea_alpha } else { 1.0 },
        );
    }
    let color_key = simulation.sea_color.to_array().map(f32::to_bits);
    if *last_alpha_color != Some(color_key) {
        for picker in &alpha_picker {
            if let Some(mut mesh) = meshes.get_mut(&picker.0) {
                *mesh = alpha_bar_mesh(simulation.sea_color);
            }
        }
        *last_alpha_color = Some(color_key);
    }
    let hue_key = simulation.sea_hue.to_bits();
    if *last_picker_hue != Some(hue_key) {
        for picker in &color_picker {
            if let Some(mut mesh) = meshes.get_mut(&picker.0) {
                *mesh = color_picker_mesh(simulation.sea_hue);
            }
        }
        *last_picker_hue = Some(hue_key);
    }
    for (mut transform, alpha) in &mut color_selector {
        if alpha.is_some() {
            transform.translation.y = toolbox::alpha_marker_y(simulation.sea_alpha);
            continue;
        }
        let (_, saturation, value) = rgb_to_hsv(simulation.sea_color);
        transform.translation.x = -627.0 + saturation * 188.0;
        transform.translation.y = -310.0 + value * 360.0;
    }
    for (readout, mut label) in &mut color_readouts {
        let channels = [
            (simulation.sea_color.x * 255.0).round() as u8,
            (simulation.sea_color.y * 255.0).round() as u8,
            (simulation.sea_color.z * 255.0).round() as u8,
            (simulation.sea_alpha * 255.0).round() as u8,
        ];
        let channel_name = ["R", "G", "B", "A"][readout.0.min(3)];
        label.0 = format!("{channel_name}: {}", channels[readout.0.min(3)]);
    }
}

fn sync_toolbox_visibility(
    simulation: Res<Simulation>,
    mut content: Query<&mut Visibility, With<ToolboxContent>>,
    mut title: Query<&mut Text2d, With<ToolboxTitle>>,
) {
    let visibility = if simulation.toolbox_collapsed {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut item_visibility in &mut content {
        *item_visibility = visibility;
    }
    for mut label in &mut title {
        label.0 = if simulation.toolbox_collapsed {
            "▶  Toolbox".to_owned()
        } else {
            "▼  Toolbox".to_owned()
        };
    }
}

fn select_tool_from_panel(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut simulation: ResMut<Simulation>,
) {
    if !simulation.show_tools || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let ui_scale = 720.0 / window.height().max(1.0);
    let x = (cursor.x - window.width() * 0.5) * ui_scale;
    let y = (window.height() * 0.5 - cursor.y) * ui_scale;
    if !(222.0..=278.0).contains(&y) {
        return;
    }
    if (-248.0..=-192.0).contains(&x) {
        simulation.tool = Tool::Break;
    } else if (-170.0..=-114.0).contains(&x) {
        simulation.tool = Tool::Dry;
    } else if (-92.0..=-36.0).contains(&x) {
        simulation.tool = Tool::Flood;
    } else if (-14.0..=42.0).contains(&x) {
        simulation.tool = Tool::Move;
    }
}

fn apply_hull_leak_inflow(
    water: &mut [f32],
    solid: &[bool],
    leaking: &[bool],
    interior: &[bool],
    positions: &[Vec2],
    width: usize,
    height: usize,
    half_height: f32,
    motion_position: Vec2,
    elapsed: f32,
    wave_amplitude: f32,
    wave_width: f32,
    gravity: f32,
    delta: f32,
    flow_scale: f32,
) {
    if width == 0
        || height == 0
        || water.len() != width * height
        || solid.len() != water.len()
        || leaking.len() != water.len()
        || interior.len() != water.len()
        || positions.len() != water.len()
    {
        return;
    }
    let cell_height = (half_height * 2.0 / height as f32).max(0.001);
    for i in 0..water.len() {
        if !leaking[i] || !solid[i] {
            continue;
        }
        let world_y = motion_position.y + positions[i].y;
        let world_x = motion_position.x + positions[i].x;
        let surface = SEA_LEVEL + wave_height(world_x, elapsed, wave_amplitude, wave_width);
        let head = surface - world_y;
        if head <= 0.0 {
            continue;
        }
        let ingress = ((2.0 * gravity.max(0.0) * head).sqrt() * delta / cell_height
            * flow_scale.max(0.0)
            * 0.02)
            .min(0.2);
        let x = i % width;
        let y = i / width;
        for (dx, dy) in EIGHT_NEIGHBORS {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 && nx < width as isize && ny < height as isize {
                let neighbor = ny as usize * width + nx as usize;
                if interior[neighbor] {
                    water[neighbor] = (water[neighbor] + ingress).min(1.0);
                }
            }
        }
    }
}

fn simulate_ship_physics(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<Simulation>,
    mut structure: ResMut<ShipStructure>,
) {
    if simulation.paused {
        return;
    }
    let delta = time.delta_secs().min(1.0 / 30.0);
    if delta <= 0.0 {
        return;
    }
    // SS2 runs its force pass once per configured physics step and schedules
    // the more expensive water pass only at water-step intervals.
    let substeps = simulation.physics_iterations.round().clamp(1.0, 200.0) as usize;
    let break_pass_scale = source_spring_break_scale(substeps, delta);
    let flow_scale =
        (simulation.water_flow / 60.0).clamp(0.0, 3.0) * simulation.water_influx.max(0.0);
    // Water enters neighboring cavity nodes through a submerged hull breach,
    // not through intact broken springs or the damaged material node itself.
    let ShipStructure {
        water,
        solid,
        leaking,
        interior,
        positions,
        width,
        height,
        half_height,
        motion_position,
        ..
    } = &mut *structure;
    apply_hull_leak_inflow(
        water,
        solid,
        leaking,
        interior,
        positions,
        *width,
        *height,
        *half_height,
        *motion_position,
        simulation.elapsed,
        simulation.wave_amplitude,
        simulation.wave_width,
        simulation.gravity,
        delta,
        flow_scale,
    );
    let cell_height = *half_height * 2.0 / *height as f32;
    let cell_width = 2.0 * SHIP_HALF_WIDTH / *width as f32;
    let maximum_local_deformation = cell_width.max(cell_height) * 4.0;
    let water_steps = simulation.water_steps.round().clamp(1.0, 32.0) as usize;
    for _ in 0..water_steps {
        let ShipStructure {
            water,
            interior,
            width,
            height,
            ..
        } = &mut *structure;
        flow_interior_water(
            water,
            interior,
            *width,
            *height,
            cell_height,
            simulation.gravity,
            delta / water_steps as f32,
            flow_scale,
            simulation.water_funk,
        );
    }
    if !simulation.ship_search_active && keys.pressed(KeyCode::KeyF) {
        if let Some(i) = structure
            .interior
            .iter()
            .position(|&is_interior| is_interior)
        {
            structure.water[i] = (structure.water[i] + delta * 0.5).min(1.0);
        }
    }
    if (!simulation.ship_search_active
        && (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::KeyP)))
        || simulation.pump_enabled
    {
        for amount in &mut structure.water {
            *amount = (*amount - 0.6 * delta).max(0.0);
        }
    }

    for _ in 0..substeps {
        let dt = delta / substeps as f32;
        // Port SS2's density-weighted buoyancy and quadratic water drag. The
        // ship translation follows mean acceleration; local point motion only
        // receives differential acceleration so it deforms without double
        // applying gravity to the hull center.
        let mut accelerations = vec![Vec2::ZERO; structure.positions.len()];
        let mut effective_masses = vec![0.0; structure.positions.len()];
        let mut mean_acceleration = Vec2::ZERO;
        let mut total_mass = 0.0;
        for i in 0..structure.positions.len() {
            if !structure.solid[i] || structure.breached[i] {
                continue;
            }
            let world_y = structure.motion_position.y + structure.positions[i].y;
            let world_x = structure.motion_position.x + structure.positions[i].x;
            let amplitude = simulation.wave_amplitude;
            let local_y = world_y - SEA_LEVEL;
            let surface = wave_height(
                world_x,
                simulation.elapsed,
                amplitude,
                simulation.wave_width,
            );
            let wet = local_y < surface;
            let fluid_density = if wet { 1025.0 } else { 1.225 };
            let x = i % structure.width;
            let y = i / structure.width;
            let mut internal_fill = structure.water[i];
            for (dx, dy) in EIGHT_NEIGHBORS {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && ny >= 0
                    && nx < structure.width as isize
                    && ny < structure.height as isize
                {
                    let neighbor = ny as usize * structure.width + nx as usize;
                    if structure.interior[neighbor] {
                        internal_fill = internal_fill.max(structure.water[neighbor]);
                    }
                }
            }
            let material = structure.materials[i].unwrap_or(MaterialProperties {
                strength: 1.0,
                tensile_strength: 1.0,
                compressive_strength: 1.0,
                mass: 2409.0,
                hull: true,
                ground: false,
                rope: false,
                invisible: false,
            });
            let density = effective_density(
                material.mass,
                internal_fill,
                material.hull,
                material.ground,
                simulation.thickness,
                simulation.water_weight,
            );
            let mut acceleration = Vec2::new(
                0.0,
                buoyancy_acceleration(
                    simulation.gravity,
                    simulation.buoyancy,
                    fluid_density,
                    density,
                ),
            );
            let previous_surface = wave_height(
                world_x,
                simulation.elapsed - dt,
                amplitude,
                simulation.wave_width,
            );
            let surface_velocity = (previous_surface - surface)
                / dt
                / (1.0 - (local_y / amplitude.max(0.001)).min(0.0));
            let old_position = structure.positions[i];
            let local_velocity = (old_position - structure.last_positions[i]) / dt;
            let relative_velocity =
                local_velocity + structure.motion_velocity + Vec2::new(0.0, surface_velocity);
            let speed = relative_velocity.length();
            let area = if is_exposed_node(&structure.solid, structure.width, structure.height, i) {
                0.5
            } else {
                0.01
            };
            if speed > 0.001 {
                let drag_acceleration =
                    0.5 * speed * speed * fluid_density * area * simulation.drag.max(0.0)
                        / density.max(0.001);
                acceleration -= relative_velocity / speed * drag_acceleration;
            }
            acceleration = acceleration.clamp_length_max(simulation.gravity.max(0.0) * 8.0);
            effective_masses[i] = density;
            accelerations[i] = acceleration;
            mean_acceleration += acceleration * density;
            total_mass += density;
        }
        if total_mass > 0.0 {
            mean_acceleration /= total_mass;
        }
        let mut spring_accelerations = vec![Vec2::ZERO; structure.positions.len()];
        {
            let ShipStructure {
                springs,
                strut_masks,
                positions,
                last_positions,
                materials,
                breached,
                ..
            } = &mut *structure;
            for spring in springs.iter_mut() {
                if spring.broken || breached[spring.a] || breached[spring.b] {
                    continue;
                }
                let mass_a = effective_masses[spring.a];
                let mass_b = effective_masses[spring.b];
                let delta_position = positions[spring.b] - positions[spring.a];
                let velocity_a = (positions[spring.a] - last_positions[spring.a]) / dt;
                let velocity_b = (positions[spring.b] - last_positions[spring.b]) / dt;
                let rope = materials[spring.a].is_some_and(|m| m.rope)
                    || materials[spring.b].is_some_and(|m| m.rope);
                let (force, elastic_load) = source_cpu_spring_force(
                    delta_position,
                    velocity_b - velocity_a,
                    spring.rest_length,
                    mass_a,
                    mass_b,
                    simulation.rigidity,
                    simulation.damping,
                    rope,
                    substeps,
                    delta,
                );
                spring_accelerations[spring.a] += force / mass_a;
                spring_accelerations[spring.b] -= force / mass_b;
                if should_break_spring(
                    elastic_load,
                    spring.tensile_strength * break_pass_scale,
                    spring.compressive_strength * break_pass_scale,
                    simulation.strength,
                ) {
                    break_spring(spring, strut_masks);
                }
            }
        }
        for i in 0..structure.positions.len() {
            if !structure.solid[i] || structure.breached[i] {
                continue;
            }
            let old_position = structure.positions[i];
            let velocity = old_position - structure.last_positions[i];
            let local_acceleration = accelerations[i] - mean_acceleration + spring_accelerations[i];
            let predicted_position = clamp_hull_deformation(
                old_position + velocity + local_acceleration * (dt * dt),
                structure.rest_positions[i],
                maximum_local_deformation,
            );
            structure.last_positions[i] = old_position;
            let local_floor = SEA_LEVEL - simulation.sea_depth - structure.motion_position.y;
            let (resolved_position, resolved_velocity) =
                resolve_floor_collision(old_position, predicted_position, local_floor, dt);
            structure.positions[i] = resolved_position;
            structure.last_positions[i] = resolved_position - resolved_velocity * dt;
        }
        structure.motion_velocity.x =
            (structure.motion_velocity.x + mean_acceleration.x * dt).clamp(-45.0, 45.0);
        structure.motion_velocity.y =
            (structure.motion_velocity.y + mean_acceleration.y * dt).clamp(-90.0, 45.0);
        let center_velocity = structure.motion_velocity;
        structure.motion_position += center_velocity * dt;
    }
    let solid_count = structure
        .solid
        .iter()
        .filter(|&&solid| solid)
        .count()
        .max(1) as f32;
    let interior_count = structure
        .interior
        .iter()
        .filter(|&&interior| interior)
        .count()
        .max(1) as f32;
    structure.flooding = structure
        .water
        .iter()
        .zip(&structure.interior)
        .filter(|(_, interior)| **interior)
        .map(|(amount, _)| *amount)
        .sum::<f32>()
        / interior_count;
    simulation.flooding = structure.flooding.clamp(0.0, 1.0);
    let mut moment = 0.0;
    for (i, amount) in structure.water.iter().enumerate() {
        if structure.interior[i] {
            moment += *amount * ((i % structure.width) as f32 / structure.width as f32 * 2.0 - 1.0);
        }
    }
    let target_angle = (-moment / solid_count * 18.0).clamp(-0.7, 0.7);
    structure.angular_velocity +=
        ((target_angle - structure.angle) * 2.4 - structure.angular_velocity * 2.1) * delta;
    structure.angle += structure.angular_velocity * delta;
}

fn update_gpu_physics_settings(
    mut drag: ResMut<tools::move_tool::MoveDragState>,
    snapshot: Res<GpuShipPhysicsSnapshot>,
    simulation: Res<Simulation>,
    structure: Res<ShipStructure>,
    mut physics: ResMut<GpuShipPhysicsAssets>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut last_breaches: Local<Option<Vec<bool>>>,
) {
    if last_breaches.as_ref() != Some(&structure.breached) {
        if let Some(mut buffer) = buffers.get_mut(&physics.masks) {
            *buffer = ShaderBuffer::from(mask_struts_data::gpu_mask_storage(current_render_masks(
                &structure, &snapshot,
            )));
        } else {
            bevy::log::error!("GPU ship physics topology buffer is missing");
        }
        *last_breaches = Some(structure.breached.clone());
    }

    let configured_iterations = simulation.physics_iterations.round().clamp(1.0, 200.0) as u32;
    let water_steps = simulation
        .water_steps
        .round()
        .clamp(1.0, configured_iterations as f32) as u32;
    let iterations = if simulation.paused || drag.dragging {
        0
    } else {
        configured_iterations
    };
    physics.iterations = iterations;
    physics.water_steps = water_steps;
    let mut settings = gpu_settings(
        &structure,
        configured_iterations,
        water_steps,
        1.0 / 60.0,
        simulation.elapsed,
        simulation.sea_depth,
        simulation.gravity,
        simulation.rigidity,
        simulation.damping,
        simulation.strength,
        simulation.drag,
        simulation.buoyancy,
        simulation.wave_width,
        simulation.wave_amplitude,
        simulation.water_influx,
        simulation.water_flow,
        simulation.water_funk,
        simulation.water_weight,
        1.0 - simulation.thickness,
    );
    let native_delta = drag.pending_translation;
    settings[5] = [native_delta.x, native_delta.y, 0.0, 0.0];
    settings[6] = physics.water_brush.take().unwrap_or([0.0; 4]);
    settings[7] = physics.break_brush.take().unwrap_or([0.0; 4]);
    drag.pending_translation = Vec2::ZERO;
    let data = ShaderBuffer::from(settings);
    if let Some(mut buffer) = buffers.get_mut(&physics.settings) {
        *buffer = data;
    } else {
        bevy::log::error!("GPU ship physics settings buffer is missing");
    }
}

fn apply_gpu_physics_readback(
    snapshot: Res<GpuShipPhysicsSnapshot>,
    mut structure: ResMut<ShipStructure>,
    mut simulation: ResMut<Simulation>,
) {
    if snapshot.positions.len() == structure.texel_positions.len() && !snapshot.positions.is_empty()
    {
        for (target, position) in structure
            .texel_positions
            .iter_mut()
            .zip(&snapshot.positions)
        {
            *target = Vec2::new(position.x, position.y);
        }
    }
    let count = structure.texel_width * structure.texel_height;
    if snapshot.water.len() == count {
        structure.water.fill(0.0);
        let mut samples = vec![0usize; structure.water.len()];
        for (index, water) in snapshot.water.iter().enumerate() {
            let x = index % structure.texel_width;
            let y = index / structure.texel_width;
            let proxy = (y / PHYSICS_NODE_PIXELS) * structure.width + x / PHYSICS_NODE_PIXELS;
            structure.water[proxy] += water.x;
            samples[proxy] += 1;
        }
        for (amount, sample_count) in structure.water.iter_mut().zip(samples) {
            if sample_count > 0 {
                *amount /= sample_count as f32;
            }
        }
        let interior_count = structure
            .interior
            .iter()
            .filter(|&&cell| cell)
            .count()
            .max(1);
        structure.flooding = structure
            .water
            .iter()
            .zip(&structure.interior)
            .filter(|(_, interior)| **interior)
            .map(|(amount, _)| *amount)
            .sum::<f32>()
            / interior_count as f32;
        simulation.flooding = structure.flooding.clamp(0.0, 1.0);
    }
}

use sea::wave_height;

fn animate_water(
    simulation: Res<Simulation>,
    camera_control: Res<CameraControlState>,
    windows: Query<&Window>,
    mut ocean_surfaces: Query<&mut OceanSurface>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OceanSurfaceMaterial>>,
) {
    let screen_width = windows
        .single()
        .map(|w| w.width() * 720.0 / w.height().max(1.0) + 16.0)
        .unwrap_or(WORLD_WIDTH);
    let bounds = camera_control.world_bounds();
    let screen_width = bounds
        .map(|(a, b)| a.x.abs().max(b.x.abs()) * 2.0 + 2.0)
        .unwrap_or(screen_width);
    let height = (simulation.wave_amplitude).max(1.0) + 2.0;
    for mut ocean in &mut ocean_surfaces {
        if (ocean.width - screen_width).abs() > 0.5 || (ocean.height - height).abs() > 0.5 {
            if let Some(mut mesh) = meshes.get_mut(&ocean.mesh) {
                *mesh = ocean_surface_mesh(screen_width, height);
            }
            ocean.width = screen_width;
            ocean.height = height;
        }
        if let Some(mut material) = materials.get_mut(&ocean.material) {
            material.color = LinearRgba::new(
                (simulation.sea_color.x * 1.15).min(1.0),
                (simulation.sea_color.y * 1.15).min(1.0),
                (simulation.sea_color.z * 1.15).min(1.0),
                1.0,
            );
            material.params = Vec4::new(
                std::f32::consts::PI / simulation.wave_width.max(0.001),
                simulation.elapsed,
                simulation.wave_amplitude,
                0.0,
            );
        }
    }
}

fn animate_sea_depth(
    simulation: Res<Simulation>,
    camera_control: Res<CameraControlState>,
    windows: Query<&Window>,
    mut oceans: Query<&mut OceanDepth>,
    mut underwater_effects: Query<&mut UnderwaterEffect>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OceanDepthMaterial>>,
    mut underwater_materials: ResMut<Assets<UnderwaterEffectMaterial>>,
) {
    let screen_width = windows
        .single()
        .map(|w| w.width() * 720.0 / w.height().max(1.0) + 16.0)
        .unwrap_or(WORLD_WIDTH);
    let bounds = camera_control.world_bounds();
    let screen_width = bounds
        .map(|(a, b)| a.x.abs().max(b.x.abs()) * 2.0 + 2.0)
        .unwrap_or(screen_width);
    // Sea.java is fullscreen: the sea depth parameter does not clip its render.
    let depth = bounds
        .map(|(a, b)| SEA_LEVEL - a.y.min(b.y) + 2.0)
        .unwrap_or(simulation.sea_depth)
        .max(simulation.sea_depth)
        .max(1.0);
    let brightness = simulation.water_darkness;
    for mut effect in &mut underwater_effects {
        if (effect.width - screen_width).abs() > 0.5 || (effect.depth - depth).abs() > 0.5 {
            if let Some(mut mesh) = meshes.get_mut(&effect.mesh) {
                *mesh = ocean_depth_mesh(screen_width, depth);
            }
            effect.width = screen_width;
            effect.depth = depth;
        }
        if let Some(mut material) = underwater_materials.get_mut(&effect.material) {
            let sea = simulation.sea_color;
            material.color = LinearRgba::new(sea.x * 0.28, sea.y * 0.52, sea.z * 0.78, 1.0);
            material.params = Vec4::new(
                simulation.elapsed,
                std::f32::consts::PI / simulation.wave_width.max(0.001),
                SEA_LEVEL,
                simulation.water_darkness,
            );
        }
    }
    for mut ocean in &mut oceans {
        if (ocean.width - screen_width).abs() > 0.5 || (ocean.depth - depth).abs() > 0.5 {
            if let Some(mut mesh) = meshes.get_mut(&ocean.mesh) {
                *mesh = ocean_depth_mesh(screen_width, depth);
            }
            ocean.width = screen_width;
            ocean.depth = depth;
        }
        if let Some(mut material) = materials.get_mut(&ocean.material) {
            let sea = simulation.sea_color;
            material.surface_color = LinearRgba::new(sea.x, sea.y, sea.z, 1.0);
            material.deep_color = LinearRgba::new(sea.x * 0.20, sea.y * 0.48, sea.z * 0.78, 1.0);
            material.params = Vec4::new(depth, brightness, SEA_LEVEL, 0.0);
        }
    }
}

fn animate_ship(
    structure: Res<ShipStructure>,
    mut ships: Query<&mut Transform, With<ShipSprite>>,
    mut mesh_ships: Query<&mut Transform, (With<ShipMesh>, Without<ShipSprite>)>,
    mut water_meshes: Query<
        &mut Transform,
        (
            With<InternalWaterMesh>,
            Without<ShipSprite>,
            Without<ShipMesh>,
        ),
    >,
) {
    for mut transform in &mut ships
        .iter_mut()
        .chain(mesh_ships.iter_mut())
        .chain(water_meshes.iter_mut())
    {
        transform.translation.x = structure.motion_position.x;
        transform.translation.y = structure.motion_position.y;
        transform.rotation = Quat::from_rotation_z(structure.angle);
    }
}

fn animate_reflection(
    simulation: Res<Simulation>,
    structure: Res<ShipStructure>,
    ships: Query<&Transform, With<ShipSprite>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ReflectionMaterial>>,
    mut reflections: Query<(&mut ReflectionMesh, &mut Transform), Without<ShipSprite>>,
) {
    let Ok(ship) = ships.single() else { return };
    let surface = SEA_LEVEL
        + wave_height(
            ship.translation.x,
            simulation.elapsed,
            simulation.wave_amplitude,
            simulation.wave_width,
        );
    for (mut reflection, mut transform) in &mut reflections {
        if (reflection.half_height - structure.half_height).abs() > 0.001 {
            if let Some(mut mesh) = meshes.get_mut(&reflection.mesh) {
                *mesh = reflection_mesh(&structure);
            }
            reflection.half_height = structure.half_height;
        }
        if let Some(mut material) = materials.get_mut(&reflection.material) {
            material.params = Vec4::new(
                std::f32::consts::PI / simulation.wave_width.max(0.001),
                simulation.elapsed,
                simulation.wave_amplitude,
                SEA_LEVEL,
            );
        }
        transform.translation = Vec3::new(ship.translation.x, surface, -2.0);
        transform.rotation = Quat::from_rotation_z(-ship.rotation.to_euler(EulerRot::XYZ).2);
    }
}

/// BASE-only ships display the derived texture; their physics retains BASE.
fn load_ship_visual_asset(
    choice: &ShipChoice,
    visual_asset: &str,
    assets: &AssetServer,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if choice.material_map && visual_asset == choice.physics_asset {
        let derived = (|| -> Result<image::RgbaImage, String> {
            let json = std::fs::read_to_string("assets/config/materials.json")
                .map_err(|error| error.to_string())?;
            let global = materials::Materials::from_json(&json)?;
            let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(
                &format!("assets/{}", choice.physics_asset),
            ))?;
            thumbnail
                .texture(&ShipLayer::default(), &global)?
                .ok_or_else(|| "missing default texture".to_owned())
        })();
        match derived {
            Ok(rgba) => {
                return images.add(texture_2d::ship_texture(rgba));
            }
            Err(error) => bevy::log::warn!("Could not derive ship appearance: {error}"),
        }
    }
    match image::open(format!("assets/{visual_asset}")) {
        Ok(image) => images.add(texture_2d::ship_texture(image.to_rgba8())),
        Err(error) => {
            bevy::log::warn!("Could not load source ship texture {visual_asset}: {error}");
            assets
                .load_builder()
                .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                    settings.sampler = texture_2d::ship_sampler();
                })
                .load(visual_asset.to_owned())
        }
    }
}

/// The source returns a black texture for a missing light map on the
/// selected layer; it does not inherit default-layer lights for other layers.
fn load_ship_light_assets(
    choice: &ShipChoice,
    layer: &ShipLayer,
    assets: &AssetServer,
    images: &mut Assets<Image>,
) -> (Handle<Image>, Handle<Image>) {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut black = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    black.sampler = bevy::image::ImageSampler::nearest();
    let black = images.add(black);
    let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(&format!(
        "assets/{}",
        choice.physics_asset
    )))
    .ok();
    let mut load = |kind| match thumbnail
        .as_ref()
        .and_then(|ship| ship.get_resource(kind, layer))
    {
        Some(ship_thumbnail::ThumbnailResource::File(resource)) => match resource.image_data() {
            Ok(image) => images.add(texture_2d::ship_texture(image)),
            Err(error) => {
                bevy::log::warn!("Could not load ship light map: {error}");
                assets
                    .load_builder()
                    .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                        settings.sampler = texture_2d::ship_sampler();
                    })
                    .load(asset_path(&resource.path))
            }
        },
        _ => black.clone(),
    };
    (
        load(ShipResourceType::InLights),
        load(ShipResourceType::ExLights),
    )
}

fn sync_ship_assets(
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
    mut materials: ResMut<Assets<ShipMaterial>>,
    mut reflection_materials: ResMut<Assets<ReflectionMaterial>>,
    mut ships: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<ShipSprite>>,
    mut mesh_ships: Query<(&ShipMesh, &mut Visibility), Without<ShipSprite>>,
    mut reflections: Query<(&ReflectionMesh, &mut Transform), Without<ShipSprite>>,
    mut last_ship: Local<Option<(usize, usize)>>,
) {
    let state = (simulation.ship_index, simulation.selected_layer);
    if *last_ship == Some(state) {
        return;
    }
    *last_ship = Some(state);
    let Some(choice) = catalog.0.get(simulation.ship_index) else {
        return;
    };
    let visual_asset = catalog
        .layer_asset(simulation.ship_index, simulation.selected_layer)
        .unwrap_or(&choice.asset)
        .to_owned();
    let image = load_ship_visual_asset(choice, &visual_asset, &assets, &mut images);
    let layer = catalog
        .1
        .get(simulation.ship_index)
        .and_then(|layers| layers.get(simulation.selected_layer))
        .map(|entry| entry.name.clone())
        .unwrap_or_default();
    let (internal_lights, external_lights) =
        load_ship_light_assets(choice, &layer, &assets, &mut images);
    for (mut sprite, mut transform, mut visibility) in &mut ships {
        sprite.image = image.clone();
        transform.scale = Vec3::ONE;
        *visibility = Visibility::Hidden;
    }
    for (ship_mesh, mut visibility) in &mut mesh_ships {
        if let Some(mut material) = materials.get_mut(&ship_mesh.1) {
            material.texture = image.clone();
            material.internal_lights = internal_lights.clone();
            material.external_lights = external_lights.clone();
        }
        *visibility = Visibility::Inherited;
    }
    for (reflection, mut transform) in &mut reflections {
        if let Some(mut material) = reflection_materials.get_mut(&reflection.material) {
            material.texture = image.clone();
        }
        transform.scale = Vec3::splat(choice.scale);
    }
}

fn sync_ship_cards(
    simulation: Res<Simulation>,
    catalog: Res<ShipCatalog>,
    mut cards: Query<
        (&ShipCard, &mut Sprite, &mut Transform, &mut Visibility),
        (
            Without<ShipThumbnail>,
            Without<ShipNameLabel>,
            Without<ToolCard>,
        ),
    >,
    mut thumbnails: Query<
        (&ShipThumbnail, &mut Transform, &mut Visibility),
        (Without<ShipCard>, Without<ShipNameLabel>, Without<ToolCard>),
    >,
    mut names: Query<
        (&ShipNameLabel, &mut Transform, &mut Visibility),
        (Without<ShipCard>, Without<ShipThumbnail>, Without<ToolCard>),
    >,
    mut tools: Query<
        (&ToolCard, &mut Sprite, &mut Visibility),
        (
            Without<ShipCard>,
            Without<ShipThumbnail>,
            Without<ShipNameLabel>,
        ),
    >,
    mut glyphs: Query<
        &mut Visibility,
        (
            With<ToolGlyph>,
            Without<ToolCard>,
            Without<ShipCard>,
            Without<ShipThumbnail>,
            Without<ShipNameLabel>,
        ),
    >,
    mut last_visual_state: Local<Option<(usize, usize, Tool, bool, bool, bool, String)>>,
) {
    let visible_choices = filtered_ship_indices(&catalog, &simulation.ship_search);
    let state = (
        simulation.ship_index,
        simulation.ship_scroll,
        simulation.tool,
        simulation.pump_enabled,
        simulation.show_tools,
        simulation.toolbox_collapsed,
        simulation.ship_search.clone(),
    );
    if last_visual_state.as_ref() == Some(&state) {
        return;
    }
    *last_visual_state = Some(state);
    for (card, mut sprite, mut transform, mut visibility) in &mut cards {
        let row = visible_choices
            .iter()
            .position(|&index| index == card.0)
            .map(|position| position as isize - simulation.ship_scroll as isize);
        let visible = row.is_some_and(|row| (0..4).contains(&row));
        transform.translation.y = row.map_or(-500.0, |row| 122.0 - row as f32 * 57.0);
        *visibility = if visible && !simulation.toolbox_collapsed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        sprite.color = if card.0 == simulation.ship_index {
            Color::srgb(0.32, 0.39, 0.58)
        } else {
            Color::srgb(0.20, 0.23, 0.32)
        };
    }
    for (thumbnail, mut transform, mut visibility) in &mut thumbnails {
        let row = visible_choices
            .iter()
            .position(|&index| index == thumbnail.0)
            .map(|position| position as isize - simulation.ship_scroll as isize);
        transform.translation.y = row.map_or(-500.0, |row| 122.0 - row as f32 * 57.0);
        *visibility =
            if row.is_some_and(|row| (0..4).contains(&row)) && !simulation.toolbox_collapsed {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
    }
    for (name, mut transform, mut visibility) in &mut names {
        let row = visible_choices
            .iter()
            .position(|&index| index == name.0)
            .map(|position| position as isize - simulation.ship_scroll as isize);
        transform.translation.y = row.map_or(-500.0, |row| 122.0 - row as f32 * 57.0);
        *visibility =
            if row.is_some_and(|row| (0..4).contains(&row)) && !simulation.toolbox_collapsed {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
    }
    for (card, mut sprite, mut visibility) in &mut tools {
        let selected = card.0 == simulation.tool;
        sprite.color = if selected {
            Color::srgb(0.96, 0.63, 0.02)
        } else {
            Color::srgb(0.60, 0.38, 0.08)
        };
        *visibility = if simulation.show_tools {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut visibility in &mut glyphs {
        *visibility = if simulation.show_tools {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_tool_panel_visibility(
    simulation: Res<Simulation>,
    mut controls: Query<&mut Visibility, With<ToolPanelUi>>,
) {
    let visibility = if simulation.show_tools {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut control in &mut controls {
        *control = visibility;
    }
}

fn animate_leaks(
    ships: Query<&Transform, With<ShipSprite>>,
    mut markers: Query<(&LeakMarker, &mut Transform), Without<ShipSprite>>,
) {
    let Ok(ship) = ships.single() else { return };
    for (marker, mut transform) in &mut markers {
        let offset = ship.rotation * marker.0.extend(0.0);
        transform.translation.x = ship.translation.x + offset.x;
        transform.translation.y = ship.translation.y + offset.y;
    }
}

fn update_hud(
    simulation: Res<Simulation>,
    structure: Res<ShipStructure>,
    markers: Query<(), With<LeakMarker>>,
    mut labels: Query<&mut Text2d, With<Hud>>,
) {
    if !simulation.is_changed() {
        return;
    }

    let leak_count = markers.iter().count();
    for mut label in &mut labels {
        let broken = structure
            .springs
            .iter()
            .filter(|spring| spring.broken)
            .count();
        label.0 = format!(
            "SINKING SIMULATOR  /  BEVY PORT\nLeaks: {leak_count}   Flooding: {:>4.1}%   Springs broken: {broken}   {}",
            simulation.flooding * 100.0,
            if simulation.paused {
                "PAUSED"
            } else {
                "RUNNING"
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn source_geometry_damage_upload_preserves_previously_broken_gpu_links() {
        let mut structure = ShipStructure::empty_fallback();
        structure.texel_width = 2;
        structure.texel_height = 2;
        structure.texel_solid = vec![true; 4];
        structure.texel_materials = vec![
            Some(MaterialProperties {
                strength: 65.0,
                tensile_strength: 65.0,
                compressive_strength: 405.0,
                mass: 2409.0,
                hull: true,
                ground: false,
                rope: false,
                invisible: false,
            });
            4
        ];
        let mut snapshot = GpuShipPhysicsSnapshot::default();
        snapshot.masks = gpu_mask_data(&structure);
        snapshot.masks[0][1] &= !2;
        snapshot.masks[0][2] &= !2;
        let masks = current_render_masks(&structure, &snapshot);
        assert_eq!(
            masks[0][1] & 2,
            0,
            "stress-broken diagonal must not be restored"
        );
        assert_eq!(
            masks[0][1] & 5,
            5,
            "unbroken east/south links remain present"
        );
        structure.breached[0] = true;
        assert!(
            current_render_masks(&structure, &snapshot)
                .iter()
                .all(|m| *m == [0; 4])
        );
    }

    #[test]
    fn material_palette_loads_array_and_single_color_entries() {
        let palette = parse_material_palette(include_str!("../assets/config/materials.json"))
            .expect("the shipped material palette should be valid JSON");

        let titanium = palette.get(&0x5D5D60).expect("Titanium Hull color");
        assert_eq!(titanium.strength, 65.0);
        assert_eq!(titanium.tensile_strength, 65.0);
        assert_eq!(titanium.compressive_strength, 405.0);
        assert_eq!(titanium.mass, 2409.0);
        assert!(titanium.hull);

        let ground = palette.get(&0x7F7F7F).expect("Ground's single color");
        assert_eq!(ground.mass, 1_000_000_000.0);
        assert!(ground.ground);
    }

    #[test]
    fn default_ship_internal_water_mesh_has_renderable_geometry() {
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
        };
        let structure = ShipStructure::load_for_choice(&choice);
        let (mesh, cells) = build_internal_water_mesh(&structure);

        assert!(!cells.is_empty(), "default ship should have interior cells");
        assert!(
            mesh.get_vertex_buffer_size() > 0,
            "default ship water mesh should have vertices"
        );
        assert!(
            mesh.get_index_buffer_bytes()
                .is_some_and(|indices| !indices.is_empty()),
            "default ship water mesh should have indices"
        );
    }

    #[test]
    fn empty_interior_water_mesh_keeps_valid_hidden_geometry() {
        let structure = ShipStructure::empty_fallback();
        let (mesh, cells) = build_internal_water_mesh(&structure);

        assert!(cells.is_empty());
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(VertexAttributeValues::as_float3)
                .map_or(0, <[[f32; 3]]>::len),
            3
        );
        assert!(
            mesh.get_index_buffer_bytes()
                .is_some_and(|indices| !indices.is_empty())
        );
    }

    #[test]
    fn default_ship_flood_and_pump_tools_change_interior_water() {
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
        };
        let mut structure = ShipStructure::load_for_choice(&choice);
        let index = structure
            .interior
            .iter()
            .position(|&is_interior| is_interior)
            .expect("default ship should contain a sealed air compartment");
        let x = index % structure.width;
        let y = index / structure.width;
        let local = Vec2::new(
            -SHIP_HALF_WIDTH + (x as f32 + 0.5) * SHIP_HALF_WIDTH * 2.0 / structure.width as f32,
            structure.half_height
                - (y as f32 + 0.5) * structure.half_height * 2.0 / structure.height as f32,
        );

        structure.flood_near(local, 10.0);
        assert!(structure.water[index] > 0.0);
        structure.pump_near(local, 10.0);
        assert_eq!(structure.water[index], 0.0);
    }

    #[test]
    fn hull_breach_tool_cuts_a_mesh_hole_and_opens_adjacent_links() {
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
        };
        let mut structure = ShipStructure::load_for_choice(&choice);
        let index = (0..structure.solid.len())
            .find(|&i| {
                structure.solid[i]
                    && is_exposed_node(&structure.solid, structure.width, structure.height, i)
            })
            .expect("default hull should contain an exposed material node");
        let x = index % structure.width;
        let y = index / structure.width;
        let local = Vec2::new(
            -SHIP_HALF_WIDTH + (x as f32 + 0.5) * SHIP_HALF_WIDTH * 2.0 / structure.width as f32,
            structure.half_height
                - (y as f32 + 0.5) * structure.half_height * 2.0 / structure.height as f32,
        );
        let before = build_deformable_ship_mesh(&structure)
            .indices()
            .map(|indices| indices.len())
            .expect("initial ship mesh should contain triangles");

        assert!(structure.breach_near(local, 0.1));

        let after = build_deformable_ship_mesh(&structure)
            .indices()
            .map(|indices| indices.len())
            .expect("breached ship mesh should retain triangles");
        assert!(structure.breached[index]);
        assert!(structure.leaking[index]);
        assert!(after < before, "breach should remove hull triangles");
        assert!(
            structure
                .springs
                .iter()
                .filter(|spring| spring.a == index || spring.b == index)
                .all(|spring| spring.broken)
        );
    }

    #[test]
    fn submerged_hull_breach_fills_diagonal_interior_cells() {
        let width = 3;
        let height = 3;
        let mut solid = vec![false; width * height];
        let mut leaking = vec![false; width * height];
        let mut interior = vec![false; width * height];
        let mut water = vec![0.0; width * height];
        solid[4] = true;
        leaking[4] = true;
        interior[0] = true;
        let positions = vec![Vec2::ZERO; width * height];

        apply_hull_leak_inflow(
            &mut water,
            &solid,
            &leaking,
            &interior,
            &positions,
            width,
            height,
            30.0,
            Vec2::new(0.0, SEA_LEVEL - 10.0),
            0.0,
            25.0,
            40.0,
            9.81,
            1.0 / 60.0,
            1.0,
        );

        assert!(water[0] > 0.0);
        assert_eq!(water[1], 0.0);
    }

    #[test]
    fn source_material_maps_build_physics_for_multiple_ships() {
        let mapped_ships = ShipCatalog::discover()
            .0
            .into_iter()
            .filter(|choice| choice.material_map)
            .collect::<Vec<_>>();
        assert!(
            mapped_ships.len() > 1,
            "catalog should discover paired maps"
        );

        for choice in mapped_ships {
            let structure = ShipStructure::load_for_choice(&choice);
            assert!(
                structure.solid.iter().any(|&solid| solid),
                "{} material map should produce solid physics nodes",
                choice.name
            );
            assert!(
                !structure.springs.is_empty(),
                "{} material map should produce structural springs",
                choice.name
            );
            assert_eq!(
                structure.texel_materials.len(),
                structure.texel_width * structure.texel_height,
                "{} must retain one material slot per source texel",
                choice.name
            );
            assert_eq!(
                structure.texel_strut_masks.len(),
                structure.texel_materials.len(),
                "{} must retain one connectivity mask per source texel",
                choice.name
            );
        }
    }

    #[test]
    fn source_texels_keep_exact_material_identity_and_connectivity() {
        let choice = ShipCatalog::discover()
            .0
            .into_iter()
            .find(|choice| {
                (choice.name == "RMS Titanic" || choice.name == "Titanic") && choice.material_map
            })
            .expect("Titanic source material map");
        let image = image::open(format!("assets/{}", choice.physics_asset))
            .expect("Titanic source material image")
            .to_rgba8();
        let palette = parse_material_palette(include_str!("../assets/config/materials.json"))
            .expect("the shipped material palette should be valid JSON");
        let structure = ShipStructure::load_for_choice(&choice);

        assert_eq!(structure.texel_width, image.width() as usize);
        assert_eq!(structure.texel_height, image.height() as usize);
        let (index, material) = structure
            .texel_materials
            .iter()
            .enumerate()
            .find_map(|(index, material)| material.map(|material| (index, material)))
            .expect("material map should contain a recognized material texel");
        let pixel = image.get_pixel(
            (index % structure.texel_width) as u32,
            (index / structure.texel_width) as u32,
        );
        let rgb = (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]);
        let expected = palette.get(&rgb).expect("texel's exact palette color");
        assert_eq!(material.mass, expected.mass);
        assert_eq!(material.strength, expected.strength);
        assert_eq!(material.tensile_strength, expected.tensile_strength);
        assert_eq!(material.compressive_strength, expected.compressive_strength);
        assert_eq!(
            structure.texel_solid[index],
            structure.texel_materials[index].is_some()
        );
        assert_eq!(
            structure.texel_strut_masks[index],
            build_strut_masks(
                &structure.texel_solid,
                structure.texel_width,
                structure.texel_height
            )[index]
        );
        assert_eq!(
            structure.texel_positions.len(),
            image.width() as usize * image.height() as usize
        );
    }

    #[test]
    fn deformable_ship_mesh_uses_source_texel_resolution() {
        let choice = ShipCatalog::discover()
            .0
            .into_iter()
            .find(|choice| {
                (choice.name == "RMS Titanic" || choice.name == "Titanic") && choice.material_map
            })
            .expect("Titanic source material map");
        let structure = ShipStructure::load_for_choice(&choice);
        let mesh = build_deformable_ship_mesh(&structure);
        let expected_vertices = structure.texel_width * structure.texel_height;
        let actual_vertices = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .map_or(0, <[[f32; 3]]>::len);

        assert_eq!(actual_vertices, expected_vertices);
    }

    #[test]
    fn clustered_material_properties_preserve_all_pixel_mass() {
        let mut samples = MaterialSampleAccumulator::default();
        samples.add(MaterialProperties {
            strength: 10.0,
            tensile_strength: 8.0,
            compressive_strength: 40.0,
            mass: 2.0,
            hull: true,
            ground: false,
            rope: false,
            invisible: false,
        });
        samples.add(MaterialProperties {
            strength: 30.0,
            tensile_strength: 28.0,
            compressive_strength: 80.0,
            mass: 5.0,
            hull: false,
            ground: false,
            rope: false,
            invisible: false,
        });

        let properties = samples.into_properties().expect("two material samples");
        assert_eq!(properties.strength, 20.0);
        assert_eq!(properties.tensile_strength, 18.0);
        assert_eq!(properties.compressive_strength, 60.0);
        assert_eq!(properties.mass, 3.5);
        assert!(!properties.hull);
    }

    #[test]
    fn palette_uses_reference_strength_fallbacks_and_tensile_override() {
        let palette = parse_material_palette(
            r##"[
                {"strength": 12, "colour": "#010203"},
                {"strength": 20, "tensileStrength": 18, "colour": "#040506"}
            ]"##,
        )
        .expect("test palette should be valid JSON");

        let fallback = palette.get(&0x010203).expect("singular colour");
        assert_eq!(fallback.tensile_strength, 12.0);
        assert_eq!(fallback.compressive_strength, 48.0);

        let override_material = palette.get(&0x040506).expect("tensile override");
        assert_eq!(override_material.tensile_strength, 18.0);
        assert_eq!(override_material.compressive_strength, 80.0);
    }

    #[test]
    fn hsv_picker_round_trips_blue_and_grayscale() {
        let blue = Vec3::new(0.0, 71.0 / 255.0, 159.0 / 255.0);
        let (hue, saturation, value) = rgb_to_hsv(blue);
        let round_trip = hsv_to_rgb(hue, saturation, value);

        assert!(round_trip.distance(blue) < 0.0001);
        assert!(hsv_to_rgb(0.0, 0.0, 0.5).distance(Vec3::splat(0.5)) < 0.0001);
    }

    #[test]
    fn ship_search_filters_names_without_reordering_catalog() {
        let catalog = ShipCatalog(
            vec![
                ShipChoice {
                    name: "RMS Titanic".to_owned(),
                    asset: "titanic.png".to_owned(),
                    physics_asset: "titanic_base.png".to_owned(),
                    material_map: true,
                    scale: 1.0,
                },
                ShipChoice {
                    name: "Queen Mary".to_owned(),
                    asset: "queen.png".to_owned(),
                    physics_asset: "queen.png".to_owned(),
                    material_map: false,
                    scale: 1.0,
                },
            ],
            Vec::new(),
        );

        assert_eq!(filtered_ship_indices(&catalog, "QUEEN"), vec![1]);
        assert_eq!(filtered_ship_indices(&catalog, ""), vec![0, 1]);
    }

    #[test]
    fn imported_ship_catalog_entries_do_not_duplicate_asset_paths() {
        let choice = ShipChoice {
            name: "Imported".to_owned(),
            asset: "user_ships/imported.png".to_owned(),
            physics_asset: "user_ships/imported.png".to_owned(),
            material_map: false,
            scale: 1.0,
        };
        let mut catalog = ShipCatalog(Vec::new(), Vec::new());

        assert_eq!(catalog.add_imported(choice.clone()), 0);
        assert_eq!(catalog.add_imported(choice), 0);
        assert_eq!(catalog.0.len(), 1);
    }

    #[test]
    fn wave_frequency_matches_the_reference_shader() {
        assert!((wave_height(20.0, 0.0, 10.0, 40.0) - 7.0).abs() < 0.0001);
    }

    #[test]
    fn toolbox_header_hit_area_matches_the_visible_header() {
        assert!(point_in_toolbox_header(Vec2::new(-465.0, 333.0)));
        assert!(point_in_toolbox_header(Vec2::new(-640.0, 316.0)));
        assert!(!point_in_toolbox_header(Vec2::new(-289.0, 333.0)));
        assert!(!point_in_toolbox_header(Vec2::new(-465.0, 315.0)));
    }

    #[test]
    fn interior_air_classification_finds_sealed_compartments() {
        let width = 7;
        let height = 7;
        let mut solid = vec![false; width * height];
        for y in 1..6 {
            for x in 1..6 {
                if x == 1 || x == 5 || y == 1 || y == 5 {
                    solid[y * width + x] = true;
                }
            }
        }

        let interior = classify_interior_air(&solid, width, height);
        assert!(interior[3 * width + 3]);
        assert!(!interior[0]);

        solid[width + 3] = false;
        let breached = classify_interior_air(&solid, width, height);
        assert!(!breached[3 * width + 3]);
    }

    #[test]
    fn diagonal_hull_gaps_do_not_trap_interior_air() {
        let width = 5;
        let height = 5;
        let mut solid = vec![false; width * height];
        for y in 1..4 {
            for x in 1..4 {
                if x == 1 || x == 3 || y == 1 || y == 3 {
                    solid[y * width + x] = true;
                }
            }
        }
        solid[width + 1] = false;

        let interior = classify_interior_air(&solid, width, height);

        assert!(!interior[2 * width + 2]);
    }

    #[test]
    fn flooding_flows_down_without_losing_water() {
        let mut water = vec![1.0, 0.0];
        let interior = vec![true, true];

        flow_interior_water(&mut water, &interior, 1, 2, 1.0, 9.81, 0.1, 1.0, 0.0);

        assert!(water[0] < 1.0);
        assert!(water[1] > 0.0);
        assert!((water.iter().sum::<f32>() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn simultaneous_inflows_respect_cell_capacity_and_conserve_water() {
        let mut water = vec![1.0, 0.0, 1.0];
        let interior = vec![true; 3];

        flow_interior_water(&mut water, &interior, 3, 1, 1.0, 9.81, 100.0, 1.0, 0.0);

        assert!(water.iter().all(|amount| (0.0..=1.0).contains(amount)));
        assert!((water.iter().sum::<f32>() - 2.0).abs() < 0.0001);
        assert!((water[1] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn repeated_water_flow_remains_bounded_and_conservative() {
        let mut water = (0..49)
            .map(|index| ((index * 37 % 101) as f32) / 100.0)
            .collect::<Vec<_>>();
        let interior = vec![true; water.len()];
        let initial_total = water.iter().sum::<f32>();

        for _ in 0..200 {
            flow_interior_water(&mut water, &interior, 7, 7, 1.0, 9.81, 1.0 / 60.0, 1.5, 0.8);
        }

        assert!(water.iter().all(|amount| (0.0..=1.0).contains(amount)));
        assert!((water.iter().sum::<f32>() - initial_total).abs() < 0.001);
    }

    #[test]
    fn strut_masks_encode_reciprocal_eight_neighbor_links() {
        let solid = vec![true, true, false, false, true, true, false, false, true];
        let masks = build_strut_masks(&solid, 3, 3);

        assert_ne!(masks[0] & (1 << 0), 0);
        assert_ne!(masks[1] & (1 << 4), 0);
        assert_ne!(masks[0] & (1 << 1), 0);
        assert_ne!(masks[4] & (1 << 5), 0);
        assert_eq!(masks[2], 0);
        assert_eq!(masks[8] & (1 << 0), 0);
        assert_eq!(masks[8] & (1 << 1), 0);
    }

    #[test]
    fn exposed_hull_nodes_use_the_source_eight_neighbor_mask() {
        let solid = vec![true; 9];
        assert!(is_exposed_node(&solid, 3, 3, 0));
        assert!(!is_exposed_node(&solid, 3, 3, 4));

        let mut diagonal_gap = solid;
        diagonal_gap[0] = false;
        assert!(is_exposed_node(&diagonal_gap, 3, 3, 4));
    }

    #[test]
    fn spring_forces_restore_rest_length_and_resist_separation() {
        let rest_force = spring_force(Vec2::new(10.0, 0.0), Vec2::ZERO, 10.0, 1.0, 1.0, 1.0, 1.0);
        let force = spring_force(
            Vec2::new(12.0, 0.0),
            Vec2::new(3.0, 0.0),
            10.0,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let compression = spring_force(Vec2::new(8.0, 0.0), Vec2::ZERO, 10.0, 1.0, 1.0, 1.0, 0.0);
        let transverse_damping = spring_force(
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 3.0),
            10.0,
            1.0,
            1.0,
            1.0,
            1.0,
        );

        assert_eq!(rest_force, Vec2::ZERO);
        assert!(force.x > 0.0);
        assert_eq!(force.y, 0.0);
        assert!(transverse_damping.y > 0.0);
        assert!(should_break_spring(force.x, 1.0, 1.0, 1.0));
        assert!(compression.x < 0.0);
        assert!(should_break_spring(compression.x, 1.0, 1.0, 1.0));
        assert!(!should_break_spring(-0.5, 1.0, 1.0, 1.0));
        assert!(!should_break_spring(2.0, 3.0, 1.0, 1.0));
    }

    #[test]
    fn spring_breakage_uses_elastic_load_not_damping_load() {
        let velocity_only_force = spring_force(
            Vec2::new(10.0, 0.0),
            Vec2::new(100.0, 100.0),
            10.0,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let elastic_load = spring_extension_force(Vec2::new(10.0, 0.0), 10.0, 1.0, 1.0, 1.0);

        assert!(velocity_only_force.length() > 1.0);
        assert_eq!(elastic_load, 0.0);
        assert!(!should_break_spring(elastic_load, 1.0, 1.0, 1.0));
    }

    #[test]
    fn spring_loads_and_break_thresholds_match_source_solver_scaling() {
        let break_scale = source_spring_break_scale(50, 1.0 / 60.0);
        assert!((break_scale - 150_000.0).abs() < 0.1);
        assert!((65.0 * break_scale - 9_750_000.0).abs() < 10.0);
    }

    #[test]
    fn default_ship_solver_does_not_tear_without_user_damage() {
        let catalog = ShipCatalog::discover();
        let structure = ShipStructure::load_for_choice(&catalog.0[0]);
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(Simulation::default());
        app.insert_resource(structure);
        app.add_systems(Update, simulate_ship_physics);

        for _ in 0..120 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.update();
        }

        let structure = app.world().resource::<ShipStructure>();
        let broken_springs = structure
            .springs
            .iter()
            .filter(|spring| spring.broken)
            .count();
        let mean_displacement = structure
            .positions
            .iter()
            .zip(&structure.rest_positions)
            .zip(&structure.solid)
            .filter_map(|((position, rest), solid)| solid.then_some(*position - *rest))
            .fold((Vec2::ZERO, 0usize), |(sum, count), displacement| {
                (sum + displacement, count + 1)
            });
        let mean_displacement = mean_displacement.0 / mean_displacement.1.max(1) as f32;
        let max_deformation = structure
            .positions
            .iter()
            .zip(&structure.rest_positions)
            .zip(&structure.solid)
            .filter_map(|((position, rest), solid)| {
                solid.then_some((*position - *rest - mean_displacement).length())
            })
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);
        assert_eq!(
            broken_springs,
            0,
            "unassisted simulation broke {broken_springs} of {} springs",
            structure.springs.len()
        );
        assert!(
            max_deformation < structure.half_height * 0.25,
            "unassisted simulation deformed a hull node by {max_deformation}"
        );
    }

    #[test]
    fn titanic_cpu_solver_does_not_tear_without_user_damage() {
        let catalog = ShipCatalog::discover();
        let choice = catalog
            .0
            .iter()
            .find(|choice| choice.name == "RMS Titanic" || choice.name == "Titanic")
            .expect("Titanic reference fixture");
        let structure = ShipStructure::load_for_choice(choice);
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(Simulation::default());
        app.insert_resource(structure);
        app.add_systems(Update, simulate_ship_physics);

        for _ in 0..120 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.update();
        }

        let structure = app.world().resource::<ShipStructure>();
        let broken_springs = structure
            .springs
            .iter()
            .filter(|spring| spring.broken)
            .count();
        let mean_displacement = structure
            .positions
            .iter()
            .zip(&structure.rest_positions)
            .zip(&structure.solid)
            .filter_map(|((position, rest), solid)| solid.then_some(*position - *rest))
            .fold((Vec2::ZERO, 0usize), |(sum, count), displacement| {
                (sum + displacement, count + 1)
            });
        let mean_displacement = mean_displacement.0 / mean_displacement.1.max(1) as f32;
        let max_deformation = structure
            .positions
            .iter()
            .zip(&structure.rest_positions)
            .zip(&structure.solid)
            .filter_map(|((position, rest), solid)| {
                solid.then_some((*position - *rest - mean_displacement).length())
            })
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);
        assert_eq!(
            broken_springs,
            0,
            "unassisted simulation broke {broken_springs} of {} springs",
            structure.springs.len()
        );
        assert!(
            max_deformation < structure.half_height * 0.25,
            "unassisted simulation deformed a hull node by {max_deformation}"
        );
    }

    #[test]
    fn cpu_spring_force_matches_original_iteration_mass_damping_and_rope_formula() {
        let delta = Vec2::new(1.01, 0.0);
        let velocity = Vec2::new(0.0, 0.5);
        let frame_delta = 1.0 / 60.0;
        let (force, elastic) = source_cpu_spring_force(
            delta,
            velocity,
            1.0,
            206.0,
            300.0,
            1.0,
            1.0,
            false,
            50,
            frame_delta,
        );
        let b = 0.03 * (50.0 / frame_delta) * 50.0;
        let expected = (delta.length() - 1.0) * 750.0 * 206.0 * b;
        assert!((elastic - expected).abs() < expected.abs() * 1.0e-5);
        assert_eq!(force.y, b * 0.5);
        let (_, rope) = source_cpu_spring_force(
            delta,
            velocity,
            1.0,
            206.0,
            300.0,
            1.0,
            1.0,
            true,
            50,
            frame_delta,
        );
        assert!((rope - elastic * 0.001).abs() < 0.01);
        let (_, doubled_iterations) = source_cpu_spring_force(
            delta,
            velocity,
            1.0,
            206.0,
            300.0,
            1.0,
            1.0,
            false,
            100,
            frame_delta,
        );
        assert!((doubled_iterations - elastic * 4.0).abs() < 1.0);
        let (_, doubled_mass) = source_cpu_spring_force(
            delta,
            velocity,
            1.0,
            412.0,
            600.0,
            1.0,
            1.0,
            false,
            50,
            frame_delta,
        );
        assert!((doubled_mass - elastic * 2.0).abs() < 1.0);
    }

    #[test]
    fn hull_deformation_is_bounded_relative_to_its_rest_position() {
        let rest = Vec2::new(4.0, -3.0);
        let displaced = clamp_hull_deformation(Vec2::new(104.0, -3.0), rest, 2.5);

        assert!((displaced - rest).length() <= 2.5);
    }

    #[test]
    fn heavier_ship_materials_stiffen_springs_with_a_bounded_scale() {
        let light = spring_force(Vec2::new(11.0, 0.0), Vec2::ZERO, 10.0, 1.0, 1.0, 1.0, 0.0);
        let heavy = spring_force(Vec2::new(11.0, 0.0), Vec2::ZERO, 10.0, 4.0, 8.0, 1.0, 0.0);
        let very_heavy = spring_force(Vec2::new(11.0, 0.0), Vec2::ZERO, 10.0, 20.0, 80.0, 1.0, 0.0);

        assert!(heavy.length() > light.length());
        assert_eq!(very_heavy, heavy);
    }

    #[test]
    fn spring_force_is_balanced_between_endpoints() {
        let force = spring_force(
            Vec2::new(11.0, 0.0),
            Vec2::new(0.0, 0.0),
            10.0,
            1.0,
            4.0,
            1.0,
            0.0,
        );
        let mass_a = 1.0;
        let mass_b = 4.0;
        let acceleration_a = force / mass_a;
        let acceleration_b = -force / mass_b;
        let momentum_change = acceleration_a * mass_a + acceleration_b * mass_b;

        assert!(momentum_change.length() < 0.0001);
    }

    #[test]
    fn breaking_spring_clears_reciprocal_structural_links() {
        let mut spring = Spring {
            a: 2,
            b: 3,
            direction: 1,
            rest_length: 1.0,
            tensile_strength: 1.0,
            compressive_strength: 1.0,
            broken: false,
        };
        let mut masks = vec![0; 5];
        masks[2] = 1 << 1;
        masks[3] = 1 << 5;
        break_spring(&mut spring, &mut masks);

        assert!(spring.broken);
        assert_eq!(masks[2] & (1 << 1), 0);
        assert_eq!(masks[3] & (1 << 5), 0);

        restore_spring(&mut spring, &mut masks);

        assert!(!spring.broken);
        assert_ne!(masks[2] & (1 << 1), 0);
        assert_ne!(masks[3] & (1 << 5), 0);
    }

    #[test]
    fn water_flows_across_diagonal_openings() {
        let mut water = vec![1.0, 0.0, 0.0, 0.0];
        let interior = vec![true, false, false, true];

        flow_interior_water(&mut water, &interior, 2, 2, 1.0, 9.81, 0.1, 1.0, 0.0);

        assert!(water[0] < 1.0);
        assert!(water[3] > 0.0);
        assert!((water.iter().sum::<f32>() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn floor_collision_keeps_nodes_above_floor_and_reverses_vertical_speed() {
        let (position, velocity) =
            resolve_floor_collision(Vec2::new(0.0, 1.0), Vec2::new(2.0, -3.0), 0.0, 0.1);

        assert!(position.y >= 0.0);
        assert!(velocity.y > 0.0);
        assert!(velocity.x.abs() < 20.0);
    }

    #[test]
    fn water_fill_increases_effective_hull_density() {
        let empty = effective_density(2409.0, 0.0, true, false, 0.085, 1.0);
        let flooded = effective_density(2409.0, 1.0, true, false, 0.085, 1.0);

        assert!(empty < flooded);
        assert!(buoyancy_acceleration(9.81, 1.0, 1025.0, empty) > 0.0);
        assert!(buoyancy_acceleration(9.81, 1.0, 1025.0, flooded) < 0.0);
    }
}

mod al_buffer;
mod al_context;
#[cfg(windows)]
mod al_capabilities;
#[cfg(windows)]
mod al_capability_tables;
mod al_context_start_reference;
mod al_device;
mod al_resource;
mod al_source;
#[cfg(windows)]
mod native_openal;
#[cfg(windows)]
mod native_vorbis;
#[cfg(windows)]
mod native_music;
#[cfg(windows)]
mod native_music_load;
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
mod float_data_texture_config;
mod float_property;
mod floor;
mod floor_camera_callback;
mod floor_companion;
mod force_data;
mod fragment_shaders;
mod fragment_shaders_texture;
mod fragment_shaders_uv;
mod fragment_shaders_white;
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
mod gui_gpu_blend;
mod gui_color_picker;
mod gui_color_picker_input;
mod gui_icon_mips;
mod gui_texture;
mod gui_kt;
mod gui_reset_ship;
mod ship_runtime_reset;
mod ship_visual_replacement;
#[cfg(test)]
mod ship_visual_replacement_tests;
#[cfg(test)]
mod ship_visual_gpu_tests;
mod live_ship_catalog;
mod gui_tool_factory;
mod i_drawable;
mod image_data;
mod input_handler;
mod input_handler_delegate;
mod int_property;
mod java_string;
mod jvm_character;
mod kotlin_helpers;
mod main_counter_callback;
mod main_counter_title;
mod main_debug;
mod main_flat_files;
mod main_frame;
mod main_globals;
mod main_initial_ship;
mod main_logging;
mod main_music_load;
mod main_screen_fbo_texture;
mod main_ship_file_predicate;
mod main_ship_file_resource;
mod main_ship_input;
mod main_shutdown;
mod main_shutdown_watchdog;
mod main_startup;
mod mask_struts_data;
mod mask_struts_ground_flag;
mod mask_struts_hull_flag;
mod mask_struts_material_masks;
mod mask_struts_rope_flag;
mod mask_struts_structural_predicate;
mod mask_struts_texture_config;
mod mask_struts_water_predicate;
mod mass_strength_data;
mod mass_strength_data_texture_config;
mod materials;
mod mem_util;
mod model;
mod monitor;
mod monitor_scale;
mod monitor_video_mode;
mod music_player;
mod music_controls;
mod music_player_icon_config;
mod music_player_icons;
mod music_player_progress_reference;
mod music_player_ui;
mod music_player_volume_reference;
mod palette_gen;
mod passes;
mod physics_full_screen;
mod pos_vel_data;
mod pos_vel_data_texture_config;
mod render_buffer;
mod render_fbo;
mod resource;
mod screen_fbo;
mod sea;
mod sea_camera_callback;
mod sea_companion;
mod sea_resolution_callback;
mod shaded_model;
mod shader;
mod shader_program;
mod ship;
mod ship_black_texture;
mod ship_black_texture_config;
mod ship_camera_callback;
mod ship_companion;
mod ship_data;
mod ship_fragment_shader;
mod ship_geometry_shader;
mod ship_physics;
mod ship_physics_shaders;
mod ship_physics_stencil;
mod ship_render;
mod ship_resource_when_mappings;
mod ship_resources;
mod ship_shaders;
mod ship_strut_shader;
mod ship_struts;
mod ship_struts_camera_callback;
mod ship_struts_companion;
mod ship_thumbnail;
mod ship_upload;
mod ship_upload_resources;
mod ship_upload_layout;
mod ship_gpu_geometry;
mod ship_coverage;
mod ship_upload_preview;
mod sky;
mod sky_camera_reference;
mod sky_companion;
mod sky_free_camera_reference;
mod sky_free_resolution_reference;
mod sky_resolution_reference;
mod sky_star_field;
mod sky_stars_texture;
mod sky_texture;
mod source_ship;
mod source_ship_physics;
mod tee_output_stream;
mod texture;
mod texture_1d;
mod texture_2d;
#[cfg(windows)]
mod native_gl_mips;
#[cfg(windows)]
mod native_gl_backend;
#[cfg(all(test, windows))]
mod native_gl_backend_tests;
#[cfg(all(test, windows))]
mod native_gl_physics_reference;
mod texture_2d_array;
mod textured_fbo;
mod time_sync;
mod time_sync_reporter;
mod toolbox;
mod toolbox_layout;
mod toolbox_tools;
mod toolbox_viewport;
mod ui_font;
mod text_edit_state;
mod editor_clipboard;
mod ui_mouse_input;
mod ui_text_viewport;
mod ui_numeric;
mod ui_window_scroll;
mod ui_description_scroll;
mod ui_text_visuals;
mod text_edit_layout;
mod source_font_advances;
mod source_font_text;
mod text_edit_undo;
mod ui_scrollbar;
mod ship_browser_ui;
mod ui_descriptions;
mod ui_classic_appearance;
mod source_ui_metrics;
mod source_tools_layout;
mod source_tools_popup;
mod ui_checkmarks;
mod ui_arrows;
mod toolbox_references;
mod toolbox_reload;
mod toolbox_reload_file_predicate;
mod toolbox_reload_filesystem;
mod toolbox_reload_loop;
mod toolbox_reload_name_comparator;
mod toolbox_render_3;
mod toolbox_settings;
mod toolbox_ship_browser;
mod tools;
mod typed_data_holder;
mod uint8_data_holder;
mod uint8_data_texture_config;
mod uv_model;
mod vao;
mod vbo;
mod vector2_property;
mod vertex_shaders;
mod vertex_shaders_none;
mod vertex_shaders_nothing;
mod vertex_shaders_transform;
mod water_data;
mod window;
mod window_bevy;
mod ui_input_capture;
mod window_keyboard;
mod window_characters;
mod window_native_characters;
mod window_framebuffer_callback;

use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::{ClearColorConfig, ScalingMode, visibility::RenderLayers},
    input::keyboard::KeyboardInput,
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
    GpuShipPhysicsAssets, GpuShipPhysicsPlugin, GpuShipPhysicsSnapshot, gpu_settings,
    make_gpu_ship_physics_assets, replace_gpu_ship_physics,
};
use ship_resources::{
    ShipLayer, ShipResource, ShipResourceFile, ShipResourceType, parse_resource_path,
};
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

struct ActiveMusicPlugin;
impl Plugin for ActiveMusicPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(windows)]
        app.add_plugins(native_music::NativeMusicPlugin);
    }
}
fn active_music_plugin() -> ActiveMusicPlugin { ActiveMusicPlugin }
fn active_default_plugins() -> bevy::app::PluginGroupBuilder {
    let plugins = DefaultPlugins.build();
    #[cfg(windows)]
    let plugins = plugins.disable::<bevy::audio::AudioPlugin>();
    plugins
}
fn main() {
    let mut app=game_app(active_default_plugins()
                .set(bevy::asset::AssetPlugin {
                    file_path: std::env::current_dir()
                        .expect("working directory")
                        .join("assets")
                        .to_string_lossy()
                        .into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    // The translated native close callback can reset shouldClose.
                    close_when_requested: false,
                    primary_window: Some(Window {
                        title: "Loading...".into(),
                        ime_enabled: true,
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: (2554, 1378).into(),
                        ..default()
                    }),
                    ..default()
                }));
    app.run();
    #[cfg(windows)]
    native_gl_mips::release_current_thread();
}

// Share the actual game systems with live runtime checks.
fn game_app(plugins: bevy::app::PluginGroupBuilder) -> App {
    main_globals::set_global_materials(std::sync::Arc::new(materials::SourceMaterials::from_file(
        std::path::Path::new("assets/config/materials.json"), |error| eprintln!("{error}"),
    )));
    let ship_catalog = ShipCatalog::discover();
    let (initial_structure,active_thumbnail)=ship_runtime_reset::choice(&ship_catalog.0[0]);
    let mut app=App::new();
    app.insert_non_send_resource(live_ship_catalog::LiveCatalog::extracted());
    app.add_plugins(plugins)
        .add_plugins((
            Material2dPlugin::<InternalWaterMaterial>::default(),
            Material2dPlugin::<tools::brush_preview::BrushMaterial>::default(),
            Material2dPlugin::<ShipMaterial>::default(),
            Material2dPlugin::<ship_coverage::CoverageMaterial>::default(),
            Material2dPlugin::<sky::SkyMaterial>::default(),
            Material2dPlugin::<ReflectionMaterial>::default(),
            Material2dPlugin::<OceanSurfaceMaterial>::default(),
            Material2dPlugin::<sea::SeaMaterial>::default(),
            Material2dPlugin::<OceanDepthMaterial>::default(),
            Material2dPlugin::<UnderwaterEffectMaterial>::default(),
            GpuShipPhysicsPlugin,
            resource::ResourcePlugin,
            window_bevy::WindowBridgePlugin,
            render_fbo::RenderFboPlugin,
            sky_star_field::StarFieldPlugin,
        ))
        .add_plugins(gui_gpu_blend::SourceGuiBlendPlugin)
        .insert_resource(ClearColor(Color::srgb(0.40, 0.68, 0.82)))
        .init_resource::<Simulation>()
        .init_resource::<ship_browser_ui::BrowserLayout>()
        .init_resource::<live_ship_catalog::PendingSelection>()
        .init_resource::<ship_upload::SourceShipUpload>()
        .init_resource::<ShipUploadUiState>()
        .init_resource::<editor_clipboard::Clipboard>()
        .init_resource::<ship_upload_layout::Layout>()
        .init_resource::<ship_upload_preview::ActivePreview>()
        .init_resource::<ship_black_texture::SharedBlackTexture>()
        .init_resource::<time_sync::TimeSync>()
        .add_systems(Startup, time_sync::register_lifecycle)
        .add_systems(Last, time_sync::sync_frame)
        .insert_resource(music_player::MusicPlayer::discover())
        .add_plugins(active_music_plugin())
        .init_resource::<CameraControlState>()
        .init_resource::<tools::move_tool::MoveDragState>()
        .insert_resource(ship_catalog)
        .insert_resource(initial_structure)
        .insert_resource(active_thumbnail)
        .add_systems(Update,live_ship_catalog::advance.before(refresh_ship_catalog).before(load_selected_ship))
        .add_systems(Startup, (setup, floor::setup, sky::setup))
        .add_systems(Startup, sea::setup.after(setup))
        .add_systems(Startup, ship_coverage::setup.after(setup))
        .add_systems(Startup,tools::brush_preview::setup_overlay.after(setup))
        .add_systems(PostUpdate,tools::brush_preview::sync_overlay
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate,tools::brush_preview::source_camera_output
            .before(bevy::camera::CameraUpdateSystems))
        .add_systems(PostUpdate, ship_coverage::sync
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate))
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
        .add_systems(Update, sync_ship_upload_ui)
        .add_systems(Startup, ship_upload_layout::spawn)
        .add_systems(Startup, ui_text_visuals::spawn)
        .add_systems(Startup, ui_text_viewport::spawn)
        .add_systems(Startup, ui_description_scroll::spawn)
        .add_systems(PostUpdate,ui_description_scroll::draw.after(ui_text_viewport::update)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate,ui_text_viewport::update.after(ship_upload_layout::sync).before(ui_text_visuals::update)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate,ui_text_visuals::update.after(ship_upload_layout::sync)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(Update, ship_upload_layout::move_window.after(handle_ship_upload_ui).after(sync_ship_upload_ui).before(ship_upload_preview::click))
        .add_systems(Update, ship_upload_preview::click.after(handle_ship_upload_ui).before(load_selected_ship))
        .add_systems(PreUpdate, ship_upload_layout::scroll.after(bevy::input::InputSystems))
        .add_systems(PostUpdate, (ship_upload_layout::route, ship_upload_layout::sync).chain()
            .after(toolbox_viewport::route_layers)
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate,ship_upload_layout::sync_window_decorations.after(ship_upload_layout::sync).before(bevy::transform::TransformSystems::Propagate))
        .add_systems(Startup,source_tools_popup::setup)
        .add_systems(Update,(source_tools_popup::scroll,source_tools_popup::drag).chain().before(select_ship_layer).before(select_toolbox_tab_and_settings))
        .add_systems(PostUpdate,source_tools_popup::route_layers.after(toolbox_viewport::route_layers).after(source_tools_layout::sync))
        .add_systems(PostUpdate,source_tools_popup::sync.after(source_tools_layout::sync).before(bevy::camera::CameraUpdateSystems).before(bevy::transform::TransformSystems::Propagate))
        .init_resource::<source_ui_metrics::SourceUiMetrics>()
        .add_systems(PreUpdate, ship_upload_layout::prepare.after(source_ui_metrics::capture).before(ui_input_capture::update))
        .add_systems(PreUpdate, source_ui_metrics::capture.before(update_tool_panel_layout))
        .add_systems(PreUpdate, source_tools_layout::update.after(source_ui_metrics::capture).before(update_tool_panel_layout))
        .add_systems(PostUpdate, source_tools_layout::sync.after(ui_font::apply).before(ui_descriptions::update).before(ui_classic_appearance::apply).before(bevy::transform::TransformSystems::Propagate).before(bevy::camera::CameraUpdateSystems))
        .add_systems(PreUpdate, update_tool_panel_layout)
        .add_systems(PreUpdate, toolbox_viewport::scroll_settings.after(update_tool_panel_layout).after(bevy::input::InputSystems).after(ui_input_capture::update))
        .add_systems(Update, ui_scrollbar::handle.before(select_toolbox_tab_and_settings))
        .add_systems(Update, ship_browser_ui::input.after(select_ship_from_panel).before(load_selected_ship))
        .add_systems(Startup, ui_arrows::spawn)
        .add_systems(Update, ui_arrows::sync.after(sync_toolbox_visibility))
        .add_systems(PostUpdate, ui_classic_appearance::apply.after(ship_upload_layout::sync).after(toolbox_viewport::sync).after(ship_browser_ui::sync))
        .add_systems(Startup, ui_checkmarks::spawn.after(setup))
        .add_systems(PostUpdate, ui_checkmarks::sync.after(toolbox_viewport::route_layers).before(bevy::sprite::update_text2d_layout))
        .add_systems(Startup, ui_descriptions::spawn)
        .add_systems(PostUpdate, ui_descriptions::update.after(ship_browser_ui::sync).after(ship_upload_layout::sync).before(bevy::sprite::update_text2d_layout))
        .add_systems(PostUpdate, ui_descriptions::position.after(ui_descriptions::update).after(bevy::sprite::update_text2d_layout).before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate, ship_browser_ui::sync.after(toolbox_viewport::sync)
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(PostUpdate, (toolbox_viewport::route_layers, toolbox_viewport::sync).chain()
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate))
        .add_systems(
            Update,
            sync_tool_panel_position.after(sync_ship_layer_dropdown),
        )
        .add_systems(
            Update,
            sync_ship_layer_dropdown.after(update_ship_layer_label),
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
                    sync_source_daylight,
                    select_ship_from_panel,
                    handle_ship_upload_ui,
                    select_ship_layer,
                    refresh_ship_catalog,
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
                    sync_tool_icons,
                    sync_tool_panel_visibility,
                    sync_settings_ui,
                    update_ship_layer_label,
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
        .add_systems(Update, music_controls::handle_sliders.before(music_player::sync))
        .add_systems(Update, music_controls::sync.after(music_player::sync))
        .add_systems(PostUpdate, ui_font::apply.before(bevy::sprite::update_text2d_layout));
    app
}

#[derive(Resource)]
struct Simulation {
    elapsed: f32,
    flooding: f32,
    tool: Tool,
    ship_index: usize,
    selected_layer: usize,
    layer_dropdown_open: bool,
    tool_panel_offset: Vec2,
    source_tools: Option<source_tools_layout::Layout>,
    layer_popup_scroll: f32,
    settings_scroll: [f32; 6],
    settings_scroll_max: [f32; 6],
    toolbox_page_bottom: [f32; 6],
    ship_search: String,
    ship_search_units:Vec<u16>,
    ship_search_active: bool,
    catalog_revision: u64,
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
    sea_color_memory: gui_color_picker_input::ColorMemory,
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
        let global_materials = main_globals::get_global_materials();
        // ShipData uses the original BASE pixels, including invisible materials
        // and RGB values with zero alpha. Visibility filtering applies solely
        // to BaseDerivedTextureShipResource, never to the physics material map.
        let source_data = if choice.material_map {
            let original =
                ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(&image_path))
                    .and_then(|thumbnail| {
                        let binding = ship_thumbnail::SourceShipDataThumbnailCurrent {
                            thumbnail: &thumbnail,
                            global: &main_globals::get_global_materials,
                        };
                        ship_data::SourceShipData::from_thumbnail(&binding)?.to_owned_adapter()
                    });
            match original {
                Ok(data) => data,
                Err(error) => {
                    bevy::log::error!("Could not construct source ship data: {error}");
                    ship_data::ShipData::new(image, global_materials.to_owned_adapter())
                }
            }
        } else {
            ship_data::ShipData::new(image, global_materials.to_owned_adapter())
        };
        Self::from_data(source_data,choice.material_map)
    }
    fn load_for_thumbnail(thumbnail:&ship_thumbnail::ShipThumbnail)->Result<Self,String>{
        let binding=ship_thumbnail::SourceShipDataThumbnailCurrent{thumbnail,global:&main_globals::get_global_materials};
        Ok(Self::from_data(ship_data::SourceShipData::from_thumbnail(&binding)?.to_owned_adapter()?,true))
    }
    fn from_data(source_data:ship_data::ShipData,material_map:bool)->Self{
        let image=&source_data.img;
        let source_width=image.width() as usize;let source_height=image.height() as usize;
        let half_height=(SHIP_HALF_WIDTH*source_height as f32/source_width.max(1) as f32).clamp(40.0,180.0);
        let width=source_width.div_ceil(PHYSICS_NODE_PIXELS);let height=source_height.div_ceil(PHYSICS_NODE_PIXELS);
        let white_background=if material_map{vec![false;source_width*source_height]}else{find_connected_white_background(image)};
        let mut solid = vec![false; width * height];
        let mut cell_materials = vec![None; width * height];
        let mut texel_solid = vec![false; source_width * source_height];
        let mut texel_materials = vec![None; source_width * source_height];
        for source_y in 0..source_height {
            for source_x in 0..source_width {
                let pixel = image.get_pixel(source_x as u32, source_y as u32).0;
                if !material_map && pixel[3] <= 8 {
                    continue;
                }
                let material = source_data
                    .material_at(source_x as u32, source_y as u32)
                    .map(MaterialProperties::from)
                    .or_else(|| {
                        (!material_map
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
            tool: Tool::None,
            // Main.java constructs ships/pacmaster by default; catalog discovery places it at index zero.
            ship_index: 0,
            selected_layer: 0,
            layer_dropdown_open: false,
            tool_panel_offset: Vec2::new(-360.0, -510.0),
            source_tools: None,
            layer_popup_scroll: 0.0,
            settings_scroll: [0.0; 6],
            settings_scroll_max: [0.0; 6],
            toolbox_page_bottom: [-350.0; 6],
            ship_search: String::new(),
            ship_search_units:Vec::new(),
            ship_search_active: false,
            catalog_revision: 0,
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
            sea_color_memory: gui_color_picker_input::ColorMemory::default(),
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
        match index {
            16 => {
                let minimum = self.water_steps.round().clamp(1.0, 1000.0);
                self.physics_iterations = (self.physics_iterations + delta)
                    .round()
                    .clamp(minimum, 1000.0);
                return;
            }
            17 => {
                let maximum = self.physics_iterations.round().clamp(1.0, 1000.0);
                self.water_steps = (self.water_steps + delta).round().clamp(1.0, maximum);
                return;
            }
            _ => {}
        }
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
            18 => {
                self.day = (self.day + delta).clamp(0.0, 1.0);
                return;
            }
            19 => {
                self.tool_size = (self.tool_size + delta).clamp(0.0, 1000.0);
                return;
            }
            20 => {
                self.music_volume = (self.music_volume + delta).clamp(0.0, 1.0);
                return;
            }
            21 => {
                self.sea_color.x = (self.sea_color.x + delta).clamp(0.0, 1.0);
                return;
            }
            22 => {
                self.sea_color.y = (self.sea_color.y + delta).clamp(0.0, 1.0);
                return;
            }
            23 => {
                self.sea_color.z = (self.sea_color.z + delta).clamp(0.0, 1.0);
                return;
            }
            24 => {
                self.sea_alpha = (self.sea_alpha + delta).clamp(0.0, 1.0);
                return;
            }
            _ => return,
        };
        let (minimum, maximum) = match index {
            0 => (0.1, 1.0e7),
            1 | 3 | 4 => (0.0, 1.0e7),
            2 => (-1.0e7, 1.0e7),
            5 | 6 => (0.0, 1000.0),
            7 => (0.0, 2.0),
            8 => (-1000.0, 1000.0),
            9 => (0.0, 1000.0),
            10 | 11 => (0.0, 2.0),
            12 => (0.0, 1.0e6),
            13 => (1.0e-6, 1.0),
            14 => (0.0, 1.0e6),
            15 => (0.0, 10.0),
            _ => (0.0, f32::MAX),
        };
        *target = (*target + delta).clamp(minimum, maximum);
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
    GeneratePalette,
}

#[derive(Clone, Copy)]
enum SettingValue {
    Number(usize),
}

#[derive(Clone, PartialEq)]
struct ShipChoice {
    name: String,
    asset: String,
    physics_asset: String,
    material_map: bool,
    scale: f32,
    source_key: Option<SourceShipKey>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceShipKey {
    directory: std::path::PathBuf,
    ship: String,
}
#[derive(Clone, PartialEq)]
struct ShipLayerChoice {
    name: ShipLayer,
    asset: String,
}

#[derive(Resource, Clone, PartialEq)]
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
                None,
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
                push_ship_choice(&mut choices, &path, &path, name, false, None);
            }
        }

        fn collect_material_maps(folder: &Path, root: &Path, choices: &mut Vec<ShipChoice>) {
            let Ok(entries) = std::fs::read_dir(folder) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
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
                push_ship_choice(choices, &texture, &path, name, true, None);
            }
        }
        collect_material_maps(root, root, &mut choices);

        // ShipResource.fromFile in the original game groups *_BASE, *_TEXTURE,
        // *_INLIGHTS and *_EXLIGHTS files by ship and optional Layer.
        let mut source_resources = Vec::new();
        collect_source_ship_resources(Path::new("assets/source_ships"), &mut source_resources);
        for (source_key, resources) in &source_resources {
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
                thumbnail.name().unwrap_or(&base.ship).to_owned(),
                true,
                Some(source_key.clone()),
            );
        }

        choices.sort_by(|a, b| crate::java_string::java_string_cmp(&a.name, &b.name));
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
                None,
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
            for (source_key, resources) in &source_resources {
                if let Some(choice_key) = &choice.source_key {
                    if choice_key != source_key {
                        continue;
                    }
                } else if resources
                    .first()
                    .is_none_or(|resource| normalized_ship_key(&resource.ship) != key)
                {
                    continue;
                }
                let Ok(thumbnail) = ship_thumbnail::ShipThumbnail::new(resources.clone()) else {
                    continue;
                };
                let source_layers = ship::ShipLayers::new(thumbnail);
                layers[index] = source_layers
                    .layers
                    .iter()
                    .filter_map(|layer| {
                        let asset = match source_layers
                            .thumbnail
                            .get_resource(ShipResourceType::Texture, layer)
                        {
                            Some(ship_thumbnail::ThumbnailResource::File(resource)) => {
                                asset_path(&resource.path)
                            }
                            Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(
                                resource,
                            )) => asset_path(&resource.base_resource.path),
                            None => return None,
                        };
                        Some(ShipLayerChoice {
                            name: layer.clone(),
                            asset,
                        })
                    })
                    .collect();
                break;
            }
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
            .map(|layer| layer.name.display_name())
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
    resources: &mut Vec<(SourceShipKey, Vec<ShipResourceFile>)>,
) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_ship_resources(&path, resources);
            continue;
        }
        let Ok(resource) = parse_resource_path(&path) else {
            continue;
        };
        let canonical =
            std::fs::canonicalize(&resource.path).unwrap_or_else(|_| resource.path.clone());
        let key = SourceShipKey {
            directory: canonical.parent().unwrap_or(folder).to_path_buf(),
            ship: resource.ship.clone(),
        };
        if let Some((_, grouped)) = resources.iter_mut().find(|(existing, _)| existing == &key) {
            grouped.push(resource);
        } else {
            resources.push((key, vec![resource]));
        }
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
    source_key: Option<SourceShipKey>,
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
        source_key,
    });
}

fn spawn_ship_catalog_card(commands: &mut Commands, assets: &AssetServer, index: usize, choice: &ShipChoice, _visible: bool) {
    commands.spawn((ToolboxContent,ShipCard(index),Sprite::from_color(Color::srgb(0.23,0.27,0.40),Vec2::ONE),Transform::from_xyz(-465.0,0.0,21.0),RenderLayers::layer(4),Visibility::Hidden));
    commands.spawn((ToolboxContent,ShipThumbnail(index),if choice.source_key.is_some() {Sprite::default()}else {Sprite::from_image(assets.load(choice.asset.clone()))},Transform::from_xyz(-465.0,0.0,22.0),RenderLayers::layer(4),Visibility::Hidden));
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
struct ToolGlyph {
    tool: Tool,
    normal: Handle<Image>,
    active: Handle<Image>,
}

#[derive(Component)]
struct ToolPanelUi;

#[derive(Component)]
struct ToolPanelPosition(Vec2);

fn tool_panel_position(x: f32, y: f32, z: f32) -> (ToolPanelPosition, Transform) {
    (
        ToolPanelPosition(Vec2::new(x, y)),
        Transform::from_xyz(x, y, z),
    )
}

fn update_tool_panel_layout(windows: Query<&Window>, mut simulation: ResMut<Simulation>) {
    let Ok(window) = windows.single() else { return };
    if window.height() > 0.0 {
        let width = window.width() * 720.0 / window.height();
        // Toolbox.render: (padding, displayHeight - padding), pivot (0, 1).
        let offset = Vec2::new(280.0 - width * 0.5, -510.0);
        if simulation.tool_panel_offset != offset {
            simulation.tool_panel_offset = offset;
        }
    }
}

fn sync_tool_panel_position(
    simulation: Res<Simulation>,
    mut widgets: Query<(&ToolPanelPosition, &mut Transform)>,
) {
    for (position, mut transform) in &mut widgets {
        let point = position.0 + simulation.tool_panel_offset;
        transform.translation.x = point.x;
        transform.translation.y = point.y;
    }
}

#[derive(Component)]
struct ShipLayerLabel;

#[derive(Component)]
struct ShipLayerDropdownBackground;

#[derive(Component)]
struct ShipLayerDropdownOption(usize);

#[derive(Component)]
struct ShipLayerDropdownOptionLabel(usize);

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ShipUploadField {
    #[default]
    Name,
    Description,
    LayerName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShipUploadAction {
    EditCurrent,
    CreateNew,
    AddLayer,
    CycleLayer,
    SelectResource(ShipResourceType),
    SaveLocally,
    Close,
}

#[derive(Resource, Default)]
struct ShipUploadUiState {
    description_scroll: ui_description_scroll::DescriptionScroll,
    focus: Option<ShipUploadField>,
    active_focus: Option<ShipUploadField>,
    editors: [text_edit_state::TextEditState; 3],
    notice: Option<String>,
    active_layer: ShipLayer,
    pending_resource: Option<ShipResourceType>,
    preview_resource: Option<ShipResourceType>,
    layer_tab_start: usize,
}

#[derive(Component)]
struct ShipUploadButton(ShipUploadAction);

#[derive(Component)]
struct ShipUploadModalContent;

#[derive(Component)]
struct ShipUploadFieldControl(ShipUploadField);

#[derive(Component)]
struct ShipUploadNameText;

#[derive(Component)]
struct ShipUploadDescriptionText;

#[derive(Component)]
struct ShipUploadLayerText;

#[derive(Component)]
struct ShipUploadResourcePreview;

#[derive(Component)]
struct ShipUploadSaveControl;

#[derive(Component)]
struct ShipUploadLayerTab(ShipLayer);

#[derive(Component)]
struct ShipUploadLayerTabLabel(ShipLayer);

#[derive(Component)]
struct ShipUploadStatusText;

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
    (mut images, black_texture, retained): (
        ResMut<Assets<Image>>,
        Res<ship_black_texture::SharedBlackTexture>,
        Res<ship_runtime_reset::ActiveThumbnail>,
    ),
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
    ship_physics::spawn_readbacks(&mut commands, &gpu_physics);
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
            order: 4,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        scaled_projection.clone(),
        RenderLayers::layer(3),
    ));
    commands.spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::None, ..default() }, scaled_projection.clone(), RenderLayers::layer(1)));
    commands.spawn((Camera2d, Camera { order: 2, clear_color: ClearColorConfig::None, ..default() }, scaled_projection, toolbox_viewport::ContentCamera, RenderLayers::layer(2)));
    commands.spawn((Camera2d,Camera {order:3,clear_color:ClearColorConfig::None,..default()},Projection::Orthographic(OrthographicProjection::default_2d()),ship_browser_ui::BrowserCamera,RenderLayers::layer(4)));
    ship_browser_ui::spawn_scrollbar(&mut commands);

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
    let (ship, internal_lights, external_lights) = if let Some(thumbnail)=&retained.0 {
        let appearance=ship_runtime_reset::appearance(thumbnail,0,&mut images,&black_texture.0);
        (appearance.texture,appearance.internal,appearance.external)
    } else {
        let ship=load_ship_visual_asset(initial_choice,&initial_choice.asset,&assets,&mut images);
        let (internal,external)=load_ship_light_assets(initial_choice,&ShipLayer::default(),&assets,&mut images,&black_texture.0);
        (ship,internal,external)
    };
    commands.spawn((
        ShipSprite,
        Sprite::from_image(ship.clone()),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0).with_scale(Vec3::ONE),
        Visibility::Hidden,
    ));
    let mesh = ship_gpu_geometry::build_mesh(&structure);
    let mesh_handle = meshes.add(mesh);
    let texture = ship.clone();

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
        positions: gpu_physics.positions.clone(),
        coverage_mode: Vec4::ZERO,
        coverage: Handle::default(),
    });
    commands.spawn((
        ShipMesh(mesh_handle.clone(), ship_material.clone()),
        ship_gpu_geometry::GpuGeometry,
        bevy::camera::visibility::NoFrustumCulling,
        MeshSyncState(structure.breached.clone(), gpu_mask_data(&structure)),
        Mesh2d(mesh_handle),
        MeshMaterial2d(ship_material.clone()),
        Visibility::Inherited,
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
    ));
    let strut_masks = gpu_mask_data(&structure);
    let strut_mesh = meshes.add(ship_gpu_geometry::build_struts_mesh(&structure));
    commands.spawn((
        ship_struts::ShipStruts {
            mesh: strut_mesh.clone(),
            dimensions: (structure.texel_width, structure.texel_height),
            masks: strut_masks,
            occupied: structure.texel_solid.clone(),
        },
        ship_gpu_geometry::GpuGeometry,
        bevy::camera::visibility::NoFrustumCulling,
        Mesh2d(strut_mesh),
        MeshMaterial2d(ship_material),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.001),
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
    // Present the source-RGB sea frame, composing the optional brush before
    // the final sRGB conversion. The unit quad is fitted to world view bounds.
    let brush_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let brush_material = brush_materials.add(tools::brush_preview::BrushMaterial {
        cursor_radius: Vec4::new(0.0, 0.0, 1.0, 0.0),
        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        background: Handle::default(),
    });
    commands.spawn((
        DamageBrushPreview,
        RenderLayers::layer(tools::brush_preview::OVERLAY_LAYER),
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
        toolbox_viewport::Backdrop,
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.32, 0.35, 0.52), Vec2::new(356.0, 706.0)),
        Transform::from_xyz(-465.0, 0.0, 19.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        toolbox_viewport::Backdrop,
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
        Text2d::new("Toolbox"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-603.0,344.0,22.0),
        bevy::sprite::Anchor::TOP_LEFT,
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.12, 0.15, 0.23), Vec2::new(330.0, 34.0)),
        Transform::from_xyz(-465.0, 294.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        toolbox_viewport::ScrollTrack,
        ToolboxContent,
        Sprite::from_color(Color::srgb(0.12, 0.14, 0.20), Vec2::new(8.0, 632.0)),
        Transform::from_xyz(-297.0, -14.0, 21.5),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        toolbox_viewport::ScrollThumb,
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
        TabPage(ToolboxTab::Ships),
        Transform::from_xyz(-514.0, 253.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolboxContent,
        ShipSearchText,
        TabPage(ToolboxTab::Ships),
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
    for (text, y) in [("Edit current ship", 214.0), ("Create new ship", 174.0)] {
        let action = if text == "Edit current ship" {
            ShipUploadAction::EditCurrent
        } else {
            ShipUploadAction::CreateNew
        };
        commands.spawn((
            ToolboxContent,
            ShipUploadButton(action),
            TabPage(ToolboxTab::Ships),
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
    commands.spawn((
        ShipUploadModalContent,
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.55), Vec2::new(1280.0, 720.0)),
        Transform::from_xyz(0.0, 0.0, 80.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        Sprite::from_color(Color::srgb(0.10, 0.12, 0.16), Vec2::new(560.0, 700.0)),
        Transform::from_xyz(0.0, 0.0, 81.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        Text2d::new("Edit Ship"),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 326.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadButton(ShipUploadAction::Close),
        Sprite::from_color(Color::srgb(0.34, 0.36, 0.58), Vec2::new(30.0, 30.0)),
        Transform::from_xyz(260.0, 326.0, 82.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        Text2d::new("×"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(260.0, 326.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    for (label, y) in [("Ship name", 300.0), ("Description", 215.0)] {
        commands.spawn((
            ShipUploadModalContent,
            Text2d::new(label),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(-205.0, y, 82.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    for (field, y, height) in [
        (ShipUploadField::Name, 263.0, 38.0),
        (ShipUploadField::Description, 150.0, 96.0),
    ] {
        commands.spawn((
            ShipUploadModalContent,
            ShipUploadFieldControl(field),
            Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(410.0, height)),
            Transform::from_xyz(0.0, y, 82.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadNameText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-196.0, 263.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadDescriptionText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-196.0, 150.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadFieldControl(ShipUploadField::LayerName),
        Sprite::from_color(Color::srgb(0.19, 0.20, 0.21), Vec2::new(330.0, 38.0)),
        Transform::from_xyz(-40.0, -108.0, 82.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadLayerText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-194.0, -108.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipUploadModalContent,
        Text2d::new("New layer"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-202.0, -82.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    for (resource, label, x, y) in [
        (ShipResourceType::Base, "BASE (Required)", -135.0, -180.0),
        (
            ShipResourceType::Materials,
            "MATERIALS (Optional)",
            135.0,
            -180.0,
        ),
        (
            ShipResourceType::Texture,
            "TEXTURE (Optional)",
            -135.0,
            -219.0,
        ),
        (
            ShipResourceType::InLights,
            "INLIGHTS (Optional)",
            135.0,
            -219.0,
        ),
        (
            ShipResourceType::ExLights,
            "EXLIGHTS (Optional)",
            0.0,
            -258.0,
        ),
    ] {
        commands.spawn((
            ShipUploadModalContent,
            ShipUploadButton(ShipUploadAction::SelectResource(resource)),
            Sprite::from_color(Color::srgb(0.24, 0.28, 0.43), Vec2::new(260.0, 34.0)),
            Transform::from_xyz(x, y, 82.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        commands.spawn((
            ShipUploadModalContent,
            Text2d::new(label),
            TextFont {
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(x, y, 83.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        ShipUploadModalContent,
        ShipUploadStatusText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -286.0, 83.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    for (action, text, x, y) in [
        (ShipUploadAction::AddLayer, "Add Layer", 190.0, -108.0),
        (ShipUploadAction::CycleLayer, "Next Layer", 190.0, -145.0),
        (
            ShipUploadAction::SaveLocally,
            "Save Locally",
            -105.0,
            -326.0,
        ),
        (ShipUploadAction::Close, "Close", 105.0, -326.0),
    ] {
        let button_entity = commands
            .spawn((
                ShipUploadModalContent,
                ShipUploadButton(action),
                Sprite::from_color(
                    Color::srgb(0.24, 0.28, 0.43),
                    Vec2::new(
                        if matches!(
                            action,
                            ShipUploadAction::AddLayer | ShipUploadAction::CycleLayer
                        ) {
                            112.0
                        } else {
                            180.0
                        },
                        38.0,
                    ),
                ),
                Transform::from_xyz(x, y, 82.0),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ))
            .id();
        if action == ShipUploadAction::SaveLocally {
            commands.entity(button_entity).insert(ShipUploadSaveControl);
        }
        let label_entity = commands
            .spawn((
                ShipUploadModalContent,
                Text2d::new(text),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(x, y, 83.0),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ))
            .id();
        if action == ShipUploadAction::SaveLocally {
            commands.entity(label_entity).insert(ShipUploadSaveControl);
        }
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
            toolbox_viewport::PageBackdrop,
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
    let color_picker_mesh = meshes.add(color_picker_mesh(simulation.sea_color_memory.read(simulation.sea_color).x));
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
            let shade = if (row + column) % 2 == 0 { 204.0 / 255.0 } else { 128.0 / 255.0 };
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
    music_controls::spawn(&mut commands, &assets);
    spawn_page_label(
        &mut commands,
        ToolboxTab::Performance,
        "Iterations per frame",
        Vec3::new(-465.0, 260.0, 25.0),
        15.0,
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
    spawn_setting_button(
        &mut commands,
        "Gen Palette",
        SettingAction::GeneratePalette,
        Vec3::new(-465.0, 252.0, 25.0),
    );

    for (index, (tool, icon)) in [
        (Tool::Break, "icons/Break.png"),
        (Tool::Flood, "icons/Flood.png"),
        (Tool::Dry, "icons/Dry.png"),
        (Tool::Move, "icons/Move.png"),
    ]
    .into_iter()
    .enumerate()
    {
        let x = -220.0 + index as f32 * 78.0;
        commands.spawn((
            ToolCard(tool),
            source_tools_layout::Role::Button(tool),
            Sprite::from_color(Color::srgb(0.78, 0.48, 0.02), Vec2::splat(56.0)),
            tool_panel_position(x, 250.0, 21.0),
            RenderLayers::layer(1),
        ));
        let normal = assets.load(icon);
        let active = assets.load(match tool {
            Tool::Break => "icons/Break2.png",
            Tool::Flood => "icons/Flood2.png",
            Tool::Dry => "icons/Dry2.png",
            Tool::Move => "icons/Move2.png",
            Tool::None => unreachable!("No icon for the source null tool slot"),
        });
        let mut glyph = Sprite::from_image(normal.clone());
        glyph.custom_size = Some(Vec2::splat(46.0));
        commands.spawn((
            ToolGlyph {
                tool,
                normal,
                active,
            },
            source_tools_layout::Role::Image(tool),
            glyph,
            tool_panel_position(x, 250.0, 22.0),
            RenderLayers::layer(1),
        ));
    }
    commands.spawn((
        ToolPanelUi,
        source_tools_layout::Role::Combo,
        Sprite::from_color(Color::srgb(0.30, 0.40, 0.54), Vec2::new(230.0, 28.0)),
        tool_panel_position(-145.0, 310.0, 21.0),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        ToolPanelUi,
        ShipLayerLabel,
        source_tools_layout::Role::ComboValue,
        Text2d::new("Default"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        tool_panel_position(-145.0, 310.0, 22.0),
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
        source_tools_layout::Role::Arrow,
        Sprite::from_color(Color::srgb(0.24, 0.29, 0.43), Vec2::new(22.0, 24.0)),
        tool_panel_position(-40.0, 310.0, 21.0),
        RenderLayers::layer(1),
    ));
    let max_layer_count = catalog.1.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let dropdown_height = max_layer_count as f32 * 24.0;
    commands.spawn((
        ShipLayerDropdownBackground,
        source_tools_layout::Role::Popup,
        Sprite::from_color(
            Color::srgba(0.07, 0.08, 0.12, 0.94),
            Vec2::new(230.0, dropdown_height),
        ),
        tool_panel_position(-145.0, 297.0 - dropdown_height * 0.5, 39.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    for index in 0..max_layer_count {
        spawn_ship_layer_dropdown_option(&mut commands, index);
    }
    commands.spawn((
        ToolPanelUi,
        SettingRow(19),
        source_tools_layout::Role::Size,
        Sprite::from_color(Color::srgb(0.30, 0.40, 0.54), Vec2::new(230.0, 28.0)),
        tool_panel_position(-145.0, 188.0, 21.0),
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
        source_tools_layout::Role::SizeValue,
        Text2d::new("1.000"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        tool_panel_position(-145.0, 188.0, 22.0),
        RenderLayers::layer(1),
    ));
    spawn_tool_panel_label(
        &mut commands,
        "You can hide the tools in the graphics settings",
        Vec3::new(-85.0, 163.0, 22.0),
        10.0,
    );
}

/// Legacy CPU rendering approximation. A readback cannot establish the latest
/// GPU topology, so this result must never be uploaded into the live solver.
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
    mut ship_meshes: Query<(&ShipMesh, &mut MeshSyncState), Without<ship_gpu_geometry::GpuGeometry>>,
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
    let entity = commands.spawn((
        ToolboxContent,
        Text2d::new(text.to_string()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        RenderLayers::layer(1),
    )).id();
    if position.y < 277.0 { commands.entity(entity).insert(TabPage(ToolboxTab::Ships)); }
}

fn spawn_tool_panel_label(commands: &mut Commands, text: &str, position: Vec3, size: f32) {
    commands.spawn((
        ToolPanelUi,
        match text {"Show Layer"=>source_tools_layout::Role::ComboLabel,"Tool Size"=>source_tools_layout::Role::SizeLabel,_=>source_tools_layout::Role::Hint},
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        ToolPanelPosition(position.truncate()),
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
    // Original dragScalar renders the right-hand label from frame.max.x +
    // ItemInnerSpacing.x, while numeric text remains centered in its frame.
    let right_hand = position.x == -391.0;
    let position = if right_hand { Vec3::new(-514.0 + 225.0 * 0.5 + 4.0, position.y, position.z) } else { position };
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(position),
        if right_hand { bevy::sprite::Anchor::CENTER_LEFT } else { bevy::sprite::Anchor::CENTER },
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
        SettingAction::GeneratePalette => ToolboxTab::Advanced,
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
            } else if matches!(action, SettingAction::GeneratePalette) {
                Vec2::new(326.0, 34.0)
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

fn handle_controls(keys: Res<ButtonInput<KeyCode>>, mut simulation: ResMut<Simulation>, capture:Option<Res<ui_input_capture::Capture>>) {
    if !simulation.ship_search_active && !capture.is_some_and(|capture|capture.keyboard) {
        for (key, tool) in [
            (KeyCode::Digit1, Tool::Break),
            (KeyCode::Digit2, Tool::Flood),
            (KeyCode::Digit3, Tool::Dry),
            (KeyCode::Digit4, Tool::Move),
        ] {
            if keys.just_pressed(key) {
                simulation.tool.toggle(tool);
            }
        }
    }
}

#[derive(Default)]
struct ShipEditorInputLocal {
    characters: bevy::ecs::message::MessageCursor<window_characters::CharacterInput>,
    mouse: ui_mouse_input::MouseInput,
}
fn handle_ship_upload_ui(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut file_events: MessageReader<FileDragAndDrop>,
    windows: Query<(&Window,Option<&bevy::window::RawHandleWrapper>)>,
    catalog: Res<ShipCatalog>,
    mut upload: ResMut<ship_upload::SourceShipUpload>,
    mut ui: ResMut<ShipUploadUiState>,
    editor_layout: Res<ship_upload_layout::Layout>,
    (active_preview,active_thumbnail): (Res<ship_upload_preview::ActivePreview>,Option<Res<ship_runtime_reset::ActiveThumbnail>>),
    buttons: Query<(&ShipUploadButton, &Transform, &Sprite, &Visibility)>,
    fields: Query<(&ShipUploadFieldControl, &Transform, &Sprite)>,
    layer_tabs: Query<(&ShipUploadLayerTab, &Transform, &Sprite, &Visibility)>,
    mut simulation: ResMut<Simulation>,
    (native_characters,mut clipboard,time,native_window): (Option<Res<Messages<window_characters::CharacterInput>>>,Option<ResMut<editor_clipboard::Clipboard>>,Option<Res<Time>>,Option<NonSend<window_bevy::LiveWindow>>),
    mut input_local: Local<ShipEditorInputLocal>,
) {
    let events: Vec<_> = keyboard_events.read().cloned().collect();
    let committed_text=window_characters::read_units(native_characters.as_deref(),&mut input_local.characters);
    let dropped_files: Vec<_> = file_events.read().cloned().collect();
    if !upload.window_open() {ui.focus=None;ui.active_focus=None;}
    if upload.window_open() {
        for event in dropped_files {
            let FileDragAndDrop::DroppedFile { path_buf, .. } = event else {
                continue;
            };
            match parse_resource_path(&path_buf) {
                Ok(resource) => {
                    if upload.ship_name().is_empty() {
                        upload.set_ship_name(&resource.ship);
                    }
                    let resource_type = ui.pending_resource.unwrap_or(resource.resource_type);
                    let selected_layer = if ui.pending_resource.is_some() {
                        ui.active_layer.clone()
                    } else {
                        resource.layer.clone()
                    };
                    let layer = ship_upload_resource_layer(resource_type, &selected_layer);
                    match upload.select_file(path_buf, resource_type, layer) {
                        Ok(()) => {
                            ui.pending_resource = None;
                            ui.preview_resource = Some(resource_type);
                            ui.notice = Some(format!("Added {} resource", resource_type.name()))
                        }
                        Err(error) => ui.notice = Some(error),
                    }
                }
                Err(error) => ui.notice = Some(error),
            }
        }
    }
    let Ok((window,raw_handle)) = windows.single() else { return };
    let field_padding=ui_text_viewport::field_padding(&editor_layout);
    let field_font=editor_layout.text_height();
    if upload.window_open() {
        let font=editor_layout.text_height();
        for editor in &mut ui.editors {editor.set_font_height(font);}
        if let Some(source)=editor_layout.source_window.as_ref() {ui.description_scroll.configure(source.gui,source.viewport.source_length_to_world(1.0));}
        if editor_layout.is_collapsed() {ui.focus=None;ui.active_focus=None;}
        if let Some((_,_,sprite))=fields.iter().find(|(field,_,_)|field.0==ShipUploadField::Description) {
            let size=sprite.custom_size.unwrap_or(Vec2::ZERO);let current=ui.editors[1].scroll_y;
            ui.editors[1].scroll_y=ui.description_scroll.begin(size,current);
        }
    }
    input_local.mouse.update(time.as_ref().map_or(1.0/60.0,|time|time.delta_secs()),window.cursor_position(),
        [mouse.pressed(MouseButton::Left),mouse.pressed(MouseButton::Right),mouse.pressed(MouseButton::Middle),
         mouse.pressed(MouseButton::Back),mouse.pressed(MouseButton::Forward)]);
    let clicked=input_local.mouse.buttons[0].clicked;
    let double_clicked=input_local.mouse.buttons[0].double_clicked;
    let cursor=input_local.mouse.position;
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );

    let pointer_moved=input_local.mouse.delta!=Vec2::ZERO;
    if upload.window_open() {
        let local = point - editor_layout.position;
        let point = editor_layout.chrome_point(point).or_else(||ship_upload_layout::content_point(&editor_layout,point))
            .unwrap_or(Vec2::splat(-1.0e6));
        let mut caret_click=None;
        if let Some((_,transform,sprite))=fields.iter().find(|(field,_,_)|field.0==ShipUploadField::Description) {
            let size=sprite.custom_size.unwrap_or(Vec2::ZERO);
            let bar_point=if ui.description_scroll.held {local-Vec2::new(0.0,editor_layout.scroll)}else {point};
            if ui.description_scroll.pointer(bar_point,mouse.pressed(MouseButton::Left),clicked,transform.translation.truncate(),size) {
                ui.editors[1].scroll_y=ui.description_scroll.window.scroll.y;
                if ui.focus!=Some(ShipUploadField::Description) {ui.focus=None;ui.active_focus=None;}
                return;
            }
        }
        if clicked {
            for (tab, transform, sprite, visibility) in &layer_tabs {
                if *visibility == Visibility::Hidden {
                    continue;
                }
                let size = sprite.custom_size.unwrap_or(Vec2::ZERO) * 0.5;
                if (point - transform.translation.truncate())
                    .abs()
                    .cmplt(size)
                    .all()
                {
                    ui.active_layer = tab.0.clone();
                    upload.set_current_layer(tab.0.clone());
                    ui.notice = Some(format!("Editing layer {}", tab.0.display_name()));
                }
            }
            let mut clicked_field=None;
            for (field, transform, sprite) in &fields {
                let size = sprite.custom_size.unwrap_or(Vec2::ZERO) * 0.5;
                if (point - transform.translation.truncate())
                    .abs()
                    .cmplt(size)
                    .all()
                {
                    clicked_field = Some(field.0);
                    let center=transform.translation.truncate();
                    let top=if field.0==ShipUploadField::Description {center.y+size.y-field_padding.y}else {center.y+field_font*0.5};
                    caret_click=Some(Vec2::new(point.x-(center.x-size.x+field_padding.x),top-point.y));
                }
            }
            ui.focus=clicked_field;
            if ui.focus!=ui.active_focus {ui.active_focus=None;}
            for (button, transform, sprite, visibility) in &buttons {
                if *visibility == Visibility::Hidden {
                    continue;
                }
                if !matches!(
                    button.0,
                    ShipUploadAction::AddLayer
                        | ShipUploadAction::CycleLayer
                        | ShipUploadAction::SelectResource(_)
                        | ShipUploadAction::SaveLocally
                        | ShipUploadAction::Close
                ) {
                    continue;
                }
                let size = sprite.custom_size.unwrap_or(Vec2::ZERO) * 0.5;
                let center = if matches!(button.0, ShipUploadAction::Close) {
                    transform.translation.truncate() - editor_layout.position
                } else { transform.translation.truncate() };
                if !(point - center)
                    .abs()
                    .cmplt(size)
                    .all()
                {
                    continue;
                }
                match button.0 {
                    ShipUploadAction::AddLayer => {
                        let name = upload.new_layer_name();
                        if upload.add_layer() {
                            ui.active_layer = ShipLayer::new(name);
                            upload.set_current_layer(ui.active_layer.clone());
                            let index = upload
                                .layers_in_order()
                                .iter()
                                .position(|layer| layer == &ui.active_layer)
                                .unwrap_or(0);
                            ui.layer_tab_start = (index / 4) * 4;
                            ui.notice = Some("Layer added".into());
                        } else {
                            ui.notice = Some("Enter a layer name first".into());
                        }
                    }
                    ShipUploadAction::CycleLayer => {
                        let layers = upload.layers_in_order();
                        if !layers.is_empty() {
                            let current = layers
                                .iter()
                                .position(|layer| layer == &ui.active_layer)
                                .unwrap_or(0);
                            let next = (current + 1) % layers.len();
                            ui.active_layer = layers[next].clone();
                            ui.layer_tab_start = (next / 4) * 4;
                            upload.set_current_layer(ui.active_layer.clone());
                            ui.notice =
                                Some(format!("Editing layer {}", ui.active_layer.display_name()));
                        }
                    }
                    ShipUploadAction::SelectResource(resource_type) => {
                        ui.pending_resource = Some(resource_type);
                        ui.preview_resource = Some(resource_type);
                        if let Some(path) = open_ship_resource_dialog(resource_type) {
                            if upload.ship_name().is_empty() {
                                if let Ok(parsed) = parse_resource_path(&path) {
                                    upload.set_ship_name(&parsed.ship);
                                }
                            }
                            let layer = ship_upload_resource_layer(resource_type, &ui.active_layer);
                            match upload.select_file(path, resource_type, layer) {
                                Ok(()) => {
                                    ui.pending_resource = None;
                                    ui.notice =
                                        Some(format!("Added {} resource", resource_type.name()));
                                }
                                Err(error) => ui.notice = Some(error),
                            }
                        }
                    }
                    ShipUploadAction::Close => {
                        upload.set_window_open(false);
                        ui.notice = None;
                    }
                    ShipUploadAction::SaveLocally if upload.ship_name_is_blank() => {
                        ui.notice = Some("Some required fields are missing".into());
                    }
                    ShipUploadAction::SaveLocally => match upload.thumbnail() {
                        Ok(Some(_)) => {
                            let materials =
                                crate::main_globals::get_global_materials().to_owned_adapter();
                            ui.notice = Some(
                                match upload.save_locally(std::path::Path::new("."), &materials) {
                                    Ok(()) => upload
                                        .success_string()
                                        .unwrap_or("Saved locally")
                                        .to_owned(),
                                    Err(error) => error,
                                },
                            );
                        }
                        Ok(None) => {
                            ui.notice = Some("A default layer BASE image is required".into())
                        }
                        Err(error) => ui.notice = Some(error),
                    },
                    _ => {}
                }
            }
        }
        use text_edit_state::Key as EditKey;
        if !upload.window_open() {ui.focus=None;ui.active_focus=None;return;}
        let Some(field)=ui.focus else {ui.active_focus=None;return;};
        let activate=ui.active_focus!=Some(field);
        ui.active_focus=Some(field);
        let index=match field {ShipUploadField::Name=>0,ShipUploadField::Description=>1,ShipUploadField::LayerName=>2};
        let ui_state:&mut ShipUploadUiState=&mut ui;
        let editor=&mut ui_state.editors[index];
        let description_scroll=&mut ui_state.description_scroll;
        if activate {editor.begin_focus(&upload.editor_units(field),field!=ShipUploadField::Description);}
        else {editor.synchronize(&upload.editor_units(field),field!=ShipUploadField::Description);}
        let ctrl=keys.pressed(KeyCode::ControlLeft)||keys.pressed(KeyCode::ControlRight);
        let shift=keys.pressed(KeyCode::ShiftLeft)||keys.pressed(KeyCode::ShiftRight);
        let alt=keys.pressed(KeyCode::AltLeft)||keys.pressed(KeyCode::AltRight);
        let super_key=keys.pressed(KeyCode::SuperLeft)||keys.pressed(KeyCode::SuperRight);
        let shortcut=ctrl && !alt && !shift && !super_key;
        if !mouse.pressed(MouseButton::Left) {editor.selected_all_mouse_lock=false;}
        if let Some(position)=caret_click {
            if double_clicked || ctrl && field!=ShipUploadField::Description {editor.select_all();editor.selected_all_mouse_lock=true;}
            else if !editor.selected_all_mouse_lock {editor.click(position.x+editor.scroll_x,if field==ShipUploadField::Description {position.y+editor.scroll_y}else {editor.font_height()*0.5});}
        } else if mouse.pressed(MouseButton::Left) && pointer_moved && !editor.selected_all_mouse_lock {
            if let Some((_,transform,sprite))=fields.iter().find(|(control,_,_)|control.0==field) {
                let center=transform.translation.truncate();let size=sprite.custom_size.unwrap_or(Vec2::ZERO)*0.5;
                let position=local-Vec2::new(0.0,editor_layout.scroll);
                let top=if field==ShipUploadField::Description {center.y+size.y-field_padding.y}else {center.y+field_font*0.5};
                editor.widget_drag(position.x-(center.x-size.x+field_padding.x)+editor.scroll_x,if field==ShipUploadField::Description {top-position.y+editor.scroll_y}else {field_font*0.5});
            }
        }
        // Source inputText drains committed characters before navigation commands.
        if !(ctrl && !alt) {
            let mut additions=committed_text;
            if native_window.is_none() {for event in events.iter().filter(|event|event.state==bevy::input::ButtonState::Pressed) {
                if let Some(text)=&event.text {additions.extend(text.chars().filter(|c|!c.is_control()).flat_map(|c|window_characters::units(c as u32)));}
            }}
            editor.insert(&editor_clipboard::filtered(&additions,field==ShipUploadField::Description));
        }
        let mut cancelled=keys.just_pressed(KeyCode::Escape);
        let mut deactivate=false;
        for event in events.iter().filter(|event|event.state==bevy::input::ButtonState::Pressed) {
            if cancelled || editor.error.is_some() {break;}
            let shift_only=shift && !ctrl && !alt && !super_key;
            let copy=shortcut && matches!(event.key_code,KeyCode::KeyC|KeyCode::Insert);
            let cut=shortcut && event.key_code==KeyCode::KeyX || shift_only && event.key_code==KeyCode::Delete;
            let paste=shortcut && event.key_code==KeyCode::KeyV || shift_only && event.key_code==KeyCode::Insert;
            if copy || cut {
                if let Some(units)=editor.clipboard_units() {
                    if let Some(clipboard)=clipboard.as_mut() {clipboard.write(editor_clipboard::owner(raw_handle),&units);}
                    if cut {if !editor.has_selection() {editor.select_all();}editor.cut();editor.cursor_follow=true;}
                }
                continue;
            }
            if paste {
                if let Some(clipboard)=clipboard.as_mut() {
                    if let Some(units)=clipboard.read(editor_clipboard::owner(raw_handle)) {
                        let units=editor_clipboard::filtered(&units,field==ShipUploadField::Description);
                        if !units.is_empty() {editor.paste(&units);editor.cursor_follow=true;}
                    }
                }
                continue;
            }
            if field==ShipUploadField::Description && ctrl && matches!(event.key_code,KeyCode::ArrowUp|KeyCode::ArrowDown) {
                description_scroll.key_scroll(event.key_code==KeyCode::ArrowDown);continue;
            }
            let command=match event.key_code {
                KeyCode::ArrowUp if field==ShipUploadField::Description && !ctrl=>Some(EditKey::Up),
                KeyCode::ArrowDown if field==ShipUploadField::Description && !ctrl=>Some(EditKey::Down),
                KeyCode::ArrowLeft=>Some(if ctrl {EditKey::WordLeft}else {EditKey::Left}),
                KeyCode::ArrowRight=>Some(if ctrl {EditKey::WordRight}else {EditKey::Right}),
                KeyCode::Home=>Some(if ctrl {EditKey::TextStart}else {EditKey::LineStart}),
                KeyCode::End=>Some(if ctrl {EditKey::TextEnd}else {EditKey::LineEnd}),
                KeyCode::Delete=>Some(EditKey::Delete),KeyCode::Backspace=>{
                    if ctrl && !editor.has_selection() {editor.on_key_pressed(EditKey::WordLeft,true);}
                    Some(EditKey::Backspace)
                },_=>None,
            };
            if let Some(command)=command {editor.on_key_pressed(command,shift);}
            else if shortcut && event.key_code==KeyCode::KeyZ {editor.on_key_pressed(EditKey::Undo,false);if editor.error.is_none() {editor.clear_selection();}}
            else if shortcut && event.key_code==KeyCode::KeyY {editor.on_key_pressed(EditKey::Redo,false);if editor.error.is_none() {editor.clear_selection();}}
            else if shortcut && event.key_code==KeyCode::KeyA {editor.select_all();editor.cursor_follow=true;}
            else if matches!(event.key_code,KeyCode::Enter|KeyCode::NumpadEnter) {
                if field==ShipUploadField::Description && !ctrl {editor.insert(&[10]);}
                else {deactivate=true;break;}
            }
            else if event.key_code==KeyCode::Escape {cancelled=true;break;}
        }
        if cancelled {upload.set_editor_units(field,&editor.cancel_units());}
        else if let Some(units)=editor.publish() {upload.set_editor_units(field,&units);}
        else if let Some(error)=editor.error {ui.notice=Some(format!("Text editor: {error}"));}
        if cancelled || deactivate {ui.focus=None;ui.active_focus=None;}
        return;
    }

    if simulation.toolbox_collapsed
        || simulation.active_tab != ToolboxTab::Ships
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    }
    for (button, transform, sprite, visibility) in &buttons {
        if *visibility == Visibility::Hidden {
            continue;
        }
        if !matches!(
            button.0,
            ShipUploadAction::EditCurrent | ShipUploadAction::CreateNew
        ) {
            continue;
        }
        let size = sprite.custom_size.unwrap_or(Vec2::ZERO) * 0.5;
        if !(point - transform.translation.truncate())
            .abs()
            .cmplt(size)
            .all()
        {
            continue;
        }
        match button.0 {
            ShipUploadAction::CreateNew => {
                upload.create_new();
                ui.editors = Default::default();
                ui.focus=None;ui.active_focus=None;
                ui.active_layer = ShipLayer::default();
                ui.pending_resource = None;
                ui.preview_resource = Some(ShipResourceType::Base);
                ui.layer_tab_start = 0;
                ui.notice = Some("Some required fields are missing".into());
            }
            ShipUploadAction::EditCurrent => {
                ui.editors = Default::default();
                ui.focus=None;ui.active_focus=None;
                let active=active_preview.thumbnail.as_ref().or_else(||active_thumbnail.as_ref().and_then(|retained|retained.0.as_ref()));
                let choice=catalog.0.get(simulation.ship_index);
                if active.is_none() && choice.is_none() {ui.notice=Some("No ship is selected".into());continue;}
                let thumbnail = if let Some(thumbnail)=active {Some(thumbnail.clone())} else if let Some(key) = choice.and_then(|choice|choice.source_key.as_ref()) {
                    let mut resources = Vec::new();
                    collect_source_ship_resources(
                        std::path::Path::new("assets/source_ships"),
                        &mut resources,
                    );
                    resources
                        .into_iter()
                        .find(|(candidate, _)| candidate == key)
                        .and_then(|(_, files)| ship_thumbnail::ShipThumbnail::new(files).ok())
                } else {
                    let choice=choice.expect("active or catalog choice checked");
                    let base_path = std::path::Path::new("assets").join(&choice.physics_asset);
                    let texture_path = std::path::Path::new("assets").join(&choice.asset);
                    let ship_name = choice.name.clone();
                    let base = ShipResourceFile::new(
                        base_path,
                        ShipResource::new(
                            ship_name.clone(),
                            ShipLayer::default(),
                            ShipResourceType::Base,
                        ),
                    );
                    base.ok().and_then(|base| {
                        let mut resources = vec![base];
                        if choice.material_map
                            && texture_path
                                != std::path::Path::new("assets").join(&choice.physics_asset)
                        {
                            if let Ok(texture) = ShipResourceFile::new(
                                texture_path,
                                ShipResource::new(
                                    ship_name,
                                    ShipLayer::default(),
                                    ShipResourceType::Texture,
                                ),
                            ) {
                                resources.push(texture);
                            }
                        }
                        ship_thumbnail::ShipThumbnail::new(resources).ok()
                    })
                };
                if let Some(thumbnail) = thumbnail {
                    upload.edit_current(&thumbnail);
                    ui.active_layer = ShipLayer::default();
                    ui.pending_resource = None;
                    ui.preview_resource = Some(ShipResourceType::Base);
                    ui.layer_tab_start = 0;
                    ui.notice = Some("Edit ship resources, then save locally".into());
                } else {
                    ui.notice = Some("This ship has no editable BASE resource".into());
                }
            }
            _ => {}
        }
    }
}

#[cfg(windows)]
#[repr(C)]
struct OpenFileNameW {
    size: u32,
    owner: *mut std::ffi::c_void,
    instance: *mut std::ffi::c_void,
    filter: *const u16,
    custom_filter: *mut u16,
    max_custom_filter: u32,
    filter_index: u32,
    file: *mut u16,
    max_file: u32,
    file_title: *mut u16,
    max_file_title: u32,
    initial_dir: *const u16,
    title: *const u16,
    flags: u32,
    file_offset: u16,
    file_extension: u16,
    default_extension: *const u16,
    custom_data: isize,
    hook: *mut std::ffi::c_void,
    template_name: *const u16,
    reserved: *mut std::ffi::c_void,
    reserved_word: u32,
    flags_ex: u32,
}

#[cfg(windows)]
#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GetOpenFileNameW(file_name: *mut OpenFileNameW) -> i32;
}

#[cfg(windows)]
fn open_ship_resource_dialog(resource_type: ShipResourceType) -> Option<std::path::PathBuf> {
    use std::os::windows::ffi::OsStringExt;

    let extension = resource_type.extension();
    let filter = format!(
        "{} resource (*.{extension})\0*.{extension}\0All files (*.*)\0*.*\0\0",
        resource_type.name()
    );
    let filter: Vec<u16> = filter.encode_utf16().collect();
    let title: Vec<u16> = format!("Select {} resource", resource_type.name())
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut file = vec![0u16; 32_768];
    let mut dialog = OpenFileNameW {
        size: std::mem::size_of::<OpenFileNameW>() as u32,
        owner: std::ptr::null_mut(),
        instance: std::ptr::null_mut(),
        filter: filter.as_ptr(),
        custom_filter: std::ptr::null_mut(),
        max_custom_filter: 0,
        filter_index: 1,
        file: file.as_mut_ptr(),
        max_file: file.len() as u32,
        file_title: std::ptr::null_mut(),
        max_file_title: 0,
        initial_dir: std::ptr::null(),
        title: title.as_ptr(),
        flags: 0x0000_1000 | 0x0000_0800 | 0x0000_0008,
        file_offset: 0,
        file_extension: 0,
        default_extension: std::ptr::null(),
        custom_data: 0,
        hook: std::ptr::null_mut(),
        template_name: std::ptr::null(),
        reserved: std::ptr::null_mut(),
        reserved_word: 0,
        flags_ex: 0,
    };
    // SAFETY: all buffers and the OPENFILENAMEW structure remain alive and writable
    // for the duration of the synchronous common-dialog call.
    let accepted = unsafe { GetOpenFileNameW(&mut dialog) } != 0;
    if !accepted {
        return None;
    }
    let length = file
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(file.len());
    Some(std::ffi::OsString::from_wide(&file[..length]).into())
}

#[cfg(not(windows))]
fn open_ship_resource_dialog(_resource_type: ShipResourceType) -> Option<std::path::PathBuf> {
    None
}

fn ship_upload_resource_layer(resource_type: ShipResourceType, selected: &ShipLayer) -> ShipLayer {
    if resource_type.is_layered() {
        selected.clone()
    } else {
        ShipLayer::default()
    }
}

fn sync_ship_upload_ui(
    mut commands: Commands,
    upload: Res<ship_upload::SourceShipUpload>,
    ui: Res<ShipUploadUiState>,
    mut visibility: Query<
        &mut Visibility,
        (
            With<ShipUploadModalContent>,
            Without<ShipUploadResourcePreview>,
            Without<ShipUploadSaveControl>,
            Without<ShipUploadLayerTab>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut save_controls: Query<
        &mut Visibility,
        (
            With<ShipUploadSaveControl>,
            Without<ShipUploadResourcePreview>,
            Without<ShipUploadLayerTab>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut previews: Query<
        (&mut Sprite, &mut Visibility),
        (
            With<ShipUploadResourcePreview>,
            Without<ShipUploadSaveControl>,
            Without<ShipUploadLayerTab>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut layer_tabs: Query<
        (
            Entity,
            &ShipUploadLayerTab,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
        ),
        (
            Without<ShipUploadSaveControl>,
            Without<ShipUploadResourcePreview>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut layer_tab_labels: Query<
        (
            Entity,
            &ShipUploadLayerTabLabel,
            &mut Transform,
            &mut Text2d,
            &mut Visibility,
        ),
        (
            Without<ShipUploadSaveControl>,
            Without<ShipUploadResourcePreview>,
            Without<ShipUploadLayerTab>,
            Without<ShipUploadNameText>,
            Without<ShipUploadDescriptionText>,
            Without<ShipUploadLayerText>,
            Without<ShipUploadStatusText>,
        ),
    >,
    mut resource_buttons: Query<
        (&ShipUploadButton, &mut Sprite),
        (
            With<ShipUploadModalContent>,
            Without<ShipUploadResourcePreview>,
            Without<ShipUploadLayerTab>,
        ),
    >,
    mut images: ResMut<Assets<Image>>,
    mut names: Query<&mut Text2d, (With<ShipUploadNameText>, Without<ShipUploadLayerTabLabel>)>,
    mut descriptions: Query<
        &mut Text2d,
        (
            With<ShipUploadDescriptionText>,
            Without<ShipUploadNameText>,
            Without<ShipUploadLayerText>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut layer_names: Query<
        &mut Text2d,
        (
            With<ShipUploadLayerText>,
            Without<ShipUploadNameText>,
            Without<ShipUploadDescriptionText>,
            Without<ShipUploadStatusText>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
    mut statuses: Query<
        &mut Text2d,
        (
            With<ShipUploadStatusText>,
            Without<ShipUploadNameText>,
            Without<ShipUploadDescriptionText>,
            Without<ShipUploadLayerText>,
            Without<ShipUploadLayerTabLabel>,
        ),
    >,
) {
    let has_required_fields =
        !upload.ship_name_is_blank() && upload.thumbnail().ok().flatten().is_some();
    let layers = upload.layers_in_order().to_vec();
    let last_tab_start = layers.len().saturating_sub(1) / 4 * 4;
    let tab_start = (ui.layer_tab_start / 4 * 4).min(last_tab_start);
    let tab_end = (tab_start + 4).min(layers.len());
    let mut existing_tabs = Vec::new();
    for (entity, tab, mut transform, mut sprite, mut tab_visibility) in &mut layer_tabs {
        let Some(index) = layers.iter().position(|layer| layer == &tab.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        existing_tabs.push(tab.0.clone());
        let on_page = (tab_start..tab_end).contains(&index);
        if on_page {
            let slot = index - tab_start;
            transform.translation.x = -200.0 + slot as f32 * 82.0;
        }
        sprite.color = if tab.0 == ui.active_layer {
            Color::srgb(0.38, 0.43, 0.62)
        } else {
            Color::srgb(0.24, 0.28, 0.43)
        };
        *tab_visibility = if upload.window_open() && on_page {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (index, layer) in layers.iter().enumerate() {
        if existing_tabs.contains(layer) {
            continue;
        }
        let slot = index.saturating_sub(tab_start);
        let visible = (tab_start..tab_end).contains(&index);
        commands.spawn((
            ShipUploadModalContent,
            ShipUploadLayerTab(layer.clone()),
            Sprite::from_color(
                if layer == &ui.active_layer {
                    Color::srgb(0.38, 0.43, 0.62)
                } else {
                    Color::srgb(0.24, 0.28, 0.43)
                },
                Vec2::new(78.0, 30.0),
            ),
            Transform::from_xyz(-200.0 + slot as f32 * 82.0, -145.0, 82.0),
            RenderLayers::layer(1),
            if upload.window_open() && visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
    }
    let mut existing_labels = Vec::new();
    for (entity, label, mut transform, mut text, mut label_visibility) in &mut layer_tab_labels {
        let Some(index) = layers.iter().position(|layer| layer == &label.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        existing_labels.push(label.0.clone());
        let on_page = (tab_start..tab_end).contains(&index);
        if on_page {
            let slot = index - tab_start;
            transform.translation.x = -200.0 + slot as f32 * 82.0;
        }
        text.0 = label.0.display_name().to_owned();
        *label_visibility = if upload.window_open() && on_page {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (index, layer) in layers.iter().enumerate() {
        if existing_labels.contains(layer) {
            continue;
        }
        let slot = index.saturating_sub(tab_start);
        let visible = (tab_start..tab_end).contains(&index);
        commands.spawn((
            ShipUploadModalContent,
            ShipUploadLayerTabLabel(layer.clone()),
            Text2d::new(layer.display_name()),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(-200.0 + slot as f32 * 82.0, -145.0, 83.0),
            RenderLayers::layer(1),
            if upload.window_open() && visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
    }
    for mut value in &mut visibility {
        *value = if upload.window_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut value in &mut save_controls {
        *value = if upload.window_open() && has_required_fields {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (button, mut sprite) in &mut resource_buttons {
        if let ShipUploadAction::SelectResource(resource_type) = button.0 {
            sprite.color = if ui.preview_resource == Some(resource_type) {
                Color::srgb(0.38, 0.43, 0.62)
            } else {
                Color::srgb(0.24, 0.28, 0.43)
            };
        }
    }
    for (mut sprite, mut preview_visibility) in &mut previews {
        if !upload.window_open() {
            *preview_visibility = Visibility::Hidden;
            continue;
        }
        let resource_type = ui.preview_resource.unwrap_or(ShipResourceType::Base);
        if resource_type == ShipResourceType::Materials {
            *preview_visibility = Visibility::Hidden;
            continue;
        }
        let layer = if matches!(
            resource_type,
            ShipResourceType::Base | ShipResourceType::Materials
        ) {
            ShipLayer::default()
        } else {
            ui.active_layer.clone()
        };
        let texture = ship_upload_resources::selected_texture_now(&upload, resource_type, &layer, &mut images);
        match texture {
            Ok(Some(handle)) => {
                sprite.image = handle.clone();
                if let Some(image) = images.get(&handle) {
                    let width = image.texture_descriptor.size.width.max(1) as f32;
                    let height = image.texture_descriptor.size.height.max(1) as f32;
                    let fit = (410.0 / width).min(62.0 / height);
                    sprite.custom_size = Some(Vec2::new(width * fit, height * fit));
                }
                *preview_visibility = Visibility::Visible;
            }
            Ok(None) => *preview_visibility = Visibility::Hidden,
            Err(error) => {
                bevy::log::warn!("Could not render ship upload preview: {error}");
                *preview_visibility = Visibility::Hidden;
            }
        }
    }
    for mut value in &mut names {
        let name = upload.ship_name();
        value.0 = if name.is_empty() {
            "Ship Name".into()
        } else {
            name
        };
    }
    for mut value in &mut descriptions {
        value.0 = upload.ship_description();
    }
    for mut value in &mut layer_names {
        let name = upload.new_layer_name();
        value.0 = if name.is_empty() {
            "Layer Name".into()
        } else {
            name
        };
    }
    let status = if !has_required_fields {
        "Some required fields are missing"
    } else {
        ui.notice
            .as_deref()
            .or_else(|| upload.error_string())
            .or_else(|| upload.success_string())
            .unwrap_or("")
    };
    for mut value in &mut statuses {
        value.0 = status.to_owned();
    }
}

fn sync_source_daylight(mut simulation: ResMut<Simulation>, windows: Query<&Window>) {
    let visible = windows
        .single()
        .is_ok_and(|window| window.physical_width() != 0 && window.physical_height() != 0);
    if visible && simulation.cycle_enabled {
        simulation.day = time_sync::daylight(simulation.elapsed, simulation.cycle_length);
    }
}
fn select_ship_layer(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    catalog: Res<ShipCatalog>,
    preview: Option<Res<ship_upload_preview::ActivePreview>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    capture: Option<Res<ui_input_capture::Capture>>,
    mut simulation: ResMut<Simulation>,
) {
    if keys.just_pressed(KeyCode::Escape) || !simulation.show_tools {
        simulation.layer_dropdown_open = false;
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
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
    if capture.is_some_and(|capture|capture.editor_point(point)) {return;}
    let world_point=point;
    let point=point-simulation.tool_panel_offset;
    let count = ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    if let Some(layout)=simulation.source_tools {
        if simulation.layer_dropdown_open {
            if layout.popup(count).contains(world_point) {
                if let Some(index)=(0..count).find(|index|source_tools_popup::option_contains(&layout,count,*index,world_point)) {
                    simulation.selected_layer=index;simulation.layer_dropdown_open=false;return;
                }
                return; // Popup padding/scrollbar clicks keep the source popup open.
            }
            simulation.layer_dropdown_open=false;
        }
        if layout.combo.contains(world_point) {simulation.layer_dropdown_open=true;simulation.layer_popup_scroll=0.0;}
        return;
    }
    if simulation.layer_dropdown_open {
        if (-260.0..=-30.0).contains(&point.x)
            && (297.0 - count as f32 * 24.0..=297.0).contains(&point.y)
        {
            let index = ((297.0 - point.y) / 24.0).floor().max(0.0) as usize;
            if index < count {
                simulation.selected_layer = index;
                simulation.layer_dropdown_open = false;
                return;
            }
        }
        simulation.layer_dropdown_open = false;
    }
    if point_in_tool_panel(point)
        && (-260.0..=-30.0).contains(&point.x)
        && (296.0..=324.0).contains(&point.y)
    {
        simulation.layer_dropdown_open = true;
    }
}

fn update_ship_layer_label(
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
    preview: Option<Res<ship_upload_preview::ActivePreview>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut labels: Query<&mut Text2d, With<ShipLayerLabel>>,
) {
    let layer = ship_upload_preview::layer(&catalog,&simulation,preview.as_deref(),retained.as_deref(),simulation.selected_layer).map_or("Default",|layer|layer.display_name());
    for mut label in &mut labels {
        label.0 = layer.to_owned();
    }
}

fn spawn_ship_layer_dropdown_option(commands: &mut Commands, index: usize) {
    let y = 285.0 - index as f32 * 24.0;
    commands.spawn((
        ShipLayerDropdownOption(index),
        source_tools_layout::Role::Option(index),
        Sprite::from_color(Color::srgb(0.16, 0.18, 0.25), Vec2::new(230.0, 24.0)),
        tool_panel_position(-145.0, y, 40.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ShipLayerDropdownOptionLabel(index),
        source_tools_layout::Role::OptionLabel(index),
        Text2d::new("Default"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        tool_panel_position(-145.0, y, 41.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
}

fn sync_ship_layer_dropdown(
    mut commands: Commands,
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
    preview: Option<Res<ship_upload_preview::ActivePreview>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut background: Query<
        (
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
            Option<&mut ToolPanelPosition>,
        ),
        (
            With<ShipLayerDropdownBackground>,
            Without<ShipLayerDropdownOption>,
            Without<ShipLayerDropdownOptionLabel>,
        ),
    >,
    mut options: Query<
        (&ShipLayerDropdownOption, &mut Sprite, &mut Visibility),
        (
            Without<ShipLayerDropdownOptionLabel>,
            Without<ShipLayerDropdownBackground>,
        ),
    >,
    mut labels: Query<
        (&ShipLayerDropdownOptionLabel, &mut Text2d, &mut Visibility),
        (
            Without<ShipLayerDropdownOption>,
            Without<ShipLayerDropdownBackground>,
        ),
    >,
) {
    let open = simulation.layer_dropdown_open && simulation.show_tools;
    let count=ship_upload_preview::layer_count(&catalog,&simulation,preview.as_deref(),retained.as_deref());
    for (mut sprite, mut transform, mut visibility, position) in &mut background {
        let height = count as f32 * 24.0;
        sprite.custom_size = Some(Vec2::new(230.0, height));
        transform.translation.y = 297.0 - height * 0.5;
        if let Some(mut position) = position {
            position.0.y = transform.translation.y;
        }
        *visibility = if open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let existing: std::collections::HashSet<usize> =
        options.iter().map(|(option, _, _)| option.0).collect();
    for index in 0..count {
        if !existing.contains(&index) {
            spawn_ship_layer_dropdown_option(&mut commands, index);
        }
    }
    for (option, mut sprite, mut visibility) in &mut options {
        let visible = open && option.0 < count;
        sprite.color = if option.0 == simulation.selected_layer {
            Color::srgb(0.30, 0.36, 0.54)
        } else {
            Color::srgb(0.16, 0.18, 0.25)
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (option, mut label, mut visibility) in &mut labels {
        let visible = open && option.0 < count;
        label.0=ship_upload_preview::layer(&catalog,&simulation,preview.as_deref(),retained.as_deref(),option.0).map_or("Default",|layer|layer.display_name()).to_owned();
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn select_ship_from_panel(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    windows: Query<&Window>,
    mut simulation: ResMut<Simulation>,
    mut search_text: Query<&mut Text2d, With<ShipSearchText>>,
    native_characters: Option<Res<Messages<window_characters::CharacterInput>>>,
    mut native_cursor: Local<bevy::ecs::message::MessageCursor<window_characters::CharacterInput>>,
    capture: Option<Res<ui_input_capture::Capture>>,
    native_window:Option<NonSend<window_bevy::LiveWindow>>,
) {
    // Drain text events even when the search box is not focused so typing elsewhere
    // cannot be replayed into the next search session.
    let keyboard_events: Vec<_> = keyboard_events.read().cloned().collect();
    let committed_text=window_characters::read_codepoints(native_characters.as_deref(),&mut native_cursor);
    if String::from_utf16_lossy(&simulation.ship_search_units)!=simulation.ship_search {simulation.ship_search_units=simulation.ship_search.encode_utf16().collect();}
    if simulation.toolbox_collapsed || simulation.active_tab != ToolboxTab::Ships {
        simulation.ship_search_active = false;
        return;
    }
    if simulation.ship_search_active {
        for point in committed_text {
            let units=window_characters::units(point);
            if simulation.ship_search_units.len()+units.len()>255 {break;}
            simulation.ship_search_units.extend(units);
        }
        if keyboard_events.iter().any(|event| {
            event.key_code == KeyCode::Backspace && event.state == bevy::input::ButtonState::Pressed
        }) {
            simulation.ship_search_units.pop();
        }
        if keys.just_pressed(KeyCode::Escape) {
            simulation.ship_search_active = false;
        }
        if native_window.is_none() {for event in &keyboard_events {
            if event.state != bevy::input::ButtonState::Pressed {
                continue;
            }
            let Some(text) = &event.text else {
                continue;
            };
            for character in text.chars().filter(|character| !character.is_control()) {
                // The source search buffer is 256 UTF-16 slots including its NUL terminator.
                let used_units = simulation.ship_search_units.len();
                if used_units + character.len_utf16() > 255 {
                    break;
                }
                simulation.ship_search_units.extend(window_characters::units(character as u32));
            }
        }}

    }
    simulation.ship_search=String::from_utf16_lossy(&simulation.ship_search_units);
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
        if capture.is_some_and(|capture|capture.editor_point(Vec2::new(x,y))) {
            simulation.ship_search_active=false;
            return;
        }
        simulation.ship_search_active =
            (-628.0..=-401.5).contains(&x) && (238.0..=268.0).contains(&y);
    }
}

fn source_ship_search_units(simulation:&Simulation)->std::borrow::Cow<'_,[u16]> {
    if String::from_utf16_lossy(&simulation.ship_search_units)==simulation.ship_search {std::borrow::Cow::Borrowed(&simulation.ship_search_units)}
    else {std::borrow::Cow::Owned(simulation.ship_search.encode_utf16().collect())}
}
fn source_filtered_ship_indices(catalog:&ShipCatalog,simulation:&Simulation)->Vec<usize> {
    let search=source_ship_search_units(simulation);
    catalog.0.iter().enumerate().filter(|(_,choice)|java_string::java_contains_ignore_case_units(&choice.name.encode_utf16().collect::<Vec<_>>(),&search)).map(|(index,_)|index).collect()
}
fn filtered_ship_indices(catalog: &ShipCatalog, search: &str) -> Vec<usize> {
    catalog
        .0
        .iter()
        .enumerate()
        .filter(|(_, choice)| crate::java_string::java_contains_ignore_case(&choice.name, search))
        .map(|(index, _)| index)
        .collect()
}

fn refresh_ship_catalog(
    time: Res<Time>,
    assets: Res<AssetServer>,
    mut catalog: ResMut<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
    mut commands: Commands,
    mut ship_cards: Query<Entity, Or<(With<ShipCard>, With<ShipThumbnail>, With<ShipNameLabel>)>>,
    mut timer: Local<Option<Timer>>,
    mut pending_scan: Local<Option<ShipCatalog>>,
    native:Option<NonSend<live_ship_catalog::LiveCatalog>>,
    retained:Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut published:Local<Option<u64>>,
) {
    if let Some(native)=native {
        if *published==Some(native.revision) {return;}
        *published=Some(native.revision);
        let next=native.ui_catalog();
        if next==*catalog {return;}
        simulation.ship_index=retained.as_ref().and_then(|retained|retained.0.as_ref()).and_then(|active|native.active_index(&next,active)).unwrap_or(usize::MAX);
        simulation.catalog_revision=simulation.catalog_revision.wrapping_add(1);
        *catalog=next;
        for entity in &mut ship_cards {commands.entity(entity).despawn();}
        for (index,choice) in catalog.0.iter().enumerate() {spawn_ship_catalog_card(&mut commands,&assets,index,choice,index<4);}
        return;
    }
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(1.0, TimerMode::Repeating));
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    // Toolbox.reloadFiles creates thumbnails from the previous shipFiles map,
    // then publishes the newly scanned map. Keep that one-scan delay here.
    let scanned = ShipCatalog::discover();
    let Some(mut next) = pending_scan.replace(scanned) else {
        return;
    };

    let selected = catalog.0.get(simulation.ship_index).cloned().map(|choice| {
        let layers = catalog
            .1
            .get(simulation.ship_index)
            .cloned()
            .unwrap_or_default();
        (choice, layers)
    });

    if let Some((choice, layers)) = &selected {
        if !next.0.iter().any(|candidate| {
            candidate.asset == choice.asset && candidate.physics_asset == choice.physics_asset
        }) {
            // Keep the currently loaded vessel selectable if its file is removed while open.
            next.0.push(choice.clone());
            next.1.push(layers.clone());
        }
    }

    if next == *catalog {
        return;
    }

    if let Some((choice, _)) = &selected {
        simulation.ship_index = next
            .0
            .iter()
            .position(|candidate| {
                candidate.asset == choice.asset && candidate.physics_asset == choice.physics_asset
            })
            .unwrap_or(0);
    } else {
        simulation.ship_index = 0;
    }
    simulation.selected_layer = 0;
    simulation.catalog_revision = simulation.catalog_revision.wrapping_add(1);
    *catalog = next;

    for entity in &mut ship_cards {
        commands.entity(entity).despawn();
    }
    for (index, choice) in catalog.0.iter().enumerate() {
        spawn_ship_catalog_card(&mut commands, &assets, index, choice, index < 4);
    }
}

fn load_selected_ship(
    catalog: Res<ShipCatalog>,
    mut simulation: ResMut<Simulation>,
    mut drag: ResMut<tools::move_tool::MoveDragState>,
    mut structure: ResMut<ShipStructure>,
    mut snapshot: ResMut<GpuShipPhysicsSnapshot>,
    mut gpu_physics: ResMut<GpuShipPhysicsAssets>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut visuals: ship_visual_replacement::VisualReplacement,
    mut preview: ResMut<ship_upload_preview::ActivePreview>,
    mut last_ship: Local<Option<(usize, u64)>>,
    mut retained: Option<ResMut<ship_runtime_reset::ActiveThumbnail>>,
    resets: Option<Res<Messages<window_characters::ResetShip>>>,
    mut reset_cursor: Local<bevy::ecs::message::MessageCursor<window_characters::ResetShip>>,
    live_catalog:Option<NonSend<live_ship_catalog::LiveCatalog>>,
    mut selection:Option<ResMut<live_ship_catalog::PendingSelection>>,
) {
    let reset_count = resets
        .as_deref()
        .map_or(0, |messages| reset_cursor.read(messages).count());
    let selected=selection.as_mut().and_then(|selection|selection.0.take());
    // Native browser indices are presentation state. Only explicit selection,
    // editor Test Ship, or native reset may replace the active source ship.
    if live_catalog.is_some() && selected.is_none() && reset_count==0 && preview.pending.is_none() {
        *last_ship=Some((simulation.ship_index,preview.revision));return;
    }
    let state = (simulation.ship_index, preview.revision);
    if selected.is_none() && reset_count == 0 && last_ship.is_none() && preview.pending.is_none() {
        *last_ship = Some(state);
        return;
    }
    if selected.is_none() && reset_count == 0 && *last_ship == Some(state) {
        return;
    }
    // A catalog rescan can move indices while the editor's unsaved ship stays active.
    if selected.is_none() && reset_count == 0 && preview.thumbnail.is_some() && preview.pending.is_none() {
        *last_ship = Some(state);
        return;
    }
    *last_ship = Some(state);
    let activate=selected.is_some() || preview.pending.is_some() || reset_count==0;
    for step in 0..reset_count+usize::from(activate) {
        let next = if step<reset_count {
            let thumbnail=retained.as_ref().and_then(|retained|retained.0.as_ref()).or(preview.thumbnail.as_ref());
            if let Some(thumbnail)=thumbnail {
                match ShipStructure::load_for_thumbnail(thumbnail) {
                    Ok(next)=>Some(next),
                    Err(error)=>{bevy::log::error!("Could not reset active thumbnail: {error}");None},
                }
            } else {catalog.0.get(simulation.ship_index).map(ShipStructure::load_for_choice)}
        } else if let Some(thumbnail)=selected.as_ref() {
            match ShipStructure::load_for_thumbnail(thumbnail) {
                Ok(next)=>{if let Some(retained)=retained.as_mut(){retained.0=Some(thumbnail.clone());}Some(next)},
                Err(error)=>{bevy::log::error!("Could not select source browser ship: {error}");None},
            }
        } else if let Some(next)=preview.pending.take() {
            if let Some(retained)=retained.as_mut() {retained.0=preview.thumbnail.clone();}
            Some(next)
        } else {
            catalog.0.get(simulation.ship_index).map(|choice| {
                let (next,thumbnail)=if let Some(thumbnail)=live_catalog.as_ref().and_then(|catalog|catalog.thumbnail(choice)) {
                    match ShipStructure::load_for_thumbnail(&thumbnail) {
                        Ok(next)=>(next,ship_runtime_reset::ActiveThumbnail(Some(thumbnail))),
                        Err(error)=>{bevy::log::error!("Could not construct cached catalog ship: {error}");ship_runtime_reset::choice(choice)},
                    }
                } else {ship_runtime_reset::choice(choice)};
                if let Some(retained)=retained.as_mut() {**retained=thumbnail;}
                next
            })
        };
        let Some(next) = next else {
            return;
        };
        // Every new Ship starts with currentLayer = 0, including Shift+R.
        simulation.selected_layer = 0;
        *structure = next;
        // Source selection constructs a new Ship, whose moving/base/offset fields
        // start at false/zero. A preceding ship's release delta must not carry over.
        *drag = tools::move_tool::MoveDragState::default();
        replace_gpu_ship_physics(&structure, &mut gpu_physics, &mut shader_buffers);
        *snapshot = GpuShipPhysicsSnapshot::default();
        visuals.construct(&structure, &gpu_physics, &simulation, &mut meshes);
    }
}

fn select_toolbox_tab_and_settings(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    tabs: Query<(&TabButton, &Transform, &Sprite)>,
    buttons: Query<(&SettingButton, &Transform, &Visibility, &Sprite)>,
    rows: Query<(&SettingRow, &Transform, &Visibility)>,
    mut simulation: ResMut<Simulation>,
    mut dragging: Local<Option<(usize, Vec2)>>,
    mut music: ResMut<music_player::MusicPlayer>,
    capture: Option<Res<ui_input_capture::Capture>>,
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
            let picker_point = toolbox_viewport::unscroll_point(&simulation, point);
            match index {
                DRAG_SEA_COLOR => apply_sea_color_point(picker_point, &mut simulation),
                DRAG_SEA_HUE => apply_sea_hue_point(picker_point, &mut simulation),
                DRAG_SEA_ALPHA => simulation.sea_alpha = toolbox::alpha_at(picker_point.y),
                _ => {let delta=point.x-previous.x;let delta=if index==19 {simulation.source_tools.map_or(delta,|layout|delta*layout.viewport.source_size.y/720.0)}else {delta};simulation.adjust(index,delta*setting_drag_speed(index));},
            }
            *dragging = Some((index, point));
        }
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // A new press belongs to the front editor; an existing widget drag keeps ownership.
    if capture.is_some_and(|capture|capture.editor_point(point)) {return;}
    if point_in_toolbox_header(point) {
        simulation.toolbox_collapsed = !simulation.toolbox_collapsed;
        return;
    }
    let tool_point = point - simulation.tool_panel_offset;
    if simulation.show_tools
        && mouse.just_pressed(MouseButton::Left)
        && simulation.source_tools.map_or((-260.0..=-30.0).contains(&tool_point.x)&&(174.0..=202.0).contains(&tool_point.y),|layout|layout.slider.contains(point))
    {
        *dragging = Some((19, point));
        return;
    }
    // The independent Tools window captures its pointer region before Toolbox widgets.
    if simulation.show_tools && simulation.source_tools.map_or(point_in_tool_panel(tool_point),|layout|layout.captures(point,simulation.layer_dropdown_open,layout.layer_count)) {
        return;
    }
    if simulation.toolbox_collapsed {
        return;
    }
    if point.x < -640.0 || point.x > -290.0 {
        return;
    }
    if let Some((tab, _, _)) = tabs.iter().find(|(_, transform, sprite)| {
        (point.x - transform.translation.x).abs()
            < sprite.custom_size.unwrap_or(Vec2::splat(0.0)).x * 0.5
            && (point.y - transform.translation.y).abs() < 14.0
    }) {
        simulation.active_tab = tab.0;
        return;
    }
    let Some(point) = toolbox_viewport::content_point(&simulation, point) else { return; };
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
    if let Some((row, _, _)) = rows.iter().find(|(_, transform, visibility)| {
        **visibility != Visibility::Hidden
            && (point.x - transform.translation.x).abs() < 112.5
            && (point.y - transform.translation.y).abs() < 17.0
    }) {
        *dragging = Some((row.0, point));
        return;
    }
    let Some((button, _, _, _)) = buttons.iter().find(|(_, transform, visibility, sprite)| {
        let size = sprite.custom_size.unwrap_or(Vec2::splat(0.0)) * 0.5;
        **visibility != Visibility::Hidden
            && (point.x - transform.translation.x).abs() < size.x
            && (point.y - transform.translation.y).abs() < size.y
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
        SettingAction::GeneratePalette => {
            let reader = crate::file_reader::FileReader::game();
            let output = reader.ss_home.join("palette.png");
            match crate::palette_gen::gen_palette(
                std::path::Path::new("config/materials.json"),
                &output,
            ) {
                Ok(()) => {
                    #[cfg(windows)]
                    if let Err(error) = std::process::Command::new("explorer")
                        .arg(&reader.ss_home)
                        .spawn()
                    {
                        bevy::log::warn!("Could not open palette folder: {error}");
                    }
                }
                Err(error) => bevy::log::error!("Could not generate palette: {error}"),
            }
        }
    }
}

fn setting_drag_speed(index: usize) -> f32 {
    match index {
        0 | 1 | 3 | 4 | 5 | 6 | 7 | 9 | 12 => 0.05,
        2 => 0.5,
        8 => 0.1,
        10 | 11 => 0.005,
        13 => 0.005,
        14 => 0.1,
        15 => 0.01,
        16 | 17 => 0.2,
        18 => 0.01,
        19 => 0.1,
        21..=24 => 1.0 / 255.0,
        _ => 0.01,
    }
}

fn apply_sea_color_point(point: Vec2, simulation: &mut Simulation) {
    let sv = gui_color_picker_input::pointer_sv(point);
    let hsv = simulation.sea_color_memory.read(simulation.sea_color);
    simulation.sea_color = simulation.sea_color_memory.edit(Vec3::new(hsv.x, sv.x, sv.y));
}

fn apply_sea_hue_point(point: Vec2, simulation: &mut Simulation) {
    let mut hsv = simulation.sea_color_memory.read(simulation.sea_color);
    hsv.x = gui_color_picker_input::pointer_hue(point.y);
    simulation.sea_color = simulation.sea_color_memory.edit(hsv);
}

fn rgb_to_hsv(rgb: Vec3) -> (f32, f32, f32) {
    let hsv = gui_color_picker_input::rgb_to_hsv(rgb);
    (hsv.x * 360.0, hsv.y, hsv.z)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    gui_color_picker_input::hsv_to_rgb(Vec3::new(hue / 360.0, saturation, value))
}
fn color_picker_mesh(hue: f32) -> Mesh {
    gui_color_picker::saturation_value(Vec2::new(188.0, 360.0), gui_color_picker_input::hsv_to_rgb(Vec3::new(hue, 1.0, 1.0)))
}

fn hue_bar_mesh() -> Mesh {
    gui_color_picker::hue(Vec2::new(18.0, 360.0))
}

fn alpha_bar_mesh(color: Vec3) -> Mesh {
    gui_color_picker::alpha(Vec2::new(14.0, 360.0), color)
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
    let picker_hue = simulation.sea_color_memory.read(simulation.sea_color).x;
    let hue_key = picker_hue.to_bits();
    if *last_picker_hue != Some(hue_key) {
        for picker in &color_picker {
            if let Some(mut mesh) = meshes.get_mut(&picker.0) {
                *mesh = color_picker_mesh(picker_hue);
            }
        }
        *last_picker_hue = Some(hue_key);
    }
    for (mut transform, alpha) in &mut color_selector {
        if alpha.is_some() {
            transform.translation.y = toolbox::alpha_marker_y(simulation.sea_alpha);
            continue;
        }
        let hsv = simulation.sea_color_memory.read(simulation.sea_color);
        let marker = gui_color_picker_input::sv_marker(hsv.y, hsv.z);
        transform.translation.x = marker.x;
        transform.translation.y = marker.y;
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
        if label.0 != "Toolbox" {label.0="Toolbox".to_owned();}
    }
}

fn select_tool_from_panel(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut simulation: ResMut<Simulation>,
    capture: Option<Res<ui_input_capture::Capture>>,
) {
    if !simulation.show_tools || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let ui_scale = 720.0 / window.height().max(1.0);
    let point=Vec2::new((cursor.x-window.width()*0.5)*ui_scale,(window.height()*0.5-cursor.y)*ui_scale);
    if capture.is_some_and(|capture|capture.editor_point(point)) {return;}
    if let Some(layout)=simulation.source_tools {
        if let Some(index)=layout.buttons.iter().position(|rect|rect.contains(point)) {
            simulation.tool.toggle([Tool::Break,Tool::Flood,Tool::Dry,Tool::Move][index]);
        }
        return;
    }
    let point=point-simulation.tool_panel_offset;
    let x=point.x;let y=point.y;
    if !(222.0..=278.0).contains(&y) {
        return;
    }
    if (-248.0..=-192.0).contains(&x) {
        simulation.tool.toggle(Tool::Break);
    } else if (-170.0..=-114.0).contains(&x) {
        simulation.tool.toggle(Tool::Flood);
    } else if (-92.0..=-36.0).contains(&x) {
        simulation.tool.toggle(Tool::Dry);
    } else if (-14.0..=42.0).contains(&x) {
        simulation.tool.toggle(Tool::Move);
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
    mut simulation: ResMut<Simulation>,
    mut structure: ResMut<ShipStructure>,
) {
    let delta = time.delta_secs().min(1.0 / 30.0);
    if delta <= 0.0 {
        return;
    }
    // SS2 runs its force pass once per configured physics step and schedules
    // the more expensive water pass only at water-step intervals.
    let substeps = simulation.physics_iterations.round().clamp(1.0, 1000.0) as usize;
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
    let water_steps = simulation.water_steps.round().clamp(
        1.0,
        simulation.physics_iterations.round().clamp(1.0, 1000.0),
    ) as usize;
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
    windows: Query<&Window>,
    mut drag: ResMut<tools::move_tool::MoveDragState>,
    simulation: Res<Simulation>,
    structure: Res<ShipStructure>,
    mut physics: ResMut<GpuShipPhysicsAssets>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    // MaskData is initialized only when creating/resetting a ship. Source
    // BreakTool and ShipPhysics mutate the live GPU state; asynchronous
    // readbacks and the legacy coarse CPU breach grid are observational only.

    let configured_iterations = simulation.physics_iterations.round().clamp(1.0, 1000.0) as u32;
    let water_steps = simulation
        .water_steps
        .round()
        .clamp(1.0, configured_iterations as f32) as u32;
    let visible = windows.single().map_or(true, |window| {
        window.physical_width() != 0 && window.physical_height() != 0
    });
    let iterations = if visible && !drag.dragging { configured_iterations } else { 0 };
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
        let derived = (|| -> Result<Handle<Image>, String> {
            let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(
                &format!("assets/{}", choice.physics_asset),
            ))?;
            // This synchronous asset path must finish BASE before the source's
            // nonblocking generated texture can return an image.
            thumbnail.base_layer()?;
            match thumbnail.get_resource(ShipResourceType::Texture, &ShipLayer::default()) {
                Some(ship_thumbnail::ThumbnailResource::File(resource)) => {
                    resource.texture_now(images)
                }
                Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(resource)) => {
                    resource.texture_now_current(images)
                }
                None => Err("missing default texture".into()),
            }
        })();
        match derived {
            Ok(texture) => return texture,
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
    black: &Handle<Image>,
) -> (Handle<Image>, Handle<Image>) {
    let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(std::path::Path::new(&format!(
        "assets/{}",
        choice.physics_asset
    )))
    .ok();
    let mut load = |kind| match thumbnail
        .as_ref()
        .and_then(|ship| ship.get_resource(kind, layer))
    {
        Some(ship_thumbnail::ThumbnailResource::File(resource)) => {
            match resource.texture_now(images) {
                Ok(texture) => texture,
                Err(error) => {
                    bevy::log::warn!("Could not load ship light map: {error}");
                    assets
                        .load_builder()
                        .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                            settings.sampler = texture_2d::ship_sampler();
                        })
                        .load(asset_path(&resource.path))
                }
            }
        }
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
    black_texture: Res<ship_black_texture::SharedBlackTexture>,
    catalog: Res<ShipCatalog>,
    simulation: Res<Simulation>,
    preview: Res<ship_upload_preview::ActivePreview>,
    retained: Option<Res<ship_runtime_reset::ActiveThumbnail>>,
    mut materials: ResMut<Assets<ShipMaterial>>,
    mut reflection_materials: ResMut<Assets<ReflectionMaterial>>,
    mut ships: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<ShipSprite>>,
    mut mesh_ships: Query<(&ShipMesh, &mut Visibility), (Without<ShipSprite>,Without<ReflectionMesh>)>,
    mut reflections: Query<(&ReflectionMesh, &mut Transform, &mut Visibility), (Without<ShipSprite>,Without<ShipMesh>)>,
    mut last_ship: Local<Option<(usize, usize,u64)>>,
) {
    let state = (simulation.ship_index, simulation.selected_layer,preview.revision);
    let changed=*last_ship!=Some(state);
    *last_ship=Some(state);
    let active=preview.thumbnail.as_ref().or_else(||retained.as_ref().and_then(|retained|retained.0.as_ref()));
    if !changed && active.is_none() {return;}
    let (image,internal_lights,external_lights,visible,scale)=if let Some(thumbnail)=active {
        let appearance=ship_runtime_reset::appearance(thumbnail,simulation.selected_layer,&mut images,&black_texture.0);
        let width=if changed {thumbnail.base_layer().map(|image|image.width()).unwrap_or(1000)}else {1000};
        (appearance.texture,appearance.internal,appearance.external,appearance.visible,(360.0/width.max(1)as f32).clamp(0.01,1.0))
    } else {
        let Some(choice)=catalog.0.get(simulation.ship_index) else {return;};
        let visual_asset=catalog.layer_asset(simulation.ship_index,simulation.selected_layer).unwrap_or(&choice.asset);
        let layer=ship_upload_preview::layer(&catalog,&simulation,None,None,simulation.selected_layer).cloned().unwrap_or_default();
        let image=load_ship_visual_asset(choice,visual_asset,&assets,&mut images);
        let (internal,external)=load_ship_light_assets(choice,&layer,&assets,&mut images,&black_texture.0);
        (image,internal,external,true,choice.scale)
    };
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
        *visibility = if visible {Visibility::Inherited}else{Visibility::Hidden};
    }
    for (reflection, mut transform, mut visibility) in &mut reflections {
        if let Some(mut material) = reflection_materials.get_mut(&reflection.material) {
            material.texture = image.clone();
        }
        if changed {transform.scale = Vec3::splat(scale);}
        *visibility = if visible {Visibility::Inherited}else{Visibility::Hidden};
    }
}

fn sync_tool_icons(
    simulation: Res<Simulation>,
    mut tools: Query<(&ToolCard, &mut Sprite, &mut Visibility), Without<ToolGlyph>>,
    mut glyphs: Query<(&ToolGlyph, &mut Sprite, &mut Visibility), Without<ToolCard>>,
) {
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
    for (glyph, mut sprite, mut visibility) in &mut glyphs {
        sprite.image = if glyph.tool == simulation.tool {
            glyph.active.clone()
        } else {
            glyph.normal.clone()
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tool_panel_anchor_keeps_render_clicks_and_camera_capture_aligned_on_resize() {
        let mut app = App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(music_player::MusicPlayer::new(Vec::new()))
            .add_systems(PreUpdate, update_tool_panel_layout)
            .add_systems(
                Update,
                (
                    sync_tool_panel_position,
                    select_toolbox_tab_and_settings,
                    select_tool_from_panel,
                )
                    .chain(),
            );
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(60.0, 620.0)));
        let icon = app
            .world_mut()
            .spawn(tool_panel_position(-220.0, 250.0, 22.0))
            .id();
        app.world_mut().spawn((
            SettingRow(8),
            Transform::from_xyz(-580.0, -260.0, 24.0),
            Visibility::Visible,
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world().get::<Transform>(icon).unwrap().translation,
            Vec3::new(-580.0, -260.0, 22.0)
        );
        assert!(app.world().resource::<Simulation>().tool == Tool::Break);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(70.0, 620.0)));
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().gravity,
            9.81,
            "tool click must not start a drag on an underlying toolbox field"
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(60.0, 620.0)));
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution = (1920, 720).into();
        {
            let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
            mouse.reset_all();
            mouse.press(MouseButton::Left);
        }
        app.update();
        let position = app
            .world()
            .get::<Transform>(icon)
            .unwrap()
            .translation
            .truncate();
        assert_eq!(position, Vec2::new(-900.0, -260.0));
        assert!(app.world().resource::<Simulation>().tool == Tool::None);
        let mut simulation = app.world_mut().resource_mut::<Simulation>();
        simulation.toolbox_collapsed = true;
        assert!(camera_control::blocked(&simulation, position));
        assert!(!camera_control::blocked(
            &simulation,
            Vec2::new(-220.0, 250.0)
        ));
    }

    #[test]
    fn ship_editor_consumes_committed_unicode_for_each_field_and_drains_closed_text() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Simulation>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ShipUploadUiState>()
            .insert_resource(editor_clipboard::Clipboard {fixture:Some(Vec::new())})
            .init_resource::<ship_upload_layout::Layout>()
            .init_resource::<ship_upload_preview::ActivePreview>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ShipCatalog(Vec::new(), Vec::new()))
            .add_message::<KeyboardInput>()
            .add_message::<FileDragAndDrop>()
            .add_message::<window_characters::CharacterInput>()
            .add_systems(Update, handle_ship_upload_ui)
            .add_systems(PostUpdate,ui_text_viewport::update);
        let mut window = Window::default();
        window.set_cursor_position(Some(Vec2::new(30.0, 30.0)));
        app.world_mut().spawn(window);
        app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Description),Transform::from_xyz(0.0,203.0,82.0),
            Sprite::from_color(Color::WHITE,Vec2::new(536.0,144.0))));
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().create_new();
        for field in [ShipUploadField::Name, ShipUploadField::Description, ShipUploadField::LayerName] {
            app.world_mut().resource_mut::<ShipUploadUiState>().focus = Some(field);
            if field==ShipUploadField::LayerName {
                let world=app.world_mut();let mut query=world.query::<&mut Window>();
                query.single_mut(world).unwrap().set_cursor_position(None);
            }
            for character in "船🚢".chars() {
                app.world_mut().write_message(window_characters::CharacterInput(character as u32));
            }
            app.update();
        }
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            query.single_mut(world).unwrap().set_cursor_position(Some(Vec2::new(30.0,30.0)));
        }
        let upload = app.world().resource::<ship_upload::SourceShipUpload>();
        assert_eq!(upload.ship_name(), "船🚢");
        assert_eq!(upload.ship_description(), "A Description船🚢");
        assert_eq!(upload.new_layer_name(), "船🚢");
        // Exercise actual native keyboard messages through the editor handler.
        app.world_mut().resource_mut::<ShipUploadUiState>().focus = Some(ShipUploadField::Name);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_name("abcd");
        app.update();
        let target = app.world_mut().query::<Entity>().iter(app.world()).next().unwrap();
        let send_key = |app: &mut App, code: KeyCode| {
            app.world_mut().write_message(KeyboardInput {
                key_code: code,
                logical_key: bevy::input::keyboard::Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                state: bevy::input::ButtonState::Pressed, text: None, repeat: false, window: target,
            });
            app.update();
        };
        // Clipboard shortcuts exercise the native handler with isolated data.
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::KeyC);
        assert_eq!(app.world().resource::<editor_clipboard::Clipboard>().fixture.as_deref(),Some(&[97,98,99,100][..]));
        send_key(&mut app,KeyCode::KeyX);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"");
        app.world_mut().resource_mut::<editor_clipboard::Clipboard>().fixture=Some(vec![65,9,10,13,127,0xe000,0xd83d,0xde80,0,66]);
        send_key(&mut app,KeyCode::KeyV);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"A🚀");
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[0].undo.records[1].delete_length,3);
        send_key(&mut app,KeyCode::Insert);
        assert_eq!(app.world().resource::<editor_clipboard::Clipboard>().fixture.as_deref(),Some(&[65,0xd83d,0xde80][..]));
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
        send_key(&mut app,KeyCode::Delete);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"");
        send_key(&mut app,KeyCode::Insert);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"A🚀");
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ShiftLeft);
        app.world_mut().resource_mut::<ShipUploadUiState>().focus=Some(ShipUploadField::Description);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::KeyX);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_description(),"A Description船🚢");
        app.world_mut().resource_mut::<editor_clipboard::Clipboard>().fixture=Some(vec![10,65,9,13]);
        send_key(&mut app,KeyCode::KeyV);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_description(),"A Description船🚢\nA");
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_description("A Description船🚢");
        app.world_mut().resource_mut::<ShipUploadUiState>().focus=Some(ShipUploadField::Name);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_name("abcd");
        app.update();
        send_key(&mut app, KeyCode::Home);
        send_key(&mut app, KeyCode::ArrowRight);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
        send_key(&mut app, KeyCode::ArrowRight);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ShiftLeft);
        app.world_mut().write_message(window_characters::CharacterInput('X' as u32));
        app.update();
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(), "aXcd");
        // The original input queue precedes Backspace even in the same frame.
        app.world_mut().write_message(window_characters::CharacterInput('Z' as u32));
        send_key(&mut app, KeyCode::Backspace);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(), "aXcd");
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_name("");
        app.update();
        for value in "abc".chars() {app.world_mut().write_message(window_characters::CharacterInput(value as u32));}
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::KeyZ);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"ab");
        send_key(&mut app,KeyCode::KeyY);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),"ab");
        let editor=&app.world().resource::<ShipUploadUiState>().editors[0];
        assert_eq!((editor.buffer.len_w,editor.cursor),(1,2));
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        app.world_mut().resource_mut::<ShipUploadUiState>().focus = Some(ShipUploadField::Description);
        send_key(&mut app, KeyCode::Enter);
        assert!(app.world().resource::<ship_upload::SourceShipUpload>().ship_description().ends_with('\n'));
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_description("Wi\nil\nWWW\n");
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::Home);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::ArrowRight);send_key(&mut app,KeyCode::ArrowRight);
        for (key,expected) in [(KeyCode::ArrowDown,5),(KeyCode::ArrowDown,7),(KeyCode::ArrowUp,5),(KeyCode::ArrowUp,2)] {
            send_key(&mut app,key);
            let editor=&app.world().resource::<ShipUploadUiState>().editors[1];
            assert_eq!(editor.cursor,expected);assert_eq!(editor.preferred_x.to_bits(),1099232706);
        }
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
        send_key(&mut app,KeyCode::ArrowDown);
        let editor=&app.world().resource::<ShipUploadUiState>().editors[1];
        assert_eq!((editor.select_start,editor.select_end),(2,5));
        send_key(&mut app,KeyCode::ArrowUp);
        let editor=&app.world().resource::<ShipUploadUiState>().editors[1];
        assert_eq!((editor.select_start,editor.select_end),(2,2));
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ShiftLeft);

        // Ctrl+vertical arrows set child-window targets without changing the caret.
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_description("a\na\na\na\na\na\na\na");
        app.update();app.update();
        let caret=app.world().resource::<ShipUploadUiState>().editors[1].cursor;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::ArrowDown);
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,0.0);
        assert_eq!(app.world().resource::<ShipUploadUiState>().description_scroll.window.target.y,18.0);
        app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,18.0);
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].cursor,caret);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_ship_description("b\na\na\na\na\na\na\na");
        app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,18.0);
        send_key(&mut app,KeyCode::ArrowUp);app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,0.0);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        // Clicking the track seeks immediately, holds the caret, and captures outside drags.
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();let mut window=query.single_mut(world).unwrap();
            let point=Vec2::new(window.width()*0.5+261.0*window.height()/720.0,window.height()*0.5-135.0*window.height()/720.0);
            window.set_cursor_position(Some(point));
        }
        let caret=app.world().resource::<ShipUploadUiState>().editors[1].cursor;
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);app.update();
        assert!(app.world().resource::<ShipUploadUiState>().description_scroll.held);
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,24.0);
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].cursor,caret);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().clear();
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();let mut window=query.single_mut(world).unwrap();
            let point=Vec2::new(window.width()*0.5+1000.0*window.height()/720.0,window.height()*0.5-272.0*window.height()/720.0);
            window.set_cursor_position(Some(point));
        }
        app.update();assert_eq!(app.world().resource::<ShipUploadUiState>().editors[1].scroll_y,0.0);
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();app.update();
        assert!(!app.world().resource::<ShipUploadUiState>().description_scroll.held);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
        send_key(&mut app,KeyCode::Enter);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::ControlLeft);
        assert_eq!(app.world().resource::<ShipUploadUiState>().focus,None);
        let description=app.world().resource::<ship_upload::SourceShipUpload>().ship_description();
        app.world_mut().write_message(window_characters::CharacterInput('q' as u32));app.update();
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_description(),description);
        app.world_mut().resource_mut::<ShipUploadUiState>().focus=Some(ShipUploadField::Name);
        app.update();
        let original_name=app.world().resource::<ship_upload::SourceShipUpload>().ship_name();
        app.world_mut().write_message(window_characters::CharacterInput('Q' as u32));app.update();
        send_key(&mut app,KeyCode::Escape);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),original_name);
        assert!(app.world().resource::<ship_upload::SourceShipUpload>().window_open());
        assert_eq!(app.world().resource::<ShipUploadUiState>().focus,None);
        send_key(&mut app,KeyCode::Escape);
        assert!(app.world().resource::<ship_upload::SourceShipUpload>().window_open());
        // Pointer activation and clicking outside a field control ownership.
        app.world_mut().spawn((ShipUploadFieldControl(ShipUploadField::Name),
            Transform::default(),Sprite::from_color(Color::WHITE,Vec2::new(100.0,30.0))));
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            let mut window=query.single_mut(world).unwrap();
            let center=Vec2::new(window.width()*0.5,window.height()*0.5);
            window.set_cursor_position(Some(center));
        }
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().focus,Some(ShipUploadField::Name));
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();
        app.update(); // Source click detection requires a released frame between presses.
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            let mut window=query.single_mut(world).unwrap();
            // Source floors mouse coordinates; stay beyond the glyph midpoint after flooring.
            let point_x=-46.0+source_font_advances::advance(97)*0.8;
            let point=Vec2::new(window.width()*0.5+point_x*window.height()/720.0,window.height()*0.5);
            window.set_cursor_position(Some(point));
        }
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().editors[0].cursor,1);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().clear();
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            let mut window=query.single_mut(world).unwrap();
            let point=Vec2::new(window.width()*0.5+400.0*window.height()/720.0,window.height()*0.5);
            window.set_cursor_position(Some(point));
        }
        app.update();
        let editor=&app.world().resource::<ShipUploadUiState>().editors[0];
        assert_eq!((editor.cursor,editor.select_start,editor.select_end),(1,1,2));
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();
        app.update();
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            query.single_mut(world).unwrap().set_cursor_position(Some(Vec2::ZERO));
        }
        let name_before_outside=app.world().resource::<ship_upload::SourceShipUpload>().ship_name();
        app.world_mut().write_message(window_characters::CharacterInput('!' as u32));
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<ShipUploadUiState>().focus,None);
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().ship_name(),name_before_outside);
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();
        // Drive the original double-click pairing through the live handler.
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(100)));
        for _ in 0..4 {app.update();}
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            let mut window=query.single_mut(world).unwrap();
            let center=Vec2::new(window.width()*0.5,window.height()*0.5);
            window.set_cursor_position(Some(center));
        }
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        let editor=&app.world().resource::<ShipUploadUiState>().editors[0];
        assert_eq!((editor.select_start,editor.select_end),(0,2));
        assert!(editor.selected_all_mouse_lock);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().clear();
        {
            let world=app.world_mut();let mut query=world.query::<&mut Window>();
            query.single_mut(world).unwrap().set_cursor_position(Some(Vec2::ZERO));
        }
        app.update();
        let editor=&app.world().resource::<ShipUploadUiState>().editors[0];
        assert_eq!((editor.select_start,editor.select_end),(0,2));
        *app.world_mut().resource_mut::<ButtonInput<MouseButton>>()=Default::default();
        app.update();
        assert!(!app.world().resource::<ShipUploadUiState>().editors[0].selected_all_mouse_lock);
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(false);
        app.world_mut().write_message(window_characters::CharacterInput('x' as u32));
        app.update();
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(true);
        app.update();
        assert_eq!(app.world().resource::<ship_upload::SourceShipUpload>().new_layer_name(), "");
    }

    #[test]
    fn ui_sync_runtime_handles_editor_queries_and_changing_layer_counts() {
        let mut app = App::new();
        app.init_resource::<Simulation>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ShipUploadUiState>()
            .init_resource::<Assets<Image>>()
            .insert_resource(ShipCatalog(
                vec![ShipChoice {
                    name: "UI fixture".into(),
                    asset: "fixture.png".into(),
                    physics_asset: "fixture_base.png".into(),
                    material_map: true,
                    scale: 1.0,
                    source_key: None,
                }],
                vec![vec![
                    ShipLayerChoice {
                        name: ShipLayer::default(),
                        asset: "fixture.png".into(),
                    },
                    ShipLayerChoice {
                        name: ShipLayer::new("exterior"),
                        asset: "exterior.png".into(),
                    },
                ]],
            ))
            .add_systems(Update, (sync_ship_layer_dropdown, sync_ship_upload_ui))
            .add_systems(
                Update,
                sync_tool_panel_position.after(sync_ship_layer_dropdown),
            );
        app.world_mut()
            .resource_mut::<Simulation>()
            .layer_dropdown_open = true;
        let background = app
            .world_mut()
            .spawn((
                ShipLayerDropdownBackground,
                Sprite::default(),
                tool_panel_position(-145.0, 273.0, 39.0),
                Visibility::Hidden,
            ))
            .id();
        let save = app
            .world_mut()
            .spawn((ShipUploadSaveControl, Visibility::Visible))
            .id();
        let status = app
            .world_mut()
            .spawn((ShipUploadStatusText, Text2d::new("")))
            .id();
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .create_new();
        app.update();
        app.update();
        fn visible_options(app: &mut App) -> usize {
            let world = app.world_mut();
            let mut query = world.query_filtered::<&Visibility, With<ShipLayerDropdownOption>>();
            query
                .iter(world)
                .filter(|value| **value == Visibility::Visible)
                .count()
        }
        assert_eq!(visible_options(&mut app), 2);
        assert_eq!(
            app.world().get::<Visibility>(save),
            Some(&Visibility::Hidden)
        );
        assert_eq!(
            app.world().get::<Text2d>(status).unwrap().0,
            "Some required fields are missing"
        );
        {
            let mut upload = app
                .world_mut()
                .resource_mut::<ship_upload::SourceShipUpload>();
            upload.select_resource(
                ShipResourceType::Base,
                ShipResourceFile::new(
                    std::path::PathBuf::from("UI_fixture_base.png"),
                    ShipResource::new(
                        "UI fixture".into(),
                        ShipLayer::default(),
                        ShipResourceType::Base,
                    ),
                )
                .unwrap(),
                ShipLayer::default(),
            );
            upload.set_ship_name("\u{00a0}\u{202f}");
        }
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(save),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_ship_name("UI fixture");
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(save),
            Some(&Visibility::Visible)
        );
        assert_eq!(app.world().get::<Text2d>(status).unwrap().0, "");
        let exterior = ShipLayer::new("exterior");
        for kind in [ShipResourceType::Base, ShipResourceType::Materials] {
            assert_eq!(
                ship_upload_resource_layer(kind, &exterior),
                ShipLayer::default()
            );
        }
        for kind in [
            ShipResourceType::Texture,
            ShipResourceType::InLights,
            ShipResourceType::ExLights,
        ] {
            assert_eq!(ship_upload_resource_layer(kind, &exterior), exterior);
        }
        app.world_mut().resource_mut::<ShipCatalog>().1[0].push(ShipLayerChoice {
            name: ShipLayer::new("interior"),
            asset: "interior.png".into(),
        });
        app.update();
        app.update();
        assert_eq!(visible_options(&mut app), 3);
        app.world_mut().resource_mut::<ShipCatalog>().1[0].truncate(1);
        app.update();
        assert_eq!(visible_options(&mut app), 1);
        assert_eq!(
            app.world().get::<Sprite>(background).unwrap().custom_size,
            Some(Vec2::new(230.0, 24.0))
        );
        assert_eq!(
            app.world()
                .get::<Transform>(background)
                .unwrap()
                .translation
                .truncate(),
            Vec2::new(-505.0, -225.0)
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.update();
        let world = app.world_mut();
        let mut tabs = world.query_filtered::<&Visibility, With<ShipUploadLayerTab>>();
        assert!(
            tabs.iter(world)
                .all(|visibility| *visibility == Visibility::Hidden)
        );
    }

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
        let _palette=main_globals::fixture_materials();
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
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
        let _palette=main_globals::fixture_materials();
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
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
        let _palette=main_globals::fixture_materials();
        let choice = ShipChoice {
            name: "RMS Titanic".to_owned(),
            asset: "source_ships/Titanic.png".to_owned(),
            physics_asset: "source_ships/Titanic.png".to_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
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
        let _palette=main_globals::fixture_materials();
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
        let _palette=main_globals::fixture_materials();
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
        let _palette=main_globals::fixture_materials();
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
                    source_key: None,
                },
                ShipChoice {
                    name: "Queen Mary".to_owned(),
                    asset: "queen.png".to_owned(),
                    physics_asset: "queen.png".to_owned(),
                    material_map: false,
                    scale: 1.0,
                    source_key: None,
                },
            ],
            Vec::new(),
        );

        assert_eq!(filtered_ship_indices(&catalog, "QUEEN"), vec![1]);
        assert_eq!(filtered_ship_indices(&catalog, ""), vec![0, 1]);
    }

    #[test]
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
        let _palette=main_globals::fixture_materials();
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
        let _palette=main_globals::fixture_materials();
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


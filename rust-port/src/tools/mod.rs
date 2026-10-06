pub(super) mod break_tool;
pub(super) mod brush_preview;
pub(super) mod dry_tool;
pub(super) mod flood_tool;
pub(super) mod move_tool;
pub(super) mod tool;
mod water_brush;

use crate::tools::tool::Tool;
use crate::{GpuShipPhysicsAssets, GpuShipPhysicsSnapshot, ShaderBuffer};
use bevy::math::Vec2;
use bevy::prelude::Assets;

pub(super) fn apply_water_brush(
    tool: Tool,
    cursor: Vec2,
    radius: f32,
    snapshot: &GpuShipPhysicsSnapshot,
    physics: &mut GpuShipPhysicsAssets,
    buffers: &mut Assets<ShaderBuffer>,
) {
    match tool {
        Tool::Flood => flood_tool::apply(cursor, radius, snapshot, physics, buffers),
        Tool::Dry => dry_tool::apply(cursor, radius, snapshot, physics, buffers),
        _ => {}
    }
}

pub(super) mod move_tool_active_texture;
pub(super) mod move_tool_texture;

pub(super) mod break_tool_active_texture;
pub(super) mod break_tool_texture;
pub(super) mod brush_callbacks;
pub(super) mod dry_tool_active_texture;
pub(super) mod dry_tool_texture;
pub(super) mod flood_tool_active_texture;
pub(super) mod flood_tool_texture;

pub(super) mod break_tool_camera_reference;
pub(super) mod break_tool_free_camera_reference;
pub(super) mod brush_runtime;
pub(super) mod dry_tool_camera_reference;
pub(super) mod dry_tool_free_camera_reference;
pub(super) mod flood_tool_camera_reference;
pub(super) mod flood_tool_free_camera_reference;

pub(super) mod brush_construction;

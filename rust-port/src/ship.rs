//! Source `Ship.java` companion geometry and geometry-shader topology.
//! Bevy has no geometry shaders; emit the same triangles on the CPU using
//! the live mask channel Z, then upload them as a triangle-list mesh.

use crate::ship_data::ShipData;
use crate::ship_resources::{ShipLayer, ShipResourceType};
use crate::ship_thumbnail::{ShipThumbnail, ThumbnailResource};
use bevy::prelude::{Mat4, Vec2, Vec4};

/// Source Ship's event state. `moving` means physics is running, not dragging.
pub(crate) struct SourceShipEvents {
    pub(crate) moving: bool,
    pub(crate) inverse_matrix: Mat4,
    pub(crate) screen_size: [i32; 2],
    pub(crate) cursor: Vec4,
    pub(crate) base: Vec4,
    pub(crate) offset: Vec4,
}

impl SourceShipEvents {
    pub(crate) fn new(matrix: Mat4, screen_size: [i32; 2]) -> Self {
        Self {
            moving: false,
            inverse_matrix: matrix.inverse(),
            screen_size,
            cursor: Vec4::ZERO,
            base: Vec4::ZERO,
            offset: Vec4::ZERO,
        }
    }

    pub(crate) fn start_drag(&mut self) {
        self.base = -self.cursor;
        self.moving = false;
    }

    pub(crate) fn on_size(&mut self, blocked: bool, height: i32, width: i32) -> bool {
        self.screen_size = [width, height];
        blocked
    }

    pub(crate) fn on_cursor(
        &mut self,
        blocked: bool,
        x: f64,
        y: f64,
        display_offset: impl FnMut(Vec2),
        strut_offset: impl FnMut(Vec2),
    ) -> bool {
        let cursor = self.inverse_matrix
            * Vec4::new(
                x as f32 * 2.0 / self.screen_size[0] as f32 - 1.0,
                -(y as f32) * 2.0 / self.screen_size[1] as f32 + 1.0,
                0.0,
                1.0,
            );
        self.on_world_cursor(blocked, cursor, display_offset, strut_offset)
    }

    /// Bevy's camera adapter has already transformed screen to world space.
    pub(crate) fn on_world_cursor(
        &mut self,
        blocked: bool,
        cursor: Vec4,
        mut display_offset: impl FnMut(Vec2),
        mut strut_offset: impl FnMut(Vec2),
    ) -> bool {
        self.cursor = cursor;
        if !blocked && !self.moving {
            self.offset = self.cursor + self.base;
            let offset = self.offset.truncate().truncate();
            display_offset(offset);
            strut_offset(offset);
        }
        blocked
    }

    pub(crate) fn on_mouse_button(
        &mut self,
        blocked: bool,
        button: i32,
        action: i32,
        mut move_by: impl FnMut(Vec2),
        mut display_offset: impl FnMut(Vec2),
        mut strut_offset: impl FnMut(Vec2),
    ) -> bool {
        if !self.moving && !blocked && button == 0 && action == 0 {
            move_by(self.offset.truncate().truncate());
            display_offset(Vec2::ZERO);
            strut_offset(Vec2::ZERO);
            // The source keeps the offset field after committing it.
            self.moving = true;
        }
        blocked
    }

    pub(crate) fn update(&self, mut physics_update: impl FnMut()) {
        if self.moving {
            physics_update();
        }
    }
}

/// Retained Ship camera callback; explicit free matches the source resource hook.
pub(crate) struct SourceShipCamera {
    pub(crate) events: std::rc::Rc<std::cell::RefCell<SourceShipEvents>>,
    camera: std::rc::Rc<crate::camera_2d::SourceCamera2D>,
    pub(crate) callback: crate::camera_2d::SourceCameraCallback,
}

impl SourceShipCamera {
    pub(crate) fn new(
        camera: std::rc::Rc<crate::camera_2d::SourceCamera2D>,
        screen_size: [i32; 2],
        transform: std::rc::Rc<dyn Fn(Mat4)>,
    ) -> Self {
        let events = std::rc::Rc::new(std::cell::RefCell::new(SourceShipEvents::new(
            *camera.matrix.borrow(),
            screen_size,
        )));
        let retained = events.clone();
        let callback = crate::camera_2d::SourceCameraCallback {
            key: crate::camera_2d::CameraCallbackKey {
                receiver: std::rc::Rc::as_ptr(&events) as usize as u64,
                owner: "com.wicpar.sinkingsimulator.ship.Ship",
                name: "cameraCallback",
                signature: "(Camera2D)Unit",
            },
            invoke: std::rc::Rc::new(move |camera| {
                let matrix = *camera.matrix.borrow();
                retained.borrow_mut().inverse_matrix = matrix.inverse();
                transform(matrix);
            }),
        };
        camera.add_camera_callback(callback.clone());
        Self {
            events,
            camera,
            callback,
        }
    }

    pub(crate) fn free(&self) {
        self.camera.remove_camera_callback(&self.callback.key);
    }
}

/// Ship.java's retained layer array and unchecked signed currentLayer setter.
pub(crate) struct ShipLayers {
    pub(crate) thumbnail: ShipThumbnail,
    pub(crate) layers: Vec<ShipLayer>,
    pub(crate) layer_names: Vec<String>,
    pub(crate) current_layer: i32,
}

impl ShipLayers {
    pub(crate) const RENDERING_SAMPLERS: [&'static str; 6] =
        ["tex", "pos", "valueMask", "water", "inlights", "exlights"];

    pub(crate) fn new(thumbnail: ShipThumbnail) -> Self {
        let layers = thumbnail.ordered_layers().to_vec();
        let layer_names = layers
            .iter()
            .map(|layer| layer.display_name().to_owned())
            .collect();
        Self {
            thumbnail,
            layers,
            layer_names,
            current_layer: 0,
        }
    }

    pub(crate) fn selected_layer(&self) -> Result<&ShipLayer, String> {
        usize::try_from(self.current_layer)
            .ok()
            .and_then(|index| self.layers.get(index))
            .ok_or_else(|| format!("layer index {} is outside ship layers", self.current_layer))
    }

    pub(crate) fn display_resource(&self) -> Result<&ThumbnailResource, String> {
        self.thumbnail
            .get_resource(ShipResourceType::Texture, self.selected_layer()?)
            .ok_or_else(|| "selected ship layer has no texture".into())
    }

    /// Missing lights return the shared black texture in the render adapter;
    /// lookup never inherits the default layer's lights.
    pub(crate) fn light_resource(
        &self,
        external: bool,
    ) -> Result<Option<&ThumbnailResource>, String> {
        let kind = if external {
            ShipResourceType::ExLights
        } else {
            ShipResourceType::InLights
        };
        Ok(self.thumbnail.get_resource(kind, self.selected_layer()?))
    }
}

pub(crate) struct Ship;

impl Ship {
    /// Original point primitives: one cell anchor whenever any corner has a
    /// material. This preserves the source's row-major primitive order.
    pub(crate) fn create_indices(dat: &ShipData) -> Vec<u32> {
        crate::ship_companion::create_indices(dat)
            .unwrap_or_else(|error| panic!("{error}"))
            .into_iter()
            .map(|index| index as u32)
            .collect()
    }

    pub(crate) fn create_vertices(dat: &ShipData) -> Vec<[f32; 2]> {
        crate::ship_companion::create_vertices(dat)
            .unwrap_or_else(|error| panic!("{error}"))
            .chunks_exact(2)
            .map(|vertex| [vertex[0], vertex[1]])
            .collect()
    }

    /// Direct translation of ShipShader's full quad and make013/make132/
    /// make012/make032 predicates. Vertex IDs refer to the source texels,
    /// whose UVs sample their centers rather than an expanded pixel border.
    pub(crate) fn triangle_indices(width: usize, height: usize, masks: &[[u32; 4]]) -> Vec<u32> {
        assert_eq!(masks.len(), width * height);
        let mut indices = Vec::new();
        for y in 0..height.saturating_sub(1) {
            for x in 0..width.saturating_sub(1) {
                let tl = y * width + x;
                // Source offsets: 0=(0,1), 1=(0,0), 2=(1,1), 3=(1,0).
                let corners = [tl + width, tl, tl + width + 1, tl + 1];
                let s = corners.map(|i| masks[i][2]);
                let mut emit = |a: usize, b: usize, c: usize| {
                    indices.extend([corners[a] as u32, corners[b] as u32, corners[c] as u32]);
                };
                if s[1] & 7 == 7 && s[0] & 1 != 0 && s[3] & 12 == 12 {
                    // GLSL triangle strip [0,1,2,3], with alternating winding.
                    emit(0, 1, 2);
                    emit(2, 1, 3);
                } else {
                    if s[1] & 5 == 5 && s[3] & 8 != 0 {
                        emit(0, 1, 3);
                    }
                    if s[1] & 3 == 3 && s[3] & 4 != 0 {
                        emit(1, 3, 2);
                    }
                    if s[1] & 6 == 6 && s[0] & 1 != 0 {
                        emit(0, 1, 2);
                    }
                    if s[3] & 12 == 12 && s[0] & 1 != 0 {
                        emit(0, 3, 2);
                    }
                }
            }
        }
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ship_drag_refreshes_blocked_cursor_and_commits_in_source_order() {
        use std::cell::RefCell;
        let mut events = SourceShipEvents::new(Mat4::IDENTITY, [200, 100]);
        assert!(!events.moving);
        events.moving = true;
        assert!(events.on_cursor(
            true,
            100.0,
            50.0,
            |_| panic!("blocked"),
            |_| panic!("blocked")
        ));
        assert_eq!(events.cursor, Vec4::new(0.0, 0.0, 0.0, 1.0));
        events.offset = Vec4::new(9.0, 8.0, 0.0, 0.0);
        events.start_drag();
        assert_eq!(events.offset.x, 9.0);
        let calls = RefCell::new(Vec::new());
        events.on_cursor(
            false,
            150.0,
            25.0,
            |offset| calls.borrow_mut().push(("display", offset)),
            |offset| calls.borrow_mut().push(("struts", offset)),
        );
        assert_eq!(events.offset, Vec4::new(0.5, 0.5, 0.0, 0.0));
        assert_eq!(
            *calls.borrow(),
            [("display", Vec2::splat(0.5)), ("struts", Vec2::splat(0.5))]
        );
        events.on_mouse_button(
            true,
            0,
            0,
            |_| panic!("blocked"),
            |_| panic!("blocked"),
            |_| panic!("blocked"),
        );
        assert!(!events.moving);
        calls.borrow_mut().clear();
        events.on_mouse_button(
            false,
            0,
            0,
            |offset| calls.borrow_mut().push(("move", offset)),
            |offset| calls.borrow_mut().push(("display", offset)),
            |offset| calls.borrow_mut().push(("struts", offset)),
        );
        assert_eq!(
            *calls.borrow(),
            [
                ("move", Vec2::splat(0.5)),
                ("display", Vec2::ZERO),
                ("struts", Vec2::ZERO)
            ]
        );
        assert!(events.moving);
        assert_eq!(events.offset.x, 0.5);
        let mut updates = 0;
        events.update(|| updates += 1);
        events.start_drag();
        events.update(|| updates += 1);
        assert_eq!(updates, 1);
        assert!(!events.on_size(false, 0, -7));
        assert_eq!(events.screen_size, [-7, 0]);
    }

    #[test]
    fn source_ship_camera_updates_inverse_then_shader_and_free_detaches() {
        use std::{cell::RefCell, rc::Rc};
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(200, 100));
        let matrices = Rc::new(RefCell::new(Vec::new()));
        let output = matrices.clone();
        let binding = SourceShipCamera::new(
            camera.clone(),
            [200, 100],
            Rc::new(move |matrix| output.borrow_mut().push(matrix)),
        );
        assert_eq!(matrices.borrow().len(), 1);
        camera.translate(16.0, 0.0);
        assert_eq!(matrices.borrow().len(), 2);
        assert_eq!(
            binding.events.borrow().inverse_matrix,
            camera.matrix.borrow().inverse()
        );
        binding.free();
        camera.translate(16.0, 0.0);
        assert_eq!(matrices.borrow().len(), 2);
        assert_ne!(
            binding.events.borrow().inverse_matrix,
            camera.matrix.borrow().inverse()
        );
    }

    #[test]
    fn source_layer_order_selection_missing_lights_and_resource_identity() {
        use crate::ship_resources::parse_resource_path;
        let exterior =
            parse_resource_path(std::path::Path::new("Test_exterior_texture.png")).unwrap();
        let base = parse_resource_path(std::path::Path::new("Test_base.png")).unwrap();
        let lights = parse_resource_path(std::path::Path::new("Test_inlights.png")).unwrap();
        let thumbnail =
            ShipThumbnail::new([exterior.clone(), base.clone(), lights.clone()]).unwrap();
        let equal_reordered = ShipThumbnail::new([base, lights, exterior]).unwrap();
        assert_eq!(thumbnail, equal_reordered); // Java Map.equals ignores iteration order.
        let mut ship = ShipLayers::new(thumbnail);
        assert_eq!(ship.layer_names, ["exterior", "Default"]);
        assert!(ship.display_resource().is_ok());
        assert!(ship.light_resource(false).unwrap().is_none());
        ship.current_layer = 1;
        assert!(ship.light_resource(false).unwrap().is_some());
        assert!(ship.light_resource(true).unwrap().is_none());
        ship.current_layer = -1;
        assert!(ship.display_resource().is_err());
        ship.current_layer = 2;
        assert!(ship.light_resource(false).is_err());
        assert_eq!(ShipLayers::RENDERING_SAMPLERS[5], "exlights");
    }

    #[test]
    fn source_geometry_full_quad_matches_triangle_strip() {
        let masks = [
            [8, 7, 7, 0],
            [8, 28, 28, 0],
            [8, 193, 193, 0],
            [8, 112, 112, 0],
        ];
        assert_eq!(Ship::triangle_indices(2, 2, &masks), vec![2, 0, 3, 3, 0, 1]);
    }

    #[test]
    fn source_geometry_broken_diagonal_changes_surface_and_empty_links_remove_it() {
        // Only source make013 is permitted: top-left east/south and
        // top-right southwest remain connected.
        let masks = [[8, 5, 5, 0], [8, 8, 8, 0], [8, 0, 0, 0], [8, 0, 0, 0]];
        assert_eq!(Ship::triangle_indices(2, 2, &masks), vec![2, 0, 1]);
        let disconnected = [[8, 0, 0, 0]; 4];
        assert!(Ship::triangle_indices(2, 2, &disconnected).is_empty());
    }
}

//! Editor Test Ship action and retained active-thumbnail adapter. No Steam.
use crate::*;
use std::rc::Rc;
pub(crate) trait PreviewOperations {
    type Ship;
    type Control;
    fn global_ship(&mut self) -> Rc<Self::Ship>;
    fn close(&mut self, ship: &Self::Ship);
    fn camera_control(&mut self, ship: &Self::Ship) -> Rc<Self::Control>;
    fn to_ship(
        &mut self,
        thumbnail: Rc<ship_thumbnail::ShipThumbnail>,
        control: Rc<Self::Control>,
    ) -> Rc<Self::Ship>;
    fn set_global_ship(&mut self, ship: Rc<Self::Ship>);
    fn clear_ship_list(&mut self);
    fn add_ship(&mut self, ship: Rc<Self::Ship>);
}
pub(crate) fn activate_preview(
    preview: Rc<ship_thumbnail::ShipThumbnail>,
    operations: &mut impl PreviewOperations,
) {
    let old = operations.global_ship();
    operations.close(&old);
    let current = operations.global_ship();
    let control = operations.camera_control(&current);
    let next = operations.to_ship(preview, control);
    operations.set_global_ship(next);
    operations.clear_ship_list();
    let current = operations.global_ship();
    operations.add_ship(current);
}
impl PreviewOperations for gui_reset_ship::SourceResetShipOperations {
    type Ship = std::cell::RefCell<source_ship::SourceShip>;
    type Control = std::cell::RefCell<camera_control::SourceCameraControl>;
    fn global_ship(&mut self) -> Rc<Self::Ship> {
        main_globals::get_global_ship()
    }
    fn close(&mut self, ship: &Self::Ship) {
        ship.borrow().close();
    }
    fn camera_control(&mut self, ship: &Self::Ship) -> Rc<Self::Control> {
        ship.borrow().camera_control.clone()
    }
    fn to_ship(
        &mut self,
        thumbnail: Rc<ship_thumbnail::ShipThumbnail>,
        control: Rc<Self::Control>,
    ) -> Rc<Self::Ship> {
        (self.constructor)(thumbnail, control).unwrap_or_else(|error| panic!("{error}"))
    }
    fn set_global_ship(&mut self, ship: Rc<Self::Ship>) {
        main_globals::set_global_ship(ship);
    }
    fn clear_ship_list(&mut self) {
        main_globals::get_ship_list().borrow_mut().clear();
    }
    fn add_ship(&mut self, ship: Rc<Self::Ship>) {
        main_globals::get_ship_list().borrow_mut().push(ship);
    }
}
#[derive(Resource, Default)]
pub(crate) struct ActivePreview {
    pub thumbnail: Option<ship_thumbnail::ShipThumbnail>,
    pub layers: Vec<ShipLayer>,
    pub revision: u64,
    pub pending: Option<ShipStructure>,
}
impl ActivePreview {
    pub(crate) fn clear(&mut self) {
        self.thumbnail = None;
        self.layers.clear();
        self.pending = None;
        self.revision = self.revision.wrapping_add(1);
    }
    fn publish(&mut self, thumbnail: ship_thumbnail::ShipThumbnail) -> Result<(), String> {
        let structure = ShipStructure::load_for_thumbnail(&thumbnail)?;
        self.layers = thumbnail.ordered_layers().to_vec();
        self.thumbnail = Some(thumbnail);
        self.pending = Some(structure);
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
}
pub(crate) fn layer<'a>(
    catalog: &'a ShipCatalog,
    simulation: &Simulation,
    preview: Option<&'a ActivePreview>,
    retained:Option<&'a ship_runtime_reset::ActiveThumbnail>,
    index: usize,
) -> Option<&'a ShipLayer> {
    if let Some(preview) = preview.filter(|preview| preview.thumbnail.is_some()) {
        preview.layers.get(index)
    } else if let Some(thumbnail)=retained.and_then(|retained|retained.0.as_ref()) {
        thumbnail.ordered_layers().get(index)
    } else {
        catalog
            .1
            .get(simulation.ship_index)
            .and_then(|layers| layers.get(index))
            .map(|layer| &layer.name)
    }
}
pub(crate) fn layer_count(
    catalog: &ShipCatalog,
    simulation: &Simulation,
    preview: Option<&ActivePreview>,
    retained:Option<&ship_runtime_reset::ActiveThumbnail>,
) -> usize {
    preview
        .filter(|preview| preview.thumbnail.is_some())
        .map_or_else(
            || {
                if let Some(thumbnail)=retained.and_then(|retained|retained.0.as_ref()) {return thumbnail.ordered_layers().len().max(1);}
                catalog
                    .1
                    .get(simulation.ship_index)
                    .map_or(1, |layers| layers.len().max(1))
            },
            |preview| preview.layers.len().max(1),
        )
}
pub(crate) fn texture(
    thumbnail: &ship_thumbnail::ShipThumbnail,
    kind: ShipResourceType,
    layer: &ShipLayer,
    images: &mut Assets<Image>,
) -> Result<Option<Handle<Image>>, String> {
    match thumbnail.get_resource(kind, layer) {
        Some(ship_thumbnail::ThumbnailResource::File(file)) => file.texture_now(images).map(Some),
        Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(resource)) => {
            resource.texture_now_current(images).map(Some)
        }
        None => Ok(None),
    }
}
pub(crate) fn click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    upload: Res<ship_upload::SourceShipUpload>,
    layout: Res<ship_upload_layout::Layout>,
    mut preview: ResMut<ActivePreview>,
    mut simulation: ResMut<Simulation>,
    mut ui: ResMut<ShipUploadUiState>,
) {
    if !mouse.just_pressed(MouseButton::Left)
        || !upload.window_open()
        || upload.ship_name_is_blank()
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let point = Vec2::new(
        cursor.x - window.width() * 0.5,
        window.height() * 0.5 - cursor.y,
    ) * (720.0 / window.height().max(1.0));
    let Some(point) = ship_upload_layout::content_point(&layout, point) else {
        return;
    };
    if !layout
        .test_bounds()
        .is_some_and(|bounds| bounds.contains(point))
    {
        return;
    }
    match upload.thumbnail() {
        Ok(Some(thumbnail)) => match preview.publish(thumbnail) {
            Ok(()) => {
                simulation.selected_layer = 0;
                simulation.layer_dropdown_open = false;
            }
            Err(error) => ui.notice = Some(error),
        },
        Ok(None) => {}
        Err(error) => ui.notice = Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_preview_reloads_global_after_close_and_reuses_that_camera_control() {
        struct Ship {
            id: i32,
            control: Rc<i32>,
        }
        struct Operations {
            current: Rc<Ship>,
            list: Vec<Rc<Ship>>,
            log: Vec<String>,
            expected: Rc<ship_thumbnail::ShipThumbnail>,
        }
        impl PreviewOperations for Operations {
            type Ship = Ship;
            type Control = i32;
            fn global_ship(&mut self) -> Rc<Ship> {
                self.log.push(format!("global:{}", self.current.id));
                self.current.clone()
            }
            fn close(&mut self, ship: &Ship) {
                self.log.push(format!("close:{}", ship.id));
                self.current = Rc::new(Ship {
                    id: 2,
                    control: Rc::new(22),
                });
            }
            fn camera_control(&mut self, ship: &Ship) -> Rc<i32> {
                self.log.push(format!("control:{}", ship.id));
                ship.control.clone()
            }
            fn to_ship(
                &mut self,
                thumbnail: Rc<ship_thumbnail::ShipThumbnail>,
                control: Rc<i32>,
            ) -> Rc<Ship> {
                assert!(Rc::ptr_eq(&thumbnail, &self.expected));
                assert!(Rc::ptr_eq(&control, &self.current.control));
                self.log.push("toShip".into());
                Rc::new(Ship { id: 3, control })
            }
            fn set_global_ship(&mut self, ship: Rc<Ship>) {
                self.log.push("set".into());
                self.current = ship;
            }
            fn clear_ship_list(&mut self) {
                self.log.push("clear".into());
                self.list.clear();
            }
            fn add_ship(&mut self, ship: Rc<Ship>) {
                self.log.push(format!("add:{}", ship.id));
                self.list.push(ship);
            }
        }
        let thumbnail = Rc::new(
            ship_thumbnail::ShipThumbnail::new([parse_resource_path(std::path::Path::new(
                "Fixture_base.png",
            ))
            .unwrap()])
            .unwrap(),
        );
        let old = Rc::new(Ship {
            id: 1,
            control: Rc::new(11),
        });
        let mut operations = Operations {
            current: old.clone(),
            list: vec![old],
            log: vec![],
            expected: thumbnail.clone(),
        };
        activate_preview(thumbnail, &mut operations);
        assert_eq!(
            operations.log,
            [
                "global:1",
                "close:1",
                "global:2",
                "control:2",
                "toShip",
                "set",
                "clear",
                "global:3",
                "add:3"
            ]
        );
        assert_eq!(operations.list.len(), 1);
        assert!(Rc::ptr_eq(&operations.list[0], &operations.current));
    }
    #[test]
    fn active_editor_test_click_retains_unsaved_resources_palette_layers_and_camera() {
        let _palette=main_globals::fixture_materials();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder =
            std::env::temp_dir().join(format!("ss2-test-ship-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        let base = folder.join("Fixture_base.png");
        let exterior = folder.join("Fixture_exterior_texture.png");
        let palette = folder.join("Fixture_materials.json");
        image::RgbaImage::from_pixel(20, 20, image::Rgba([0x12, 0x34, 0x56, 255]))
            .save(&base)
            .unwrap();
        image::RgbaImage::from_pixel(20, 20, image::Rgba([10, 20, 30, 255]))
            .save(&exterior)
            .unwrap();
        std::fs::write(&palette, r##"[{"color":"#123456","mass":17}]"##).unwrap();
        let mut upload = ship_upload::SourceShipUpload::default();
        upload.create_new();
        upload.set_ship_name("Fixture");
        upload
            .select_file(base.clone(), ShipResourceType::Base, ShipLayer::default())
            .unwrap();
        upload
            .select_file(
                palette.clone(),
                ShipResourceType::Materials,
                ShipLayer::default(),
            )
            .unwrap();
        upload.set_new_layer_name("exterior");
        assert!(upload.add_layer());
        upload
            .select_file(
                exterior.clone(),
                ShipResourceType::Texture,
                ShipLayer::new("exterior"),
            )
            .unwrap();
        let initial = ShipStructure::empty_fallback();
        let old_mesh = build_deformable_ship_mesh(&initial);
        let mut buffers = Assets::<ShaderBuffer>::default();
        let gpu = make_gpu_ship_physics_assets(&initial, &mut buffers);
        let choice = ShipChoice {
            name: "Catalog ship".into(),
            asset: "unused.png".into(),
            physics_asset: "unused.png".into(),
            material_map: true,
            scale: 1.0,
            source_key: None,
        };
        let mut app = App::new();
        app.insert_resource(upload)
            .insert_resource(initial)
            .insert_resource(buffers)
            .insert_resource(gpu)
            .insert_resource(ShipCatalog(
                vec![choice],
                vec![vec![ShipLayerChoice {
                    name: ShipLayer::default(),
                    asset: "unused.png".into(),
                }]],
            ))
            .init_resource::<GpuShipPhysicsSnapshot>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<ShipUploadUiState>()
            .init_resource::<ship_upload_layout::Layout>()
            .init_resource::<ActivePreview>()
            .init_resource::<Simulation>()
            .init_resource::<tools::move_tool::MoveDragState>()
            .init_resource::<Assets<Image>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<window_characters::ResetShip>()
            .add_systems(Startup, ship_upload_layout::spawn)
            .add_systems(
                Update,
                (ship_upload_layout::sync, click, load_selected_ship).chain(),
            );
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        let camera = app
            .world_mut()
            .spawn((WorldCamera, Transform::from_xyz(123.0, 45.0, 1000.0)))
            .id();
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(old_mesh);
        let ship = app
            .world_mut()
            .spawn((
                ShipMesh(mesh.clone(), Handle::default()),
                Mesh2d(mesh.clone()),
                MeshSyncState(vec![true], vec![[1, 2, 3, 4]]),
            ))
            .id();
        app.update();
        {
            let mut drag = app.world_mut().resource_mut::<tools::move_tool::MoveDragState>();
            drag.release(false);
            drag.offset = Vec2::new(17.0, 19.0);
            drag.pending_translation = Vec2::new(31.0, 37.0);
        }
        {
            let mut physics = app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
            physics.water_brush = Some([1.0; 4]);
            physics.break_brush = Some([2.0; 4]);
        }
        app.world_mut()
            .resource_mut::<GpuShipPhysicsSnapshot>()
            .positions
            .push(Vec4::ONE);
        let bounds = app
            .world()
            .resource::<ship_upload_layout::Layout>()
            .test_bounds()
            .unwrap();
        // Test activation uses the editor's moved screen position, while the scene camera stays put.
        let editor_position = Vec2::new(123.0, -17.0);
        app.world_mut()
            .resource_mut::<ship_upload_layout::Layout>()
            .position = editor_position;
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(
                640.0 + bounds.center().x + editor_position.x,
                360.0 - bounds.center().y - editor_position.y,
            )));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        let active = app.world().resource::<ActivePreview>();
        assert_eq!(active.revision, 1);
        assert_eq!(active.thumbnail.as_ref().unwrap().name(), Some("Fixture"));
        assert!(active.layers.contains(&ShipLayer::new("exterior")));
        assert!(active.pending.is_none());
        let pending = app.world().resource::<ShipStructure>();
        assert_eq!((pending.texel_width, pending.texel_height), (20, 20));
        let gpu = app.world().resource::<GpuShipPhysicsAssets>();
        assert_eq!((gpu.width, gpu.height), (20, 20));
        assert_eq!(gpu.iterations, 0);
        assert!(gpu.water_brush.is_none());
        assert!(gpu.break_brush.is_none());
        let drag = app.world().resource::<tools::move_tool::MoveDragState>();
        assert!(drag.dragging);
        assert_eq!(drag.offset, Vec2::ZERO);
        assert_eq!(drag.pending_translation, Vec2::ZERO);
        assert!(
            app.world()
                .resource::<GpuShipPhysicsSnapshot>()
                .positions
                .is_empty()
        );
        let current=&app.world().get::<ShipMesh>(ship).unwrap().0;
        assert_ne!(*current,mesh,"Test Ship constructs fresh model geometry");
        assert_eq!(*current,app.world().get::<Mesh2d>(ship).unwrap().0);
        assert!(app.world().resource::<Assets<Mesh>>().get(&mesh).is_some(),"External old model handle remains retained");
        assert!(app.world().get::<MeshSyncState>(ship).unwrap().1.is_empty());
        assert!(
            app.world()
                .get::<MeshSyncState>(ship)
                .unwrap()
                .0
                .iter()
                .all(|breached| !*breached)
        );
        assert!(
            pending
                .texel_materials
                .iter()
                .all(|material| material.is_some_and(|material| material.mass == 17.0))
        );
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(123.0, 45.0, 1000.0)
        );
        assert!(
            app.world()
                .resource::<ship_upload::SourceShipUpload>()
                .window_open()
        );
        app.world_mut()
            .resource_mut::<ship_upload::SourceShipUpload>()
            .set_window_open(false);
        app.update();
        assert!(app.world().resource::<ActivePreview>().thumbnail.is_some());
        app.world_mut().resource_mut::<Simulation>().ship_index = 1;
        app.world_mut().resource_mut::<tools::move_tool::MoveDragState>().release(false);
        app.update();
        assert_eq!(app.world().resource::<ShipStructure>().texel_width, 20);
        assert!(!app.world().resource::<tools::move_tool::MoveDragState>().dragging);
        // Reset uses the retained unsaved thumbnail even when catalog indices move.
        app.world_mut().resource_mut::<Simulation>().selected_layer=1;
        let generation=app.world().resource::<GpuShipPhysicsAssets>().generation;
        app.world_mut().resource_mut::<GpuShipPhysicsAssets>().water_brush=Some([1.0;4]);
        app.world_mut().resource_mut::<GpuShipPhysicsAssets>().break_brush=Some([2.0;4]);
        app.world_mut().resource_mut::<ShipStructure>().motion_position=Vec2::new(99.0,77.0);
        app.world_mut().resource_mut::<ShipStructure>().breached.fill(true);
        std::fs::write(&base,b"changed after original thumbnail load").unwrap();
        app.world_mut().write_message(window_characters::ResetShip);app.world_mut().write_message(window_characters::ResetShip);
        app.update();
        let physics=app.world().resource::<GpuShipPhysicsAssets>();assert_eq!(physics.generation,generation+2);
        assert!(physics.water_brush.is_none() && physics.break_brush.is_none());
        assert_eq!(app.world().resource::<Simulation>().selected_layer,0,"Replacement Ship starts on its first layer");
        assert_eq!(app.world().resource::<Simulation>().ship_index,1);
        assert_eq!(app.world().resource::<ActivePreview>().revision,1);
        let structure=app.world().resource::<ShipStructure>();assert_eq!(structure.texel_width,20);
        assert_eq!(structure.motion_position,Vec2::new(0.0,SEA_LEVEL));assert!(structure.breached.iter().all(|v|!*v));
        assert!(structure.texel_materials.iter().all(|m|m.is_some_and(|m|m.mass==17.0)));
        assert!(app.world().resource::<tools::move_tool::MoveDragState>().dragging);
        assert_eq!(app.world().get::<Transform>(camera).unwrap().translation,Vec3::new(123.0,45.0,1000.0));
        app.world_mut().resource_mut::<ActivePreview>().clear();
        assert!(app.world().resource::<ActivePreview>().thumbnail.is_none());
        for path in [base, exterior, palette] {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
}

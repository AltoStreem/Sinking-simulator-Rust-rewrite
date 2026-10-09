//! Retain the active catalog thumbnail's lazy resources for source Shift+R.
use crate::*;
#[derive(Resource, Default)]
pub(crate) struct ActiveThumbnail(pub Option<ship_thumbnail::ShipThumbnail>);
pub(crate) fn choice(choice: &ShipChoice) -> (ShipStructure, ActiveThumbnail) {
    if choice.material_map {
        let path = std::path::Path::new("assets").join(&choice.physics_asset);
        if let Ok(thumbnail) = ship_thumbnail::ShipThumbnail::from_base_file(&path) {
            if let Ok(structure) = ShipStructure::load_for_thumbnail(&thumbnail) {
                return (structure, ActiveThumbnail(Some(thumbnail)));
            }
        }
    }
    (
        ShipStructure::load_for_choice(choice),
        ActiveThumbnail(None),
    )
}

pub(crate) struct Appearance {
    pub texture: Handle<Image>,
    pub internal: Handle<Image>,
    pub external: Handle<Image>,
    pub visible: bool,
}
/// Both startup and later layer changes use the active source thumbnail cache.
pub(crate) fn appearance(
    thumbnail: &ship_thumbnail::ShipThumbnail,
    index: usize,
    images: &mut Assets<Image>,
    black: &Handle<Image>,
) -> Appearance {
    let layer = thumbnail.ordered_layers().get(index);
    let mut texture = |kind| {
        layer.and_then(|layer| {
            ship_upload_preview::texture(thumbnail, kind, layer, images).unwrap_or_else(|error| {
                bevy::log::warn!("Could not load active ship resource: {error}");
                None
            })
        })
    };
    let image = texture(ShipResourceType::Texture);
    let visible = image.is_some();
    Appearance {
        texture: image.unwrap_or_else(|| black.clone()),
        internal: texture(ShipResourceType::InLights).unwrap_or_else(|| black.clone()),
        external: texture(ShipResourceType::ExLights).unwrap_or_else(|| black.clone()),
        visible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_reset_uses_retained_image_and_restores_all_ship_buffers_without_camera_reset() {
        let _palette = main_globals::fixture_materials();
        let folder = std::env::temp_dir().join(format!(
            "ss2_retained_reset_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let base = folder.join("cached_base.png");
        image::RgbaImage::from_pixel(2, 2, image::Rgba([0x5d, 0x5d, 0x60, 255]))
            .save(&base)
            .unwrap();
        let choice = ShipChoice {
            name: "cached".into(),
            asset: base.to_string_lossy().into_owned(),
            physics_asset: base.to_string_lossy().into_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
        };
        let (structure, retained) = super::choice(&choice);
        assert!(retained.0.is_some());
        assert_eq!(structure.texel_width, 2);
        let rest = structure.texel_rest_positions.clone();
        let mut buffers = Assets::<ShaderBuffer>::default();
        let physics = make_gpu_ship_physics_assets(&structure, &mut buffers);
        let handles = [
            physics.positions.clone(),
            physics.materials.clone(),
            physics.masks.clone(),
            physics.forces.clone(),
            physics.water.clone(),
            physics.water_outflow_1.clone(),
            physics.water_outflow_2.clone(),
            physics.water_velocity_1.clone(),
            physics.water_velocity_2.clone(),
        ];
        let expected: Vec<_> = handles
            .iter()
            .map(|h| buffers.get(h).unwrap().data.clone())
            .collect();
        let mut app = App::new();
        app.insert_resource(structure)
            .insert_resource(retained)
            .insert_resource(buffers)
            .insert_resource(physics)
            .insert_resource(ShipCatalog(vec![choice], Vec::new()))
            .init_resource::<Simulation>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<GpuShipPhysicsSnapshot>()
            .init_resource::<ship_upload_preview::ActivePreview>()
            .init_resource::<tools::move_tool::MoveDragState>()
            .add_message::<window_characters::ResetShip>()
            .add_systems(Update, load_selected_ship);
        let camera = app
            .world_mut()
            .spawn((
                WorldCamera,
                Transform::from_xyz(99.0, 77.0, 55.0),
                Projection::Orthographic(OrthographicProjection {
                    scale: 3.25,
                    ..OrthographicProjection::default_2d()
                }),
            ))
            .id();
        app.update();
        {
            let mut structure = app.world_mut().resource_mut::<ShipStructure>();
            structure.motion_position = Vec2::splat(99.0);
            structure.breached.fill(true);
            structure.texel_positions.fill(Vec2::splat(44.0));
        }
        {
            let mut drag = app
                .world_mut()
                .resource_mut::<tools::move_tool::MoveDragState>();
            drag.release(false);
            drag.offset = Vec2::splat(88.0);
            drag.pending_translation = Vec2::splat(77.0);
        }
        {
            let mut gpu = app.world_mut().resource_mut::<GpuShipPhysicsAssets>();
            gpu.water_brush = Some([1.0; 4]);
            gpu.break_brush = Some([2.0; 4]);
        }
        {
            let mut snapshot = app.world_mut().resource_mut::<GpuShipPhysicsSnapshot>();
            snapshot.positions.push(Vec4::ONE);
            snapshot.water.push(Vec4::ONE);
            snapshot.masks.push([1; 4]);
        }
        for handle in &handles {
            *app.world_mut()
                .resource_mut::<Assets<ShaderBuffer>>()
                .get_mut(handle)
                .unwrap() = ShaderBuffer::from(vec![[99.0f32; 4]]);
        }
        std::fs::write(
            &base,
            b"disk changed after source resource cache was populated",
        )
        .unwrap();
        app.world_mut().write_message(window_characters::ResetShip);
        app.update();
        let structure = app.world().resource::<ShipStructure>();
        assert_eq!(structure.texel_positions, rest);
        assert_eq!(structure.motion_position, Vec2::new(0.0, SEA_LEVEL));
        assert!(structure.breached.iter().all(|v| !*v));
        let gpu = app.world().resource::<GpuShipPhysicsAssets>();
        assert_eq!(gpu.generation, 1);
        assert!(gpu.water_brush.is_none() && gpu.break_brush.is_none());
        let buffers = app.world().resource::<Assets<ShaderBuffer>>();
        let fresh=[gpu.positions.clone(),gpu.materials.clone(),gpu.masks.clone(),gpu.forces.clone(),
            gpu.water.clone(),gpu.water_outflow_1.clone(),gpu.water_outflow_2.clone(),gpu.water_velocity_1.clone(),gpu.water_velocity_2.clone()];
        for (handle, expected) in fresh.iter().zip(expected) {
            assert!(!handles.contains(handle),"Reset must construct fresh source physics state");
            assert_eq!(buffers.get(handle).unwrap().data, expected);
        }
        for handle in &handles {
            assert_eq!(buffers.get(handle).unwrap().data,ShaderBuffer::from(vec![[99.0f32;4]]).data,
                "Retained old physics must not be overwritten by the new Ship");
        }
        let snapshot = app.world().resource::<GpuShipPhysicsSnapshot>();
        assert!(
            snapshot.positions.is_empty() && snapshot.water.is_empty() && snapshot.masks.is_empty()
        );
        let drag = app.world().resource::<tools::move_tool::MoveDragState>();
        assert!(drag.dragging);
        assert_eq!(drag.offset, Vec2::ZERO);
        assert_eq!(drag.pending_translation, Vec2::ZERO);
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(99.0, 77.0, 55.0)
        );
        let Projection::Orthographic(projection) = app.world().get::<Projection>(camera).unwrap()
        else {
            panic!()
        };
        assert_eq!(projection.scale, 3.25);
        // Source Main.globalMaterials is retained until its property is replaced;
        // a new Ship must then use the new reference, never reload the disk file.
        main_globals::set_global_materials(std::sync::Arc::new(
            materials::SourceMaterials::from_json(r##"[{"color":"#5D5D60","mass":13}]"##).unwrap(),
        ));
        assert!(
            app.world()
                .resource::<ShipStructure>()
                .texel_materials
                .iter()
                .all(|material| material.is_some_and(|material| material.mass == 2409.0))
        );
        app.world_mut().resource_mut::<Simulation>().selected_layer = 7;
        app.world_mut().write_message(window_characters::ResetShip);
        app.update();
        assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().generation, 2);
        assert_eq!(app.world().resource::<Simulation>().selected_layer, 0);
        assert!(
            app.world()
                .resource::<ShipStructure>()
                .texel_materials
                .iter()
                .all(|material| material.is_some_and(|material| material.mass == 13.0))
        );
        std::fs::remove_file(base).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
    fn folder(label: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "ss2_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&folder).unwrap();
        folder
    }
    #[test]
    fn live_palette_replacement_rebuilds_physics_but_retains_generated_texture_identity() {
        let _serial = main_globals::fixture_materials();
        let folder = folder("live_palette");
        let base = folder.join("Palette_base.png");
        image::RgbaImage::from_pixel(2, 1, image::Rgba([0x12, 0x34, 0x56, 161]))
            .save(&base)
            .unwrap();
        let palette = |mass, invisible| {
            std::sync::Arc::new(
                materials::SourceMaterials::from_json(&format!(
                    r##"[{{"color":"#123456","mass":{mass},"invisible":{invisible}}}]"##
                ))
                .unwrap(),
            )
        };
        main_globals::set_global_materials(palette(17, true));
        let choice = ShipChoice {
            name: "Palette".into(),
            asset: base.to_string_lossy().into_owned(),
            physics_asset: base.to_string_lossy().into_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
        };
        let (original, retained) = super::choice(&choice);
        assert!(
            original
                .texel_materials
                .iter()
                .all(|m| m.is_some_and(|m| m.mass == 17.0)),
            "Invisible materials remain physical"
        );
        let thumbnail = retained.0.as_ref().unwrap();
        let mut images = Assets::<Image>::default();
        let hidden = ship_upload_preview::texture(
            thumbnail,
            ShipResourceType::Texture,
            &ShipLayer::default(),
            &mut images,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            images.get(&hidden).unwrap().data.as_deref().unwrap(),
            &[0; 12]
        );
        main_globals::set_global_materials(palette(29, false));
        std::fs::write(&base, b"disk file is no longer a PNG").unwrap();
        let next = ShipStructure::load_for_thumbnail(thumbnail).unwrap();
        assert!(
            next.texel_materials
                .iter()
                .all(|m| m.is_some_and(|m| m.mass == 29.0))
        );
        assert!(
            original
                .texel_materials
                .iter()
                .all(|m| m.is_some_and(|m| m.mass == 17.0))
        );
        let cached = ship_upload_preview::texture(
            thumbnail,
            ShipResourceType::Texture,
            &ShipLayer::default(),
            &mut images,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            cached, hidden,
            "New Ship retains shared thumbnail texture identity"
        );
        assert_eq!(
            images.get(&cached).unwrap().data.as_deref().unwrap(),
            &[0; 12],
            "Cached generated pixels are not silently regenerated after a global palette change"
        );
        let Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(derived)) =
            thumbnail.get_resource(ShipResourceType::Texture, &ShipLayer::default())
        else {
            panic!()
        };
        derived.free_resources();
        let visible = ship_upload_preview::texture(
            thumbnail,
            ShipResourceType::Texture,
            &ShipLayer::default(),
            &mut images,
        )
        .unwrap()
        .unwrap();
        assert_ne!(visible, hidden);
        assert_eq!(
            images.get(&visible).unwrap().data.as_deref().unwrap(),
            &[
                0x12, 0x34, 0x56, 161, 0x12, 0x34, 0x56, 161, 0x12, 0x34, 0x56, 161
            ]
        );
        std::fs::remove_file(base).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn live_appearance_uses_retained_catalog_and_unsaved_preview_after_catalog_entry_disappears() {
        let _serial = main_globals::fixture_materials();
        main_globals::set_global_materials(std::sync::Arc::new(
            materials::SourceMaterials::from_json(r##"[{"color":"#123456","mass":7}]"##).unwrap(),
        ));
        let folder = folder("live_appearance");
        let base = folder.join("Appearance_base.png");
        let exterior = folder.join("Appearance_exterior_texture.png");
        let lights = folder.join("Appearance_exterior_inlights.png");
        image::RgbaImage::from_pixel(2, 1, image::Rgba([0x12, 0x34, 0x56, 161]))
            .save(&base)
            .unwrap();
        image::RgbaImage::from_pixel(2, 1, image::Rgba([10, 20, 30, 255]))
            .save(&exterior)
            .unwrap();
        image::RgbaImage::from_pixel(2, 1, image::Rgba([40, 50, 60, 255]))
            .save(&lights)
            .unwrap();
        let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(&base).unwrap();
        thumbnail.base_layer().unwrap();
        let index = thumbnail
            .ordered_layers()
            .iter()
            .position(|layer| layer.display_name() == "exterior")
            .unwrap();
        let mut images = Assets::<Image>::default();
        let black = images.add(texture_2d::ship_texture(image::RgbaImage::new(1, 1)));
        let expected = super::appearance(&thumbnail, index, &mut images, &black);
        assert_eq!(
            images
                .get(&expected.texture)
                .unwrap()
                .data
                .as_deref()
                .unwrap(),
            &[10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255]
        );
        assert_eq!(
            images
                .get(&expected.internal)
                .unwrap()
                .data
                .as_deref()
                .unwrap(),
            &[40, 50, 60, 255, 40, 50, 60, 255, 40, 50, 60, 255]
        );
        assert_eq!(expected.external, black);
        assert_eq!(
            images
                .get(&expected.texture)
                .unwrap()
                .texture_descriptor
                .mip_level_count,
            2
        );
        assert_eq!(
            images
                .get(&expected.texture)
                .unwrap()
                .texture_descriptor
                .size
                .width,
            2
        );
        std::fs::write(&exterior, b"changed cached exterior").unwrap();
        std::fs::write(&lights, b"changed cached lights").unwrap();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .insert_resource(images)
            .insert_resource(ship_black_texture::SharedBlackTexture(black.clone()))
            .insert_resource(ShipCatalog(Vec::new(), Vec::new()))
            .insert_resource(ActiveThumbnail(Some(thumbnail.clone())))
            .insert_resource(ship_upload_preview::ActivePreview {
                thumbnail: Some(thumbnail.clone()),
                layers: thumbnail.ordered_layers().to_vec(),
                revision: 1,
                pending: None,
            })
            .init_resource::<Simulation>()
            .init_resource::<Assets<ShipMaterial>>()
            .init_resource::<Assets<ReflectionMaterial>>()
            .add_systems(Update, sync_ship_assets);
        {
            let mut simulation = app.world_mut().resource_mut::<Simulation>();
            simulation.ship_index = 99;
            simulation.selected_layer = index;
        }
        let hull_material =
            app.world_mut()
                .resource_mut::<Assets<ShipMaterial>>()
                .add(ShipMaterial {
                    params: Vec4::ZERO,
                    sea_color: Vec4::ZERO,
                    texture: black.clone(),
                    internal_lights: black.clone(),
                    external_lights: black.clone(),
                    water: Handle::default(),
                    masks: Handle::default(),
                    positions: Handle::default(),
                    coverage_mode: Vec4::ZERO,
                    coverage: black.clone(),
                });
        let reflection_material = app
            .world_mut()
            .resource_mut::<Assets<ReflectionMaterial>>()
            .add(ReflectionMaterial {
                texture: black.clone(),
                tint: LinearRgba::WHITE,
                params: Vec4::ZERO,
            });
        let hull = app
            .world_mut()
            .spawn((
                ShipMesh(Handle::default(), hull_material.clone()),
                Visibility::Hidden,
            ))
            .id();
        let reflection = app
            .world_mut()
            .spawn((
                ReflectionMesh {
                    mesh: Handle::default(),
                    material: reflection_material.clone(),
                    half_height: 1.0,
                },
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        let sprite = app
            .world_mut()
            .spawn((
                ShipSprite,
                Sprite::from_image(black.clone()),
                Transform::default(),
                Visibility::Inherited,
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Sprite>(sprite).unwrap().image,
            expected.texture,
            "Unsaved preview renders even with no catalog entry"
        );
        let material = app
            .world()
            .resource::<Assets<ShipMaterial>>()
            .get(&hull_material)
            .unwrap();
        assert_eq!(material.texture, expected.texture);
        assert_eq!(material.internal_lights, expected.internal);
        assert_eq!(material.external_lights, black);
        assert_eq!(
            *app.world().get::<Visibility>(hull).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            *app.world().get::<Visibility>(reflection).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            app.world()
                .resource::<Assets<ReflectionMaterial>>()
                .get(&reflection_material)
                .unwrap()
                .texture,
            expected.texture
        );
        // Source Ship.render calls its resource getters every frame, even when
        // the layer/index/revision is unchanged. Cached access must refresh lastUsed.
        let light=match thumbnail.get_resource(ShipResourceType::InLights,&ShipLayer::new("exterior")).unwrap() {
            ship_thumbnail::ThumbnailResource::File(file)=>file,
            _=>panic!("File light map expected"),
        };
        let before=light.last_used_ms();std::thread::sleep(std::time::Duration::from_millis(5));
        app.world_mut().get_mut::<Sprite>(sprite).unwrap().image=black.clone();app.update();
        assert_eq!(app.world().get::<Sprite>(sprite).unwrap().image,expected.texture,"Unchanged state still queries active resources");
        assert!(light.last_used_ms()>before,"Rendering keeps the used source light map alive");
        app.world_mut().get_mut::<Sprite>(sprite).unwrap().image = black.clone();
        app.world_mut()
            .resource_mut::<ship_upload_preview::ActivePreview>()
            .clear();
        app.update();
        assert_eq!(
            app.world().get::<Sprite>(sprite).unwrap().image,
            expected.texture,
            "Retained catalog thumbnail, rather than disk rediscovery, supplies its existing handle"
        );
        app.world_mut().resource_mut::<Simulation>().selected_layer = 0;
        app.update();
        let handle = &app.world().get::<Sprite>(sprite).unwrap().image;
        let images = app.world().resource::<Assets<Image>>();
        assert_eq!(
            images.get(handle).unwrap().data.as_deref().unwrap(),
            &[
                0x12, 0x34, 0x56, 161, 0x12, 0x34, 0x56, 161, 0x12, 0x34, 0x56, 161
            ]
        );
        let material = app
            .world()
            .resource::<Assets<ShipMaterial>>()
            .get(&hull_material)
            .unwrap();
        assert_eq!(&material.texture, handle);
        assert_eq!(material.internal_lights, black);
        assert_eq!(material.external_lights, black);
        assert_eq!(
            &app.world()
                .resource::<Assets<ReflectionMaterial>>()
                .get(&reflection_material)
                .unwrap()
                .texture,
            handle
        );
        let defaults = super::appearance(
            &thumbnail,
            0,
            &mut app.world_mut().resource_mut::<Assets<Image>>(),
            &black,
        );
        assert_eq!(
            defaults.internal, black,
            "Layers do not inherit each other's lights"
        );
        for file in [base, exterior, lights] {
            std::fs::remove_file(file).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
}

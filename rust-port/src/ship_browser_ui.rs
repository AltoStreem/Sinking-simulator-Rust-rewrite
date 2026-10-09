//! Active ShipScroll image layout, reusing the translated dslfix sizing rules.
//! Exact DPI dimensions and hover descriptions remain pending.
use crate::*;
use bevy::camera::Viewport;
#[derive(Component)]
pub(crate) struct BrowserCamera;
#[derive(Component)]
pub(crate) struct BrowserScrollbar(bool);
pub(crate) fn spawn_scrollbar(commands: &mut Commands) {
    for thumb in [false, true] {
        commands.spawn((
            BrowserScrollbar(thumb),
            Sprite::from_color(
                if thumb {
                    Color::srgb(0.23, 0.28, 0.43)
                } else {
                    Color::srgb(0.12, 0.14, 0.20)
                },
                Vec2::ONE,
            ),
            Transform::from_xyz(-309.0, 0.0, 26.0),
            RenderLayers::layer(3),
            Visibility::Hidden,
        ));
    }
}
#[derive(Resource, Default)]
pub(crate) struct BrowserLayout {
    pub rows: Vec<(usize, Rect, Vec2)>,
    pub scroll: f32,
    pub maximum: f32,
    search: String,
    search_units:Vec<u16>,
}
#[derive(Clone, Copy)]
pub(crate) struct Dimensions(u32, u32);
fn browser_texture(
    choice: &ShipChoice,
    images: &mut Assets<Image>,
) -> Result<(Handle<Image>, Dimensions), String> {
    let handle = if choice.material_map {
        let thumbnail = ship_thumbnail::ShipThumbnail::from_base_file(
            &std::path::Path::new("assets").join(&choice.physics_asset),
        )?;
        match thumbnail.get_resource(ShipResourceType::Texture, &ShipLayer::default()) {
            Some(ship_thumbnail::ThumbnailResource::File(resource)) => {
                resource.texture_now(images)?
            }
            Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(resource)) => {
                thumbnail.base_layer()?;
                resource.texture_now_current(images)?
            }
            None => return Err("missing default-layer TEXTURE".into()),
        }
    } else {
        let image = image::open(std::path::Path::new("assets").join(&choice.asset))
            .map_err(|error| error.to_string())?;
        images.add(texture_2d::ship_texture(image.to_rgba8()))
    };
    let texture = images
        .get(&handle)
        .ok_or("browser texture is unavailable")?;
    let dimensions = Dimensions(
        texture.texture_descriptor.size.width,
        texture.texture_descriptor.size.height,
    );
    Ok((handle, dimensions))
}
fn source_browser_texture(thumbnail:&ship_thumbnail::ShipThumbnail,images:&mut Assets<Image>)->Result<Option<(Handle<Image>,Dimensions)>,String> {
    let texture=match thumbnail.get_resource(ShipResourceType::Texture,&ShipLayer::default()) {
        Some(ship_thumbnail::ThumbnailResource::File(file))=>file.texture(images)?,
        Some(ship_thumbnail::ThumbnailResource::BaseDerivedTexture(derived))=>derived.texture_current(images)?,
        None=>None,
    };
    let Some(handle)=texture else {return Ok(None);};
    let image=images.get(&handle).ok_or("source browser texture unavailable")?;
    Ok(Some((handle,Dimensions(image.texture_descriptor.size.width,image.texture_descriptor.size.height))))
}

impl dslfix::TextureDimensions for Dimensions {
    fn width(&self) -> i32 {
        self.0 as i32
    }
    fn height(&self) -> i32 {
        self.1 as i32
    }
}
struct Geometry {
    width: f32,
    height: f32,
    scroll_max: f32,
    cursor_x: f32,
}
impl dslfix::GeometryBackend for Geometry {
    fn gui_scale(&mut self) -> f32 {
        1.0
    }
    fn window_content_region_width(&mut self) -> f32 {
        self.width - if self.scroll_max > 0.0 { 14.0 } else { 0.0 }
    }
    fn window_height(&mut self) -> f32 {
        self.height
    }
    fn frame_padding(&mut self) -> Vec2 {
        Vec2::new(4.0, 3.0)
    }
    fn scroll_max_y(&mut self) -> f32 {
        self.scroll_max
    }
    fn window_width(&mut self) -> f32 {
        self.width
    }
    fn scrollbar_size(&mut self) -> f32 {
        14.0
    }
    fn set_cursor_pos_x(&mut self, x: f32) {
        self.cursor_x = x;
    }
}
pub(crate) fn rect(simulation: &Simulation) -> Rect {
    Rect::from_corners(
        Vec2::new(-628.0, toolbox_viewport::bottom(simulation) + 8.0),
        Vec2::new(-302.0, 153.0),
    )
}
fn rows(
    dimensions: &[(usize, Dimensions)],
    rect: Rect,
    scrollbar: bool,
) -> (Vec<(usize, Rect, Vec2)>, f32) {
    let mut geometry = Geometry {
        width: rect.width(),
        height: rect.height(),
        scroll_max: if scrollbar { 1.0 } else { 0.0 },
        cursor_x: 0.0,
    };
    let padding = Vec2::splat(3.0);
    let mut y = rect.max.y;
    let mut rows = vec![];
    for (index, dimensions) in dimensions {
        let image =
            dslfix::DslFix::adjust_texture_size_default(dimensions, 0.0, padding, 2, &mut geometry);
        dslfix::DslFix::center_next_element(image, padding, &mut geometry);
        let frame = image + 2.0 * padding;
        let center = Vec2::new(
            rect.min.x + geometry.cursor_x + frame.x * 0.5,
            y - frame.y * 0.5,
        );
        rows.push((*index, Rect::from_center_size(center, frame), image));
        y -= frame.y + 4.0;
    }
    let maximum = (rect.min.y - y - 4.0).max(0.0);
    (rows, maximum)
}
pub(crate) fn sync(
    windows: Query<&Window>,
    simulation: Res<Simulation>,
    catalog: Res<ShipCatalog>,
    mut layout: ResMut<BrowserLayout>,
    mut textures: ResMut<Assets<Image>>,
    mut dimensions_cache: Local<HashMap<String, Option<(Handle<Image>, Dimensions)>>>,
    mut dimensions_revision: Local<Option<u64>>,
    native:Option<NonSend<live_ship_catalog::LiveCatalog>>,
    mut cards: Query<
        (&ShipCard, &mut Sprite, &mut Transform, &mut Visibility),
        Without<ShipThumbnail>,
    >,
    mut images: Query<
        (&ShipThumbnail, &mut Sprite, &mut Transform, &mut Visibility),
        Without<ShipCard>,
    >,
    mut cameras: Query<
        (&mut Camera, &mut Projection, &mut Transform),
        (
            With<BrowserCamera>,
            Without<ShipCard>,
            Without<ShipThumbnail>,
        ),
    >,
    mut bars: Query<
        (
            &BrowserScrollbar,
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
        ),
        (
            Without<BrowserCamera>,
            Without<ShipCard>,
            Without<ShipThumbnail>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let rect = rect(&simulation);
    if *dimensions_revision != Some(simulation.catalog_revision) {
        dimensions_cache.clear();
        *dimensions_revision = Some(simulation.catalog_revision);
    }
    let active = !simulation.toolbox_collapsed && simulation.active_tab == ToolboxTab::Ships;
    let filtered = if active {source_filtered_ship_indices(&catalog,&simulation)}else {Vec::new()};
    let dimensions: Vec<_> = filtered
        .into_iter()
        .filter_map(|index| {
            let choice = &catalog.0[index];
            let key = format!("{}\0{}", choice.physics_asset, choice.asset);
            if let Some(native)=native.as_ref().filter(|_|choice.source_key.is_some()) {
                let thumbnail=native.thumbnail(choice)?;
                match source_browser_texture(&thumbnail,&mut textures) {
                    Ok(Some((handle,dimensions)))=>{dimensions_cache.insert(key,Some((handle,dimensions)));return Some((index,dimensions));},
                    Ok(None)=>{dimensions_cache.remove(&key);return None;},
                    Err(error)=>{dimensions_cache.remove(&key);bevy::log::warn!("Source browser texture failed: {error}");return None;},
                }
            }
            let texture = dimensions_cache.entry(key).or_insert_with(|| {
                browser_texture(choice, &mut textures)
                    .map_err(|error| {
                        bevy::log::warn!(
                            "Could not load browser thumbnail {}: {error}",
                            choice.name
                        );
                    })
                    .ok()
            });
            Some((index, texture.as_ref()?.1))
        })
        .collect();
    let (mut positioned, mut maximum) = rows(&dimensions, rect, false);
    if maximum > 0.0 {
        (positioned, maximum) = rows(&dimensions, rect, true);
    }
    if layout.search != simulation.ship_search || layout.search_units.as_slice()!=source_ship_search_units(&simulation).as_ref() {
        layout.scroll = 0.0;
        layout.search = simulation.ship_search.clone();
        layout.search_units=source_ship_search_units(&simulation).into_owned();
    }
    layout.maximum = maximum;
    layout.scroll = layout.scroll.clamp(0.0, maximum);
    layout.rows = positioned;
    let scrollbar = ui_scrollbar::Geometry::new(
        rect.max.y,
        rect.min.y,
        rect.height(),
        rect.height() + layout.maximum,
        10.0,
    );
    for (bar, mut sprite, mut transform, mut visibility) in &mut bars {
        *visibility = if active && layout.maximum > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        sprite.custom_size = Some(Vec2::new(
            if bar.0 { 8.0 } else { 14.0 },
            if bar.0 { scrollbar.grab } else { rect.height() },
        ));
        transform.translation.y = if bar.0 {
            scrollbar.center(layout.scroll)
        } else {
            rect.center().y
        };
    }
    for (card, mut sprite, mut transform, mut visibility) in &mut cards {
        let row = layout.rows.iter().find(|row| row.0 == card.0);
        *visibility = if active && row.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some((_, bounds, _)) = row {
            sprite.custom_size = Some(bounds.size());
            sprite.color = Color::srgb(0.23, 0.27, 0.40);
            transform.translation.x = bounds.center().x;
            transform.translation.y = bounds.center().y;
        }
    }
    for (image, mut sprite, mut transform, mut visibility) in &mut images {
        let row = layout.rows.iter().find(|row| row.0 == image.0);
        *visibility = if active && row.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some((_, bounds, size)) = row {
            let choice = &catalog.0[image.0];
            let key = format!("{}\0{}", choice.physics_asset, choice.asset);
            if let Some(Some((handle, _))) = dimensions_cache.get(&key) {
                sprite.image = handle.clone();
            }
            sprite.custom_size = Some(*size);
            transform.translation.x = bounds.center().x;
            transform.translation.y = bounds.center().y;
        }
    }
    let scale = window.physical_height() as f32 / 720.0;
    let virtual_width = window.physical_width() as f32 / scale.max(0.001);
    let start = Vec2::new(
        (rect.min.x + virtual_width * 0.5) * scale,
        (360.0 - rect.max.y) * scale,
    )
    .round()
    .max(Vec2::ZERO)
    .as_uvec2();
    let end = Vec2::new(
        (rect.max.x + virtual_width * 0.5) * scale,
        (360.0 - rect.min.y) * scale,
    )
    .round()
    .max(Vec2::ZERO)
    .as_uvec2()
    .min(UVec2::new(
        window.physical_width(),
        window.physical_height(),
    ));
    for (mut camera, mut projection, mut transform) in &mut cameras {
        camera.is_active = active && end.cmpgt(start).all();
        if !camera.is_active {
            continue;
        }
        camera.viewport = Some(Viewport {
            physical_position: start,
            physical_size: end - start,
            ..default()
        });
        transform.translation.x = (start.x + end.x) as f32 * 0.5 / scale - virtual_width * 0.5;
        transform.translation.y = 360.0 - (start.y + end.y) as f32 * 0.5 / scale - layout.scroll;
        if let Projection::Orthographic(p) = &mut *projection {
            p.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: (end.y - start.y) as f32 / scale,
            };
            p.scale = 1.0;
        }
    }
}
pub(crate) fn input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    capture: Option<Res<ui_input_capture::Capture>>,
    mut simulation: ResMut<Simulation>,
    mut layout: ResMut<BrowserLayout>,
    mut preview: ResMut<ship_upload_preview::ActivePreview>,
    mut dragging: Local<Option<f32>>,
    native:Option<NonSend<live_ship_catalog::LiveCatalog>>,
    catalog:Option<Res<ShipCatalog>>,
    mut selection:Option<ResMut<live_ship_catalog::PendingSelection>>,
) {
    let events: Vec<_> = wheel
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => e.y,
            MouseScrollUnit::Pixel => e.y / 40.0,
        })
        .collect();
    if simulation.toolbox_collapsed
        || simulation.active_tab != ToolboxTab::Ships
        || layout.search != simulation.ship_search
        || layout.search_units.as_slice()!=source_ship_search_units(&simulation).as_ref()
    {
        *dragging = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );
    let rect = rect(&simulation);
    if !mouse.pressed(MouseButton::Left) {
        *dragging = None;
    }
    let scrollbar = ui_scrollbar::Geometry::new(
        rect.max.y,
        rect.min.y,
        rect.height(),
        rect.height() + layout.maximum,
        10.0,
    );
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = None;
        if capture.as_ref().is_some_and(|capture|capture.editor_point(point)) {return;}
        if layout.maximum > 0.0
            && (rect.max.x - 11.0..=rect.max.x - 3.0).contains(&point.x)
            && (scrollbar.top - scrollbar.size..=scrollbar.top).contains(&point.y)
        {
            let (scroll, offset) = scrollbar.activate(layout.scroll, point.y);
            layout.scroll = scroll.clamp(0.0, layout.maximum);
            *dragging = Some(offset);
            return;
        }
    } else if let Some(offset) = *dragging {
        layout.scroll = scrollbar.drag(point.y, offset).clamp(0.0, layout.maximum);
        return;
    }
    if capture.is_some_and(|capture|capture.editor_point(point)) || !rect.contains(point) {
        return;
    }
    if !keys.pressed(KeyCode::ControlLeft)
        && !keys.pressed(KeyCode::ControlRight)
        && !keys.pressed(KeyCode::ShiftLeft)
        && !keys.pressed(KeyCode::ShiftRight)
    {
        for delta in events {
            layout.scroll = (layout.scroll - delta * ui_scrollbar::wheel_step(18.0, rect.height()))
                .clamp(0.0, layout.maximum);
        }
    }
    if mouse.just_pressed(MouseButton::Left) {
        let point = point - Vec2::new(0.0, layout.scroll);
        if let Some(row) = layout.rows.iter().find(|row| row.1.contains(point)) {
            if let (Some(native),Some(catalog),Some(selection))=(native.as_ref(),catalog.as_ref(),selection.as_mut()) {
                if let Some(thumbnail)=catalog.0.get(row.0).and_then(|choice|native.thumbnail(choice)) {selection.0=Some(thumbnail);}
            }
            preview.clear();
            simulation.ship_index = row.0;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_uses_default_texture_and_derives_missing_texture_with_local_palette() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder =
            std::env::temp_dir().join(format!("ss2-browser-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        let base = folder.join("Fixture_base.png");
        let texture = folder.join("Fixture_texture.png");
        let exterior = folder.join("Fixture_exterior_texture.png");
        let palette = folder.join("Fixture_materials.json");
        image::RgbaImage::from_pixel(4, 1, image::Rgba([0x12, 0x34, 0x56, 255]))
            .save(&base)
            .unwrap();
        image::RgbaImage::from_pixel(3, 2, image::Rgba([10, 20, 30, 255]))
            .save(&texture)
            .unwrap();
        image::RgbaImage::from_pixel(7, 5, image::Rgba([40, 50, 60, 255]))
            .save(&exterior)
            .unwrap();
        std::fs::write(&palette, r##"[{"color":"#123456","invisible":true}]"##).unwrap();
        let choice = ShipChoice {
            name: "Fixture".into(),
            asset: exterior.to_string_lossy().into_owned(),
            physics_asset: base.to_string_lossy().into_owned(),
            material_map: true,
            scale: 1.0,
            source_key: None,
        };
        let mut images = Assets::<Image>::default();
        let (handle, dimensions) = browser_texture(&choice, &mut images).unwrap();
        assert_eq!((dimensions.0, dimensions.1), (3, 2));
        assert_eq!(
            &images.get(&handle).unwrap().data.as_ref().unwrap()[..4],
            &[10, 20, 30, 255]
        );
        std::fs::remove_file(&texture).unwrap();
        let (handle, dimensions) = browser_texture(&choice, &mut images).unwrap();
        assert_eq!((dimensions.0, dimensions.1), (4, 1));
        assert!(
            images
                .get(&handle)
                .unwrap()
                .data
                .as_ref()
                .unwrap()
                .iter()
                .all(|byte| *byte == 0)
        );
        for path in [base, exterior, palette] {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn active_browser_has_variable_rows_clipping_and_scrolled_image_selection() {
        let mut app = App::new();
        let choices = (0..10)
            .map(|index| ShipChoice {
                name: format!("ship {index}"),
                asset: "ships/Titanic.png".into(),
                physics_asset: "ships/Titanic_base.png".into(),
                material_map: false,
                scale: 1.0,
                source_key: None,
            })
            .collect();
        app.init_resource::<Simulation>()
            .init_resource::<BrowserLayout>()
            .init_resource::<ship_upload_preview::ActivePreview>()
            .init_resource::<Assets<Image>>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ShipCatalog(choices, vec![]))
            .add_message::<MouseWheel>()
            .add_systems(Update, (input, sync).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1280, 720).into(),
                ..default()
            })
            .id();
        let camera = app
            .world_mut()
            .spawn((
                Camera2d,
                BrowserCamera,
                Projection::Orthographic(OrthographicProjection::default_2d()),
                Transform::default(),
            ))
            .id();
        let card = app
            .world_mut()
            .spawn((
                ShipCard(0),
                Sprite::default(),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        let image = app
            .world_mut()
            .spawn((
                ShipThumbnail(0),
                Sprite::default(),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        app.update();
        let layout = app.world().resource::<BrowserLayout>();
        assert_eq!(layout.rows.len(), 10);
        assert!(layout.maximum > 0.0);
        assert_eq!(
            app.world().get::<Sprite>(card).unwrap().custom_size,
            Some(layout.rows[0].1.size())
        );
        assert_eq!(
            app.world().get::<Sprite>(image).unwrap().custom_size,
            Some(layout.rows[0].2)
        );
        let viewport = app
            .world()
            .get::<Camera>(camera)
            .unwrap()
            .viewport
            .as_ref()
            .unwrap();
        assert_eq!(viewport.physical_position, UVec2::new(12, 207));
        assert_eq!(viewport.physical_size, UVec2::new(326, 321));
        let row = layout.rows[5].1;
        let scroll = (100.0 - row.center().y).min(layout.maximum);
        app.world_mut().resource_mut::<BrowserLayout>().scroll = scroll;
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(
                640.0 + row.center().x,
                360.0 - (row.center().y + scroll),
            )));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Simulation>().ship_index, 5);
        assert!(
            (app.world().get::<Transform>(camera).unwrap().translation.y + 7.5 + scroll).abs()
                < 0.0001
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.world_mut().resource_mut::<Simulation>().ship_search = "no matches".into();
        // Search input runs before browser input; old image bounds must not select a ship.
        app.world_mut().resource_mut::<Simulation>().ship_index = 0;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Simulation>().ship_index, 0);
        assert!(app.world().resource::<BrowserLayout>().rows.is_empty());
        assert_eq!(app.world().resource::<BrowserLayout>().scroll, 0.0);
        assert_eq!(
            app.world().get::<Visibility>(card).unwrap(),
            &Visibility::Hidden
        );
        app.world_mut()
            .resource_mut::<Simulation>()
            .toolbox_collapsed = true;
        app.update();
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);
    }
    #[test]
    fn source_thumbnail_sizes_keep_aspect_and_half_child_height_and_centering() {
        let rect = Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(326.0, 400.0));
        let dimensions = [
            (0, Dimensions(1, 2000)),
            (1, Dimensions(1000, 250)),
            (2, Dimensions(100, 100)),
            (3, Dimensions(100, 100)),
        ];
        let (items, maximum) = rows(&dimensions, rect, true);
        assert_eq!(items.len(), 4);
        assert!(maximum > 0.0);
        assert_eq!(items[0].1.height(), 200.0);
        assert!((items[0].1.width() - 0.1).abs() < 0.00001);
        assert_eq!(items[1].1.size(), Vec2::new(312.0, 78.0));
        assert_eq!(items[2].1.size(), Vec2::new(100.0, 100.0));
        for item in items {
            assert!((item.1.center().x - 156.0).abs() < 0.00001);
        }
    }
    #[test]
    fn open_editor_only_occludes_covered_browser_rows() {
        let mut app=App::new();
        app.init_resource::<Simulation>().init_resource::<BrowserLayout>()
            .init_resource::<ship_upload_preview::ActivePreview>()
            .init_resource::<ship_upload::SourceShipUpload>()
            .init_resource::<ship_upload_layout::Layout>()
            .init_resource::<ui_input_capture::Capture>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>()
            .add_systems(PreUpdate,ui_input_capture::update).add_systems(Update,input);
        let point=rect(app.world().resource::<Simulation>()).center();
        app.world_mut().resource_mut::<BrowserLayout>().rows.push((1,Rect::from_center_size(point,Vec2::splat(20.0)),Vec2::splat(20.0)));
        app.world_mut().resource_mut::<ship_upload::SourceShipUpload>().set_window_open(true);
        app.world_mut().resource_mut::<ship_upload_layout::Layout>().position=point;
        let mut window=Window {resolution:(1280,720).into(),..default()};
        window.set_cursor_position(Some(Vec2::new(640.0+point.x,360.0-point.y)));
        app.world_mut().spawn(window);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();assert_eq!(app.world().resource::<Simulation>().ship_index,0);
        app.world_mut().resource_mut::<ship_upload_layout::Layout>().position=Vec2::new(450.0,0.0);
        let mut mouse=app.world_mut().resource_mut::<ButtonInput<MouseButton>>();mouse.reset_all();mouse.press(MouseButton::Left);
        app.update();assert_eq!(app.world().resource::<Simulation>().ship_index,1);
    }

}

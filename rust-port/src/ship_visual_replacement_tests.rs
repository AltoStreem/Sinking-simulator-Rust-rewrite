//! Exercise the live load system, not the legacy in-place buffer reset.
use crate::*;

fn structure(w: usize, h: usize) -> ShipStructure {
    let mut ship = ShipStructure::empty_fallback();
    ship.texel_width = w; ship.texel_height = h;
    ship.texel_solid = vec![true; w * h];
    ship.texel_strut_masks = vec![255; w * h];
    ship.texel_materials = vec![Some(MaterialProperties::from(&materials::Material::default())); w * h];
    ship.texel_rest_positions = (0..h).flat_map(|y| (0..w).map(move |x| Vec2::new(x as f32, y as f32))).collect();
    ship.texel_positions = ship.texel_rest_positions.clone(); ship
}
fn material(physics: &GpuShipPhysicsAssets) -> ShipMaterial {
    ShipMaterial { params: Vec4::ZERO, sea_color: Vec4::ZERO, texture: Handle::default(),
        internal_lights: Handle::default(), external_lights: Handle::default(),
        water: physics.water.clone(), masks: physics.masks.clone(), positions: physics.positions.clone(),
        coverage_mode: Vec4::ZERO, coverage: Handle::default() }
}
fn assert_material_same(a: &ShipMaterial, b: &ShipMaterial) {
    assert_eq!(a.params, b.params); assert_eq!(a.sea_color, b.sea_color);
    assert_eq!(a.positions, b.positions); assert_eq!(a.masks, b.masks); assert_eq!(a.water, b.water);
    assert_eq!(a.coverage, b.coverage); assert_eq!(a.coverage_mode, b.coverage_mode);
    assert_eq!(a.texture, b.texture); assert_eq!(a.internal_lights,b.internal_lights); assert_eq!(a.external_lights,b.external_lights);
}

#[test]
fn live_replacement_rebinds_all_draws_and_preserves_retained_ship_assets() {
    let initial = structure(2,2);
    let mut buffers = Assets::<ShaderBuffer>::default();
    let physics = make_gpu_ship_physics_assets(&initial, &mut buffers);
    let mut meshes = Assets::<Mesh>::default();
    let hull_mesh = meshes.add(ship_gpu_geometry::build_mesh(&initial));
    let strut_mesh = meshes.add(ship_gpu_geometry::build_struts_mesh(&initial));
    let (water_data,cells) = build_internal_water_mesh(&initial);
    let water_mesh = meshes.add(water_data); let reflection_mesh_handle = meshes.add(reflection_mesh(&initial));
    let mut materials = Assets::<ShipMaterial>::default();
    let hull_material = materials.add(material(&physics));
    let strut_material = materials.add(material(&physics));
    let untouched = materials.add(material(&physics));
    let untouched_value = materials.get(&untouched).unwrap().clone();
    let mut waters = Assets::<InternalWaterMaterial>::default();
    let water_material = waters.add(InternalWaterMaterial {color: LinearRgba::WHITE, params: Vec4::ONE});
    let mut reflections = Assets::<ReflectionMaterial>::default();
    let reflection_material = reflections.add(ReflectionMaterial {texture:Handle::default(), tint:LinearRgba::WHITE, params:Vec4::ONE});
    let mut app = App::new();
    app.insert_resource(initial).insert_resource(physics).insert_resource(buffers).insert_resource(meshes)
        .insert_resource(materials).insert_resource(waters).insert_resource(reflections)
        .init_resource::<Assets<Image>>().init_resource::<Assets<ship_coverage::CoverageMaterial>>()
        .insert_resource(ShipCatalog(Vec::new(),Vec::new())).init_resource::<Simulation>()
        .init_resource::<GpuShipPhysicsSnapshot>().init_resource::<ship_upload_preview::ActivePreview>()
        .init_resource::<tools::move_tool::MoveDragState>()
        .add_systems(Startup,ship_coverage::setup)
        .add_systems(Update,(load_selected_ship,fragment_shaders::update_ship_lighting).chain())
        .add_systems(PostUpdate,ship_coverage::sync);
    app.world_mut().spawn(Window {resolution:(32,32).into(), ..default()});
    app.world_mut().spawn((WorldCamera,Transform::default(),Projection::Orthographic(OrthographicProjection::default_2d()),Camera::default()));
    let hull = app.world_mut().spawn((ShipMesh(hull_mesh.clone(),hull_material.clone()),
        Mesh2d(hull_mesh.clone()),MeshMaterial2d(hull_material.clone()),MeshSyncState(Vec::new(),Vec::new()),Transform::default())).id();
    let strut = app.world_mut().spawn((ship_struts::ShipStruts {mesh:strut_mesh.clone(),dimensions:(2,2),masks:Vec::new(),occupied:vec![true;4]},
        Mesh2d(strut_mesh.clone()),MeshMaterial2d(strut_material),Transform::default())).id();
    let water = app.world_mut().spawn((InternalWaterMesh {mesh:water_mesh.clone(),material:water_material.clone(),cells,width:32,height:12,interior:Vec::new()},
        Mesh2d(water_mesh.clone()),MeshMaterial2d(water_material.clone()))).id();
    let reflection = app.world_mut().spawn((ReflectionMesh {mesh:reflection_mesh_handle.clone(),material:reflection_material.clone(),half_height:1.0},
        Mesh2d(reflection_mesh_handle.clone()),MeshMaterial2d(reflection_material.clone()))).id();
    app.update();
    let coverage = app.world_mut().query_filtered::<Entity,With<ship_coverage::CoverageHull>>().single(app.world()).unwrap();
    let original_strut = app.world().get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0.clone();
    let original_coverage = app.world().get::<MeshMaterial2d<ship_coverage::CoverageMaterial>>(coverage).unwrap().0.clone();
    let old_coverage = app.world().resource::<Assets<ship_coverage::CoverageMaterial>>().get(&original_coverage).unwrap().clone();
    let old_hull = app.world().resource::<Assets<ShipMaterial>>().get(&hull_material).unwrap().clone();
    let old_strut = app.world().resource::<Assets<ShipMaterial>>().get(&original_strut).unwrap().clone();
    let old_positions = app.world().resource::<Assets<ShaderBuffer>>().get(&old_hull.positions).unwrap().data.clone();
    let old_vertices = format!("{:?}",app.world().resource::<Assets<Mesh>>().get(&hull_mesh).unwrap().attribute(Mesh::ATTRIBUTE_POSITION));
    let mut previous = (hull_mesh.clone(),hull_material.clone(),original_strut.clone(),original_coverage.clone());
    for (generation,(w,h)) in [(2,2),(3,4),(1,1)].into_iter().enumerate() {
        { let mut preview = app.world_mut().resource_mut::<ship_upload_preview::ActivePreview>();
          preview.pending=Some(structure(w,h)); preview.revision+=1; }
        app.world_mut().resource_mut::<Simulation>().day=0.15+generation as f32*0.2;
        app.update();
        let ship = app.world().get::<ShipMesh>(hull).unwrap();
        let strut_handle = &app.world().get::<MeshMaterial2d<ShipMaterial>>(strut).unwrap().0;
        let coverage_handle = &app.world().get::<MeshMaterial2d<ship_coverage::CoverageMaterial>>(coverage).unwrap().0;
        assert_ne!(ship.0,previous.0); assert_ne!(ship.1,previous.1);
        assert_ne!(*strut_handle,previous.2); assert_ne!(*coverage_handle,previous.3);
        assert_eq!(app.world().get::<Mesh2d>(hull).unwrap().0,ship.0);
        assert_eq!(app.world().get::<MeshMaterial2d<ShipMaterial>>(hull).unwrap().0,ship.1);
        assert_eq!(app.world().get::<Mesh2d>(coverage).unwrap().0,ship.0);
        let gpu=app.world().resource::<GpuShipPhysicsAssets>();
        assert_eq!(gpu.generation,(generation+1) as u64);
        let materials=app.world().resource::<Assets<ShipMaterial>>();
        for handle in [&ship.1,strut_handle] {
            let current=materials.get(handle).unwrap();
            assert_eq!(current.positions,gpu.positions); assert_eq!(current.masks,gpu.masks); assert_eq!(current.water,gpu.water);
            assert_eq!((current.params.z,current.params.w),(w as f32,h as f32));
        }
        assert_eq!(materials.get(strut_handle).unwrap().coverage_mode,Vec4::X);
        let current=app.world().resource::<Assets<ship_coverage::CoverageMaterial>>().get(coverage_handle).unwrap();
        assert_eq!(current.positions,gpu.positions); assert_eq!(current.masks,gpu.masks);
        assert_eq!((current.params.z,current.params.w),(w as f32,h as f32));
        assert_material_same(materials.get(&hull_material).unwrap(),&old_hull);
        assert_material_same(materials.get(&original_strut).unwrap(),&old_strut);
        assert_material_same(materials.get(&untouched).unwrap(),&untouched_value);
        let retained=app.world().resource::<Assets<ship_coverage::CoverageMaterial>>().get(&original_coverage).unwrap();
        assert_eq!(retained.params,old_coverage.params); assert_eq!(retained.positions,old_coverage.positions); assert_eq!(retained.masks,old_coverage.masks);
        assert_eq!(app.world().resource::<Assets<ShaderBuffer>>().get(&old_hull.positions).unwrap().data,old_positions);
        assert_eq!(format!("{:?}",app.world().resource::<Assets<Mesh>>().get(&hull_mesh).unwrap().attribute(Mesh::ATTRIBUTE_POSITION)),old_vertices);
        let current_water=app.world().get::<InternalWaterMesh>(water).unwrap();
        assert_ne!(current_water.mesh,water_mesh); assert_ne!(current_water.material,water_material);
        assert_eq!(current_water.mesh,app.world().get::<Mesh2d>(water).unwrap().0);
        assert_eq!(current_water.material,app.world().get::<MeshMaterial2d<InternalWaterMaterial>>(water).unwrap().0);
        let current_reflection=app.world().get::<ReflectionMesh>(reflection).unwrap();
        assert_ne!(current_reflection.mesh,reflection_mesh_handle); assert_ne!(current_reflection.material,reflection_material);
        assert_eq!(current_reflection.mesh,app.world().get::<Mesh2d>(reflection).unwrap().0);
        assert_eq!(current_reflection.material,app.world().get::<MeshMaterial2d<ReflectionMaterial>>(reflection).unwrap().0);
        previous=(ship.0.clone(),ship.1.clone(),strut_handle.clone(),coverage_handle.clone());
    }
}

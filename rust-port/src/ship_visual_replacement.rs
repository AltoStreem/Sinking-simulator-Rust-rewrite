//! Per-Ship draw state: independent meshes/material bindings with shared shader/image assets.
//! Native GL program IDs, close callbacks and target/context lifetimes remain separate work.
use crate::*;
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(crate) struct VisualReplacement<'w, 's> {
    ship_materials: Option<ResMut<'w, Assets<ShipMaterial>>>,
    water_materials: Option<ResMut<'w, Assets<InternalWaterMaterial>>>,
    reflection_materials: Option<ResMut<'w, Assets<ReflectionMaterial>>>,
    coverage_materials: Option<ResMut<'w, Assets<ship_coverage::CoverageMaterial>>>,
    hulls: Query<'w, 's, (&'static mut ShipMesh, Option<&'static mut Mesh2d>,
        Option<&'static mut MeshMaterial2d<ShipMaterial>>, &'static mut MeshSyncState),
        (Without<ship_struts::ShipStruts>, Without<InternalWaterMesh>, Without<ReflectionMesh>)>,
    struts: Query<'w, 's, (&'static mut ship_struts::ShipStruts, Option<&'static mut Mesh2d>,
        Option<&'static mut MeshMaterial2d<ShipMaterial>>),
        (Without<ShipMesh>, Without<InternalWaterMesh>, Without<ReflectionMesh>)>,
    water: Query<'w, 's, (&'static mut InternalWaterMesh, Option<&'static mut Mesh2d>,
        Option<&'static mut MeshMaterial2d<InternalWaterMaterial>>),
        (Without<ShipMesh>, Without<ship_struts::ShipStruts>, Without<ReflectionMesh>)>,
    reflections: Query<'w, 's, (&'static mut ReflectionMesh, Option<&'static mut Mesh2d>,
        Option<&'static mut MeshMaterial2d<ReflectionMaterial>>),
        (Without<ShipMesh>, Without<ship_struts::ShipStruts>, Without<InternalWaterMesh>)>,
    coverage: Query<'w, 's, &'static mut MeshMaterial2d<ship_coverage::CoverageMaterial>,
        With<ship_coverage::CoverageHull>>,
}

pub(crate) fn bind(material: &mut ShipMaterial, physics: &GpuShipPhysicsAssets, simulation: &Simulation) {
    material.positions = physics.positions.clone();
    material.masks = physics.masks.clone();
    material.water = physics.water.clone();
    material.params = Vec4::new(simulation.day, u8::from(simulation.show_internal_water) as f32,
        physics.width as f32, physics.height as f32);
    material.sea_color = simulation.water_color();
}

impl VisualReplacement<'_, '_> {
    pub(crate) fn construct(&mut self, structure: &ShipStructure, physics: &GpuShipPhysicsAssets,
        simulation: &Simulation, meshes: &mut Assets<Mesh>) {
        for (mut ship, mesh2d, material2d, mut state) in &mut self.hulls {
            let mesh = meshes.add(ship_gpu_geometry::build_mesh(structure));
            ship.0 = mesh.clone();
            if let Some(mut mesh2d) = mesh2d { mesh2d.0 = mesh; }
            if let Some(materials) = self.ship_materials.as_mut() {
                let mut material = materials.get(&ship.1).expect("active hull material missing").clone();
                bind(&mut material, physics, simulation);
                material.coverage_mode = Vec4::ZERO;
                let handle = materials.add(material);
                ship.1 = handle.clone();
                if let Some(mut material2d) = material2d { material2d.0 = handle; }
            }
            state.0.clone_from(&structure.breached);
            state.1.clear();
        }
        for (mut strut, mesh2d, material2d) in &mut self.struts {
            let mesh = meshes.add(ship_gpu_geometry::build_struts_mesh(structure));
            strut.mesh = mesh.clone();
            strut.dimensions = (structure.texel_width, structure.texel_height);
            strut.masks = gpu_mask_data(structure);
            strut.occupied.clone_from(&structure.texel_solid);
            if let Some(mut mesh2d) = mesh2d { mesh2d.0 = mesh; }
            if let (Some(materials), Some(mut material2d)) = (self.ship_materials.as_mut(), material2d) {
                let mut material = materials.get(&material2d.0).expect("active strut material missing").clone();
                bind(&mut material, physics, simulation);
                material.coverage_mode = Vec4::X;
                material2d.0 = materials.add(material);
            }
        }
        for (mut water, mesh2d, material2d) in &mut self.water {
            let (data, cells) = build_internal_water_mesh(structure);
            let mesh = meshes.add(data);
            water.mesh = mesh.clone(); water.cells = cells;
            water.width = structure.width; water.height = structure.height;
            water.interior.clone_from(&structure.interior);
            if let Some(mut mesh2d) = mesh2d { mesh2d.0 = mesh; }
            if let Some(materials) = self.water_materials.as_mut() {
                let material = materials.get(&water.material).expect("active internal-water material missing").clone();
                let handle = materials.add(material); water.material = handle.clone();
                if let Some(mut material2d) = material2d { material2d.0 = handle; }
            }
        }
        for (mut reflection, mesh2d, material2d) in &mut self.reflections {
            let mesh = meshes.add(reflection_mesh(structure));
            reflection.mesh = mesh.clone(); reflection.half_height = structure.half_height;
            if let Some(mut mesh2d) = mesh2d { mesh2d.0 = mesh; }
            if let Some(materials) = self.reflection_materials.as_mut() {
                let material = materials.get(&reflection.material).expect("active reflection material missing").clone();
                let handle = materials.add(material); reflection.material = handle.clone();
                if let Some(mut material2d) = material2d { material2d.0 = handle; }
            }
        }
        if let Some(materials) = self.coverage_materials.as_mut() {
            for mut material2d in &mut self.coverage {
                let mut material = materials.get(&material2d.0).expect("active coverage material missing").clone();
                material.positions = physics.positions.clone(); material.masks = physics.masks.clone();
                material.params.z = physics.width as f32; material.params.w = physics.height as f32;
                material2d.0 = materials.add(material);
            }
        }
    }
}

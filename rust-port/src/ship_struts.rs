//! Source `ShipStruts.java` vertex/index construction and shader link filtering.
use crate::ship_data::ShipData;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};

#[allow(dead_code)]
pub(crate) struct SourceShipStrutShader(std::sync::Arc<crate::shader::Shader>);
#[allow(dead_code)]
impl SourceShipStrutShader {
    pub fn new(
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        Ok(Self(std::sync::Arc::new(crate::shader::Shader::new(
            crate::ship_strut_shader::SOURCE,
            36313,
            vec![],
            backend,
            context,
            runtime,
        )?)))
    }
    pub fn shader(&self) -> std::sync::Arc<crate::shader::Shader> {
        self.0.clone()
    }
}

/// Native object construction and inherited lifecycle of ShipStruts.java.
/// The active Bevy mesh adapter below remains a separate rendering backend.
#[allow(dead_code)]
pub(crate) struct SourceShipStruts {
    pub model: crate::shaded_model::ShadedModel,
    pub shader: std::sync::Arc<crate::shader_program::ShaderProgram>,
    camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    callback: crate::camera_2d::SourceCameraCallback,
}
#[allow(dead_code)]
impl SourceShipStruts {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dat: &impl crate::ship_companion::ShipGeometryData,
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        rendering_samplers: &[String],
        vertex: std::sync::Arc<crate::shader::Shader>,
        geometry: std::sync::Arc<crate::shader::Shader>,
        fragment: std::sync::Arc<crate::shader::Shader>,
        programs: std::sync::Arc<std::sync::Mutex<dyn crate::shader_program::ProgramBackend>>,
        buffers: std::sync::Arc<std::sync::Mutex<dyn crate::vbo::BufferBackend>>,
        vaos: std::sync::Arc<std::sync::Mutex<dyn crate::vao::VertexArrayBackend>>,
        draws: std::sync::Arc<std::sync::Mutex<dyn crate::model::ModelBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        use std::{rc::Rc, sync::Arc};
        let indices = crate::ship_struts_companion::create_indices(dat)
            .unwrap_or_else(|error| panic!("{error}"));
        let vertices = crate::ship_struts_companion::create_vertices(dat)
            .unwrap_or_else(|error| panic!("{error}"));
        let shader = Arc::new(crate::shader_program::ShaderProgram::new(
            vec![vertex, geometry, fragment],
            programs,
            context.clone(),
            runtime,
        ));
        let model = crate::shaded_model::ShadedModel::from_arrays(
            &indices,
            &vertices,
            2,
            shader.clone(),
            1,
            buffers,
            vaos,
            draws,
            context,
            runtime,
        );
        let identity = Rc::new(());
        let callback = crate::ship_struts_camera_callback::create(identity, shader.clone());
        for (index, name) in rendering_samplers.iter().enumerate() {
            shader
                .set_ints(shader.uniform_location(name), &[index as i32])
                .unwrap_or_else(|error| panic!("{error}"));
        }
        shader
            .set_floats(
                shader.uniform_location("resolution"),
                &[dat.width() as f32, dat.height() as f32],
            )
            .unwrap_or_else(|error| panic!("{error}"));
        camera_control
            .borrow()
            .camera()
            .add_camera_callback(callback.clone());
        shader.validate(model.model.vao.id());
        let cleanup_control = camera_control.clone();
        let key = callback.key.clone();
        model.model.resource_handle().before_free_local(move || {
            cleanup_control
                .borrow()
                .camera()
                .remove_camera_callback(&key);
        });
        Self {
            model,
            shader,
            camera_control,
            callback,
        }
    }
    pub fn camera_control(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>> {
        self.camera_control.clone()
    }
    pub fn camera_callback(&self) -> crate::camera_2d::SourceCameraCallback {
        self.callback.clone()
    }
    pub fn set_time(&self, time: f32) {
        self.shader
            .set_floats(self.shader.uniform_location("time"), &[time])
            .unwrap_or_else(|error| panic!("{error}"));
    }
    pub fn set_displacement(&self, displacement: Vec2) {
        self.shader
            .set_floats(
                self.shader.uniform_location("u_mouse"),
                &displacement.to_array(),
            )
            .unwrap_or_else(|error| panic!("{error}"));
    }
    pub fn free(&self) {
        self.camera_control
            .borrow()
            .camera()
            .remove_camera_callback(&self.callback.key);
    }
    pub fn close(&self) {
        self.model.model.close();
    }
    pub fn freed(&self) -> bool {
        self.model.model.freed()
    }
}
impl crate::i_drawable::IDrawable for SourceShipStruts {
    fn render(&self) {
        crate::i_drawable::IDrawable::render(&self.model);
    }
}

#[derive(Component)]
pub(crate) struct ShipStruts {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) dimensions: (usize, usize),
    pub(crate) masks: Vec<[u32; 4]>,
    pub(crate) occupied: Vec<bool>,
}

impl ShipStruts {
    pub(crate) fn create_vertices(dat: &ShipData) -> Vec<[f32; 2]> {
        crate::ship_struts_companion::create_vertices(dat)
            .unwrap_or_else(|error| panic!("{error}"))
            .chunks_exact(2)
            .map(|vertex| [vertex[0], vertex[1]])
            .collect()
    }

    pub(crate) fn create_indices(dat: &ShipData) -> Vec<u32> {
        crate::ship_struts_companion::create_indices(dat)
            .unwrap_or_else(|error| panic!("{error}"))
            .into_iter()
            .map(|index| index as u32)
            .collect()
    }

    pub(crate) fn live_indices(
        width: usize,
        height: usize,
        occupied: &[bool],
        masks: &[[u32; 4]],
    ) -> Vec<u32> {
        assert_eq!(masks.len(), width * height);
        Self::indices(width, height, occupied, Some(masks))
    }

    fn indices(
        width: usize,
        height: usize,
        occupied: &[bool],
        masks: Option<&[[u32; 4]]>,
    ) -> Vec<u32> {
        assert_eq!(occupied.len(), width * height);
        let mut indices = Vec::new();
        // Kotlin loops over X then Y. Each undirected edge is emitted once:
        // east, southeast, south, southwest (mask bits 0,1,2,3).
        for x in 0..width {
            for y in 0..height {
                let a = y * width + x;
                if !occupied[a] {
                    continue;
                }
                for (bit, (dx, dy)) in [(1isize, 0isize), (1, 1), (0, 1), (-1, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                        continue;
                    }
                    let b = ny as usize * width + nx as usize;
                    if occupied[b] && masks.is_none_or(|m| m[a][1] & (1 << bit) != 0) {
                        indices.extend([a as u32, b as u32]);
                    }
                }
            }
        }
        indices
    }
}

pub(crate) fn build_mesh(structure: &crate::ShipStructure, masks: &[[u32; 4]]) -> Mesh {
    let width = structure.texel_width;
    let height = structure.texel_height;
    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(ShipStruts::live_indices(
        width,
        height,
        &structure.texel_solid,
        masks,
    )));
    mesh
}

pub(crate) fn sync_ship_struts(
    structure: Res<crate::ShipStructure>,
    snapshot: Res<crate::GpuShipPhysicsSnapshot>,
    mut meshes: ResMut<Assets<Mesh>>,
    surfaces: Query<&Transform, (With<crate::ShipMesh>, Without<ShipStruts>)>,
    mut struts: Query<
        (
            &mut ShipStruts,
            &mut Transform,
            Option<&crate::ship_gpu_geometry::GpuGeometry>,
        ),
        Without<crate::ShipMesh>,
    >,
) {
    let dimensions = (structure.texel_width, structure.texel_height);
    for (mut state, mut transform, gpu) in &mut struts {
        if let Ok(surface) = surfaces.single() {
            *transform = *surface;
            // Active struts render after the hull and reject hull-covered pixels
            // through the coverage target; legacy CPU lines retain their order.
            transform.translation.z += if gpu.is_some() { 0.001 } else { -0.001 };
        }
        if gpu.is_some() {
            if state.dimensions != dimensions || state.occupied != structure.texel_solid {
                if let Some(mut mesh) = meshes.get_mut(&state.mesh) {
                    *mesh = crate::ship_gpu_geometry::build_struts_mesh(&structure);
                    state.dimensions = dimensions;
                    state.occupied.clone_from(&structure.texel_solid);
                }
            }
            continue;
        }
        let masks = crate::current_render_masks(&structure, &snapshot);
        let Some(mut mesh) = meshes.get_mut(&state.mesh) else {
            continue;
        };
        if state.dimensions != dimensions || state.masks != masks {
            *mesh = build_mesh(&structure, &masks);
            state.dimensions = dimensions;
            state.masks.clone_from(&masks);
        } else if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for (vertex, position) in vertices.iter_mut().zip(&structure.texel_positions) {
                *vertex = [position.x, position.y, 0.0];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_strut_shader_literal_matches_original_java() {
        let original = std::fs::read_to_string(
            "../SS2/decompiled/com/wicpar/sinkingsimulator/ship/ShipStruts.java",
        )
        .unwrap();
        assert!(original.contains(&format!(
            "new Shader({:?}, 36313",
            crate::ship_strut_shader::SOURCE
        )));
    }
    #[test]
    fn source_struts_constructor_draw_updates_and_deferred_callback_cleanup() {
        use std::{
            cell::RefCell,
            rc::Rc,
            sync::{Arc, Mutex},
        };
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(vec![]));
        let programs = Arc::new(Mutex::new(crate::shader_program::tests::Backend(
            log.clone(),
        )));
        let geometry = Arc::new(Mutex::new(crate::shaded_model::tests::Backend(log.clone())));
        let vertices = crate::vertex_shaders::SourceVertexShaders::new(
            programs.clone(),
            context.clone(),
            &runtime,
        );
        let vertex = vertices.none().unwrap();
        let strut_shader =
            SourceShipStrutShader::new(programs.clone(), context.clone(), &runtime).unwrap();
        assert!(Arc::ptr_eq(&strut_shader.shader(), &strut_shader.shader()));
        assert!(strut_shader.shader().bindings.is_empty());
        let fragment = Arc::new(
            crate::shader::Shader::new(
                crate::ship_fragment_shader::SOURCE,
                35632,
                vec![],
                programs.clone(),
                context.clone(),
                &runtime,
            )
            .unwrap(),
        );
        let dat = crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(crate::materials::SourceMaterials::default()),
            Rc::new(RefCell::new(vec![
                Some(Arc::new(
                    crate::materials::Material::default()
                ));
                4
            ])),
            2,
            2,
        );
        let (window, _, _) = crate::window::tests::fixture();
        let camera = Rc::new(crate::camera_2d::SourceCamera2D::new(400, 200));
        let control = Rc::new(RefCell::new(
            crate::camera_control::SourceCameraControl::with_camera(window, camera.clone()),
        ));
        log.lock().unwrap().clear();
        let struts = SourceShipStruts::new(
            &dat,
            control.clone(),
            &["tex".into(), "pos".into(), "pos".into()],
            vertex,
            strut_shader.shader(),
            fragment,
            programs,
            geometry.clone(),
            geometry.clone(),
            geometry,
            context.clone(),
            &runtime,
        );
        assert!(Rc::ptr_eq(&struts.camera_control(), &control));
        assert!(Rc::ptr_eq(
            &struts.camera_callback().invoke,
            &struts.camera_callback().invoke
        ));
        let setup = log.lock().unwrap().clone();
        let validations: Vec<_> = setup
            .iter()
            .enumerate()
            .filter_map(|(i, event)| (event == "validate").then_some(i))
            .collect();
        assert_eq!(validations.len(), 2);
        let first_sampler = setup
            .iter()
            .position(|event| event == "int:-1:[0]")
            .unwrap();
        let resolution = setup
            .iter()
            .position(|event| event == "float:-1:[2.0, 2.0]")
            .unwrap();
        let matrix = setup
            .iter()
            .position(|event| event.starts_with("matrix:"))
            .unwrap();
        assert!(
            validations[0] < first_sampler
                && first_sampler < resolution
                && resolution < matrix
                && matrix < validations[1]
        );
        assert_eq!(
            setup
                .iter()
                .filter(|event| event.starts_with("int:"))
                .cloned()
                .collect::<Vec<_>>(),
            ["int:-1:[0]", "int:-1:[1]", "int:-1:[2]"]
        );
        log.lock().unwrap().clear();
        crate::i_drawable::IDrawable::render(&struts);
        assert_eq!(
            *log.lock().unwrap(),
            [
                "use:9",
                "vao:21",
                "enable:0",
                "draw:1:12:5125:0",
                "disable:0",
                "vao:0",
                "use:0"
            ]
        );
        log.lock().unwrap().clear();
        struts.set_time(3.5);
        struts.set_displacement(Vec2::new(-2.0, 7.0));
        assert_eq!(
            *log.lock().unwrap(),
            [
                "use:9",
                "float:-1:[3.5]",
                "use:0",
                "use:9",
                "float:-1:[-2.0, 7.0]",
                "use:0"
            ]
        );
        struts.close();
        assert!(struts.freed());
        log.lock().unwrap().clear();
        camera.translate(1.0, 2.0);
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|event| event.starts_with("matrix:"))
        );
        runtime.run_main();
        log.lock().unwrap().clear();
        camera.translate(1.0, 2.0);
        assert!(log.lock().unwrap().is_empty());
        context.close();
        runtime.run_main();
    }
    #[test]
    fn source_geometry_struts_preserve_source_order_and_mask_channel() {
        let occupied = [true; 4];
        let masks = [[8, 7, 0, 0], [8, 28, 0, 0], [8, 193, 0, 0], [8, 112, 0, 0]];
        assert_eq!(
            ShipStruts::live_indices(2, 2, &occupied, &masks),
            vec![0, 1, 0, 3, 0, 2, 2, 3, 1, 3, 1, 2]
        );
        let mut broken = masks;
        broken[0][1] &= !2;
        assert_eq!(
            ShipStruts::live_indices(2, 2, &occupied, &broken),
            vec![0, 1, 0, 2, 2, 3, 1, 3, 1, 2]
        );
        assert!(ShipStruts::live_indices(2, 2, &occupied, &[[8, 0, 255, 0]; 4]).is_empty());
    }
}

//! Ship geometry-shader expansion adapted to live GPU vertex evaluation.
//! Two live triangle slots preserve the source quad/fallback emission order.
use crate::*;

#[derive(Component)]
pub(crate) struct GpuGeometry;

struct Occupancy<'a>(usize, usize, &'a [bool]);
impl crate::ship_companion::ShipGeometryData for Occupancy<'_> {
    fn width(&self) -> i32 {
        self.0 as i32
    }
    fn height(&self) -> i32 {
        self.1 as i32
    }
    fn occupied(&self, index: i32) -> Result<bool, String> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.2.get(index))
            .copied()
            .ok_or_else(|| "source strut index outside material list".into())
    }
}

pub(crate) fn build_struts_mesh(structure: &ShipStructure) -> Mesh {
    structure.validate_texel_data();
    let width = structure.texel_width;
    let height = structure.texel_height;
    let indices = crate::ship_struts_companion::create_indices(&Occupancy(
        width,
        height,
        &structure.texel_solid,
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut metadata = Vec::new();
    for edge in indices.chunks_exact(2) {
        let a = edge[0] as usize;
        let b = edge[1] as usize;
        let x = a % width;
        let y = a / width;
        let dx = (b % width) as i32 - x as i32;
        let dy = (b / width) as i32 - y as i32;
        let bit = if dx == 1 { dy } else { 2 - dx };
        for index in [a, b] {
            let position = structure.texel_rest_positions[index];
            positions.push([position.x, position.y, 0.0]);
            uvs.push([
                ((index % width) as f32 + 0.5) / width as f32,
                ((index / width) as f32 + 0.5) / height as f32,
            ]);
            metadata.push([x as f32, y as f32, 8.0 + bit as f32]);
        }
    }
    if positions.is_empty() {
        positions = vec![[0.0; 3]; 2];
        uvs = vec![[0.0; 2]; 2];
        metadata = vec![[0.0, 0.0, -1.0]; 2];
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, metadata);
    mesh
}

pub(crate) const TRIANGLES: [[usize; 3]; 6] = [
    [0, 1, 2],
    [2, 1, 3], // source full triangle strip
    [0, 1, 3],
    [1, 3, 2],
    [0, 1, 2],
    [0, 3, 2],
];

pub(crate) fn enabled(kind: usize, s: [u32; 4]) -> bool {
    let full = s[1] & 7 == 7 && s[0] & 1 != 0 && s[3] & 12 == 12;
    match kind {
        0 | 1 => full,
        2 => !full && s[1] & 5 == 5 && s[3] & 8 != 0,
        3 => !full && s[1] & 3 == 3 && s[3] & 4 != 0,
        4 => !full && s[1] & 6 == 6 && s[0] & 1 != 0,
        5 => !full && s[3] & 12 == 12 && s[0] & 1 != 0,
        _ => false,
    }
}

pub(crate) fn build_mesh(structure: &ShipStructure) -> Mesh {
    structure.validate_texel_data();
    let width = structure.texel_width;
    let height = structure.texel_height;
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut metadata = Vec::new();
    for y in 0..height.saturating_sub(1) {
        for x in 0..width.saturating_sub(1) {
            let anchor = y * width + x;
            let corners = [anchor + width, anchor, anchor + width + 1, anchor + 1];
            // Ship.createIndices keeps a point if any of its four corners has material.
            if !corners
                .iter()
                .any(|index| structure.texel_materials[*index].is_some())
            {
                continue;
            }
            for (slot, triangle) in TRIANGLES[..2].iter().enumerate() {
                for (lane, &corner) in triangle.iter().enumerate() {
                    let index = corners[corner];
                    let position = structure.texel_rest_positions[index];
                    positions.push([position.x, position.y, 0.0]);
                    uvs.push([lane as f32, 0.0]);
                    // Normal is unused by the source fragment. Carry exact cell/branch identity.
                    metadata.push([x as f32, y as f32, slot as f32]);
                }
            }
        }
    }
    if positions.is_empty() {
        positions = vec![[0.0; 3]; 3];
        uvs = vec![[0.0; 2]; 3];
        metadata = vec![[0.0, 0.0, -1.0]; 3];
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, metadata);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0; 4]; mesh.count_vertices()]);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    fn strut_fixture() -> ShipStructure {
        let mut structure = ShipStructure::empty_fallback();
        structure.texel_width = 2;
        structure.texel_height = 2;
        structure.texel_solid = vec![true; 4];
        structure.texel_strut_masks = vec![255; 4];
        structure.texel_materials =
            vec![Some(MaterialProperties::from(&materials::Material::default())); 4];
        structure.texel_rest_positions = vec![
            Vec2::new(3.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, -1.0),
        ];
        structure.texel_positions = vec![Vec2::new(99.0, 99.0); 4];
        structure
    }
    #[test]
    fn strut_metadata_matches_source_edge_order_and_all_root_masks() {
        let structure = strut_fixture();
        let mesh = build_struts_mesh(&structure);
        assert_eq!(mesh.count_vertices(), 12);
        assert!(mesh.indices().is_none());
        let Some(VertexAttributeValues::Float32x3(metadata)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("no edge metadata")
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("no texel UVs")
        };
        let expected = [
            [0.0, 0.0, 8.0],
            [0.0, 0.0, 9.0],
            [0.0, 0.0, 10.0],
            [0.0, 1.0, 8.0],
            [1.0, 0.0, 10.0],
            [1.0, 0.0, 11.0],
        ];
        for (pair, expected) in metadata.chunks_exact(2).zip(expected) {
            assert_eq!(pair, [expected, expected]);
        }
        for root in 0..256u32 {
            let mut masks = vec![[0, 255, 0, 0]; 4];
            masks[0][1] = root;
            let actual: Vec<_> = metadata
                .chunks_exact(2)
                .zip(uvs.chunks_exact(2))
                .filter(|(pair, _)| {
                    let anchor = pair[0][1] as usize * 2 + pair[0][0] as usize;
                    masks[anchor][1] & (1 << (pair[0][2] as u32 - 8)) != 0
                })
                .flat_map(|(_, uv)| {
                    uv.iter()
                        .map(|uv| (uv[1] * 2.0) as u32 * 2 + (uv[0] * 2.0) as u32)
                })
                .collect();
            assert_eq!(
                actual,
                crate::ship_struts::ShipStruts::live_indices(2, 2, &structure.texel_solid, &masks)
            );
        }
    }
    #[test]
    fn live_strut_sync_ignores_readback_and_rebuilds_when_source_occupancy_changes() {
        let structure = strut_fixture();
        let initial = build_struts_mesh(&structure);
        let positions = initial.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().clone();
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<GpuShipPhysicsSnapshot>()
            .insert_resource(structure)
            .add_systems(Update, crate::ship_struts::sync_ship_struts);
        let handle = app.world_mut().resource_mut::<Assets<Mesh>>().add(initial);
        let entity = app
            .world_mut()
            .spawn((
                crate::ship_struts::ShipStruts {
                    mesh: handle.clone(),
                    dimensions: (2, 2),
                    masks: vec![],
                    occupied: vec![true; 4],
                },
                GpuGeometry,
                Transform::default(),
            ))
            .id();
        app.world_mut().spawn((
            ShipMesh(Handle::default(), Handle::default()),
            Transform::from_xyz(40.0, 30.0, 2.0),
        ));
        app.world_mut()
            .resource_mut::<GpuShipPhysicsSnapshot>()
            .masks = vec![[0; 4]; 4];
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&handle)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION),
            Some(&positions)
        );
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().translation,
            // Source struts follow the hull draw; coverage now handles masking.
            Vec3::new(40.0, 30.0, 2.001)
        );
        let mut structure = app.world_mut().resource_mut::<ShipStructure>();
        structure.texel_solid = vec![false; 4];
        structure.texel_materials = vec![None; 4];
        drop(structure);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&handle)
                .unwrap()
                .count_vertices(),
            2
        );
        assert_eq!(
            app.world()
                .get::<crate::ship_struts::ShipStruts>(entity)
                .unwrap()
                .occupied,
            vec![false; 4]
        );
    }
    #[test]
    fn mesh_keeps_occupied_source_anchors_and_two_live_triangle_slots() {
        let mut structure = ShipStructure::empty_fallback();
        structure.texel_width = 2;
        structure.texel_height = 2;
        structure.texel_solid = vec![false, false, false, true];
        structure.texel_strut_masks = vec![0; 4];
        structure.texel_materials = vec![
            None,
            None,
            None,
            Some(MaterialProperties::from(&materials::Material::default())),
        ];
        structure.texel_rest_positions = vec![
            Vec2::new(3.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, -1.0),
        ];
        let mesh = build_mesh(&structure);
        assert_eq!(mesh.count_vertices(), 6);
        assert!(
            mesh.indices().is_none(),
            "source live topology is evaluated on the GPU"
        );
        let Some(VertexAttributeValues::Float32x3(metadata)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing live cell metadata")
        };
        assert_eq!(
            metadata,
            &vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0]
            ]
        );
        let Some(VertexAttributeValues::Float32x2(lanes)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("missing source vertex lanes")
        };
        assert_eq!(
            lanes,
            &vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [2.0, 0.0],
                [0.0, 0.0],
                [1.0, 0.0],
                [2.0, 0.0]
            ]
        );
        structure.texel_solid = vec![false; 4];
        structure.texel_materials = vec![None; 4];
        let empty = build_mesh(&structure);
        assert_eq!(empty.count_vertices(), 3);
        let Some(VertexAttributeValues::Float32x3(metadata)) =
            empty.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing empty sentinel")
        };
        assert!(metadata.iter().all(|value| value[2] < 0.0));
    }
    #[test]
    fn live_vertex_shader_validates_for_ship_and_strut_attribute_layouts() {
        let source = include_str!("../assets/shaders/ship_gpu_vertex.wgsl");
        let source = source.split_once("}\n").unwrap().1;
        for normals in [false, true] {
            for colors in [false, true] {
                let mut selected = String::new();
                let mut enabled = true;
                for line in source.lines() {
                    if line.starts_with("#ifdef ") {
                        enabled = match line.trim_start_matches("#ifdef ") {
                            "VERTEX_NORMALS" => normals,
                            "VERTEX_COLORS" => colors,
                            _ => panic!("unexpected shader definition"),
                        };
                    } else if line.starts_with("#endif") {
                        enabled = true;
                    } else if enabled {
                        selected.push_str(line);
                        selected.push('\n');
                    }
                }
                let stubs = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}
fn get_world_from_local(instance: u32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0,0.0,0.0,0.0),vec4<f32>(0.0,1.0,0.0,0.0),vec4<f32>(0.0,0.0,1.0,0.0),vec4<f32>(0.0,0.0,0.0,1.0));
}
fn mesh2d_position_local_to_world(matrix: mat4x4<f32>,position: vec4<f32>) -> vec4<f32> { return matrix*position; }
fn mesh2d_position_world_to_clip(position: vec4<f32>) -> vec4<f32> { return position; }
"#;
                let composed = format!(
                    "{stubs}\n{}",
                    selected
                        .replace("#{MATERIAL_BIND_GROUP}", "0")
                        .replace("mesh_functions::", "")
                );
                let module = naga::front::wgsl::parse_str(&composed).unwrap();
                naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                )
                .validate(&module)
                .unwrap();
            }
        }
    }
    #[test]
    fn live_geometry_branch_selection_matches_source_for_all_local_mask_combinations() {
        // Only these four bits participate in the source's four-corner emission.
        for a in 0..16u32 {
            for b in 0..16u32 {
                for c in 0..16u32 {
                    let s = [a, b, 0, c];
                    let masks = [[0, 0, b, 0], [0, 0, c, 0], [0, 0, a, 0], [0; 4]];
                    let corners = [2u32, 0, 3, 1];
                    let actual: Vec<_> = TRIANGLES
                        .iter()
                        .enumerate()
                        .filter(|(kind, _)| enabled(*kind, s))
                        .flat_map(|(_, tri)| tri.map(|i| corners[i]))
                        .collect();
                    assert!(
                        actual.len() <= 6,
                        "source never emits more than its declared six vertices"
                    );
                    assert_eq!(actual, ship::Ship::triangle_indices(2, 2, &masks), "{s:?}");
                }
            }
        }
    }
}

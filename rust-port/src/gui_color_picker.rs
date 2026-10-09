//! Drawing primitives from bundled widgetsColorEditorPicker and DrawList.
//! Coordinates use Bevy's upward Y axis; vertex/index order follows ImGui.
use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

/// Generic_helpersKt.F32_TO_INT8_SAT, followed by GL's normalized UBYTE read.
pub(crate) fn packed_channel(value: f32) -> f32 {
    ((value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8) as f32 / 255.0
}
fn packed(rgb: Vec3, alpha: f32) -> [f32; 4] {
    [
        packed_channel(rgb.x),
        packed_channel(rgb.y),
        packed_channel(rgb.z),
        packed_channel(alpha),
    ]
}
#[derive(Default)]
struct Draw {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}
impl Draw {
    fn rect(&mut self, left: f32, top: f32, right: f32, bottom: f32, colors: [[f32; 4]; 4]) {
        let base = self.positions.len() as u32;
        self.positions.extend([
            [left, top, 0.0],
            [right, top, 0.0],
            [right, bottom, 0.0],
            [left, bottom, 0.0],
        ]);
        self.colors.extend(colors);
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}
pub(crate) fn saturation_value(size: Vec2, hue: Vec3) -> Mesh {
    let mut draw = Draw::default();
    let half = size * 0.5;
    let white = [1.0; 4];
    let hue = packed(hue, 1.0);
    // Two draw calls in source order: white-to-hue, then transparent-to-black.
    // Blending the second quad also preserves the source's alpha equation.
    draw.rect(-half.x, half.y, half.x, -half.y, [white, hue, hue, white]);
    draw.rect(
        -half.x,
        half.y,
        half.x,
        -half.y,
        [
            [0.0; 4],
            [0.0; 4],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    );
    draw.mesh()
}
pub(crate) fn hue(size: Vec2) -> Mesh {
    let mut draw = Draw::default();
    let half = size * 0.5;
    let colors = [
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
    ];
    for index in 0..6 {
        let top = half.y - index as f32 * (size.y / 6.0);
        let bottom = half.y - (index + 1) as f32 * (size.y / 6.0);
        draw.rect(
            -half.x,
            top,
            half.x,
            bottom,
            [
                colors[index],
                colors[index],
                colors[index + 1],
                colors[index + 1],
            ],
        );
    }
    draw.mesh()
}
pub(crate) fn alpha(size: Vec2, rgb: Vec3) -> Mesh {
    let mut draw = Draw::default();
    let half = size * 0.5;
    let step = size.x / 2.0;
    // renderColorRectWithAlphaCheckerboard: gray 204 background, then gray
    // 128 square cells, starting at the top left, with clipped final cells.
    draw.rect(
        -half.x,
        half.y,
        half.x,
        -half.y,
        [[204.0 / 255.0, 204.0 / 255.0, 204.0 / 255.0, 1.0]; 4],
    );
    let mut row = 0;
    let mut top = half.y;
    while top > -half.y {
        let bottom = (top - step).max(-half.y);
        let left = -half.x + (row % 2) as f32 * step;
        draw.rect(
            left,
            top,
            (left + step).min(half.x),
            bottom,
            [[128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0]; 4],
        );
        row += 1;
        top = bottom;
    }
    let opaque = packed(rgb, 1.0);
    let transparent = packed(rgb, 0.0);
    draw.rect(
        -half.x,
        half.y,
        half.x,
        -half.y,
        [opaque, opaque, transparent, transparent],
    );
    draw.mesh()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn colors(mesh: &Mesh) -> &Vec<[f32; 4]> {
        let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("Source packed colors")
        };
        colors
    }
    #[test]
    fn packed_colors_saturate_round_and_normalize_source_bytes() {
        for (input, byte) in [
            (-1.0, 0),
            (0.0, 0),
            (0.5, 128),
            (0.33333334, 85),
            (1.0, 255),
            (2.0, 255),
        ] {
            assert_eq!(packed_channel(input), byte as f32 / 255.0);
        }
    }
    #[test]
    fn saturation_value_uses_two_ordered_source_quads_and_packed_hue() {
        let mesh = saturation_value(Vec2::new(188.0, 360.0), Vec3::new(0.0, 1.0 / 3.0, 1.0));
        assert_eq!(mesh.count_vertices(), 8);
        assert_eq!(
            mesh.indices().unwrap().iter().collect::<Vec<_>>(),
            [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]
        );
        assert_eq!(colors(&mesh)[1], [0.0, 85.0 / 255.0, 1.0, 1.0]);
        assert_eq!(
            &colors(&mesh)[4..],
            &[
                [0.0; 4],
                [0.0; 4],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        );
    }
    #[test]
    fn hue_is_six_continuous_source_segments_and_alpha_uses_square_checks() {
        let bar = hue(Vec2::new(18.0, 360.0));
        assert_eq!(bar.count_vertices(), 24);
        assert_eq!(colors(&bar)[0], [1.0, 0.0, 0.0, 1.0]);
        for index in 0..5 {
            assert_eq!(colors(&bar)[index * 4 + 2], colors(&bar)[(index + 1) * 4]);
        }
        let bar = alpha(Vec2::new(14.0, 360.0), Vec3::new(0.1, 0.2, 0.3));
        assert_eq!(bar.count_vertices(), (1 + 52 + 1) * 4);
        let colors = colors(&bar);
        assert_eq!(colors[0], [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(
            colors[4],
            [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0]
        );
        assert_eq!(colors.last().unwrap()[3], 0.0);
    }
}

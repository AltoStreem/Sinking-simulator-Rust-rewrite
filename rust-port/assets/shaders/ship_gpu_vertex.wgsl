#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var<storage, read> masks: array<vec4<u32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var<storage, read> positions: array<vec4<f32>>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
#ifdef VERTEX_NORMALS
    @location(1) metadata: vec3<f32>,
#endif
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};

fn source_triangle_enabled(kind: u32, anchor: u32, width: u32) -> bool {
    let s0 = masks[anchor + width].z;
    let s1 = masks[anchor].z;
    let s3 = masks[anchor + 1u].z;
    let full = (s1 & 7u) == 7u && (s0 & 1u) != 0u && (s3 & 12u) == 12u;
    if kind < 2u { return full; }
    if full { return false; }
    switch kind {
        case 2u: { return (s1 & 5u) == 5u && (s3 & 8u) != 0u; }
        case 3u: { return (s1 & 3u) == 3u && (s3 & 4u) != 0u; }
        case 4u: { return (s1 & 6u) == 6u && (s0 & 1u) != 0u; }
        case 5u: { return (s3 & 12u) == 12u && (s0 & 1u) != 0u; }
        default: { return false; }
    }
}

fn source_triangle_kind(slot: u32, anchor: u32, width: u32) -> u32 {
    var emitted = 0u;
    for (var kind = 0u; kind < 6u; kind += 1u) {
        if source_triangle_enabled(kind, anchor, width) {
            if emitted == slot { return kind; }
            emitted += 1u;
        }
    }
    return 6u;
}

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    let dimensions = vec2<u32>(params.zw);
    var uv = input.uv;
    var emitted_vertex = true;
#ifdef VERTEX_NORMALS
    if input.metadata.z < 0.0 {
        emitted_vertex = false;
        uv = vec2<f32>(0.0);
    } else if input.metadata.z >= 8.0 {
        let anchor_xy = vec2<u32>(input.metadata.xy);
        let anchor = anchor_xy.y * dimensions.x + anchor_xy.x;
        let bit = u32(input.metadata.z - 8.0);
        emitted_vertex = (masks[anchor].y & (1u << bit)) != 0u;
    } else {
        let anchor_xy = vec2<u32>(input.metadata.xy);
        let anchor = anchor_xy.y * dimensions.x + anchor_xy.x;
        let kind = source_triangle_kind(u32(input.metadata.z), anchor, dimensions.x);
        if kind == 6u {
            emitted_vertex = false;
            uv = vec2<f32>(0.0);
        } else {
            let corners = array<u32,18>(0u,1u,2u,2u,1u,3u,0u,1u,3u,1u,3u,2u,0u,1u,2u,0u,3u,2u);
            let offsets = array<vec2<u32>,4>(vec2<u32>(0u,1u),vec2<u32>(0u,0u),vec2<u32>(1u,1u),vec2<u32>(1u,0u));
            let corner = corners[kind * 3u + u32(input.uv.x)];
            uv = (vec2<f32>(anchor_xy + offsets[corner]) + vec2<f32>(0.5)) / params.zw;
        }
    }
#endif
    let texel = min(vec2<u32>(uv * params.zw), dimensions - vec2<u32>(1u));
    let index = texel.y * dimensions.x + texel.x;
    let world_from_local = mesh_functions::get_world_from_local(input.instance_index);
    var out: VertexOutput;
    out.uv = uv;
    out.world_normal = vec3<f32>(0.0, 0.0, 1.0);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(world_from_local, vec4<f32>(positions[index].xy, 0.0, 1.0));
    out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
    // All three vertices of an inactive slot collapse to the same clip point.
    if !emitted_vertex {
        // Outside the near plane also suppresses zero-length line rasterization.
        out.position = vec4<f32>(0.0, 0.0, -2.0, 1.0);
    }
#ifdef VERTEX_COLORS
    out.color = input.color;
#endif
    return out;
}

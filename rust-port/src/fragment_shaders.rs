//! Rust counterparts of `FragmentShaders.java` and Ship's texture fragment.
//! The generic fragment bodies were recovered from the extracted lambda
//! classes because CFR omitted their bodies from FragmentShaders.java.
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{render_resource::AsBindGroup, storage::ShaderBuffer},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

pub(crate) struct FragmentShaders;

impl FragmentShaders {
    pub(crate) fn white() -> Vec4 {
        Vec4::ONE
    }
    pub(crate) fn uv(uv: Vec2) -> Vec4 {
        Vec4::new(uv.x, uv.y, 0.0, 1.0)
    }
    pub(crate) fn texture(sample: Vec4) -> Vec4 {
        sample
    }

    /// Source texture fragment evaluated in the original RGB sample space.
    pub(crate) fn ship_color(
        mut color: Vec4,
        internal: Vec4,
        external: Vec4,
        flags: u32,
        water: f32,
        day: f32,
        sea: Vec3,
    ) -> Vec4 {
        let t = ((day + 0.2) / 1.0).clamp(0.0, 1.0);
        let daylight = t * t * (3.0 - 2.0 * t);
        let brightness =
            (Vec3::splat(daylight) + external.truncate() * external.w).clamp(Vec3::ZERO, Vec3::ONE);
        if flags & 2 == 0 {
            color = color.lerp(sea.extend(1.0), water.clamp(0.0, 1.0) * 0.75);
        }
        let rgb = color.truncate() * brightness
            + internal.truncate() * internal.w * (Vec3::ONE - brightness);
        rgb.extend(color.w)
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct ShipMaterial {
    /// day, show flood water, BASE width, BASE height.
    #[uniform(0)]
    pub(crate) params: Vec4,
    #[uniform(1)]
    pub(crate) sea_color: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub(crate) texture: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub(crate) internal_lights: Handle<Image>,
    #[texture(6)]
    #[sampler(7)]
    pub(crate) external_lights: Handle<Image>,
    #[storage(8, read_only)]
    pub(crate) water: Handle<ShaderBuffer>,
    #[storage(9, read_only)]
    pub(crate) masks: Handle<ShaderBuffer>,
}

impl Material2d for ShipMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ship_texture.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub(crate) fn update_ship_lighting(
    simulation: Res<crate::Simulation>,
    physics: Res<crate::GpuShipPhysicsAssets>,
    mut materials: ResMut<Assets<ShipMaterial>>,
) {
    for (_, material) in materials.iter_mut() {
        material.params = Vec4::new(
            simulation.day,
            u8::from(simulation.show_internal_water) as f32,
            physics.width as f32,
            physics.height as f32,
        );
        material.sea_color = simulation.water_color();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_fragment_flooding_changes_non_hull_color_and_alpha_only() {
        let base = Vec4::new(0.8, 0.4, 0.2, 0.2);
        let sea = Vec3::new(0.0, 0.278431373, 0.623529411);
        let wet = FragmentShaders::ship_color(base, Vec4::ZERO, Vec4::ZERO, 8, 1.0, 1.0, sea);
        assert!(wet.distance(base.lerp(sea.extend(1.0), 0.75)) < 0.00001);
        assert_eq!(
            FragmentShaders::ship_color(base, Vec4::ZERO, Vec4::ZERO, 10, 1.0, 1.0, sea),
            base
        );
    }

    #[test]
    fn source_fragment_internal_lights_survive_night_and_external_lights_restore_brightness() {
        let base = Vec4::new(0.8, 0.4, 0.2, 0.7);
        let internal = Vec4::new(0.4, 0.2, 0.1, 0.5);
        let night =
            FragmentShaders::ship_color(base, internal, Vec4::ZERO, 10, 0.0, -0.2, Vec3::ZERO);
        assert!(night.distance(Vec4::new(0.2, 0.1, 0.05, 0.7)) < 0.00001);
        assert_eq!(
            FragmentShaders::ship_color(base, internal, Vec4::ONE, 10, 0.0, -0.2, Vec3::ZERO),
            base
        );
    }

    #[test]
    fn source_fragment_wgsl_parses_and_validates() {
        let shader = include_str!("../assets/shaders/ship_texture.wgsl");
        let body = shader
            .lines()
            .filter(|line| !line.starts_with("#import"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("#{MATERIAL_BIND_GROUP}", "2");
        // Installed Bevy Mesh2d VertexOutput interface without optional fields.
        let interface = "struct VertexOutput { @builtin(position) position:vec4<f32>, @location(0) world_position:vec4<f32>, @location(1) world_normal:vec3<f32>, @location(2) uv:vec2<f32>, };\n";
        let source = format!("{interface}{body}");
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("source material shader must type-check");
    }
}

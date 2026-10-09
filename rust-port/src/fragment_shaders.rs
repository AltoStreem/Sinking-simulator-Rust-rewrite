//! Rust counterparts of `FragmentShaders.java` and Ship's texture fragment.
//! The generic fragment bodies were recovered from the extracted lambda
//! classes because CFR omitted their bodies from FragmentShaders.java.
use bevy::{
    prelude::*,
    reflect::TypePath,
    mesh::MeshVertexBufferLayoutRef,
    render::{render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation,
        BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError}, storage::ShaderBuffer},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey},
};

pub(crate) struct FragmentShaders;

/// The three source Kotlin synchronized lazy shader delegates. Their default
/// binding lists are empty; the fragment output is not explicitly rebound.
#[allow(dead_code)]
pub(crate) struct SourceFragmentShaders {
    backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
    context: crate::resource::ResourceHandle,
    runtime: crate::resource::ResourceRuntime,
    slots: [std::sync::Mutex<Option<std::sync::Arc<crate::shader::Shader>>>; 3],
}
#[allow(dead_code)]
impl SourceFragmentShaders {
    pub fn new(
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::shader::ShaderBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        Self {
            backend,
            context,
            runtime: runtime.clone(),
            slots: std::array::from_fn(|_| std::sync::Mutex::new(None)),
        }
    }
    fn get(
        &self,
        index: usize,
        source: &str,
    ) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        let mut slot = self.slots[index].lock().unwrap();
        if let Some(shader) = &*slot {
            return Ok(shader.clone());
        }
        let shader = std::sync::Arc::new(crate::shader::Shader::new(
            source,
            35632,
            vec![],
            self.backend.clone(),
            self.context.clone(),
            &self.runtime,
        )?);
        *slot = Some(shader.clone());
        Ok(shader)
    }
    pub fn white(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(0, crate::fragment_shaders_white::SOURCE)
    }
    pub fn uv(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(1, crate::fragment_shaders_uv::SOURCE)
    }
    pub fn texture(&self) -> Result<std::sync::Arc<crate::shader::Shader>, String> {
        self.get(2, crate::fragment_shaders_texture::SOURCE)
    }
}

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
    #[storage(10, read_only)]
    pub(crate) positions: Handle<ShaderBuffer>,
    #[uniform(11)]
    pub(crate) coverage_mode: Vec4,
    #[texture(12)]
    pub(crate) coverage: Handle<Image>,
}

impl Material2d for ShipMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/ship_gpu_vertex.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/ship_texture.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(source_ship_blend());
            }
        }
        Ok(())
    }
}

fn source_ship_blend() -> BlendState {
    // Ship.render uses glBlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA), which
    // applies to alpha as well as RGB. Bevy's default uses ONE for source alpha.
    let component = BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };
    BlendState { color: component, alpha: component }
}

pub(crate) fn update_ship_lighting(
    simulation: Res<crate::Simulation>,
    physics: Res<crate::GpuShipPhysicsAssets>,
    mut materials: ResMut<Assets<ShipMaterial>>,
    active: Query<&MeshMaterial2d<ShipMaterial>, Or<(With<crate::ShipMesh>, With<crate::ship_struts::ShipStruts>)>>,
) {
    for handle in &active {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            crate::ship_visual_replacement::bind(&mut material, &physics, &simulation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn source_ship_blending_retains_source_alpha_for_sea_reflections() {
        let blend = source_ship_blend();
        assert_eq!(blend.color, blend.alpha);
        assert_eq!(blend.alpha.src_factor, BlendFactor::SrcAlpha);
        assert_eq!(blend.alpha.dst_factor, BlendFactor::OneMinusSrcAlpha);
        for alpha in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let source_alpha = alpha * alpha + 1.0 * (1.0 - alpha);
            assert!((source_alpha - (1.0 - alpha * (1.0 - alpha))).abs() < 1e-6);
            if alpha > 0.0 && alpha < 1.0 { assert!(source_alpha < 1.0); }
        }
        let ship = include_str!("../assets/shaders/ship_texture.wgsl");
        let sky = include_str!("../assets/shaders/sky.wgsl");
        let sea = include_str!("../assets/shaders/sea.wgsl");
        assert!(!ship.contains("output_rgb("));
        assert!(ship.contains("return color;"));
        assert!(sky.contains("return vec4<f32>(rgb, 1.0);"));
        assert!(sea.contains("return vec4<f32>(linear, max(prev.a, color.a));"));
    }
    #[derive(Default)]
    struct Compiler {
        sources: Vec<String>,
        fail: bool,
    }
    impl crate::shader::ShaderBackend for Compiler {
        fn create_shader(&mut self, kind: i32) -> i32 {
            assert_eq!(kind, 35632);
            self.sources.len() as i32 + 1
        }
        fn shader_source(&mut self, _: i32, source: &str) {
            self.sources.push(source.to_owned());
        }
        fn compile_shader(&mut self, _: i32) {}
        fn shader_info_log(&mut self, _: i32) -> String {
            String::new()
        }
        fn shader_compiled(&mut self, _: i32) -> bool {
            !self.fail
        }
        fn attach_shader(&mut self, _: i32, _: i32) {}
        fn detach_shader(&mut self, _: i32, _: i32) {}
        fn delete_shader(&mut self, _: i32) {}
    }
    #[test]
    fn source_fragment_lazy_registry_preserves_bodies_empty_bindings_and_retry() {
        let runtime = crate::resource::ResourceRuntime::default();
        let backend = Arc::new(Mutex::new(Compiler::default()));
        let shaders =
            SourceFragmentShaders::new(backend.clone(), runtime.allocate(&[], || {}), &runtime);
        assert!(backend.lock().unwrap().sources.is_empty());
        backend.lock().unwrap().fail = true;
        assert!(shaders.texture().is_err());
        backend.lock().unwrap().fail = false;
        let texture = shaders.texture().unwrap();
        let white = shaders.white().unwrap();
        let uv = shaders.uv().unwrap();
        assert!(Arc::ptr_eq(&texture, &shaders.texture().unwrap()));
        assert!(Arc::ptr_eq(&white, &shaders.white().unwrap()));
        assert!(Arc::ptr_eq(&uv, &shaders.uv().unwrap()));
        assert!(!Arc::ptr_eq(&white, &uv));
        assert!(texture.bindings.is_empty() && white.bindings.is_empty() && uv.bindings.is_empty());
        assert_eq!(
            backend.lock().unwrap().sources,
            [
                crate::fragment_shaders_texture::SOURCE,
                crate::fragment_shaders_texture::SOURCE,
                crate::fragment_shaders_white::SOURCE,
                crate::fragment_shaders_uv::SOURCE
            ]
        );
        texture.close();
        assert!(Arc::ptr_eq(&texture, &shaders.texture().unwrap()));
    }

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

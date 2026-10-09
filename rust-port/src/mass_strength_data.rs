//! Translation of SS2 `MassStrengthDataHolder` GPU packing.
use crate::ShipStructure;

#[allow(dead_code)]
pub(crate) struct SourceMassStrengthDataHolder {
    texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>,
    pub(crate) air: f32,
    pub(crate) water: f32,
}
#[allow(dead_code)]
impl SourceMassStrengthDataHolder {
    pub(crate) fn create_mass<
        P: crate::ship_data::SourceMaterialLookup<
                Material = std::sync::Arc<crate::materials::Material>,
            >,
    >(
        dat: &crate::ship_data::SourceShipData<P>,
        air: f32,
        water: f32,
    ) -> Result<crate::image_data::ImageData, String> {
        let count = dat.width.wrapping_mul(dat.height);
        let count =
            usize::try_from(count).map_err(|_| "negative source mass array size".to_owned())?;
        let mut channels = [
            Vec::with_capacity(count),
            Vec::with_capacity(count),
            Vec::with_capacity(count),
            Vec::with_capacity(count),
        ];
        // Source builds four complete arrays before interleaving their values.
        for (channel, values) in channels.iter_mut().enumerate() {
            for index in 0..count {
                let materials = dat.material_buffer.borrow();
                let material = materials
                    .get(index)
                    .ok_or_else(|| "source material list index out of bounds".to_owned())?;
                let value = material.as_ref().map_or(0.0, |material| match channel {
                    0 => material.mass,
                    1 => material.tensile_strength.unwrap_or(material.strength),
                    2 => material
                        .compressive_strength
                        .unwrap_or(material.strength * 4.0),
                    _ => {
                        if material.is_hull || material.is_ground {
                            material.mass * 0.1 + water * 80.0
                        } else {
                            material.mass * 0.1 + 0.9 * air
                        }
                    }
                });
                values.push(value);
            }
        }
        let mut bytes = Vec::with_capacity(count * 16);
        for index in 0..count {
            for values in &channels {
                bytes.extend_from_slice(&values[index].to_ne_bytes());
            }
        }
        let mut image = crate::image_data::ImageData::new(bytes, dat.width, dat.height, 6408);
        image.format = 5126;
        Ok(image)
    }
    pub(crate) fn new<
        P: crate::ship_data::SourceMaterialLookup<
                Material = std::sync::Arc<crate::materials::Material>,
            >,
    >(
        dat: &crate::ship_data::SourceShipData<P>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        // Bytecode invokes createMass at offset 17, then assigns AIR at 40
        // and WATER at 46. JVM instance fields are zero before those writes.
        let image = Self::create_mass(dat, 0.0, 0.0)?;
        let texture = crate::texture_2d::SourceTexture2D::from_image(
            &image,
            34836,
            false,
            std::sync::Arc::new(crate::mass_strength_data_texture_config::configure),
            backend,
            context,
            runtime,
        );
        Ok(Self {
            texture: std::sync::Arc::new(texture),
            air: 1.225,
            water: 1025.0,
        })
    }
}
impl crate::gl_data_holder::SourceGlDataHolder for SourceMassStrengthDataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        self.texture.clone()
    }
}

/// Match `MassStrengthDataHolder.createMass` exactly: R is raw mass, G is
/// tensile strength, B is compressive strength, and A is buoyancy-adjusted
/// mass. Source physics samples A for springs and inertia; water updates A
/// from the unchanged raw mass R. Neither strength channel is overwritten.
pub(super) fn gpu_material_data(structure: &ShipStructure) -> Vec<[f32; 4]> {
    structure
        .texel_materials
        .iter()
        .map(|material| {
            material.map_or([0.0; 4], |material| {
                // The original constructor builds the upload before assigning
                // its AIR/WATER fields. Both fluid terms initially use zero.
                let weighted_mass = material.mass * 0.1 + 0.0;
                [
                    material.mass,
                    material.tensile_strength,
                    material.compressive_strength,
                    weighted_mass,
                ]
            })
        })
        .collect()
}

/// Reference of the source mass update shader; only alpha/effective mass changes.
#[cfg(test)]
fn update_effective_mass(
    mut channels: [f32; 4],
    hull: bool,
    water: f32,
    water_weight: f32,
    thickness: f32,
) -> [f32; 4] {
    let wet = water.clamp(0.0, 1.0);
    let fluid = 1.225 + (1025.0 * if hull { 1.0 } else { water_weight } - 1.225) * wet;
    channels[3] = channels[0] * (1.0 - thickness) + fluid * thickness;
    channels
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_data_holder::SourceGlDataHolder;
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    #[test]
    fn native_mass_constructor_uploads_before_density_fields_are_initialized() {
        let palette = Arc::new(crate::materials::SourceMaterials::from_json(r##"[
          {"color":"#123456","mass":7800,"strength":65,"isHull":true},
          {"color":"#abcdef","mass":100,"strength":7,"tensileStrength":0,"compressiveStrength":-2,"invisible":true},
          {"color":"#010203","mass":200,"strength":3,"isGround":true}
        ]"##).unwrap());
        let dat = crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            palette.clone(),
            Rc::new(RefCell::new(vec![
                palette.get(0x123456),
                palette.get(0xabcdef),
                palette.get(0x010203),
                None,
            ])),
            4,
            1,
        );
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(Vec::new()));
        let holder = SourceMassStrengthDataHolder::new(
            &dat,
            crate::render_fbo::source_tests::backend(log.clone()),
            context.clone(),
            &runtime,
        )
        .unwrap();
        assert_eq!((holder.air, holder.water), (1.225, 1025.0));
        let texture = holder.source_texture();
        assert!(Arc::ptr_eq(&texture, &holder.source_texture()));
        let bytes = texture.img.as_ref().unwrap().lock().unwrap();
        let values: Vec<_> = bytes
            .chunks_exact(4)
            .map(|bytes| f32::from_ne_bytes(bytes.try_into().unwrap()))
            .collect();
        assert_eq!(
            values,
            vec![
                7800., 65., 260., 780., 100., 0., -2., 10., 200., 3., 12., 20., 0., 0., 0., 0.
            ]
        );
        drop(bytes);
        let after =
            SourceMassStrengthDataHolder::create_mass(&dat, holder.air, holder.water).unwrap();
        let values: Vec<_> = after
            .buffer
            .chunks_exact(4)
            .map(|bytes| f32::from_ne_bytes(bytes.try_into().unwrap()))
            .collect();
        assert_eq!(values[3], 82780.0);
        assert_eq!(values[7], 100.0 * 0.1 + 0.9 * 1.225);
        assert_eq!(values[11], 82020.0);
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"image2:1:3553:34836:[4, 1]:6408:5126:false".into()));
        assert!(!events.iter().any(|event| event.starts_with("mips:")));
        let parameters: Vec<_> = events
            .iter()
            .filter(|event| event.starts_with("parameter:"))
            .cloned()
            .collect();
        assert_eq!(
            parameters,
            [
                "parameter:3553:10240:9728",
                "parameter:3553:10241:9728",
                "parameter:3553:10242:33069",
                "parameter:3553:10243:33069"
            ]
        );
        context.close();
        assert!(!log.lock().unwrap().contains(&"delete_tex:1".into()));
        runtime.run_main();
        assert!(log.lock().unwrap().contains(&"delete_tex:1".into()));
        dat.material_buffer.borrow_mut().clear();
        assert!(SourceMassStrengthDataHolder::create_mass(&dat, 0., 0.).is_err());
        let dat = crate::ship_data::SourceShipData::new(
            dat.img.clone(),
            palette,
            Rc::new(RefCell::new(Vec::new())),
            i32::MIN,
            1,
        );
        assert!(SourceMassStrengthDataHolder::create_mass(&dat, 0., 0.).is_err());
    }

    #[test]
    fn active_gpu_initial_mass_preserves_original_constructor_initialization_order() {
        let choice = crate::ShipCatalog::discover()
            .0
            .into_iter()
            .find(|choice| choice.material_map)
            .unwrap();
        let structure = crate::ShipStructure::load_for_choice(&choice);
        let packed = gpu_material_data(&structure);
        assert!(!packed.is_empty());
        for (material, channels) in structure.texel_materials.iter().zip(packed) {
            if let Some(material) = material {
                assert_eq!(channels[0], material.mass);
                assert_eq!(channels[1], material.tensile_strength);
                assert_eq!(channels[2], material.compressive_strength);
                assert_eq!(channels[3].to_bits(), (material.mass * 0.1 + 0.0).to_bits());
            } else {
                assert_eq!(channels, [0.; 4]);
            }
        }
        let original = std::fs::read_to_string("../SS2/decompiled/com/wicpar/sinkingsimulator/ship/physics/data/MassStrengthDataHolder.java").unwrap();
        let constructor = original
            .split("public MassStrengthDataHolder(")
            .nth(1)
            .unwrap();
        assert!(
            constructor.find("this.createMass(dat)").unwrap()
                < constructor.find("this.AIR = 1.225f").unwrap()
        );
        assert!(
            constructor.find("this.AIR = 1.225f").unwrap()
                < constructor.find("this.WATER = 1025.0f").unwrap()
        );
    }
    #[test]
    fn water_updates_alpha_from_raw_mass_without_feedback() {
        let input = [7800.0, 65.0, 260.0, 82780.0];
        let first = update_effective_mass(input, true, 1.0, 3.0, 0.915);
        assert_eq!(&first[..3], &input[..3]);
        assert!((first[3] - (7800.0 * 0.085 + 1025.0 * 0.915)).abs() < 0.001);
        assert_eq!(first, update_effective_mass(first, true, 1.0, 3.0, 0.915));
        let interior = update_effective_mass(input, false, 2.0, 3.0, 1.0);
        assert_eq!(interior[3], 3075.0);
    }
}

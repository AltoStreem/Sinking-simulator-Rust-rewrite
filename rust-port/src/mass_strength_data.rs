//! Translation of SS2 `MassStrengthDataHolder` GPU packing.
use crate::ShipStructure;

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
                let weighted_mass = if material.hull || material.ground {
                    material.mass * 0.1 + 1025.0 * 80.0
                } else {
                    material.mass * 0.1 + 1.225 * 0.9
                };
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

//! Translation of SS2 `MaskStrutsDataHolder` pixel masks and spring links.
use crate::ShipStructure;

#[allow(dead_code)]
pub(crate) struct SourceMaskStrutsDataHolder {
    texture: std::sync::Arc<crate::texture_2d::SourceTexture2D>,
}
#[allow(dead_code)]
impl SourceMaskStrutsDataHolder {
    pub(crate) fn create_mask<
        P: crate::ship_data::SourceMaterialLookup<
                Material = std::sync::Arc<crate::materials::Material>,
            >,
    >(
        dat: &crate::ship_data::SourceShipData<P>,
    ) -> Result<crate::image_data::ImageData, String> {
        let count = dat.width.wrapping_mul(dat.height);
        let count =
            usize::try_from(count).map_err(|_| "negative source mask array size".to_owned())?;
        let mut flags = Vec::with_capacity(count);
        for index in 0..count {
            let materials = dat.material_buffer.borrow();
            let material = materials
                .get(index)
                .ok_or_else(|| "source material list index out of bounds".to_owned())?;
            flags.push(material.as_ref().map_or(0u32, |material| {
                crate::mask_struts_material_masks::MATERIAL_MASKS
                    .iter()
                    .fold(1u32, |mask, predicate| {
                        mask * 2 + u32::from(predicate(material))
                    })
            }));
        }
        let mut links = [Vec::with_capacity(count), Vec::with_capacity(count)];
        let predicates: [fn(Option<&crate::materials::Material>) -> bool; 2] = [
            crate::mask_struts_structural_predicate::invoke,
            crate::mask_struts_water_predicate::invoke,
        ];
        for (output, predicate) in links.iter_mut().zip(predicates) {
            for index in 0..count {
                if dat.width == 0 {
                    return Err("source integer division by zero".into());
                }
                let index = index as i32;
                let x = index % dat.width;
                let y = index / dat.width;
                let materials = dat.material_buffer.borrow();
                let good = |index: i32| -> Result<bool, String> {
                    let material = usize::try_from(index)
                        .ok()
                        .and_then(|index| materials.get(index))
                        .ok_or_else(|| "source material list index out of bounds".to_owned())?;
                    Ok(predicate(material.as_deref()))
                };
                let mut mask = 0u32;
                for offset in (0..8).rev() {
                    let target = match offset {
                        0 if x != dat.width.wrapping_sub(1) => Some(index.wrapping_add(1)),
                        1 if x != dat.width.wrapping_sub(1) && y != dat.height.wrapping_sub(1) => {
                            Some(index.wrapping_add(1).wrapping_add(dat.width))
                        }
                        2 if y != dat.height.wrapping_sub(1) => Some(index.wrapping_add(dat.width)),
                        3 if x != 0 && y != dat.height.wrapping_sub(1) => {
                            Some(index.wrapping_sub(1).wrapping_add(dat.width))
                        }
                        4 if x != 0 => Some(index.wrapping_sub(1)),
                        5 if x != 0 && y != 0 => {
                            Some(index.wrapping_sub(1).wrapping_sub(dat.width))
                        }
                        6 if y != 0 => Some(index.wrapping_sub(dat.width)),
                        7 if x != dat.width.wrapping_sub(1) && y != 0 => {
                            Some(index.wrapping_add(1).wrapping_sub(dat.width))
                        }
                        _ => None,
                    };
                    let exists = if good(index)? {
                        match target {
                            Some(target) => good(target)?,
                            None => false,
                        }
                    } else {
                        false
                    };
                    mask = mask * 2 + u32::from(exists);
                }
                output.push(mask);
            }
        }
        let mut bytes = Vec::with_capacity(count * 12);
        for index in 0..count {
            for value in [flags[index], links[0][index], links[1][index]] {
                bytes.extend_from_slice(&value.to_ne_bytes());
            }
        }
        let mut image = crate::image_data::ImageData::new(bytes, dat.width, dat.height, 36248);
        image.format = 5125;
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
        let image = Self::create_mask(dat)?;
        let texture = crate::texture_2d::SourceTexture2D::from_image(
            &image,
            36220,
            false,
            std::sync::Arc::new(crate::mask_struts_texture_config::configure),
            backend,
            context,
            runtime,
        );
        Ok(Self {
            texture: std::sync::Arc::new(texture),
        })
    }
}
impl crate::gl_data_holder::SourceGlDataHolder for SourceMaskStrutsDataHolder {
    fn source_texture(&self) -> std::sync::Arc<crate::texture_2d::SourceTexture2D> {
        self.texture.clone()
    }
}

/// Pack occupancy/material flags and reciprocal spring direction masks.
pub(super) fn gpu_mask_data(structure: &ShipStructure) -> Vec<[u32; 4]> {
    let solid: Vec<bool> = structure
        .texel_solid
        .iter()
        .enumerate()
        .map(|(index, &is_solid)| {
            let x = index % structure.texel_width;
            let y = index / structure.texel_width;
            let proxy =
                (y / crate::PHYSICS_NODE_PIXELS) * structure.width + x / crate::PHYSICS_NODE_PIXELS;
            is_solid && !structure.breached[proxy]
        })
        .collect();
    let water_solid: Vec<bool> = structure
        .texel_materials
        .iter()
        .zip(&solid)
        .map(|(material, &is_solid)| is_solid && material.is_some_and(|material| !material.rope))
        .collect();
    let struts = build_strut_masks(&solid, structure.texel_width, structure.texel_height);
    let water_struts =
        build_strut_masks(&water_solid, structure.texel_width, structure.texel_height);
    structure
        .texel_materials
        .iter()
        .zip(struts)
        .zip(water_struts)
        .zip(solid)
        .map(|(((material, struts), water_struts), is_solid)| {
            let flags = if is_solid {
                material.map_or(0, |material| {
                    8 | (u32::from(material.ground) << 2)
                        | (u32::from(material.hull) << 1)
                        | u32::from(material.rope)
                })
            } else {
                0
            };
            [flags, u32::from(struts), u32::from(water_struts), 0]
        })
        .collect()
}

/// Create one reciprocal bit for each solid 8-neighbour spring edge.
pub(super) fn build_strut_masks(solid: &[bool], width: usize, height: usize) -> Vec<u8> {
    if width == 0 || height == 0 || solid.len() != width * height {
        return vec![0; solid.len()];
    }
    const FORWARD_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (1, 1), (0, 1), (-1, 1)];
    let mut masks = vec![0u8; solid.len()];
    for y in 0..height {
        for x in 0..width {
            let a = y * width + x;
            if !solid[a] {
                continue;
            }
            for (direction, (dx, dy)) in FORWARD_DIRECTIONS.iter().copied().enumerate() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    continue;
                }
                let b = ny as usize * width + nx as usize;
                if solid[b] {
                    masks[a] |= 1 << direction;
                    masks[b] |= 1 << (direction + 4);
                }
            }
        }
    }
    masks
}

/// Live plane followed by the stable input plane used by the source final pass.
pub(super) fn gpu_mask_storage(mut live: Vec<[u32; 4]>) -> Vec<[u32; 4]> {
    live.extend_from_within(..);
    live
}

/// CPU reference of ShipPhysics finalPass's opposite-link intersection.
#[cfg(test)]
fn reconcile_links(width: usize, height: usize, input: &[[u32; 4]]) -> Vec<[u32; 4]> {
    let mut output = input.to_vec();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            for (direction, (dx, dy)) in crate::EIGHT_NEIGHBORS.iter().copied().enumerate() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                let other = if nx >= 0 && ny >= 0 && nx < width as isize && ny < height as isize {
                    input[ny as usize * width + nx as usize][1]
                } else {
                    0
                };
                if other & (1 << ((direction + 4) & 7)) == 0 {
                    output[index][1] &= !(1 << direction);
                    output[index][2] &= !(1 << direction);
                }
            }
        }
    }
    output
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

    fn source_data(
        width: i32,
        height: i32,
        materials: Vec<Option<Arc<crate::materials::Material>>>,
    ) -> crate::ship_data::SourceShipData<crate::materials::SourceMaterials> {
        crate::ship_data::SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(crate::materials::SourceMaterials::default()),
            Rc::new(RefCell::new(materials)),
            width,
            height,
        )
    }

    #[test]
    fn native_masks_pack_source_flags_and_distinguish_rope_water_connections() {
        let palette = crate::materials::SourceMaterials::from_json(
            r##"[
          {"color":"#123456","isHull":true,"invisible":true},
          {"color":"#abcdef","isRope":true},
          {"color":"#010203","isGround":true}
        ]"##,
        )
        .unwrap();
        let mut materials = vec![palette.get(0x123456); 9];
        materials[2] = None;
        materials[4] = palette.get(0xabcdef);
        materials[6] = palette.get(0x010203);
        let dat = source_data(3, 3, materials.clone());
        let image = SourceMaskStrutsDataHolder::create_mask(&dat).unwrap();
        assert_eq!(
            (image.width, image.height, image.image_format, image.format),
            (3, 3, 36248, 5125)
        );
        assert_eq!(image.buffer.len(), 9 * 12);
        let values: Vec<_> = image
            .buffer
            .chunks_exact(12)
            .map(|bytes| {
                std::array::from_fn::<_, 3, _>(|index| {
                    u32::from_ne_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
                })
            })
            .collect();
        assert_eq!(values[0], [10, 7, 5]);
        assert_eq!(values[1], [10, 30, 26]);
        assert_eq!(values[2], [0, 0, 0]);
        assert_eq!(values[3], [10, 199, 198]);
        assert_eq!(values[4], [9, 127, 0]);
        assert_eq!(values[6][0], 12);
        let solid: Vec<_> = materials.iter().map(Option::is_some).collect();
        let wet: Vec<_> = materials
            .iter()
            .map(|material| material.as_ref().is_some_and(|material| !material.is_rope))
            .collect();
        let structural = build_strut_masks(&solid, 3, 3);
        let water = build_strut_masks(&wet, 3, 3);
        for index in 0..9 {
            assert_eq!(values[index][1], u32::from(structural[index]));
            assert_eq!(values[index][2], u32::from(water[index]));
        }
        let runtime = crate::resource::ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(Vec::new()));
        let holder = SourceMaskStrutsDataHolder::new(
            &dat,
            crate::render_fbo::source_tests::backend(log.clone()),
            context.clone(),
            &runtime,
        )
        .unwrap();
        let texture = holder.source_texture();
        assert!(Arc::ptr_eq(&texture, &holder.source_texture()));
        assert_eq!(*texture.img.as_ref().unwrap().lock().unwrap(), image.buffer);
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"image2:1:3553:36220:[3, 3]:36248:5125:false".into()));
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
        runtime.run_main();
        assert!(log.lock().unwrap().contains(&"delete_tex:1".into()));
    }

    #[test]
    fn native_flags_cover_all_material_combinations_and_preserve_source_failures() {
        let materials = (0..8)
            .map(|bits| {
                let mut material = crate::materials::Material::default();
                material.is_ground = bits & 4 != 0;
                material.is_hull = bits & 2 != 0;
                material.is_rope = bits & 1 != 0;
                Some(Arc::new(material))
            })
            .collect();
        let dat = source_data(1, 8, materials);
        let image = SourceMaskStrutsDataHolder::create_mask(&dat).unwrap();
        for (index, pixel) in image.buffer.chunks_exact(12).enumerate() {
            assert_eq!(
                u32::from_ne_bytes(pixel[..4].try_into().unwrap()),
                8 | index as u32
            );
        }
        assert!(SourceMaskStrutsDataHolder::create_mask(&source_data(-1, 1, vec![])).is_err());
        assert!(SourceMaskStrutsDataHolder::create_mask(&source_data(1, 1, vec![])).is_err());
        assert!(
            SourceMaskStrutsDataHolder::create_mask(&source_data(0, 1, vec![]))
                .unwrap()
                .buffer
                .is_empty()
        );
    }
    #[test]
    fn source_reciprocal_pass_cuts_the_other_end_and_preserves_flags() {
        let input = [[8, 1, 1, 0], [10, 0, 0, 0]];
        assert_eq!(
            reconcile_links(2, 1, &input),
            vec![[8, 0, 0, 0], [10, 0, 0, 0]]
        );
        let input = [[8, 1, 1, 0], [10, 16, 16, 0]];
        assert_eq!(reconcile_links(2, 1, &input), input);
        assert_eq!(gpu_mask_storage(input.to_vec()), [input, input].concat());
    }
}

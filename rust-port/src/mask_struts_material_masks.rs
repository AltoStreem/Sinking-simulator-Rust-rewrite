//! MaskStrutsDataHolder.Companion's retained predicate order.
pub(crate) const MATERIAL_MASKS: [fn(&crate::materials::Material) -> bool; 3] = [
    crate::mask_struts_ground_flag::invoke,
    crate::mask_struts_hull_flag::invoke,
    crate::mask_struts_rope_flag::invoke,
];

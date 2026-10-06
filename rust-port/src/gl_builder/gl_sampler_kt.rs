//! GLSamplerKt.sample generates the exact source texelFetch expression.
use super::{gl_reference::GlReference, gl_sampler_2d_type::GlSampler2DType, gl_type::GlType};
pub(crate) fn sample<
    T: GlSampler2DType,
    I: super::gl_vector_type::GlVectorType<Size = super::two::Two>,
>(
    sampler: &GlReference<T>,
    index: &GlReference<I>,
) -> GlReference<T::Base>
where
    T::Base: Clone,
    I::Base: GlType<Value = i32>,
{
    GlReference::new(
        sampler.gl_type.base_type().clone(),
        format!("texelFetch({}, {}, 0)", sampler.value, index),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_builder::{
        gl_builder::{GlBuilder, HolderIdentity},
        gl_int::GlInt,
        gl_ivec2::GlIVec2,
        gl_type::GlType,
        gl_uint::GlUInt,
    };
    #[test]
    fn scalar_families_keep_source_type_names_and_sampler_declarations() {
        assert_eq!(GlInt.type_name(), "int");
        assert_eq!(GlUInt.type_name(), "int");
        let mut context = GlBuilder::new(1, 1).pass().context();
        let signed = context.input_int(&HolderIdentity::new(), GlInt);
        let unsigned = context.input_uint(&HolderIdentity::new(), GlUInt);
        assert_eq!(signed.gl_type.type_name(), "isampler2D");
        assert_eq!(unsigned.gl_type.type_name(), "usampler2D");
        assert_eq!(
            &context.global[1..],
            &["uniform isampler2D in_b;", "uniform usampler2D in_c;"]
        );
        let fetched = sample(&unsigned, &GlReference::new(GlIVec2, "idx"));
        assert_eq!(fetched.gl_type.type_name(), "int");
        assert_eq!(fetched.value, "texelFetch(in_c, idx, 0)");
    }
}

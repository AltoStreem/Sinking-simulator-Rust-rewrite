//! GLOpsKt.java: expression construction, without evaluation or type reinterpretation.
use super::{gl_reference::GlReference, gl_type::GlType};
fn binary<T: GlType + Clone>(
    left: &GlReference<T>,
    right: &GlReference<T>,
    operation: char,
) -> GlReference<T> {
    GlReference::new(
        left.gl_type.clone(),
        format!("({}{}{})", left.value, operation, right.value),
    )
}
macro_rules! operation {
    ($name:ident, $symbol:literal) => {
        pub(crate) fn $name<T: GlType + Clone>(
            left: &GlReference<T>,
            right: &GlReference<T>,
        ) -> GlReference<T> {
            binary(left, right, $symbol)
        }
    };
}
operation!(plus, '+');
operation!(minus, '-');
operation!(times, '*');
operation!(div, '/');
operation!(and, '&');
operation!(or, '|');
operation!(xor, '^');
pub(crate) fn inv<T: GlType + Clone>(value: &GlReference<T>) -> GlReference<T> {
    GlReference::new(value.gl_type.clone(), format!("(~{})", value.value))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_builder::{gl_float::GlFloat, gl_ivec2::GlIVec2};
    #[test]
    fn every_source_operator_preserves_expression_parentheses_and_type() {
        let a = GlReference::new(GlFloat, "a");
        let b = GlReference::new(GlFloat, "b");
        let results = [
            plus(&a, &b),
            minus(&a, &b),
            times(&a, &b),
            div(&a, &b),
            and(&a, &b),
            or(&a, &b),
            xor(&a, &b),
            inv(&a),
        ];
        let expected = [
            "(a+b)", "(a-b)", "(a*b)", "(a/b)", "(a&b)", "(a|b)", "(a^b)", "(~a)",
        ];
        for (actual, text) in results.iter().zip(expected) {
            assert_eq!(actual.value, text);
            assert_eq!(actual.gl_type.type_name(), "float");
        }
        assert_eq!(times(&plus(&a, &b), &inv(&b)).value, "((a+b)*(~b))");
        let v = GlReference::new(GlIVec2, "v");
        assert_eq!(plus(&v, &v).value, "(v+v)");
        assert_eq!(inv(&v).gl_type.type_name(), "ivec2");
    }
}

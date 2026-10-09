//! Generic_helpersKt.floor/round preserve Kotlin Number.toInt conversions.
pub(crate) fn floor(value:f32)->f32 {value as i32 as f32}
pub(crate) fn round(value:f32)->f32 {floor(value+0.5)}
pub(crate) fn min(a:f32,b:f32)->f32 {
    if a.is_nan() {a}else if b.is_nan() {b}else if a==0.0&&b==0.0 {f32::from_bits(a.to_bits()|b.to_bits())}else {a.min(b)}
}
pub(crate) fn max(a:f32,b:f32)->f32 {
    if a.is_nan() {a}else if b.is_nan() {b}else if a==0.0&&b==0.0 {f32::from_bits(a.to_bits()&b.to_bits())}else {a.max(b)}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn jvm_integer_conversion_preserves_negative_nan_and_saturation_rules() {
        assert_eq!(floor(-1.9),-1.0);assert_eq!(floor(-0.9).to_bits(),0);
        assert_eq!(floor(f32::NAN),0.0);assert_eq!(floor(f32::INFINITY),i32::MAX as f32);
        assert_eq!(floor(f32::NEG_INFINITY),i32::MIN as f32);
        assert_eq!(round(-1.9),-1.0);assert_eq!(round(1.5),2.0);
        assert_eq!([-1.9,-0.9,1.9,f32::INFINITY,f32::NEG_INFINITY,f32::NAN].map(|v|floor(v).to_bits() as i32),
            [-1082130432,0,1065353216,1325400064,-822083584,0]);
        assert_eq!(min(0.0,-0.0).to_bits(),(-0.0f32).to_bits());assert_eq!(max(-0.0,0.0).to_bits(),0);
    }
}

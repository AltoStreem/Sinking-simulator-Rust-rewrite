//! Main$main$counter$1$1.class: title construction occurs when the queued callback runs.
fn round_to_int(value: f64) -> i32 {
    assert!(!value.is_nan(), "Cannot round NaN value.");
    // Kotlin roundToInt delegates to Math.round: ties toward positive infinity,
    // then clamps to Int's range. Avoid adding 0.5 before floor (precision loss).
    let floor = value.floor();
    let rounded = if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    rounded as i32
}
pub(crate) fn title(resources: f64, speed: f64) -> String {
    format!(
        "Resources: {:>5}% usage, {:>5}% speed",
        round_to_int(resources * 100.0),
        round_to_int(speed * 100.0)
    )
}
pub(crate) fn invoke(resources: f64, speed: f64, set_title: impl FnOnce(String)) {
    set_title(title(resources, speed));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kotlin_rounding_padding_clamping_and_nan() {
        assert_eq!(round_to_int(-1.5), -1);
        assert_eq!(round_to_int(1.5), 2);
        assert_eq!(round_to_int(f64::INFINITY), i32::MAX);
        assert_eq!(round_to_int(f64::NEG_INFINITY), i32::MIN);
        assert_eq!(
            title(-0.015, 0.985),
            "Resources:    -1% usage,    99% speed"
        );
        assert_eq!(
            title(1234.56, 1.0),
            "Resources: 123456% usage,   100% speed"
        );
        assert!(std::panic::catch_unwind(|| title(f64::NAN, 1.0)).is_err());
    }
}

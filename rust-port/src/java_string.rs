//! Java-compatible primitive float formatting used by translated Kotlin `toString` methods.
#![allow(dead_code)]

pub(crate) fn java_float_to_string(value: f32) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == f32::INFINITY {
        return "Infinity".into();
    }
    if value == f32::NEG_INFINITY {
        return "-Infinity".into();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .into();
    }

    let raw = format!("{value:?}");
    let (negative, raw) = raw
        .strip_prefix('-')
        .map_or((false, raw.as_str()), |s| (true, s));
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map_or((raw, 0i32), |(m, e)| (m, e.parse::<i32>().unwrap_or(0)));
    let decimal_at = mantissa.find('.').unwrap_or(mantissa.len()) as i32 + exponent;
    let mut digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let leading_zeroes = digits.bytes().take_while(|b| *b == b'0').count();
    digits.drain(..leading_zeroes);
    let decimal_at = decimal_at - leading_zeroes as i32;
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
    }
    if digits.is_empty() {
        return if negative { "-0.0" } else { "0.0" }.into();
    }

    let scientific_exponent = decimal_at - 1;
    let sign = if negative { "-" } else { "" };
    if scientific_exponent >= 7 || scientific_exponent < -3 {
        let first = &digits[..1];
        let rest = &digits[1..];
        let fraction = if rest.is_empty() { "0" } else { rest };
        return format!("{sign}{first}.{fraction}E{scientific_exponent}");
    }

    if decimal_at <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-decimal_at) as usize))
    } else if decimal_at as usize >= digits.len() {
        format!(
            "{sign}{}{zeros}.0",
            digits,
            zeros = "0".repeat(decimal_at as usize - digits.len())
        )
    } else {
        let split = decimal_at as usize;
        format!("{sign}{}.{}", &digits[..split], &digits[split..])
    }
}

/// Java String.regionMatches(ignoreCase=true) semantics over UTF-16 code units,
/// used by Kotlin's String.contains(other, ignoreCase=true).
pub(crate) fn java_contains_ignore_case(haystack: &str, needle: &str) -> bool {
    let haystack: Vec<u16> = haystack.encode_utf16().collect();
    let needle: Vec<u16> = needle.encode_utf16().collect();
    java_contains_ignore_case_units(&haystack,&needle)
}
pub(crate) fn java_contains_ignore_case_units(haystack:&[u16],needle:&[u16])->bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|window| {
        window
            .iter()
            .copied()
            .zip(needle.iter().copied())
            .all(|(left, right)| java_char_equals_ignore_case(left, right))
    })
}

fn java_char_equals_ignore_case(left: u16, right: u16) -> bool {
    if left == right {
        return true;
    }
    let upper_left = java_char_uppercase(left);
    let upper_right = java_char_uppercase(right);
    upper_left == upper_right || java_char_lowercase(upper_left) == java_char_lowercase(upper_right)
}

// Java Character mappings are single UTF-16 chars. Rust's full Unicode case
// iterators can expand a character (for example, sharp s), which Java char
// case conversion does not do, so expansions fall back to the original unit.
fn java_char_uppercase(unit: u16) -> u16 {
    java_single_case_mapping(unit, true)
}

fn java_char_lowercase(unit: u16) -> u16 {
    java_single_case_mapping(unit, false)
}

fn java_single_case_mapping(unit: u16, uppercase: bool) -> u16 {
    let Some(character) = char::from_u32(u32::from(unit)) else {
        return unit;
    };
    let (first, expands) = if uppercase {
        let mut mapped = character.to_uppercase();
        (mapped.next(), mapped.next().is_some())
    } else {
        let mut mapped = character.to_lowercase();
        (mapped.next(), mapped.next().is_some())
    };
    let Some(first) = first else {
        return unit;
    };
    if expands || first as u32 > u16::MAX as u32 {
        unit
    } else {
        first as u16
    }
}

/// Java String.compareTo compares unsigned UTF-16 code units lexicographically.
pub(crate) fn java_string_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

//! LWJGL capability constructor and availability contracts from supplied classes.
use std::collections::{BTreeMap, BTreeSet};
pub(crate) struct Symbol {
    pub name: &'static str,
    pub device: bool,
}
pub(crate) struct Flag {
    pub name: &'static str,
    pub required: &'static [&'static str],
}
#[derive(Debug)]
pub(crate) struct Capabilities {
    pub addresses: Vec<(&'static str, usize)>,
    pub flags: BTreeMap<&'static str, bool>,
    pub missing: Vec<&'static str>,
}
impl Capabilities {
    pub fn new(
        symbols: &[Symbol],
        flags: &[Flag],
        reported: &BTreeSet<String>,
        mut lookup: impl FnMut(&Symbol) -> usize,
    ) -> Self {
        let addresses: Vec<_> = symbols
            .iter()
            .map(|symbol| (symbol.name, lookup(symbol)))
            .collect();
        let mut missing = Vec::new();
        let flags = flags
            .iter()
            .map(|flag| {
                let reported = reported.contains(flag.name);
                let present = flag.required.iter().all(|name| {
                    addresses
                        .iter()
                        .find(|(key, _)| key == name)
                        .is_some_and(|(_, pointer)| *pointer != 0)
                });
                if reported && !present {
                    missing.push(flag.name);
                }
                (flag.name, reported && present)
            })
            .collect();
        Self {
            addresses,
            flags,
            missing,
        }
    }
    pub fn enabled(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }
    pub fn address(&self, name: &str) -> usize {
        self.addresses
            .iter()
            .find(|(key, _)| *key == name)
            .map_or(0, |(_, value)| *value)
    }
}
pub(crate) fn version_extensions(prefix: &str, major: i32, minor: i32) -> BTreeSet<String> {
    (0..=1)
        .filter(|&version| major > 1 || (major == 1 && minor >= version))
        .map(|version| format!("{prefix}1{version}"))
        .collect()
}
pub(crate) fn extension_tokens(text: &str) -> impl Iterator<Item = &str> {
    // Java StringTokenizer's default five delimiters; Unicode whitespace is
    // not an extension separator in the bundled constructor.
    text.split([' ', '\t', '\n', '\r', '\u{c}'])
        .filter(|token| !token.is_empty())
}
pub(crate) fn parse_version(text: &str) -> Result<(i32, i32), String> {
    // APIUtil.apiParseVersion with a null prefix. Java's default \s and \S
    // are ASCII, and Java dot excludes five line terminators. Require the
    // complete match, including any optional revision/implementation suffix.
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = PATTERN.get_or_init(|| regex::Regex::new(
        r"\A([0-9]+)[.]([0-9]+)([.][^ \t\n\x0B\x0C\r]+)?[ \t\n\x0B\x0C\r]*([^\n\r\u{0085}\u{2028}\u{2029}]+)?\z"
    ).unwrap());
    let malformed = || format!("Malformed API version string [{text}]");
    let captures = pattern.captures(text).ok_or_else(malformed)?;
    let major = captures[1].parse::<i32>().map_err(|_| malformed())?;
    let minor = captures[2].parse::<i32>().map_err(|_| malformed())?;
    Ok((major, minor))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::al_capability_tables::*;
    #[test]
    fn supplied_capability_tables_cover_all_original_fields() {
        assert_eq!((ALC_SYMBOLS.len(), ALC_FLAGS.len()), (30, 16));
        assert_eq!((AL_SYMBOLS.len(), AL_FLAGS.len()), (118, 33));
        for (symbols, flags) in [(ALC_SYMBOLS, ALC_FLAGS), (AL_SYMBOLS, AL_FLAGS)] {
            let reported = flags.iter().map(|flag| flag.name.to_owned()).collect();
            let caps = Capabilities::new(symbols, flags, &reported, |_| 1);
            assert!(caps.flags.values().all(|flag| *flag));
            for symbol in symbols {
                let broken = Capabilities::new(symbols, flags, &reported, |entry| {
                    usize::from(entry.name != symbol.name)
                });
                for flag in flags {
                    assert_eq!(
                        broken.enabled(flag.name),
                        !flag.required.contains(&symbol.name)
                    );
                }
            }
        }
    }
    #[test]
    fn version_and_java_token_delimiters_match_source() {
        assert!(version_extensions("OpenAL", 0, 99).is_empty());
        assert_eq!(
            version_extensions("OpenAL", 1, 0),
            BTreeSet::from(["OpenAL10".into()])
        );
        assert_eq!(version_extensions("OpenALC", 2, 0).len(), 2);
        assert_eq!(
            extension_tokens(" a\tb\nc\rd\u{c}e\u{a0}f ").collect::<Vec<_>>(),
            ["a", "b", "c", "d", "e\u{a0}f"]
        );
    }
    #[test]
    fn original_api_version_pattern_accepts_and_rejects_complete_strings() {
        for text in [
            "1.1",
            "1.1 ALSOFT 1.20.1",
            "1.1.0 vendor",
            "01.001",
            "1.1\r\n",
            "1.1.\u{2028}",
        ] {
            assert_eq!(parse_version(text).unwrap(), (1, 1), "{text:?}");
        }
        for text in [
            "",
            "OpenAL 1.1",
            " 1.1",
            "1",
            "1.x",
            "1.1 vendor\nextra",
            "2147483648.1",
            "1.2147483648",
        ] {
            assert!(parse_version(text).is_err(), "{text:?}");
        }
    }
}

//! Materials(File): read, deserialize and build the palette, or report failure
//! and retain an empty map. The JVM stack-trace ABI remains a backend concern.
use super::SourceMaterials;
use std::path::Path;

impl SourceMaterials {
    pub(crate) fn from_file(path: &Path, report: impl FnOnce(String)) -> Self {
        Self::construct(
            || {
                crate::file_reader::FileReader::game()
                    .read_file(path)
                    .map_err(|error| error.to_string())
            },
            report,
        )
    }

    fn construct(
        read: impl FnOnce() -> Result<String, String>,
        report: impl FnOnce(String),
    ) -> Self {
        match read().and_then(|json| Self::from_json(&json)) {
            Ok(materials) => materials,
            Err(error) => {
                report(error);
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructor_reports_once_and_discards_the_entire_failed_palette() {
        let mut reads = 0;
        let mut failures = Vec::new();
        let palette = SourceMaterials::construct(
            || {
                reads += 1;
                Ok(r##"[{"name":"good","color":"#123456"},{"name":"bad","color":"bad"}]"##.into())
            },
            |error| failures.push(error),
        );
        assert_eq!(reads, 1);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("Color 'bad'"));
        assert!(palette.materials.is_empty());
        assert_eq!(palette.to_json().unwrap(), "[]");
        failures.clear();
        let palette =
            SourceMaterials::construct(|| Err("read failed".into()), |error| failures.push(error));
        assert!(palette.materials.is_empty());
        assert_eq!(failures, ["read failed"]);
    }
    #[test]
    fn file_constructor_uses_actual_reader_and_retains_alias_identity() {
        let path = std::env::temp_dir().join(format!(
            "ss2-material-constructor-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            r##"[{"name":"steel","color":"#123456","colour":"#abcdef"}]"##,
        )
        .unwrap();
        let palette = SourceMaterials::from_file(&path, |_| panic!("valid palette"));
        assert!(std::sync::Arc::ptr_eq(
            &palette.get(0x123456).unwrap(),
            &palette.get(0xabcdef).unwrap()
        ));
        std::fs::write(&path, "[invalid").unwrap();
        let mut reported = false;
        let palette = SourceMaterials::from_file(&path, |_| reported = true);
        assert!(reported);
        assert!(palette.materials.is_empty());
        std::fs::remove_file(path).unwrap();
    }
}

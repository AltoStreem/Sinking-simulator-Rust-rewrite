//! Main$main$1.class: recursive getFlatFiles, in the directory's listing order.
use std::path::{Path, PathBuf};

pub(crate) trait Files {
    fn is_directory(&self, path: &Path) -> bool;
    fn list_files(&self, path: &Path) -> Option<Vec<PathBuf>>;
}
pub(crate) struct NativeFiles;
impl Files for NativeFiles {
    fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }
    fn list_files(&self, path: &Path) -> Option<Vec<PathBuf>> {
        // Java listFiles returns null on failure; never sort the listing.
        std::fs::read_dir(path)
            .ok()?
            .map(|entry| entry.ok().map(|e| e.path()))
            .collect()
    }
}
pub(crate) fn invoke(path: &Path) -> Vec<PathBuf> {
    invoke_with(path, &NativeFiles)
}

/// Kotlin `File.extension`: text after the last dot, or empty when no dot exists.
pub(crate) fn kotlin_file_extension(name: &str) -> &str {
    name.rfind('.').map_or("", |index| &name[index + 1..])
}

/// Kotlin `File.nameWithoutExtension`: text before the last dot, or the full name when absent.
pub(crate) fn kotlin_file_name_without_extension(name: &str) -> &str {
    name.rfind('.').map_or(name, |index| &name[..index])
}
pub(crate) fn invoke_with(path: &Path, files: &impl Files) -> Vec<PathBuf> {
    if !files.is_directory(path) {
        return vec![path.to_owned()];
    }
    let Some(children) = files.list_files(path) else {
        return Vec::new();
    };
    children
        .into_iter()
        .flat_map(|child| invoke_with(&child, files))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture;
    impl Files for Fixture {
        fn is_directory(&self, path: &Path) -> bool {
            matches!(path.to_str(), Some("root" | "nested" | "failed" | "empty"))
        }
        fn list_files(&self, path: &Path) -> Option<Vec<PathBuf>> {
            match path.to_str().unwrap() {
                "root" => Some(
                    ["z", "nested", "failed", "empty", "a"]
                        .map(PathBuf::from)
                        .into(),
                ),
                "nested" => Some(["second", "first"].map(PathBuf::from).into()),
                "empty" => Some(vec![]),
                _ => None,
            }
        }
    }
    #[test]
    fn preserves_depth_first_listing_order_and_null_listing() {
        assert_eq!(
            invoke_with(Path::new("root"), &Fixture),
            ["z", "second", "first", "a"].map(PathBuf::from)
        );
        assert_eq!(
            invoke_with(Path::new("missing"), &Fixture),
            vec![PathBuf::from("missing")]
        );
        assert!(invoke_with(Path::new("failed"), &Fixture).is_empty());
    }
    #[test]
    fn native_listing_keeps_files_and_missing_paths() {
        let root = std::env::temp_dir().join(format!("ss2-main-files-{}", std::process::id()));
        std::fs::create_dir_all(root.join("nested")).unwrap();
        let file = root.join("nested").join("Titanic_BASE.png");
        std::fs::write(&file, []).unwrap();
        assert_eq!(invoke(&root), vec![file.clone()]);
        assert!(crate::main_ship_file_predicate::invoke(&file));
        assert!(!crate::main_ship_file_predicate::invoke(&root));
        let absent = root.join("absent");
        assert_eq!(invoke(&absent), vec![absent.clone()]);
        assert!(!crate::main_ship_file_predicate::invoke(&absent));
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(root.join("nested")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}

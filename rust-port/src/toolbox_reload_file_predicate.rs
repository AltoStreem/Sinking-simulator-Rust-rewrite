//! Toolbox$reloadFiles$reloaded$1$1.class predicate: File.isFile, with no extension restriction.
pub fn invoke<F>(file: &F, is_file: impl FnOnce(&F) -> bool) -> bool {
    is_file(file)
}

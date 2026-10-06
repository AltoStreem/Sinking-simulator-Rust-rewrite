//! Filesystem and translated ship-resource binding for Toolbox.reloadFiles.
//! Steam services are explicitly supplied; this adapter does not silently substitute an offline service.
use crate::{
    ship_resources::{ShipLayer, ShipResourceFile, ShipResourceType},
    ship_thumbnail::ShipThumbnail,
    toolbox_reload::ReloadBackend,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

pub trait SteamFolderAccess {
    fn is_running(&mut self) -> bool;
    fn all_workshop_folders(&mut self) -> Vec<PathBuf>;
}
#[derive(Clone, Debug)]
pub(crate) struct SourceFile {
    pub path: PathBuf,
    comparison: Vec<u16>,
}
impl SourceFile {
    pub fn new(path: PathBuf) -> Self {
        #[cfg(windows)]
        let units = {
            use std::os::windows::ffi::OsStrExt;
            path.as_os_str().encode_wide().collect::<Vec<_>>()
        };
        #[cfg(not(windows))]
        let units = path.to_string_lossy().encode_utf16().collect::<Vec<_>>();
        // All cache keys are canonical paths. Raw constructor normalization/WinNT path grammar remains pending.
        let comparison = units
            .into_iter()
            .map(|unit| {
                let unit = if unit == 47 { 92 } else { unit };
                crate::jvm_character::lower(crate::jvm_character::upper(unit))
            })
            .collect();
        Self { path, comparison }
    }
}
impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.comparison == other.comparison
    }
}
impl Eq for SourceFile {}

/// Resource map values retain identity, as FileShipResource inherits Object.equals.
/// Comparing all original layer/type entries also compares the base/material references used by synthesized textures.
pub(crate) struct LoadedThumbnail {
    pub translated: ShipThumbnail,
    resources: HashMap<(ShipLayer, ShipResourceType), Rc<ShipResourceFile>>,
}
impl PartialEq for LoadedThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.resources.len() == other.resources.len()
            && self.resources.iter().all(|(key, value)| {
                other
                    .resources
                    .get(key)
                    .is_some_and(|other| Rc::ptr_eq(value, other))
            })
    }
}
pub(crate) struct FilesystemReloadBackend {
    pub working_directory: PathBuf,
    steam: Box<dyn SteamFolderAccess>,
}
impl FilesystemReloadBackend {
    pub fn new(working_directory: PathBuf, steam: Box<dyn SteamFolderAccess>) -> Self {
        Self {
            working_directory,
            steam,
        }
    }
}
// Preserve UTF-16 OS path units, including unpaired surrogates, when removing Rust's canonical prefix.
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let units: Vec<_> = path.as_os_str().encode_wide().collect();
        if units.starts_with(&[92, 92, 63, 92]) {
            let result = if units[4..].starts_with(&[85, 78, 67, 92]) {
                let mut unc = vec![92, 92];
                unc.extend_from_slice(&units[8..]);
                unc
            } else {
                units[4..].to_vec()
            };
            return PathBuf::from(std::ffi::OsString::from_wide(&result));
        }
    }
    path
}
fn walk(path: &Path, out: &mut Vec<SourceFile>) {
    if path.is_dir() {
        out.push(SourceFile::new(path.to_path_buf()));
        // Kotlin default onFail is null: listFiles failure ends this directory's children silently.
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        // Preserve the OS encounter order rather than introducing alphabetical sorting.
        let entries: Vec<_> = entries.collect();
        // Java listFiles returns an array or null; a partially failed Rust enumeration is treated as null.
        if entries.iter().any(Result::is_err) {
            return;
        }
        for entry in entries {
            walk(&entry.unwrap().path(), out);
        }
    } else if path.is_file() {
        out.push(SourceFile::new(path.to_path_buf()));
    }
}
impl ReloadBackend for FilesystemReloadBackend {
    type File = SourceFile;
    type Resource = ShipResourceFile;
    type Thumbnail = LoadedThumbnail;
    fn local_ship_root(&mut self, path: &str) -> SourceFile {
        SourceFile::new(self.working_directory.join(path))
    }
    fn steam_running(&mut self) -> bool {
        self.steam.is_running()
    }
    fn workshop_folders(&mut self) -> Vec<SourceFile> {
        self.steam
            .all_workshop_folders()
            .into_iter()
            .map(SourceFile::new)
            .collect()
    }
    fn walk_top_down(&mut self, root: &SourceFile) -> Vec<SourceFile> {
        let mut result = Vec::new();
        walk(&root.path, &mut result);
        result
    }
    fn is_file(&mut self, file: &SourceFile) -> bool {
        file.path.is_file()
    }
    fn canonical_file(&mut self, file: &SourceFile) -> Result<SourceFile, String> {
        let path = std::fs::canonicalize(&file.path).map_err(|error| error.to_string())?;
        let path = strip_verbatim_prefix(path);
        Ok(SourceFile::new(path))
    }
    fn resource_from_file(&mut self, file: &SourceFile) -> Result<Rc<ShipResourceFile>, String> {
        crate::ship_resources::parse_resource_path(&file.path).map(Rc::new)
    }
    fn resource_file(&mut self, resource: &ShipResourceFile) -> SourceFile {
        SourceFile::new(resource.path.clone())
    }
    fn parent_file(&mut self, file: &SourceFile) -> Option<SourceFile> {
        file.path
            .parent()
            .map(|parent| SourceFile::new(parent.to_path_buf()))
    }
    fn resource_ship_name(&mut self, resource: &ShipResourceFile) -> Vec<u16> {
        resource.ship.encode_utf16().collect()
    }
    fn construct_thumbnail(
        &mut self,
        resources: &[Rc<ShipResourceFile>],
    ) -> Result<Rc<LoadedThumbnail>, String> {
        let translated =
            ShipThumbnail::new(resources.iter().map(|resource| resource.as_ref().clone()))?;
        let resources = resources
            .iter()
            .map(|resource| {
                (
                    (resource.layer.clone(), resource.resource_type),
                    resource.clone(),
                )
            })
            .collect();
        Ok(Rc::new(LoadedThumbnail {
            translated,
            resources,
        }))
    }
    fn thumbnail_name(&mut self, thumbnail: &LoadedThumbnail) -> Vec<u16> {
        thumbnail
            .translated
            .name()
            .expect("NullPointerException: default BASE missing")
            .encode_utf16()
            .collect()
    }
    fn print_error(&mut self, text: &str) {
        println!("{text}");
    }
    fn print_stack_trace(&mut self, error: &str) {
        eprintln!("{error}");
    } // Full JVM exception stacks remain unavailable.
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Steam {
        running: bool,
        roots: Vec<PathBuf>,
    }
    impl SteamFolderAccess for Steam {
        fn is_running(&mut self) -> bool {
            self.running
        }
        fn all_workshop_folders(&mut self) -> Vec<PathBuf> {
            assert!(self.running);
            self.roots.clone()
        }
    }
    struct Directory(PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            assert!(self.0.starts_with(std::env::temp_dir()));
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[cfg(windows)]
    #[test]
    fn toolbox_filesystem_canonical_prefix_retains_raw_utf16_path_units() {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let units = vec![92, 92, 63, 92, 67, 58, 92, 0xd800];
        let path = PathBuf::from(std::ffi::OsString::from_wide(&units));
        let result = strip_verbatim_prefix(path);
        assert_eq!(
            result.as_os_str().encode_wide().collect::<Vec<_>>(),
            vec![67, 58, 92, 0xd800]
        );
        let path = PathBuf::from("\\\\?\\UNC\\server\\share\\A.png");
        assert_eq!(
            strip_verbatim_prefix(path),
            PathBuf::from("\\\\server\\share\\A.png")
        );
    }
    fn fixture() -> Directory {
        let path = std::env::temp_dir().join(format!(
            "ss2-reload-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(path.join("ships")).unwrap();
        Directory(path)
    }
    fn image(path: &Path) {
        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]))
            .save(path)
            .unwrap();
    }
    #[test]
    fn toolbox_filesystem_scans_real_maps_and_retains_resource_and_thumbnail_objects() {
        let dir = fixture();
        image(&dir.0.join("ships/A_base.png"));
        image(&dir.0.join("ships/A_texture.png"));
        image(&dir.0.join("ships/B_base.png"));
        std::fs::write(dir.0.join("ships/readme.txt"), "not a ship resource").unwrap();
        let mut backend = FilesystemReloadBackend::new(
            dir.0.clone(),
            Box::new(Steam {
                running: false,
                roots: vec![],
            }),
        );
        let mut toolbox = crate::toolbox::SourceToolbox::default();
        toolbox.reload_files(&mut backend).unwrap();
        assert_eq!(toolbox.catalog().ship_files.len(), 3);
        assert_eq!(toolbox.catalog().excluded_files.len(), 1);
        assert!(toolbox.catalog().ship_thumbnails.is_empty());
        let resource = toolbox.catalog().ship_files[0].1.clone();
        toolbox.reload_files(&mut backend).unwrap();
        assert_eq!(toolbox.catalog().ship_thumbnails.len(), 2);
        assert!(Rc::ptr_eq(&resource, &toolbox.catalog().ship_files[0].1));
        assert_eq!(
            toolbox.catalog().ship_thumbnails[0].translated.name(),
            Some("A")
        );
        assert_eq!(
            toolbox.catalog().ship_thumbnails[0]
                .translated
                .base_layer()
                .unwrap()
                .dimensions(),
            (2, 2)
        );
        let thumbnail = toolbox.catalog().ship_thumbnails[0].clone();
        toolbox.reload_files(&mut backend).unwrap();
        assert!(Rc::ptr_eq(
            &thumbnail,
            &toolbox.catalog().ship_thumbnails[0]
        ));
        // Equal resource values allocated afresh are not Object.equals-equal in the source.
        let originals: Vec<_> = thumbnail.resources.values().cloned().collect();
        let copied: Vec<_> = originals
            .iter()
            .map(|r| Rc::new(r.as_ref().clone()))
            .collect();
        let different = backend.construct_thumbnail(&copied).unwrap();
        assert!(thumbnail.as_ref() != different.as_ref());
    }
    #[test]
    fn toolbox_filesystem_workshop_branch_and_supplied_source_assets_load() {
        let dir = fixture();
        let workshop = dir.0.join("workshop");
        std::fs::create_dir_all(&workshop).unwrap();
        image(&workshop.join("Workshop_base.png"));
        let mut backend = FilesystemReloadBackend::new(
            dir.0.clone(),
            Box::new(Steam {
                running: true,
                roots: vec![workshop],
            }),
        );
        let mut toolbox = crate::toolbox::SourceToolbox::default();
        toolbox.reload_files(&mut backend).unwrap();
        toolbox.reload_files(&mut backend).unwrap();
        assert_eq!(
            toolbox.catalog().ship_thumbnails[0].translated.name(),
            Some("Workshop")
        );
        let home = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("SS2");
        let mut backend = FilesystemReloadBackend::new(
            home,
            Box::new(Steam {
                running: false,
                roots: vec![],
            }),
        );
        let mut toolbox = crate::toolbox::SourceToolbox::default();
        toolbox.reload_files(&mut backend).unwrap();
        toolbox.reload_files(&mut backend).unwrap();
        assert!(!toolbox.catalog().ship_thumbnails.is_empty());
        for thumbnail in &toolbox.catalog().ship_thumbnails {
            assert!(thumbnail.translated.name().is_some());
            thumbnail.translated.base_layer().unwrap();
        }
    }
    #[test]
    fn toolbox_filesystem_canonical_keys_use_original_case_mapping() {
        assert_eq!(
            SourceFile::new(PathBuf::from("C:/Ships/A.png")),
            SourceFile::new(PathBuf::from("c:\\ships\\a.PNG"))
        );
        assert_eq!(
            SourceFile::new(PathBuf::from("C:/\u{131}.png")),
            SourceFile::new(PathBuf::from("c:/I.PNG"))
        );
    }
}

//! Main.java's initial ships/pacmaster walk, construction and publication.
//! FileTreeWalk is consumed lazily: a resource failure stops further directory traversal.
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
pub(crate) trait Files: crate::main_flat_files::Files {
    fn is_file(&self, path: &Path) -> bool;
}
impl Files for crate::main_flat_files::NativeFiles {
    fn is_file(&self, path: &Path) -> bool {
        crate::main_ship_file_predicate::invoke(path)
    }
}
pub(crate) trait Operations {
    type Resource;
    type Thumbnail;
    type Ship: Clone;
    fn resource(&mut self, path: &Path) -> Result<Self::Resource, String>;
    fn thumbnail(&mut self, resources: Vec<Self::Resource>) -> Result<Self::Thumbnail, String>;
    fn ship(&mut self, thumbnail: Self::Thumbnail) -> Result<Self::Ship, String>;
    fn publish(&mut self, ship: Self::Ship);
    fn append(&mut self, ship: Self::Ship);
}
fn visit<O: Operations>(
    path: &Path,
    directory: bool,
    files: &impl Files,
    ops: &mut O,
    resources: &mut Vec<O::Resource>,
) -> Result<(), String> {
    if files.is_file(path) {
        resources.push(ops.resource(path)?);
    }
    if directory {
        if let Some(children) = files.list_files(path) {
            for child in children {
                let directory = files.is_directory(&child);
                visit(&child, directory, files, ops, resources)?;
            }
        }
    }
    Ok(())
}
pub(crate) fn load<O: Operations>(
    root: &Path,
    files: &impl Files,
    ops: &mut O,
) -> Result<O::Ship, String> {
    let directory = files.is_directory(root);
    let mut resources = Vec::new();
    // FileTreeWalk has no root state when the root is neither directory nor file.
    if directory || files.is_file(root) {
        visit(root, directory, files, ops, &mut resources)?;
    }
    let thumbnail = ops.thumbnail(resources)?;
    let ship = ops.ship(thumbnail)?;
    ops.publish(ship.clone());
    ops.append(ship.clone()); // startup appends; it does not clear existing ships
    Ok(ship)
}
pub(crate) struct NativeOperations {
    pub factory: crate::gui_reset_ship::SourceResetShipOperations,
    pub control: Rc<RefCell<crate::camera_control::SourceCameraControl>>,
}
impl Operations for NativeOperations {
    type Resource = crate::ship_resources::ShipResourceFile;
    type Thumbnail = Rc<crate::ship_thumbnail::ShipThumbnail>;
    type Ship = crate::main_globals::Ship;
    fn resource(&mut self, path: &Path) -> Result<Self::Resource, String> {
        crate::main_ship_file_resource::invoke(path)
    }
    fn thumbnail(&mut self, resources: Vec<Self::Resource>) -> Result<Self::Thumbnail, String> {
        crate::ship_thumbnail::ShipThumbnail::new(resources).map(Rc::new)
    }
    fn ship(&mut self, thumbnail: Self::Thumbnail) -> Result<Self::Ship, String> {
        (self.factory.constructor)(thumbnail, self.control.clone())
    }
    fn publish(&mut self, ship: Self::Ship) {
        crate::main_globals::set_global_ship(ship);
    }
    fn append(&mut self, ship: Self::Ship) {
        crate::main_globals::get_ship_list().borrow_mut().push(ship);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::main_flat_files::Files as FlatFiles;
    struct Fixture {
        events: Rc<RefCell<Vec<String>>>,
    }
    impl FlatFiles for Fixture {
        fn is_directory(&self, path: &Path) -> bool {
            self.events
                .borrow_mut()
                .push(format!("directory:{}", path.display()));
            matches!(path.to_str(), Some("root" | "nested" | "unreadable"))
        }
        fn list_files(&self, path: &Path) -> Option<Vec<PathBuf>> {
            self.events
                .borrow_mut()
                .push(format!("list:{}", path.display()));
            match path.to_str().unwrap() {
                "root" => Some(
                    ["first_BASE.png", "nested", "unreadable", "last_TEXTURE.png"]
                        .map(PathBuf::from)
                        .into(),
                ),
                "nested" => Some(vec!["middle_INLIGHTS.png".into()]),
                _ => None,
            }
        }
    }
    impl Files for Fixture {
        fn is_file(&self, path: &Path) -> bool {
            self.events
                .borrow_mut()
                .push(format!("filter:{}", path.display()));
            path.extension().is_some()
        }
    }
    struct Ops {
        events: Rc<RefCell<Vec<String>>>,
        global: usize,
        list: Vec<usize>,
        fail: bool,
    }
    impl Operations for Ops {
        type Resource = String;
        type Thumbnail = Vec<String>;
        type Ship = usize;
        fn resource(&mut self, path: &Path) -> Result<String, String> {
            self.events
                .borrow_mut()
                .push(format!("resource:{}", path.display()));
            if self.fail {
                Err("resource failure".into())
            } else {
                Ok(path.to_string_lossy().into_owned())
            }
        }
        fn thumbnail(&mut self, resources: Vec<String>) -> Result<Vec<String>, String> {
            self.events
                .borrow_mut()
                .push(format!("thumbnail:{}", resources.join(",")));
            Ok(resources)
        }
        fn ship(&mut self, _: Vec<String>) -> Result<usize, String> {
            self.events.borrow_mut().push("ship".into());
            Ok(7)
        }
        fn publish(&mut self, ship: usize) {
            self.events.borrow_mut().push("global".into());
            self.global = ship;
        }
        fn append(&mut self, ship: usize) {
            assert_eq!(self.global, ship);
            self.events.borrow_mut().push("append".into());
            self.list.push(ship);
        }
    }
    fn fixture() -> (Fixture, Ops) {
        let events = Rc::new(RefCell::new(vec![]));
        (
            Fixture {
                events: events.clone(),
            },
            Ops {
                events,
                global: 99,
                list: vec![99],
                fail: false,
            },
        )
    }
    #[test]
    fn resources_are_loaded_during_walk_then_ship_is_published_before_append() {
        let (files, mut ops) = fixture();
        assert_eq!(load(Path::new("root"), &files, &mut ops), Ok(7));
        assert_eq!(ops.list, [99, 7]);
        let events = ops.events.borrow();
        assert!(
            events
                .iter()
                .position(|s| s == "resource:first_BASE.png")
                .unwrap()
                < events.iter().position(|s| s == "list:nested").unwrap()
        );
        assert_eq!(
            &events[events.len() - 4..],
            [
                "thumbnail:first_BASE.png,middle_INLIGHTS.png,last_TEXTURE.png",
                "ship",
                "global",
                "append"
            ]
        );
    }
    #[test]
    fn resource_failure_stops_walk_without_thumbnail_or_publication() {
        let (files, mut ops) = fixture();
        ops.fail = true;
        assert_eq!(
            load(Path::new("root"), &files, &mut ops),
            Err("resource failure".into())
        );
        assert_eq!(ops.global, 99);
        assert_eq!(ops.list, [99]);
        assert!(
            !ops.events
                .borrow()
                .iter()
                .any(|s| s == "list:nested" || s == "ship" || s == "global")
        );
    }
    #[test]
    fn actual_original_pacmaster_files_construct_a_source_thumbnail() {
        struct NativeThumbnail {
            resources: usize,
        }
        impl Operations for NativeThumbnail {
            type Resource = crate::ship_resources::ShipResourceFile;
            type Thumbnail = crate::ship_thumbnail::ShipThumbnail;
            type Ship = Rc<crate::ship_thumbnail::ShipThumbnail>;
            fn resource(&mut self, path: &Path) -> Result<Self::Resource, String> {
                self.resources += 1;
                crate::main_ship_file_resource::invoke(path)
            }
            fn thumbnail(&mut self, r: Vec<Self::Resource>) -> Result<Self::Thumbnail, String> {
                crate::ship_thumbnail::ShipThumbnail::new(r)
            }
            fn ship(&mut self, t: Self::Thumbnail) -> Result<Self::Ship, String> {
                Ok(Rc::new(t))
            }
            fn publish(&mut self, _: Self::Ship) {}
            fn append(&mut self, _: Self::Ship) {}
        }
        let mut ops = NativeThumbnail { resources: 0 };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../SS2/ships/pacmaster");
        let thumbnail = load(&root, &crate::main_flat_files::NativeFiles, &mut ops).unwrap();
        assert_eq!(ops.resources, 3);
        for kind in [
            crate::ship_resources::ShipResourceType::Base,
            crate::ship_resources::ShipResourceType::Texture,
            crate::ship_resources::ShipResourceType::InLights,
        ] {
            assert!(
                thumbnail
                    .get_resource(kind, &crate::ship_resources::ShipLayer::default())
                    .is_some()
            );
        }
    }
}

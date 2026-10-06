//! Toolbox.reloadFiles(), including its original one-scan thumbnail delay and retained object identities.
use std::{marker::PhantomData, rc::Rc};

pub trait ReloadBackend {
    type File: Clone + Eq;
    type Resource;
    type Thumbnail: PartialEq;
    fn local_ship_root(&mut self, path: &str) -> Self::File;
    fn steam_running(&mut self) -> bool;
    fn workshop_folders(&mut self) -> Vec<Self::File>;
    /// FileTreeWalk top-down sequence, including directory entries; Kotlin's walk failure policy belongs here.
    fn walk_top_down(&mut self, root: &Self::File) -> Vec<Self::File>;
    fn is_file(&mut self, file: &Self::File) -> bool;
    /// IOException from canonicalization is outside the per-resource source catch.
    fn canonical_file(&mut self, file: &Self::File) -> Result<Self::File, String>;
    fn resource_from_file(&mut self, file: &Self::File) -> Result<Rc<Self::Resource>, String>;
    fn resource_file(&mut self, resource: &Self::Resource) -> Self::File;
    fn parent_file(&mut self, file: &Self::File) -> Option<Self::File>;
    fn resource_ship_name(&mut self, resource: &Self::Resource) -> Vec<u16>;
    fn construct_thumbnail(
        &mut self,
        resources: &[Rc<Self::Resource>],
    ) -> Result<Rc<Self::Thumbnail>, String>;
    fn thumbnail_name(&mut self, thumbnail: &Self::Thumbnail) -> Vec<u16>;
    fn print_error(&mut self, text: &str);
    fn print_stack_trace(&mut self, error: &str);
}
/// Ordered entries reproduce Kotlin LinkedHashMap/LinkedHashSet traversal.
/// Publication is one engine-thread operation; original synchronized cross-thread publication remains pending.
pub struct ReloadState<B: ReloadBackend> {
    pub excluded_files: Vec<B::File>,
    pub ship_files: Vec<(B::File, Rc<B::Resource>)>,
    pub ship_thumbnails: Vec<Rc<B::Thumbnail>>,
    backend_type: PhantomData<B>,
}
impl<B: ReloadBackend> Default for ReloadState<B> {
    fn default() -> Self {
        Self {
            excluded_files: Vec::new(),
            ship_files: Vec::new(),
            ship_thumbnails: Vec::new(),
            backend_type: PhantomData,
        }
    }
}
impl<B: ReloadBackend> ReloadState<B> {
    pub fn reload_files(&mut self, backend: &mut B) -> Result<(), String> {
        let mut roots = vec![backend.local_ship_root("./ships")];
        if backend.steam_running() {
            roots.extend(backend.workshop_folders());
        }
        let mut scanned = Vec::new();
        for root in roots {
            for file in backend.walk_top_down(&root) {
                if crate::toolbox_reload_file_predicate::invoke(&file, |file| backend.is_file(file))
                {
                    scanned.push(file);
                }
            }
        }
        // Source map(canonicalFile).toSet() completes before exclusions are subtracted.
        let canonical: Vec<_> = scanned
            .iter()
            .map(|file| backend.canonical_file(file))
            .collect::<Result<_, _>>()?;
        let mut reloaded = Vec::new();
        for file in canonical {
            if !reloaded.contains(&file) {
                reloaded.push(file);
            }
        }
        reloaded.retain(|file| !self.excluded_files.contains(file));
        let mut new_files = Vec::new();
        for file in reloaded {
            let cached = self
                .ship_files
                .iter()
                .find(|(key, _)| key == &file)
                .map(|(_, resource)| resource.clone());
            let resource = match cached {
                Some(resource) => Ok(resource),
                None => backend.resource_from_file(&file),
            };
            match resource {
                Ok(resource) => new_files.push((file, resource)),
                Err(error) => {
                    backend.print_error(&format!("Error: {error}"));
                    if !self.excluded_files.contains(&file) {
                        self.excluded_files.push(file);
                    }
                }
            }
        }
        // Confirmed in bytecode: aload_0/getfield shipFiles/Map.values; it reads OLD shipFiles.
        let mut parents: Vec<(Option<B::File>, Vec<Rc<B::Resource>>)> = Vec::new();
        for (_, resource) in &self.ship_files {
            let file = backend.resource_file(resource);
            let parent = backend.parent_file(&file);
            if let Some((_, group)) = parents.iter_mut().find(|(key, _)| key == &parent) {
                group.push(resource.clone());
            } else {
                parents.push((parent, vec![resource.clone()]));
            }
        }
        let mut new_thumbnails = Vec::new();
        for (_, resources) in parents {
            let mut names: Vec<(Vec<u16>, Vec<Rc<B::Resource>>)> = Vec::new();
            for resource in resources {
                let name = backend.resource_ship_name(&resource);
                if let Some((_, group)) = names.iter_mut().find(|(key, _)| key == &name) {
                    group.push(resource);
                } else {
                    names.push((name, vec![resource]));
                }
            }
            for (_, resources) in names {
                match backend.construct_thumbnail(&resources) {
                    Ok(thumbnail) => {
                        // associateWith replaces equal keys' values: the last matching old instance wins.
                        let retained = self
                            .ship_thumbnails
                            .iter()
                            .rev()
                            .find(|old| old.as_ref() == thumbnail.as_ref())
                            .cloned()
                            .unwrap_or(thumbnail);
                        new_thumbnails.push(retained);
                    }
                    Err(error) => {
                        backend.print_stack_trace(&error);
                        for resource in resources {
                            let file = backend.resource_file(&resource);
                            if !self.excluded_files.contains(&file) {
                                self.excluded_files.push(file);
                            }
                        }
                    }
                }
            }
        }
        new_thumbnails.sort_by(|a, b| {
            let a = backend.thumbnail_name(a);
            let b = backend.thumbnail_name(b);
            crate::toolbox_reload_name_comparator::compare(&a, &b)
        });
        self.ship_thumbnails = new_thumbnails;
        self.ship_files = new_files;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, PartialEq)]
    struct Resource {
        file: String,
        name: Vec<u16>,
    }
    #[derive(Debug, PartialEq)]
    struct Thumbnail {
        resources: Vec<Rc<Resource>>,
    }
    #[derive(Default)]
    struct Backend {
        files: Vec<String>,
        parsed: Vec<String>,
        built: usize,
        errors: Vec<String>,
        traces: Vec<String>,
        canonical_failure: bool,
        running: bool,
        workshop_queries: usize,
    }
    impl ReloadBackend for Backend {
        type File = String;
        type Resource = Resource;
        type Thumbnail = Thumbnail;
        fn local_ship_root(&mut self, path: &str) -> String {
            assert_eq!(path, "./ships");
            path.into()
        }
        fn steam_running(&mut self) -> bool {
            self.running
        }
        fn workshop_folders(&mut self) -> Vec<String> {
            self.workshop_queries += 1;
            vec!["workshop".into()]
        }
        fn walk_top_down(&mut self, root: &String) -> Vec<String> {
            if root == "./ships" {
                let mut files = vec![root.clone()];
                files.extend(self.files.clone());
                files
            } else {
                vec!["a/Alpha.png".into()]
            }
        }
        fn is_file(&mut self, file: &String) -> bool {
            file != "./ships"
        }
        fn canonical_file(&mut self, file: &String) -> Result<String, String> {
            if self.canonical_failure {
                Err("canonical error".into())
            } else {
                Ok(file.trim_start_matches("alias:").into())
            }
        }
        fn resource_from_file(&mut self, file: &String) -> Result<Rc<Resource>, String> {
            self.parsed.push(file.clone());
            if file.ends_with("error.txt") {
                return Err("bad resource".into());
            }
            let name = file
                .rsplit('/')
                .next()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .encode_utf16()
                .collect();
            Ok(Rc::new(Resource {
                file: file.clone(),
                name,
            }))
        }
        fn resource_file(&mut self, r: &Resource) -> String {
            r.file.clone()
        }
        fn parent_file(&mut self, f: &String) -> Option<String> {
            f.rsplit_once('/').map(|(p, _)| p.into())
        }
        fn resource_ship_name(&mut self, r: &Resource) -> Vec<u16> {
            r.name.clone()
        }
        fn construct_thumbnail(
            &mut self,
            resources: &[Rc<Resource>],
        ) -> Result<Rc<Thumbnail>, String> {
            self.built += 1;
            if String::from_utf16(&resources[0].name).unwrap() == "bad" {
                return Err("bad thumbnail".into());
            }
            Ok(Rc::new(Thumbnail {
                resources: resources.to_vec(),
            }))
        }
        fn thumbnail_name(&mut self, t: &Thumbnail) -> Vec<u16> {
            t.resources[0].name.clone()
        }
        fn print_error(&mut self, text: &str) {
            self.errors.push(text.into());
        }
        fn print_stack_trace(&mut self, error: &str) {
            self.traces.push(error.into());
        }
    }
    #[test]
    fn toolbox_reload_preserves_one_scan_delay_resource_identity_and_equal_thumbnail_reuse() {
        let mut state = ReloadState::<Backend>::default();
        let mut b = Backend {
            files: vec![
                "a/Zeta.png".into(),
                "a/Alpha.png".into(),
                "alias:a/Alpha.png".into(),
            ],
            ..Default::default()
        };
        state.reload_files(&mut b).unwrap();
        assert!(state.ship_thumbnails.is_empty());
        assert_eq!(state.ship_files.len(), 2);
        let resource = state.ship_files[0].1.clone();
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.parsed.len(), 2);
        assert!(Rc::ptr_eq(&resource, &state.ship_files[0].1));
        assert_eq!(
            String::from_utf16(&state.ship_thumbnails[0].resources[0].name).unwrap(),
            "Alpha"
        );
        let thumb = state.ship_thumbnails[0].clone();
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.built, 4); // fresh temporary thumbnails still constructed before equal-object reuse.
        assert!(Rc::ptr_eq(&thumb, &state.ship_thumbnails[0]));
        assert_eq!(b.workshop_queries, 0);
        b.running = true;
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.workshop_queries, 1);
        assert_eq!(state.ship_files.len(), 2);
    }
    #[test]
    fn toolbox_reload_deleted_files_remain_in_thumbnails_for_one_scan() {
        let mut state = ReloadState::<Backend>::default();
        let mut b = Backend {
            files: vec!["a/Alpha.png".into()],
            ..Default::default()
        };
        state.reload_files(&mut b).unwrap();
        state.reload_files(&mut b).unwrap();
        b.files.clear();
        state.reload_files(&mut b).unwrap();
        assert!(state.ship_files.is_empty());
        assert_eq!(state.ship_thumbnails.len(), 1);
        state.reload_files(&mut b).unwrap();
        assert!(state.ship_thumbnails.is_empty());
    }
    #[test]
    fn toolbox_reload_failures_permanently_exclude_files_and_group_resources() {
        let mut state = ReloadState::<Backend>::default();
        let mut b = Backend {
            files: vec![
                "a/error.txt".into(),
                "a/bad.png".into(),
                "a/bad.json".into(),
            ],
            ..Default::default()
        };
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.errors, vec!["Error: bad resource"]);
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.traces, vec!["bad thumbnail"]);
        assert_eq!(
            state.excluded_files,
            vec!["a/error.txt", "a/bad.png", "a/bad.json"]
        );
        assert_eq!(state.ship_files.len(), 2); // excluded during thumbnail pass, after new-file map was computed.
        state.reload_files(&mut b).unwrap();
        assert!(state.ship_files.is_empty());
        assert_eq!(b.traces.len(), 2);
        state.reload_files(&mut b).unwrap();
        assert_eq!(b.parsed.len(), 3);
        assert_eq!(b.errors.len(), 1);
    }
    #[test]
    fn toolbox_reload_groups_same_names_separately_by_parent_and_aborts_canonical_errors() {
        let mut state = ReloadState::<Backend>::default();
        let mut b = Backend {
            files: vec!["a/Alpha.png".into(), "b/Alpha.png".into()],
            ..Default::default()
        };
        state.reload_files(&mut b).unwrap();
        state.reload_files(&mut b).unwrap();
        assert_eq!(state.ship_thumbnails.len(), 2);
        assert_eq!(state.ship_thumbnails[0].resources[0].file, "a/Alpha.png");
        assert_eq!(state.ship_thumbnails[1].resources[0].file, "b/Alpha.png");
        let thumb = state.ship_thumbnails[0].clone();
        let resource = state.ship_files[0].1.clone();
        b.canonical_failure = true;
        assert_eq!(state.reload_files(&mut b), Err("canonical error".into()));
        assert!(Rc::ptr_eq(&thumb, &state.ship_thumbnails[0]));
        assert!(Rc::ptr_eq(&resource, &state.ship_files[0].1));
        assert!(b.errors.is_empty());
        assert!(state.excluded_files.is_empty());
    }
    #[test]
    fn toolbox_reload_name_comparator_uses_java_utf16_order() {
        assert_eq!(
            crate::toolbox_reload_name_comparator::compare(&[0xd800, 0xdc00], &[0xe000]),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            crate::toolbox_reload_name_comparator::compare(&[65], &[97]),
            std::cmp::Ordering::Less
        );
    }
}

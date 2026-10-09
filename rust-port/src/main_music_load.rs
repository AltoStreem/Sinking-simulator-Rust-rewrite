//! Main$main$2.class: non-suspending startup music coroutine body.
//! GlobalScope scheduling, shared HashMap race semantics and JVM ABI remain pending.
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
pub(crate) fn track_name(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    (crate::main_flat_files::kotlin_file_extension(&name) == "ogg")
        .then(|| crate::main_flat_files::kotlin_file_name_without_extension(&name).to_owned())
}
pub(crate) trait Operations {
    type Buffer;
    fn load_buffer(&mut self, path: &Path) -> Result<Self::Buffer, String>;
    fn put_all(&mut self, tracks: Vec<(String, Self::Buffer)>) -> Result<(), String>;
    fn next_track(&mut self) -> Result<(), String>;
}
pub(crate) fn selected_files(files: Vec<PathBuf>) -> Vec<(String, PathBuf)> {
    files
        .into_iter()
        .filter_map(|path| track_name(&path).map(|name| (name, path)))
        .collect()
}
pub(crate) fn invoke<O: Operations>(
    label: i32,
    result: Result<(), String>,
    root: &Path,
    files: &impl crate::main_flat_files::Files,
    ops: &mut O,
) -> Result<(), String> {
    if label != 0 {
        return Err("call to 'resume' before 'invoke' with coroutine".into());
    }
    result?;
    let selected = selected_files(crate::main_flat_files::invoke_with(root, files));
    let mut associated: Vec<(String, O::Buffer)> = Vec::new();
    for (name, path) in selected {
        let buffer = ops.load_buffer(&path)?;
        if let Some((_, previous)) = associated.iter_mut().find(|(key, _)| key == &name) {
            // LinkedHashMap replacement keeps the first insertion position.
            // Source does not explicitly close the overwritten ALBuffer.
            *previous = buffer;
        } else {
            associated.push((name, buffer));
        }
    }
    ops.put_all(associated)?;
    ops.next_track()
}
pub(crate) type Tracks = Vec<(String, Arc<crate::al_buffer::AlBuffer>)>;
pub(crate) struct NativeOperations {
    pub backend: Arc<Mutex<dyn crate::al_buffer::AlBufferBackend>>,
    pub runtime: crate::resource::ResourceRuntime,
    // Captured map supplies its own Java HashMap iteration semantics.
    pub publish: Box<dyn FnMut(Tracks) -> Result<(), String>>,
    pub player: Box<dyn FnMut() -> crate::main_globals::Player>,
}
impl Operations for NativeOperations {
    type Buffer = Arc<crate::al_buffer::AlBuffer>;
    fn load_buffer(&mut self, path: &Path) -> Result<Self::Buffer, String> {
        crate::al_buffer::AlBuffer::from_file(path, self.backend.clone(), &self.runtime)
            .map(Arc::new)
            .map_err(|e| e.to_string())
    }
    fn put_all(&mut self, tracks: Tracks) -> Result<(), String> {
        (self.publish)(tracks)
    }
    fn next_track(&mut self) -> Result<(), String> {
        (self.player)().borrow_mut().next_track();
        Ok(())
    }
}
impl NativeOperations {
    pub(crate) fn use_main_player(&mut self) {
        self.player = Box::new(crate::main_globals::get_player);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Files;
    impl crate::main_flat_files::Files for Files {
        fn is_directory(&self, p: &Path) -> bool {
            p == Path::new("music")
        }
        fn list_files(&self, _: &Path) -> Option<Vec<PathBuf>> {
            Some(
                ["one/dup.ogg", "upper.OGG", "z.ogg", "two/dup.ogg", ".ogg"]
                    .map(PathBuf::from)
                    .into(),
            )
        }
    }
    #[derive(Default)]
    struct Ops {
        loaded: usize,
        tracks: Vec<(String, usize)>,
        events: Vec<String>,
        fail: Option<usize>,
    }
    impl Operations for Ops {
        type Buffer = usize;
        fn load_buffer(&mut self, p: &Path) -> Result<usize, String> {
            self.loaded += 1;
            self.events.push(format!("load:{}", p.display()));
            if self.fail == Some(self.loaded) {
                return Err("decode failure".into());
            }
            Ok(self.loaded)
        }
        fn put_all(&mut self, t: Vec<(String, usize)>) -> Result<(), String> {
            self.events.push("putAll".into());
            self.tracks.extend(t);
            Ok(())
        }
        fn next_track(&mut self) -> Result<(), String> {
            self.events.push("getPlayer/nextTrack".into());
            Ok(())
        }
    }
    #[test]
    fn association_retains_first_key_and_last_buffer_before_publication_and_playback() {
        let mut ops = Ops::default();
        invoke(0, Ok(()), Path::new("music"), &Files, &mut ops).unwrap();
        assert_eq!(ops.loaded, 4);
        assert_eq!(
            ops.tracks,
            [("dup".into(), 3), ("z".into(), 2), ("".into(), 4)]
        );
        assert_eq!(&ops.events[4..], ["putAll", "getPlayer/nextTrack"]);
        assert_eq!(
            track_name(Path::new("archive.part.ogg")),
            Some("archive.part".into())
        );
        assert_eq!(track_name(Path::new("upper.OGG")), None);
        assert_eq!(track_name(Path::new("ogg")), None);
    }
    #[test]
    fn decode_and_continuation_failures_leave_captured_map_untouched() {
        let mut ops = Ops {
            tracks: vec![("existing".into(), 99)],
            fail: Some(3),
            ..Default::default()
        };
        assert_eq!(
            invoke(0, Ok(()), Path::new("music"), &Files, &mut ops),
            Err("decode failure".into())
        );
        assert_eq!(ops.tracks, [("existing".into(), 99)]);
        assert_eq!(ops.events.len(), 3);
        ops.events.clear();
        assert_eq!(
            invoke(
                0,
                Err("incoming".into()),
                Path::new("music"),
                &Files,
                &mut ops
            ),
            Err("incoming".into())
        );
        assert!(
            invoke(
                1,
                Err("incoming".into()),
                Path::new("music"),
                &Files,
                &mut ops
            )
            .unwrap_err()
            .contains("resume")
        );
        assert!(ops.events.is_empty());
    }
    #[test]
    fn native_buffer_construction_preserves_both_allocations_without_explicit_close() {
        let runtime = crate::resource::ResourceRuntime::default();
        let (_initial, backend, log) = crate::al_buffer::tests::fixture(&runtime);
        let mut native = NativeOperations {
            backend,
            runtime: runtime.clone(),
            publish: Box::new(|_| Ok(())),
            player: Box::new(|| panic!("late player must not be read during loading")),
        };
        let first = native.load_buffer(Path::new("one/dup.ogg")).unwrap();
        let second = native.load_buffer(Path::new("two/dup.ogg")).unwrap();
        assert_ne!(first.id(), second.id());
        native
            .put_all(vec![("dup".into(), second.clone())])
            .unwrap();
        runtime.run_main();
        assert!(!first.freed());
        assert!(!second.freed());
        let events = log.lock().unwrap();
        assert_eq!(events.iter().filter(|v| v.starts_with("file:")).count(), 2);
        assert!(!events.iter().any(|v| v.starts_with("delete:")));
    }
}

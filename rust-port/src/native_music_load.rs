//! Background Main$main$2 operations, with ordered engine-thread publication.
use crate::{
    al_buffer::{AlBuffer, AlBufferBackend},
    main_music_load::{Operations, Tracks},
    resource::ResourceRuntime,
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};
pub(crate) enum Event {
    Publish(Tracks),
    NextTrack,
    Failed(String),
}
pub(crate) struct Loader {
    pub events: mpsc::Receiver<Event>,
    pub worker: Option<thread::JoinHandle<()>>,
}
struct Worker {
    backend: Arc<Mutex<dyn AlBufferBackend>>,
    runtime: ResourceRuntime,
    events: mpsc::Sender<Event>,
}
impl Operations for Worker {
    type Buffer = Arc<AlBuffer>;
    fn load_buffer(&mut self, path: &Path) -> Result<Self::Buffer, String> {
        AlBuffer::from_file(path, self.backend.clone(), &self.runtime)
            .map(Arc::new)
            .map_err(|error| error.to_string())
    }
    fn put_all(&mut self, tracks: Tracks) -> Result<(), String> {
        self.events
            .send(Event::Publish(tracks))
            .map_err(|_| "Music publication receiver closed".into())
    }
    fn next_track(&mut self) -> Result<(), String> {
        self.events
            .send(Event::NextTrack)
            .map_err(|_| "Music player receiver closed".into())
    }
}
impl Loader {
    pub fn spawn(
        backend: Arc<Mutex<dyn AlBufferBackend>>,
        runtime: ResourceRuntime,
        root: PathBuf,
        before_load: impl FnOnce() + Send + 'static,
    ) -> Result<Self, String> {
        let (events, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("source-music-loader".into())
            .spawn(move || {
                before_load();
                let mut operations = Worker {
                    backend,
                    runtime,
                    events,
                };
                if let Err(error) = crate::main_music_load::invoke(
                    0,
                    Ok(()),
                    &root,
                    &crate::main_flat_files::NativeFiles,
                    &mut operations,
                ) {
                    let _ = operations.events.send(Event::Failed(error));
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            events: receiver,
            worker: Some(worker),
        })
    }
    pub fn join(&mut self) -> Result<(), String> {
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "Native music loader panicked".to_owned())?;
        }
        Ok(())
    }
}

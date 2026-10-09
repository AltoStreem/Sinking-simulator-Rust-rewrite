//! Live Windows adapter for the translated MusicPlayer and native ALBuffer.
use crate::{
    music_player::{MusicPlayer, MusicTracks, SourceMusicPlayer},
    native_openal::{NativeAudioBuffers, NativeOpenAl},
    native_vorbis::NativeVorbis,
};
use bevy::prelude::*;
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};
#[derive(Default)]
struct Tracks(crate::main_music_load::Tracks);
impl MusicTracks for Tracks {
    fn keys(&self) -> Vec<String> {
        self.0.iter().map(|(name, _)| name.clone()).collect()
    }
    fn get(&self, name: &str) -> Option<Arc<crate::al_buffer::AlBuffer>> {
        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, buffer)| buffer.clone())
    }
}
pub(crate) struct NativeMusic {
    session: Result<Session, String>,
}
struct Session {
    native: NativeOpenAl,
    device: Arc<crate::al_device::AlDevice>,
    context: Arc<crate::al_context::AlContext>,
    runtime: crate::resource::ResourceRuntime,
    player: Rc<RefCell<SourceMusicPlayer>>,
    revision: u64,
    next_requests: u64,
    tracks: Rc<RefCell<Tracks>>,
    catalog: Vec<crate::music_player::Track>,
    loader: Option<crate::native_music_load::Loader>,
    load_done: bool,
    load_error: Option<String>,
    global_player: bool,
    closed: bool,
}
impl Session {
    fn new_with_runtime(
        model: &MusicPlayer,
        volume: f32,
        paused: bool,
        runtime: crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        #[cfg(not(test))]
        let class = crate::al_device::class();
        #[cfg(test)]
        let class = &crate::al_device::AlDeviceClass::default();
        Self::new_in_runtime(model, volume, paused, || {}, class, runtime)
    }
    fn new(model: &MusicPlayer, volume: f32, paused: bool) -> Result<Self, String> {
        Self::new_with_start(model, volume, paused, || {})
    }
    fn new_with_start(
        model: &MusicPlayer,
        volume: f32,
        paused: bool,
        before_load: impl FnOnce() + Send + 'static,
    ) -> Result<Self, String> {
        // Native application uses the single source loaded-class default. Each
        // test invocation models its own loaded class, like a separate JVM.
        #[cfg(not(test))]
        let class = crate::al_device::class();
        #[cfg(test)]
        let class = &crate::al_device::AlDeviceClass::default();
        Self::new_with_class(model, volume, paused, before_load, class)
    }
    fn new_with_class(
        model: &MusicPlayer,
        volume: f32,
        paused: bool,
        before_load: impl FnOnce() + Send + 'static,
        class: &crate::al_device::AlDeviceClass,
    ) -> Result<Self, String> {
        Self::new_in_runtime(
            model,
            volume,
            paused,
            before_load,
            class,
            crate::resource::ResourceRuntime::default(),
        )
    }
    fn new_in_runtime(
        model: &MusicPlayer,
        volume: f32,
        paused: bool,
        before_load: impl FnOnce() + Send + 'static,
        class: &crate::al_device::AlDeviceClass,
        runtime: crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        let directory = Path::new("assets/native/windows/x64");
        let native = NativeOpenAl::load(&directory.join("OpenAL.dll"))?;
        let decoder = NativeVorbis::load(&directory.join("lwjgl_stb.dll"))?;
        let backend = Arc::new(Mutex::new(native.clone()));
        let device = class.default_device(backend.clone(), &runtime);
        if device.id() == 0 {
            device.close();
            runtime.run_main();
            return Err("Native default audio device could not open".into());
        }
        let context = crate::al_context::AlContext::new(device.clone(), backend.clone(), &runtime);
        if context.id() == 0 {
            context.close();
            device.close();
            runtime.run_main();
            return Err("Native audio context could not be created".into());
        }
        if let Err(error) = context.try_start() {
            context.close();
            device.close();
            runtime.run_main();
            return Err(error);
        }
        let tracks = Rc::new(RefCell::new(Tracks::default()));
        let player = Rc::new(RefCell::new(SourceMusicPlayer::new(
            tracks.clone(),
            backend,
            &runtime,
        )));
        player.borrow().set_volume(volume);
        player.borrow_mut().set_paused(paused);
        // Own the source resource graph before launching the worker so an
        // initialization error still runs translated closeAll/runMain.
        let mut session = Self {
            native: native.clone(),
            device,
            context,
            runtime: runtime.clone(),
            player,
            revision: model.revision,
            next_requests: model.next_requests,
            tracks,
            catalog: model.tracks.clone(),
            loader: None,
            load_done: false,
            load_error: None,
            global_player: false,
            closed: false,
        };
        session.loader = Some(crate::native_music_load::Loader::spawn(
            Arc::new(Mutex::new(NativeAudioBuffers::new(native, decoder))),
            runtime,
            Path::new("assets/music").to_owned(),
            before_load,
        )?);
        Ok(session)
    }
    fn tick(&mut self, model: &mut MusicPlayer, simulation: &mut crate::Simulation) {
        if self.closed {
            return;
        }
        self.resolve_receiver();
        let mut player = self.player.borrow_mut();
        player.autonext = model.autonext;
        player.shuffle = model.shuffle;
        player.repeat = model.repeat;
        player.set_volume(simulation.music_volume);
        // The source setter is a UI write, not an every-frame command. Calling
        // it on a stopped source would restart playback before autonext runs.
        if player.paused != !simulation.music_playing {
            player.set_paused(!simulation.music_playing);
        }
        if self.next_requests != model.next_requests {
            for _ in 0..model.next_requests.wrapping_sub(self.next_requests) {
                player.next_track();
            }
            self.next_requests = model.next_requests;
            self.revision = model.revision;
        } else if self.revision != model.revision {
            if let Some(index) = model.current {
                player.play_track(model.tracks[index].name.clone());
            }
            self.revision = model.revision;
        }
        if let Some(position) = model.pending_seek.take() {
            player.set_progress(position.as_secs_f32());
        }
        Self::mirror(&player, &self.catalog, model, simulation);
    }
    fn advance(&mut self, model: &mut MusicPlayer, simulation: &mut crate::Simulation) {
        if self.closed {
            return;
        }
        self.resolve_receiver();
        self.runtime.run_main();
        self.poll_loading(model);
        let mut player = self.player.borrow_mut();
        // Main calls update before gui.handle; UI skip/pause must not be
        // followed by a second automatic advancement in the same frame.
        player.update();
        Self::mirror(&player, &self.catalog, model, simulation);
    }
    fn reset_model(model: &mut MusicPlayer) {
        // Main's captured buffer map is empty until putAll. Exposing discovered
        // but unloaded names lets an early skip select a nonexistent buffer.
        model.tracks.clear();
        model.queue.clear();
        model.current = None;
        model.progress = 0.;
        model.max_progress = 0.;
    }
    fn resolve_receiver(&mut self) {
        if self.global_player {
            self.player = crate::main_globals::get_player();
        }
    }
    fn poll_loading(&mut self, model: &mut MusicPlayer) {
        use crate::native_music_load::Event;
        let mut disconnected = false;
        loop {
            let event = match self.loader.as_ref().unwrap().events.try_recv() {
                Ok(event) => event,
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            };
            match event {
                Event::Publish(mut loaded) => {
                    let capacity = crate::music_player::java_hash_map_capacity(loaded.len());
                    loaded.sort_by_key(|(name, _)| {
                        crate::music_player::java_track_bucket(name, capacity)
                    });
                    model.tracks = loaded
                        .iter()
                        .map(|(name, _)| crate::music_player::Track {
                            name: name.clone(),
                            asset: self
                                .catalog
                                .iter()
                                .find(|track| &track.name == name)
                                .map(|track| track.asset.clone())
                                .unwrap_or_default(),
                        })
                        .collect();
                    self.tracks.borrow_mut().0 = loaded;
                }
                Event::NextTrack => {
                    // Resolve the session's current receiver now, not a player
                    // captured when the worker began loading.
                    self.resolve_receiver();
                    self.player.borrow_mut().next_track();
                    self.load_done = true;
                }
                Event::Failed(error) => {
                    bevy::log::error!("Native soundtrack loading failed: {error}");
                    self.load_error = Some(error);
                    self.load_done = true;
                }
            }
        }
        if disconnected && !self.load_done {
            let error = "Native music loader exited before publication completed".to_owned();
            bevy::log::error!("{error}");
            self.load_error = Some(error);
            self.load_done = true;
        }
        let loader = self.loader.as_mut().unwrap();
        if loader
            .worker
            .as_ref()
            .is_some_and(|worker| worker.is_finished())
        {
            if let Err(error) = loader.join() {
                bevy::log::error!("{error}");
                self.load_error = Some(error);
                self.load_done = true;
            }
        }
    }
    fn mirror(
        player: &SourceMusicPlayer,
        catalog: &[crate::music_player::Track],
        model: &mut MusicPlayer,
        simulation: &mut crate::Simulation,
    ) {
        // The current receiver may have a different map from the one captured
        // by the loader. UI state follows that receiver's actual source map.
        model.tracks = player
            .tracks
            .borrow()
            .keys()
            .into_iter()
            .map(|name| {
                let asset = catalog
                    .iter()
                    .find(|track| track.name == name)
                    .map(|track| track.asset.clone())
                    .unwrap_or_default();
                crate::music_player::Track { name, asset }
            })
            .collect();
        model.current = model
            .tracks
            .iter()
            .position(|track| track.name == player.current_track);
        model.queue = player
            .queue
            .iter()
            .filter_map(|name| model.tracks.iter().position(|track| &track.name == name))
            .collect();
        model.paused = player.paused;
        model.progress = player.progress();
        model.max_progress = player.max_progress();
        simulation.music_playing = !player.paused;
    }
}
impl Session {
    fn shutdown(&mut self) {
        if self.closed {
            return;
        }
        // Join the Rust worker before source closeAll/runMain so it cannot
        // register buffers after the graph has been closed. The source context
        // and default device now participate in that same translated graph.
        if let Some(loader) = &mut self.loader {
            let _ = loader.join();
        }
        self.runtime.close_all();
        self.runtime.run_main();
        self.closed = true;
        // Source free destroys the context without clearing ALContext.current
        // or restoring a previous context. Device closure owns its dependent.
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.shutdown();
    }
}
pub(crate) struct NativeMusicPlugin;
impl Plugin for NativeMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::resource::ResourceRuntime>();
        let simulation = app.world().resource::<crate::Simulation>();
        let mut session = Session::new_with_runtime(
            app.world().resource::<MusicPlayer>(),
            simulation.music_volume,
            !simulation.music_playing,
            app.world()
                .resource::<crate::resource::ResourceRuntime>()
                .clone(),
        );
        if let Err(error) = &session {
            bevy::log::error!("Native soundtrack initialization failed: {error}");
        }
        if let Ok(session) = &mut session {
            crate::main_globals::initialize_player(session.player.clone());
            session.global_player = true;
        }
        if session.is_ok() {
            Session::reset_model(&mut app.world_mut().resource_mut::<MusicPlayer>());
        }
        app.insert_non_send_resource(NativeMusic { session });
        app.add_systems(Last, shutdown.before(crate::resource::shutdown));
        app.add_systems(
            Update,
            advance
                .before(crate::select_toolbox_tab_and_settings)
                .before(crate::music_controls::handle_sliders)
                .before(crate::music_player::sync),
        );
    }
}
fn shutdown(mut native: NonSendMut<NativeMusic>, mut exit: MessageReader<bevy::app::AppExit>) {
    if exit.read().next().is_some() {
        if let Ok(session) = &mut native.session {
            session.shutdown();
        }
    }
}
fn advance(
    mut native: NonSendMut<NativeMusic>,
    mut model: ResMut<MusicPlayer>,
    mut simulation: ResMut<crate::Simulation>,
) {
    if let Ok(session) = &mut native.session {
        session.advance(&mut model, &mut simulation);
    }
}
pub(crate) fn sync(
    mut native: NonSendMut<NativeMusic>,
    assets: Res<AssetServer>,
    mut player: ResMut<MusicPlayer>,
    mut simulation: ResMut<crate::Simulation>,
    mut labels: Query<&mut Text2d, With<crate::MusicStatus>>,
    mut icons: Query<(&crate::music_player::MusicIcon, &mut Sprite)>,
) {
    if let Ok(session) = &mut native.session {
        session.tick(&mut player, &mut simulation);
    }
    crate::music_player::sync_display(&assets, &player, &mut labels, &mut icons);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Simulation;
    fn finish_loading(session: &mut Session, model: &mut MusicPlayer, simulation: &mut Simulation) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !session.load_done {
            session.advance(model, simulation);
            session.tick(model, simulation);
            assert!(
                model.tracks.is_empty() || model.tracks.len() == 10,
                "No partial track-map publication"
            );
            assert!(
                std::time::Instant::now() < deadline,
                "Native loading timed out"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(session.load_error.is_none(), "{:?}", session.load_error);
    }
    struct ReleaseOnDrop(Option<std::sync::mpsc::Sender<()>>);
    impl ReleaseOnDrop {
        fn release(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }
    impl Drop for ReleaseOnDrop {
        fn drop(&mut self) {
            self.release();
        }
    }
    #[test]
    #[ignore = "Requires original audio libraries, soundtrack and default device; muted"]
    fn live_source_audio_graph_global_default_and_current_identity() {
        use crate::al_context::AlContext;
        let mut model = MusicPlayer::discover();
        let mut simulation = Simulation::default();
        simulation.music_volume = 0.;
        simulation.music_playing = false;
        let (release, wait) = std::sync::mpsc::channel();
        // Use the actual production class singleton, not a test's scoped class.
        let class = crate::al_device::class();
        let mut session = Session::new_with_class(
            &model,
            0.,
            true,
            move || {
                wait.recv().unwrap();
            },
            class,
        )
        .unwrap();
        let mut release = ReleaseOnDrop(Some(release));
        let context = session.context.clone();
        let device = session.device.clone();
        assert!(Arc::ptr_eq(&context, &AlContext::current().unwrap()));
        assert!(Arc::ptr_eq(&context.device, &device));
        assert_eq!(session.native.current_context_id(), context.id());
        let alc = context.alc_capabilities();
        let al = context.al_capabilities();
        assert_eq!(
            alc.downcast_ref::<crate::native_openal::NativeAlcCapabilities>()
                .unwrap()
                .id,
            context.id()
        );
        let unrelated_runtime = crate::resource::ResourceRuntime::default();
        let same_default = class.default_device(
            Arc::new(Mutex::new(session.native.clone())),
            &unrelated_runtime,
        );
        assert!(
            Arc::ptr_eq(&device, &same_default),
            "default getter doesn't reopen a device"
        );
        let other = AlContext::new(
            same_default,
            Arc::new(Mutex::new(session.native.clone())),
            &session.runtime,
        );
        other.start();
        assert!(Arc::ptr_eq(&other, &AlContext::current().unwrap()));
        context.start();
        assert!(Arc::ptr_eq(&context, &AlContext::current().unwrap()));
        assert!(Arc::ptr_eq(&alc, &context.alc_capabilities()));
        assert!(Arc::ptr_eq(&al, &context.al_capabilities()));
        other.close();
        session.runtime.run_main();
        assert_eq!(session.native.current_context_id(), context.id());
        Session::reset_model(&mut model);
        release.release();
        finish_loading(&mut session, &mut model, &mut simulation);
        let buffers: Vec<_> = session
            .tracks
            .borrow()
            .0
            .iter()
            .map(|(_, buffer)| buffer.clone())
            .collect();
        assert_eq!(buffers.len(), 10);
        let player = session.player.clone();
        let native = session.native.clone();
        drop(session);
        assert!(context.freed() && device.freed() && player.borrow().source_freed());
        assert!(buffers.iter().all(|buffer| buffer.freed()));
        assert_eq!(native.current_context_id(), 0);
        assert!(
            Arc::ptr_eq(&context, &AlContext::current().unwrap()),
            "source free retains its global current reference"
        );
        let closed_default = class.default_device(Arc::new(Mutex::new(native)), &unrelated_runtime);
        assert!(
            Arc::ptr_eq(&device, &closed_default) && closed_default.freed(),
            "closed source default identity remains retained"
        );
        println!(
            "Live source audio graph: production default singleton, context/global/device identity, cached switch-back, ten background buffers, shared closeAll and retained closed current/default passed"
        );
    }
    #[test]
    #[ignore = "Requires native audio libraries, soundtrack and output device"]
    fn asynchronous_native_loading_keeps_empty_map_and_retains_early_controls() {
        let mut model = MusicPlayer::discover();
        let mut simulation = Simulation::default();
        simulation.music_volume = 0.;
        simulation.music_playing = false;
        let (release, wait) = std::sync::mpsc::channel();
        let mut session = Session::new_with_start(&model, 0., true, move || {
            wait.recv().unwrap();
        })
        .unwrap();
        // Declared after session: on panic this releases the gate before join.
        let mut release = ReleaseOnDrop(Some(release));
        Session::reset_model(&mut model);
        session.advance(&mut model, &mut simulation);
        assert!(session.tracks.borrow().0.is_empty() && model.tracks.is_empty());
        assert!(!session.load_done && model.current.is_none());
        model.next_track();
        session.tick(&mut model, &mut simulation);
        assert!(
            model.current.is_none(),
            "An early skip cannot select an unloaded name"
        );
        model.repeat = false;
        session.tick(&mut model, &mut simulation);
        release.release();
        finish_loading(&mut session, &mut model, &mut simulation);
        assert_eq!(session.tracks.borrow().0.len(), 10);
        assert!(model.current.is_none() && model.queue.is_empty() && model.paused);
        assert!(
            !session.player.borrow().repeat,
            "Publication must preserve earlier repeat-off"
        );
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../tools/fixtures/source-vorbis-track-pcm.json"
        ))
        .unwrap();
        for (name, buffer) in &session.tracks.borrow().0 {
            let track = expected["tracks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|track| track["file"].as_str() == Some(&format!("{name}.ogg")))
                .unwrap();
            assert_eq!(
                buffer.channels() as i64,
                track["channels"].as_i64().unwrap()
            );
            assert_eq!(buffer.frequency() as i64, track["rate"].as_i64().unwrap());
            assert_eq!(buffer.samples() as u64, track["frames"].as_u64().unwrap());
        }
        crate::al_util_kt::al_check(&mut session.native).unwrap();
        println!(
            "Gated native startup: frames/early skip on empty map, all ten background uploads, atomic publication and retained repeat-off passed"
        );
    }
    #[test]
    #[ignore = "Requires native audio libraries, soundtrack and output device"]
    fn asynchronous_native_publication_resolves_current_receiver() {
        let mut model = MusicPlayer::discover();
        let mut simulation = Simulation::default();
        simulation.music_volume = 0.;
        simulation.music_playing = false;
        let (release, wait) = std::sync::mpsc::channel();
        let mut session = Session::new_with_start(&model, 0., true, move || {
            wait.recv().unwrap();
        })
        .unwrap();
        let mut release = ReleaseOnDrop(Some(release));
        Session::reset_model(&mut model);
        let old_player = session.player.clone();
        crate::main_globals::initialize_player(old_player.clone());
        session.global_player = true;
        let replacement = Rc::new(RefCell::new(SourceMusicPlayer::new(
            Rc::new(RefCell::new(Tracks::default())),
            Arc::new(Mutex::new(session.native.clone())),
            &session.runtime,
        )));
        replacement.borrow().set_volume(0.);
        replacement.borrow_mut().set_paused(true);
        crate::main_globals::initialize_player(replacement.clone());
        release.release();
        finish_loading(&mut session, &mut model, &mut simulation);
        assert_eq!(
            session.tracks.borrow().0.len(),
            10,
            "Captured map receives the loaded buffers"
        );
        assert!(
            old_player.borrow().current_buffer().is_none(),
            "Old receiver is not advanced"
        );
        assert!(
            replacement.borrow().current_track.is_empty(),
            "Current receiver's empty map controls nextTrack"
        );
        assert!(
            model.tracks.is_empty() && model.current.is_none(),
            "UI follows current receiver map"
        );
        crate::al_util_kt::al_check(&mut session.native).unwrap();
        println!(
            "Native loader: captured map publication plus late current-player receiver and UI-map identity passed"
        );
    }
    #[test]
    #[ignore = "Requires native audio libraries, soundtrack and output device"]
    fn asynchronous_native_shutdown_joins_worker_before_context_close() {
        let model = MusicPlayer::discover();
        let session = Session::new(&model, 0., true).unwrap();
        assert!(!session.load_done);
        let player = session.player.clone();
        drop(session);
        assert!(player.borrow().source_freed());
        println!(
            "Immediate native shutdown joined the in-flight loader and closed the shared source device/context/source/buffer resource graph"
        );
    }
    #[derive(Resource, Default)]
    struct Next {
        skip: bool,
        playing: Option<bool>,
    }
    fn request_next(
        mut next: ResMut<Next>,
        mut model: ResMut<MusicPlayer>,
        mut simulation: ResMut<Simulation>,
    ) {
        if let Some(playing) = next.playing.take() {
            simulation.music_playing = playing;
        }
        if std::mem::take(&mut next.skip) {
            model.next_track();
        }
    }
    #[test]
    #[ignore = "Requires original audio libraries and output device; gated loading and muted shutdown"]
    fn registered_native_exit_joins_loader_before_shared_registry_cleanup() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::resource::ResourcePlugin));
        let runtime = app
            .world()
            .resource::<crate::resource::ResourceRuntime>()
            .clone();
        let freed = Arc::new(AtomicBool::new(false));
        let observed = freed.clone();
        let marker = runtime.allocate(&[], move || {
            observed.store(true, Ordering::SeqCst);
        });
        let model = MusicPlayer::discover();
        let class = crate::al_device::AlDeviceClass::default();
        let (release, wait) = std::sync::mpsc::channel();
        let (ready, started) = std::sync::mpsc::channel();
        let session = Session::new_in_runtime(
            &model,
            0.,
            true,
            move || {
                ready.send(()).unwrap();
                wait.recv().unwrap();
            },
            &class,
            runtime,
        )
        .unwrap();
        let context = session.context.clone();
        let device = session.device.clone();
        app.insert_non_send_resource(NativeMusic {
            session: Ok(session),
        })
        .add_systems(Last, shutdown.before(crate::resource::shutdown));
        let mut release = ReleaseOnDrop(Some(release));
        started
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        let release = release.0.take().unwrap();
        let premature = freed.clone();
        let helper = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(25));
            let closed_before_release = premature.load(Ordering::SeqCst);
            release.send(()).unwrap();
            closed_before_release
        });
        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.update();
        assert!(
            !helper.join().unwrap(),
            "shared registry cannot free the context while loader is gated"
        );
        assert!(marker.freed() && freed.load(Ordering::SeqCst));
        let native = app.world().non_send_resource::<NativeMusic>();
        let session = native.session.as_ref().unwrap();
        assert!(session.closed && context.freed() && device.freed());
        assert!(
            session.loader.as_ref().unwrap().worker.is_none(),
            "worker joined before graph close"
        );
        let mut count = 0;
        for event in session.loader.as_ref().unwrap().events.try_iter() {
            match event {
                crate::native_music_load::Event::Publish(tracks) => {
                    count = tracks.len();
                    assert!(tracks.iter().all(|(_, buffer)| buffer.freed()));
                }
                crate::native_music_load::Event::Failed(error) => {
                    panic!("loader lost context before completion: {error}")
                }
                crate::native_music_load::Event::NextTrack => {}
            }
        }
        assert_eq!(
            count, 10,
            "all worker buffers registered before shared closeAll"
        );
        assert_eq!(session.native.current_context_id(), 0);
        println!(
            "Registered AppExit: gated worker remains current until joined, then shared registry closes context/device/source and all ten newly loaded buffers"
        );
    }
    #[test]
    #[ignore = "Requires native audio libraries, soundtrack and output device"]
    fn registered_native_music_system_updates_ui_and_advances_before_controls() {
        use crate::al_source::AlSourceBackend;
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            crate::resource::ResourcePlugin,
        ));
        let marker = app
            .world()
            .resource::<crate::resource::ResourceRuntime>()
            .allocate(&[], || {});
        let mut simulation = crate::Simulation::default();
        simulation.music_volume = 0.;
        simulation.music_playing = false;
        app.insert_resource(simulation)
            .insert_resource(MusicPlayer::discover())
            .init_resource::<Next>()
            .add_plugins(NativeMusicPlugin)
            .add_systems(
                Update,
                request_next
                    .after(advance)
                    .before(crate::music_player::sync),
            )
            .add_systems(Update, crate::music_player::sync);
        let label = app
            .world_mut()
            .spawn((crate::MusicStatus, Text2d::new("")))
            .id();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            app.update();
            let native = app.world().non_send_resource::<NativeMusic>();
            let session = native.session.as_ref().unwrap();
            if session.load_done {
                assert!(session.load_error.is_none());
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(app.world().resource::<MusicPlayer>().current, Some(0));
        assert_eq!(
            app.world().get::<Text2d>(label).unwrap().0,
            app.world().resource::<MusicPlayer>().tracks[0].name
        );
        assert!(app.world().resource::<MusicPlayer>().max_progress > 60.);
        app.world_mut().resource_mut::<Next>().playing = Some(true);
        app.update();
        let (mut native, source, buffer) = {
            let native = app.world().non_send_resource::<NativeMusic>();
            let session = native.session.as_ref().unwrap();
            let player = session.player.borrow();
            assert!(player.source_playing());
            (
                session.native.clone(),
                player.source_id(),
                player.current_buffer().unwrap(),
            )
        };
        native.stop(source);
        // Source update advances track zero to one first; the UI skip then
        // consumes the next source queue entry, selecting track two.
        app.world_mut().resource_mut::<Next>().skip = true;
        app.update();
        assert_eq!(app.world().resource::<MusicPlayer>().current, Some(2));
        assert_eq!(
            app.world().get::<Text2d>(label).unwrap().0,
            app.world().resource::<MusicPlayer>().tracks[2].name
        );
        assert_eq!(native.source_integer(source, 4117), 1);
        assert_eq!(app.world().resource::<MusicPlayer>().queue.len(), 7);
        {
            let world = app.world_mut();
            let mut playback = world.query::<&AudioPlayer>();
            assert!(
                playback.iter(world).next().is_none(),
                "The registered Windows adapter must not spawn Bevy playback"
            );
        }
        crate::al_util_kt::al_check(&mut native).unwrap();
        drop(app);
        assert!(
            marker.freed(),
            "native session shutdown closes the same application registry"
        );
        assert!(buffer.freed(), "App teardown closes native buffers");
        println!(
            "Registered native sync: track label/duration, muted playback, source update then UI skip, one native buffer queue, no Bevy playback and app cleanup passed"
        );
    }
    #[test]
    #[ignore = "Requires native audio libraries, soundtrack and output device"]
    fn live_native_music_bridge_controls_queue_seek_and_cleanup() {
        let mut model = MusicPlayer::discover();
        let mut simulation = crate::Simulation::default();
        simulation.music_volume = 0.;
        simulation.music_playing = false;
        let mut session = Session::new(&model, 0., true).unwrap();
        Session::reset_model(&mut model);
        finish_loading(&mut session, &mut model, &mut simulation);
        session.tick(&mut model, &mut simulation);
        assert!(model.paused);
        assert_eq!(model.current, Some(0));
        assert!(model.max_progress > 60.);
        simulation.music_playing = true;
        session.tick(&mut model, &mut simulation);
        assert!(!model.paused);
        assert!(session.player.borrow().source_playing());
        simulation.music_playing = false;
        session.tick(&mut model, &mut simulation);
        model.pending_seek = Some(std::time::Duration::from_secs_f32(1.25));
        session.tick(&mut model, &mut simulation);
        assert_eq!(model.progress, 1.25);
        model.autonext = false;
        model.next_track();
        session.tick(&mut model, &mut simulation);
        assert_eq!(model.current, Some(1));
        assert!(model.paused);
        model.repeat = false;
        for _ in 1..model.tracks.len() {
            model.next_track();
            session.tick(&mut model, &mut simulation);
        }
        assert!(model.paused && model.queue.is_empty());
        let device = session.device.id();
        let source = session.player.borrow().source_id();
        let mut native = session.native.clone();
        drop(session);
        assert!(!native.is_source(source));
        crate::al_util_kt::al_check(&mut native).ok(); // no context remains
        println!(
            "Native live bridge: ten tracks, play/pause, exact paused seek, next/exhaustion and source/context/device ({device:#x}) teardown passed"
        );
    }
}

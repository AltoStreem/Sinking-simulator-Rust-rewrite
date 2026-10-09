//! Translation of MusicPlayer.java: remove-on-play queue, optional shuffle,
//! repeat of the entire queue, pause/resume, volume and playback position.
use crate::{MusicStatus, SettingAction, Simulation};
use bevy::asset::AssetId;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use bevy::{
    audio::{AudioSinkPlayback, Decodable, Volume},
    prelude::*,
};
use rand::{RngExt, SeedableRng};
use rodio::Source;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::{path::Path, time::Duration};

#[derive(Clone, Debug)]
pub(crate) struct Track {
    pub name: String,
    pub asset: String,
}

#[derive(Resource)]
pub(crate) struct MusicPlayer {
    pub tracks: Vec<Track>,
    pub(crate) queue: Vec<usize>,
    pub current: Option<usize>,
    pub paused: bool,
    pub autonext: bool,
    pub shuffle: bool,
    pub repeat: bool,
    pub progress: f32,
    pub max_progress: f32,
    pub pending_seek: Option<Duration>,
    pub(crate) revision: u64,
    pub(crate) next_requests: u64,
    duration_checked: bool,
}
impl MusicPlayer {
    pub fn new(tracks: Vec<Track>) -> Self {
        let mut player = Self {
            tracks,
            queue: Vec::new(),
            current: None,
            paused: false,
            autonext: true,
            shuffle: false,
            repeat: true,
            progress: 0.0,
            max_progress: 0.0,
            pending_seek: None,
            revision: 0,
            next_requests: 0,
            duration_checked: false,
        };
        player.next_track();
        player
    }
    pub fn discover() -> Self {
        let root = Path::new("assets/music");
        let mut tracks = Vec::new();
        // Main.main$1 is the recursive unsorted getFlatFiles traversal used by
        // the source coroutine before filtering exact lowercase .ogg extensions.
        for path in crate::main_flat_files::invoke(root) {
            let Some(file_name) = path.file_name().map(|name| name.to_string_lossy()) else {
                continue;
            };
            if crate::main_flat_files::kotlin_file_extension(&file_name) == "ogg" {
                let name = crate::main_flat_files::kotlin_file_name_without_extension(&file_name)
                    .to_owned();
                let asset = path
                    .strip_prefix("assets")
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if let Some(track) = tracks
                    .iter_mut()
                    .find(|track: &&mut Track| track.name == name)
                {
                    track.asset = asset;
                } else {
                    tracks.push(Track { name, asset });
                }
            }
        }
        // Main puts its insertion-ordered discovery map into java.util.HashMap.
        // Its keySet iterates table buckets, not alphabetically or by filename.
        let capacity = java_hash_map_capacity(tracks.len());
        tracks.sort_by_key(|track| java_track_bucket(&track.name, capacity));
        // Main constructs with an empty map, populates it, then calls nextTrack.
        // No pause is introduced while repeat is enabled.
        Self::new(tracks)
    }
    pub fn play_track(&mut self, track: usize) {
        assert!(track < self.tracks.len());
        self.current = Some(track);
        self.revision = self.revision.wrapping_add(1);
        self.progress = 0.0;
        self.max_progress = 0.0;
        self.duration_checked = false;
        self.pending_seek = None;
    }
    pub fn next_track(&mut self) {
        let seed = std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        self.next_with(|len| rng.random_range(0..len));
        self.next_requests = self.next_requests.wrapping_add(1);
    }
    fn next_with(&mut self, choose: impl FnOnce(usize) -> usize) {
        if self.queue.is_empty() {
            if self.repeat {
                self.queue.extend(0..self.tracks.len());
            } else {
                self.paused = true;
            }
        }
        if !self.queue.is_empty() {
            let selected = if self.shuffle {
                choose(self.queue.len())
            } else {
                0
            };
            let track = self.queue.remove(selected);
            self.play_track(track);
        }
    }
    pub fn update_stopped(&mut self, stopped: bool) {
        if stopped && self.autonext {
            self.next_track();
        }
    }
}

/// Main creates a default HashMap, then fills its unallocated table with putAll.
/// HashMap pre-sizes that first table from the source map size before inserting keys.
pub(crate) fn java_hash_map_capacity(entries: usize) -> usize {
    if entries == 0 {
        return 16; // No bucket is observed when the source map is empty.
    }
    const MAXIMUM_CAPACITY: usize = 1 << 30;
    let requested = entries as f32 / 0.75_f32 + 1.0_f32;
    if !requested.is_finite() || requested >= MAXIMUM_CAPACITY as f32 {
        return MAXIMUM_CAPACITY;
    }
    let required = (requested as usize).max(1);
    let mut capacity = required
        .checked_next_power_of_two()
        .unwrap_or(MAXIMUM_CAPACITY)
        .min(MAXIMUM_CAPACITY);
    while entries > capacity * 3 / 4 && capacity < MAXIMUM_CAPACITY {
        capacity = (capacity * 2).min(MAXIMUM_CAPACITY);
    }
    capacity
}

pub(crate) fn java_track_bucket(name: &str, capacity: usize) -> usize {
    let hash = name.encode_utf16().fold(0u32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(unit as u32)
    });
    ((hash ^ (hash >> 16)) as usize) & (capacity - 1)
}

fn decoded_duration(source: &AudioSource) -> f32 {
    let decoder = source.decoder();
    decoder
        .total_duration()
        .map(|value| value.as_secs_f32())
        .unwrap_or_else(|| {
            // ALBuffer likewise determines duration from fully decoded samples.
            let channels = decoder.channels().get() as f32;
            let sample_rate = decoder.sample_rate().get() as f32;
            decoder.count() as f32 / (channels * sample_rate)
        })
}

#[derive(Default)]
pub(crate) struct DurationCache {
    ready: HashMap<AssetId<AudioSource>, f32>,
    pending: HashMap<AssetId<AudioSource>, Task<f32>>,
}

#[derive(Component)]
pub(crate) struct Playback {
    revision: u64,
}

#[cfg(windows)]
pub(crate) use crate::native_music::sync;

#[cfg(not(windows))]
pub(crate) fn sync(
    mut commands: Commands,
    assets: Res<AssetServer>,
    sources: Res<Assets<AudioSource>>,
    mut player: ResMut<MusicPlayer>,
    mut simulation: ResMut<Simulation>,
    mut sinks: Query<(Entity, &Playback, Option<&mut AudioSink>)>,
    mut labels: Query<&mut Text2d, With<MusicStatus>>,
    mut icons: Query<(&MusicIcon, &mut Sprite)>,
    mut durations: Local<DurationCache>,
) {
    let finished: Vec<_> = durations
        .pending
        .iter_mut()
        .filter_map(|(id, task)| block_on(poll_once(task)).map(|duration| (*id, duration)))
        .collect();
    for (id, duration) in finished {
        durations.pending.remove(&id);
        durations.ready.insert(id, duration);
    }
    let stopped = sinks.iter().any(|(_, playback, sink)| {
        playback.revision == player.revision && sink.as_ref().is_some_and(|sink| sink.empty())
    });
    player.paused = !simulation.music_playing;
    player.update_stopped(stopped);
    simulation.music_playing = !player.paused;
    let revision = player.revision;
    let mut current_exists = false;
    for (entity, playback, sink) in &mut sinks {
        if playback.revision != revision {
            commands.entity(entity).despawn();
            continue;
        }
        current_exists = true;
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(simulation.music_volume));
            if player.paused {
                sink.pause();
            } else if sink.is_paused() {
                sink.play();
            }
            if let Some(position) = player.pending_seek.take() {
                if let Err(error) = sink.try_seek(position) {
                    bevy::log::warn!("Could not seek soundtrack: {error}");
                }
            }
            player.progress = sink.position().as_secs_f32();
        }
    }
    if let Some(index) = player.current {
        let handle: Handle<AudioSource> = assets.load(player.tracks[index].asset.clone());
        let id = handle.id();
        if let Some(duration) = durations.ready.get(&id) {
            player.max_progress = *duration;
            player.duration_checked = true;
        } else if !durations.pending.contains_key(&id) {
            if let Some(source) = sources.get(&handle) {
                let source = source.clone();
                let task =
                    AsyncComputeTaskPool::get().spawn(async move { decoded_duration(&source) });
                durations.pending.insert(id, task);
            }
        }
        if !current_exists {
            let mut settings =
                PlaybackSettings::ONCE.with_volume(Volume::Linear(simulation.music_volume));
            settings.paused = player.paused;
            commands.spawn((AudioPlayer::new(handle), settings, Playback { revision }));
        }
    }
    sync_display(&assets, &player, &mut labels, &mut icons);
}

pub(crate) fn sync_display(
    assets: &AssetServer,
    player: &MusicPlayer,
    labels: &mut Query<&mut Text2d, With<MusicStatus>>,
    icons: &mut Query<(&MusicIcon, &mut Sprite)>,
) {
    for (icon, mut sprite) in icons.iter_mut() {
        let name = match icon.0 {
            SettingAction::ToggleShuffle => {
                if player.shuffle {
                    "shuffle-disabled"
                } else {
                    "shuffle"
                }
            }
            SettingAction::ToggleRepeat => {
                if player.repeat {
                    "repeat-off"
                } else {
                    "repeat"
                }
            }
            SettingAction::ToggleMusic => {
                if player.paused {
                    "play"
                } else {
                    "pause"
                }
            }
            _ => "skip-next",
        };
        sprite.image = assets.load(format!("icons/music/{name}.png"));
    }
    for mut label in labels.iter_mut() {
        let name = player
            .current
            .map(|index| player.tracks[index].name.as_str())
            .unwrap_or("");
        label.0 = name.to_owned();
    }
}

#[derive(Component)]
pub(crate) struct MusicIcon(pub(crate) SettingAction);

#[cfg(test)]
mod tests {
    use super::*;
    fn tracks() -> Vec<Track> {
        (0..3)
            .map(|i| Track {
                name: i.to_string(),
                asset: String::new(),
            })
            .collect()
    }
    #[test]
    fn source_music_java_hash_buckets_preserve_keyset_order() {
        assert_eq!(java_track_bucket("Aa", 16), 0);
        assert_eq!(java_track_bucket("BB", 16), 0);
        assert_eq!(java_track_bucket("a", 16), 1);
        assert_eq!(java_track_bucket("b", 16), 2);
    }
    #[test]
    fn source_music_original_track_duration_decodes() {
        let bytes = std::fs::read("assets/music/Kevin Macleod - Night Vigil.ogg").unwrap();
        let source = AudioSource {
            bytes: bytes.into(),
        };
        let duration = decoded_duration(&source);
        assert!(
            duration > 60.0 && duration < 1800.0,
            "unexpected original soundtrack duration {duration}"
        );
    }
    #[test]
    fn source_music_plays_queue_once_then_refills_when_repeating() {
        let mut p = MusicPlayer::new(tracks());
        assert_eq!(p.current, Some(0));
        assert_eq!(p.queue, vec![1, 2]);
        p.next_track();
        assert_eq!(p.current, Some(1));
        p.next_track();
        assert_eq!(p.current, Some(2));
        p.next_track();
        assert_eq!(p.current, Some(0));
        assert_eq!(p.queue, vec![1, 2]);
    }
    #[test]
    fn source_music_exhaustion_pauses_without_restarting_current_track() {
        let mut p = MusicPlayer::new(tracks());
        p.repeat = false;
        p.next_track();
        p.next_track();
        let revision = p.revision;
        p.next_track();
        assert!(p.paused);
        assert_eq!(p.revision, revision);
        assert_eq!(p.current, Some(2));
    }
    #[test]
    fn source_music_shuffle_removes_chosen_track_and_preserves_pause() {
        let mut p = MusicPlayer::new(tracks());
        p.shuffle = true;
        p.paused = true;
        p.next_with(|len| len - 1);
        assert_eq!(p.current, Some(2));
        assert_eq!(p.queue, vec![1]);
        assert!(p.paused);
    }
    #[test]
    fn source_music_autonext_and_empty_playlist_follow_source_rules() {
        let mut p = MusicPlayer::new(tracks());
        p.autonext = false;
        p.update_stopped(true);
        assert_eq!(p.current, Some(0));
        p.autonext = true;
        p.update_stopped(false);
        assert_eq!(p.current, Some(0));
        p.update_stopped(true);
        assert_eq!(p.current, Some(1));
        let mut empty = MusicPlayer::new(vec![]);
        empty.next_track();
        assert_eq!(empty.current, None);
        empty.repeat = false;
        empty.next_track();
        assert!(empty.paused);
    }
}

/// Source Map<String, ALBuffer> operations; key iteration order comes from the
/// supplied map, rather than an invented alphabetic sort.
pub(crate) trait MusicTracks {
    fn keys(&self) -> Vec<String>;
    fn get(&self, name: &str) -> Option<std::sync::Arc<crate::al_buffer::AlBuffer>>;
}
/// Audio behavior from MusicPlayer.java. Source icon loading and drawUI still
/// use the existing Bevy adapter and require further native rendering conversion.
pub(crate) struct SourceMusicPlayer {
    pub tracks: std::rc::Rc<std::cell::RefCell<dyn MusicTracks>>,
    pub queue: Vec<String>,
    pub current_track: String,
    pub paused: bool,
    pub autonext: bool,
    pub shuffle: bool,
    pub repeat: bool,
    source: crate::al_source::AlSource,
    pub icons: Option<crate::music_player_icons::MusicIcons>,
}
impl SourceMusicPlayer {
    #[cfg(test)]
    pub(crate) fn source_playing(&self) -> bool { self.source.playing() }
    #[cfg(test)]
    pub(crate) fn source_id(&self) -> i32 { self.source.id() }
    #[cfg(test)]
    pub(crate) fn source_freed(&self) -> bool { self.source.freed() }
    pub fn new(
        tracks: std::rc::Rc<std::cell::RefCell<dyn MusicTracks>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::al_source::AlSourceBackend>>,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Self {
        let source = crate::al_source::AlSource::new(backend, runtime);
        let queue = tracks.borrow().keys();
        let current_track = queue.first().cloned().unwrap_or_default();
        let mut player = Self {
            tracks,
            queue,
            current_track,
            source,
            paused: false,
            autonext: true,
            shuffle: false,
            repeat: true,
            icons: None,
        };
        player.next_track();
        player
    }
    pub fn new_with_icons(
        tracks: std::rc::Rc<std::cell::RefCell<dyn MusicTracks>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::al_source::AlSourceBackend>>,
        runtime: &crate::resource::ResourceRuntime,
        load: impl FnMut(&str) -> Result<crate::texture_2d::SourceTexture2D, String>,
    ) -> Result<Self, String> {
        let mut player = Self::new(tracks, backend, runtime);
        player.icons = Some(crate::music_player_icons::MusicIcons::load(load)?);
        Ok(player)
    }
    pub fn new_native(
        tracks: std::rc::Rc<std::cell::RefCell<dyn MusicTracks>>,
        backend: std::sync::Arc<std::sync::Mutex<dyn crate::al_source::AlSourceBackend>>,
        textures: std::sync::Arc<std::sync::Mutex<dyn crate::texture::TextureBackend>>,
        context: crate::resource::ResourceHandle,
        runtime: &crate::resource::ResourceRuntime,
    ) -> Result<Self, String> {
        Self::new_with_icons(tracks, backend, runtime, |name| {
            crate::music_player_icon_config::load_icon(
                name,
                textures.clone(),
                context.clone(),
                runtime,
            )
        })
    }
    pub fn current_buffer(&self) -> Option<std::sync::Arc<crate::al_buffer::AlBuffer>> {
        self.tracks.borrow().get(&self.current_track)
    }
    pub fn set_paused(&mut self, value: bool) {
        if value && self.source.playing() {
            self.source.pause();
        } else if !value && !self.source.playing() {
            self.source.play();
        }
        self.paused = value;
    }
    pub fn volume(&self) -> f32 {
        self.source.volume()
    }
    pub fn set_volume(&self, value: f32) {
        self.source.set_volume(value);
    }
    pub fn progress(&self) -> f32 {
        self.source.seconds_offset()
    }
    pub fn set_progress(&self, value: f32) {
        self.source.set_seconds_offset(value);
    }
    pub fn max_progress(&self) -> f32 {
        self.current_buffer()
            .map(|buffer| buffer.duration())
            .unwrap_or(0.)
    }
    pub fn play_track(&mut self, name: String) {
        self.current_track = name;
        self.source.stop();
        self.source.unqueue_finished_buffers();
        if let Some(buffer) = self.current_buffer() {
            self.source.queue_buffer(&buffer);
        }
        if self.paused {
            self.source.pause();
        } else {
            self.source.play();
        }
    }
    pub fn next_track(&mut self) {
        let seed = std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        self.next_with(|len| rng.random_range(0..len));
    }
    pub fn next_with(&mut self, choose: impl FnOnce(usize) -> usize) {
        if self.queue.is_empty() {
            if self.repeat {
                self.queue.extend(self.tracks.borrow().keys());
            } else {
                self.set_paused(true);
            }
        }
        if !self.queue.is_empty() {
            let index = if self.shuffle {
                choose(self.queue.len())
            } else {
                0
            };
            let track = self.queue.remove(index);
            self.play_track(track);
        }
    }
    pub fn update(&mut self) {
        self.source.unqueue_finished_buffers();
        if self.source.stopped() && self.autonext {
            self.next_track();
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum MusicNumber {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}
impl MusicNumber {
    pub fn float_value(self) -> f32 {
        match self {
            Self::Int(v) => v as f32,
            Self::Long(v) => v as f32,
            Self::Float(v) => v,
            Self::Double(v) => v as f32,
        }
    }
}

#[cfg(test)]
mod source_audio_tests {
    use super::*;
    #[test]
    fn source_music_ui_preserves_controls_descriptions_and_property_writes() {
        struct Ui {
            events: Vec<String>,
            player: std::rc::Rc<std::cell::RefCell<SourceMusicPlayer>>,
        }
        impl crate::music_player_ui::Backend for Ui {
            fn frame_height(&mut self) -> f32 {
                self.events.push("height".into());
                24.
            }
            fn image_button(
                &mut self,
                _: i32,
                size: [f32; 2],
                uv0: [f32; 2],
                uv1: [f32; 2],
                padding: i32,
                background: [f32; 4],
                tint: [f32; 4],
            ) -> bool {
                assert_eq!((size, uv0, uv1, padding), ([24.; 2], [0.; 2], [1.; 2], 0));
                assert_eq!((background, tint), ([0.; 4], [1.; 4]));
                self.events.push("button".into());
                true
            }
            fn description(&mut self, text: &str) {
                // Native callbacks may reenter the player while describing an item.
                let _player = self.player.borrow_mut();
                self.events.push(text.into());
            }
            fn same_line(&mut self, offset: f32, spacing: f32) {
                assert_eq!((offset, spacing), (0., -1.));
                self.events.push("line".into());
            }
            fn volume_slider(
                &mut self,
                reference: crate::music_player_volume_reference::VolumeReference,
                label: &str,
                min: f32,
                max: f32,
                format: &str,
                power: f32,
            ) {
                assert_eq!(
                    (label, min, max, format, power),
                    ("Volume", 0., 1., "%.3f", 2.)
                );
                reference.set(Some(MusicNumber::Float(0.25)));
                self.events.push("volume".into());
            }
            fn progress_slider(
                &mut self,
                reference: crate::music_player_progress_reference::ProgressReference,
                label: &str,
                min: f32,
                max: f32,
                format: &str,
                power: f32,
            ) {
                assert_eq!(
                    (label, min, max, format, power),
                    ("Playing", 0., 0.5, "B", 1.)
                );
                reference.set(Some(MusicNumber::Float(0.125)));
                self.events.push("progress".into());
            }
        }
        let runtime = crate::resource::ResourceRuntime::default();
        let (_, tracks, backend, _) = fixture(&runtime, false);
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let textures = crate::render_fbo::source_tests::backend(events);
        let context = runtime.allocate(&[], || {});
        let player = std::rc::Rc::new(std::cell::RefCell::new(
            SourceMusicPlayer::new_native(tracks, backend, textures, context, &runtime).unwrap(),
        ));
        let mut ui = Ui {
            events: Vec::new(),
            player: player.clone(),
        };
        crate::music_player_ui::draw(player.clone(), &mut ui);
        assert_eq!(
            ui.events,
            [
                "height",
                "button",
                "Disable shuffling",
                "line",
                "button",
                "Enable loop",
                "line",
                "volume",
                "button",
                "Unpause music",
                "line",
                "button",
                "Skip track",
                "line",
                "progress"
            ]
        );
        assert!(player.borrow().shuffle && player.borrow().paused);
        assert!(!player.borrow().repeat);
        assert_eq!(player.borrow().volume(), 0.25);
        assert_eq!(player.borrow().progress(), 0.125);
    }
    #[test]
    fn native_constructor_loads_all_original_icons_after_starting_audio_and_stops_on_failure() {
        let runtime = crate::resource::ResourceRuntime::default();
        let (_, tracks, backend, audio) = fixture(&runtime, false);
        audio.lock().unwrap().clear();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let textures = crate::render_fbo::source_tests::backend(events.clone());
        let context = runtime.allocate(&[], || {});
        let mut names = Vec::new();
        let player =
            SourceMusicPlayer::new_with_icons(tracks.clone(), backend.clone(), &runtime, |name| {
                assert_eq!(audio.lock().unwrap().last().unwrap(), "play");
                names.push(name.to_owned());
                crate::music_player_icon_config::load_icon(
                    name,
                    textures.clone(),
                    context.clone(),
                    &runtime,
                )
            })
            .unwrap();
        assert_eq!(
            names,
            [
                "play",
                "pause",
                "repeat",
                "repeat-off",
                "shuffle",
                "shuffle-disabled",
                "skip-next"
            ]
        );
        let icons = player.icons.as_ref().unwrap();
        for icon in [
            &icons.play,
            &icons.pause,
            &icons.repeat,
            &icons.repeat_off,
            &icons.shuffle,
            &icons.shuffle_disabled,
            &icons.skip_next,
        ] {
            assert!(icon.width > 0 && icon.height > 0);
            assert_eq!(icon.internal_format, 32856);
        }
        names.clear();
        let failed = SourceMusicPlayer::new_with_icons(tracks, backend, &runtime, |name| {
            names.push(name.to_owned());
            if name == "repeat" {
                return Err("icon failure".into());
            }
            crate::music_player_icon_config::load_icon(
                name,
                textures.clone(),
                context.clone(),
                &runtime,
            )
        });
        assert!(matches!(failed, Err(error) if error == "icon failure"));
        assert_eq!(names, ["play", "pause", "repeat"]);
    }
    #[test]
    fn native_startup_music_publishes_captured_map_before_fetching_player() {
        use crate::main_music_load::{NativeOperations, invoke};
        struct Files;
        impl crate::main_flat_files::Files for Files {
            fn is_directory(&self, path: &Path) -> bool {
                path == Path::new("music")
            }
            fn list_files(&self, _: &Path) -> Option<Vec<std::path::PathBuf>> {
                Some(
                    ["first/A.ogg", "second/A.ogg"]
                        .map(std::path::PathBuf::from)
                        .into(),
                )
            }
        }
        let runtime = ResourceRuntime::default();
        let (player, tracks, _, log) = fixture(&runtime, true);
        assert_eq!(player.current_track, "");
        let player = Rc::new(RefCell::new(player));
        let captured = tracks.clone();
        let late_tracks = tracks.clone();
        let late_player = player.clone();
        let calls = Rc::new(std::cell::Cell::new(0));
        let late_calls = calls.clone();
        let (_, buffer_backend, buffer_log) = crate::al_buffer::tests::fixture(&runtime);
        let mut operations = NativeOperations {
            backend: buffer_backend,
            runtime: runtime.clone(),
            publish: Box::new(move |loaded| {
                captured.borrow_mut().0.extend(loaded);
                Ok(())
            }),
            player: Box::new(move || {
                assert_eq!(late_tracks.borrow().0.len(), 1);
                late_calls.set(late_calls.get() + 1);
                late_player.clone()
            }),
        };
        assert_eq!(calls.get(), 0);
        invoke(0, Ok(()), Path::new("music"), &Files, &mut operations).unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(player.borrow().current_track, "A");
        assert!(player.borrow().queue.is_empty());
        assert!(!player.borrow().paused);
        let buffer = tracks.borrow().0[0].1.clone();
        assert!(Arc::ptr_eq(
            &player.borrow().current_buffer().unwrap(),
            &buffer
        ));
        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "generate".to_owned(),
                "stop".into(),
                "query:4118".into(),
                format!("queue:{}", buffer.id()),
                "play".into()
            ]
        );
        assert_eq!(
            buffer_log
                .lock()
                .unwrap()
                .iter()
                .filter(|s| s.starts_with("file:"))
                .count(),
            2
        );
    }
    use crate::{al_buffer::AlBuffer, al_source::AlSourceBackend, resource::ResourceRuntime};
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, Mutex},
    };
    struct Tracks(Vec<(String, Arc<AlBuffer>)>);
    impl MusicTracks for Tracks {
        fn keys(&self) -> Vec<String> {
            self.0.iter().map(|(name, _)| name.clone()).collect()
        }
        fn get(&self, name: &str) -> Option<Arc<AlBuffer>> {
            self.0
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, buffer)| buffer.clone())
        }
    }
    struct Backend {
        log: Arc<Mutex<Vec<String>>>,
        state: i32,
        volume: f32,
        offset: f32,
    }
    impl AlSourceBackend for Backend {
        fn generate_source(&mut self) -> i32 {
            self.log.lock().unwrap().push("generate".into());
            3
        }
        fn delete_source(&mut self, _: i32) {}
        fn source_integer(&mut self, _: i32, p: i32) -> i32 {
            self.log.lock().unwrap().push(format!("query:{p}"));
            if p == 4118 {
                0
            } else if p == 4112 {
                self.state
            } else {
                0
            }
        }
        fn source_float(&mut self, _: i32, p: i32) -> f32 {
            if p == 4106 { self.volume } else { self.offset }
        }
        fn set_integer(&mut self, _: i32, _: i32, _: i32) {}
        fn set_float(&mut self, _: i32, p: i32, v: f32) {
            if p == 4106 {
                self.volume = v
            } else {
                self.offset = v
            }
        }
        fn queue_buffer(&mut self, _: i32, b: i32) {
            self.log.lock().unwrap().push(format!("queue:{b}"));
        }
        fn queue_buffers(&mut self, _: i32, _: &[i32]) {}
        fn unqueue_buffer(&mut self, _: i32) -> i32 {
            panic!("no processed buffers")
        }
        fn play(&mut self, _: i32) {
            self.state = 4114;
            self.log.lock().unwrap().push("play".into());
        }
        fn pause(&mut self, _: i32) {
            self.state = 4115;
            self.log.lock().unwrap().push("pause".into());
        }
        fn stop(&mut self, _: i32) {
            self.state = 4116;
            self.log.lock().unwrap().push("stop".into());
        }
    }
    fn fixture(
        runtime: &ResourceRuntime,
        empty: bool,
    ) -> (
        SourceMusicPlayer,
        Rc<RefCell<Tracks>>,
        Arc<Mutex<Backend>>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let (buffer, _, _) = crate::al_buffer::tests::fixture(runtime);
        let buffer = Arc::new(buffer);
        let tracks = Rc::new(RefCell::new(Tracks(if empty {
            vec![]
        } else {
            vec![("A".into(), buffer.clone()), ("B".into(), buffer)]
        })));
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            state: 4113,
            volume: 1.,
            offset: 0.,
        }));
        (
            SourceMusicPlayer::new(tracks.clone(), backend.clone(), runtime),
            tracks,
            backend,
            log,
        )
    }
    #[test]
    fn source_constructor_play_pause_missing_track_and_autonext_order() {
        let runtime = ResourceRuntime::default();
        let (mut player, _, backend, log) = fixture(&runtime, false);
        assert_eq!(player.current_track, "A");
        assert_eq!(player.queue, ["B"]);
        assert!(!player.paused);
        assert_eq!(
            *log.lock().unwrap(),
            ["generate", "stop", "query:4118", "queue:7", "play"]
        );
        assert_eq!(player.max_progress(), 0.5);
        log.lock().unwrap().clear();
        player.set_paused(true);
        player.set_paused(true);
        player.set_paused(false);
        assert_eq!(
            *log.lock().unwrap(),
            ["query:4112", "pause", "query:4112", "query:4112", "play"]
        );
        log.lock().unwrap().clear();
        player.play_track("missing".into());
        assert!(player.current_buffer().is_none());
        assert_eq!(player.max_progress(), 0.);
        assert_eq!(*log.lock().unwrap(), ["stop", "query:4118", "play"]);
        player.autonext = false;
        backend.lock().unwrap().state = 4116;
        player.update();
        assert_eq!(player.current_track, "missing");
        player.autonext = true;
        player.update();
        assert_eq!(player.current_track, "B");
    }
    #[test]
    fn empty_map_population_shuffle_exhaustion_and_property_references() {
        let runtime = ResourceRuntime::default();
        let (mut player, tracks, _, _) = fixture(&runtime, true);
        assert_eq!(player.current_track, "");
        assert!(!player.paused);
        let (buffer, _, _) = crate::al_buffer::tests::fixture(&runtime);
        let buffer = Arc::new(buffer);
        tracks
            .borrow_mut()
            .0
            .extend([("X".into(), buffer.clone()), ("Y".into(), buffer)]);
        player.shuffle = true;
        player.next_with(|len| len - 1);
        assert_eq!(player.current_track, "Y");
        assert_eq!(player.queue, ["X"]);
        player.repeat = false;
        player.next_track();
        assert_eq!(player.current_track, "X");
        player.next_track();
        assert!(player.paused);
        let player = Rc::new(RefCell::new(player));
        let volume = crate::music_player_volume_reference::VolumeReference {
            receiver: player.clone(),
        };
        let progress =
            crate::music_player_progress_reference::ProgressReference { receiver: player };
        assert_eq!(volume.name(), "volume");
        assert_eq!(volume.signature(), "getVolume()F");
        assert_eq!(progress.signature(), "getProgress()F");
        volume.set(Some(MusicNumber::Double(0.25)));
        assert_eq!(volume.get(), 0.25);
        progress.set(Some(MusicNumber::Long(16_777_217)));
        assert_eq!(progress.get(), 16_777_216.);
        volume.set(Some(MusicNumber::Float(f32::from_bits(0x7fc00001))));
        assert_eq!(volume.get().to_bits(), 0x7fc00001);
    }
    #[test]
    fn active_discovery_keeps_source_music_unpaused() {
        let player = MusicPlayer::discover();
        assert!(!player.tracks.is_empty());
        assert!(!player.paused);
        assert!(Simulation::default().music_playing);
    }
}

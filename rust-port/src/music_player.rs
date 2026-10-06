//! Translation of MusicPlayer.java: remove-on-play queue, optional shuffle,
//! repeat of the entire queue, pause/resume, volume and playback position.
use crate::{
    MusicStatus, SettingAction, SettingButton, Simulation, TabPage, ToolboxContent, ToolboxTab,
};
use bevy::asset::AssetId;
use bevy::camera::visibility::RenderLayers;
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
    queue: Vec<usize>,
    pub current: Option<usize>,
    pub paused: bool,
    pub autonext: bool,
    pub shuffle: bool,
    pub repeat: bool,
    pub progress: f32,
    pub max_progress: f32,
    pub pending_seek: Option<Duration>,
    revision: u64,
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
            duration_checked: false,
        };
        player.next_track();
        player
    }
    pub fn discover() -> Self {
        fn visit(path: &Path, tracks: &mut Vec<Track>) {
            if path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        visit(&entry.path(), tracks);
                    }
                }
            } else if path.extension().is_some_and(|extension| extension == "ogg") {
                let name = path.file_stem().unwrap().to_string_lossy().into_owned();
                let asset = path
                    .strip_prefix("assets")
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if let Some(track) = tracks.iter_mut().find(|track| track.name == name) {
                    track.asset = asset;
                } else {
                    tracks.push(Track { name, asset });
                }
            }
        }
        let mut tracks = Vec::new();
        visit(Path::new("assets/music"), &mut tracks);
        // Main puts its insertion-ordered discovery map into java.util.HashMap.
        // Its keySet iterates table buckets, not alphabetically or by filename.
        let capacity = ((tracks.len() * 4 / 3 + 1).max(1)).next_power_of_two();
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

fn java_track_bucket(name: &str, capacity: usize) -> usize {
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

pub(crate) fn sync(
    mut commands: Commands,
    assets: Res<AssetServer>,
    sources: Res<Assets<AudioSource>>,
    mut player: ResMut<MusicPlayer>,
    mut simulation: ResMut<Simulation>,
    mut sinks: Query<(Entity, &Playback, Option<&mut AudioSink>)>,
    mut labels: Query<&mut Text2d, With<MusicStatus>>,
    mut icons: Query<(&MusicIcon, &mut Sprite)>,
    mut progress: Query<&mut Text2d, (With<MusicProgress>, Without<MusicStatus>)>,
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
    for (icon, mut sprite) in &mut icons {
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
    for mut label in &mut progress {
        label.0 = format!("{:.1} / {:.1} s", player.progress, player.max_progress);
    }
    for mut label in &mut labels {
        let name = player
            .current
            .map(|index| player.tracks[index].name.as_str())
            .unwrap_or("");
        label.0 = format!(
            "{}  •  {}",
            name,
            if player.paused { "PAUSED" } else { "PLAYING" }
        );
    }
}

#[derive(Component)]
pub(crate) struct MusicIcon(SettingAction);
#[derive(Component)]
pub(crate) struct MusicProgress;

pub(crate) fn spawn_controls(commands: &mut Commands, assets: &AssetServer) {
    for (action, name, x) in [
        (SettingAction::ToggleShuffle, "shuffle", -545.0),
        (SettingAction::ToggleRepeat, "repeat-off", -505.0),
        (SettingAction::ToggleMusic, "play", -465.0),
        (SettingAction::NextTrack, "skip-next", -425.0),
    ] {
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            SettingButton(action),
            Sprite::from_color(Color::srgb(0.23, 0.25, 0.40), Vec2::splat(30.0)),
            Transform::from_xyz(x, 110.0, 24.5),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
        let mut sprite = Sprite::from_image(assets.load(format!("icons/music/{name}.png")));
        sprite.custom_size = Some(Vec2::splat(24.0));
        commands.spawn((
            ToolboxContent,
            TabPage(ToolboxTab::Music),
            MusicIcon(action),
            sprite,
            Transform::from_xyz(x, 110.0, 25.0),
            RenderLayers::layer(1),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        ToolboxContent,
        TabPage(ToolboxTab::Music),
        Sprite::from_color(Color::srgb(0.23, 0.25, 0.27), Vec2::new(230.0, 26.0)),
        Transform::from_xyz(-455.0, -5.0, 24.5),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
    commands.spawn((
        ToolboxContent,
        TabPage(ToolboxTab::Music),
        MusicProgress,
        Text2d::new("0.0 / 0.0 s"),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        Transform::from_xyz(-455.0, -5.0, 25.0),
        RenderLayers::layer(1),
        Visibility::Hidden,
    ));
}

pub(crate) fn handle_seek(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    simulation: Res<Simulation>,
    mut player: ResMut<MusicPlayer>,
) {
    if simulation.toolbox_collapsed
        || !matches!(simulation.active_tab, ToolboxTab::Music)
        || !mouse.pressed(MouseButton::Left)
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = 720.0 / window.height().max(1.0);
    let point = Vec2::new(
        (cursor.x - window.width() * 0.5) * scale,
        (window.height() * 0.5 - cursor.y) * scale,
    );
    if (-570.0..=-340.0).contains(&point.x) && (-18.0..=8.0).contains(&point.y) {
        let fraction = ((point.x + 570.0) / 230.0).clamp(0.0, 1.0);
        player.pending_seek = Some(Duration::from_secs_f32(player.max_progress * fraction));
    }
}

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
}
impl SourceMusicPlayer {
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
        };
        player.next_track();
        player
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

//! MusicPlayer.java drawUI call order and retained property references.
use crate::music_player::SourceMusicPlayer;
use std::{cell::RefCell, rc::Rc};
pub(crate) trait Backend {
    fn frame_height(&mut self) -> f32;
    fn image_button(
        &mut self,
        texture: i32,
        size: [f32; 2],
        uv0: [f32; 2],
        uv1: [f32; 2],
        padding: i32,
        background: [f32; 4],
        tint: [f32; 4],
    ) -> bool;
    fn description(&mut self, text: &str);
    fn same_line(&mut self, offset: f32, spacing: f32);
    fn volume_slider(
        &mut self,
        reference: crate::music_player_volume_reference::VolumeReference,
        label: &str,
        min: f32,
        max: f32,
        format: &str,
        power: f32,
    );
    fn progress_slider(
        &mut self,
        reference: crate::music_player_progress_reference::ProgressReference,
        label: &str,
        min: f32,
        max: f32,
        format: &str,
        power: f32,
    );
}
pub(crate) fn draw(player: Rc<RefCell<SourceMusicPlayer>>, ui: &mut impl Backend) {
    let id = {
        let p = player.borrow();
        let icons = p.icons.as_ref().expect("music icons not initialized");
        if p.shuffle {
            icons.shuffle_disabled.texture.id()
        } else {
            icons.shuffle.texture.id()
        }
    };
    let size = [ui.frame_height(); 2];
    if ui.image_button(id, size, [0.; 2], [1.; 2], 0, [0.; 4], [1.; 4]) {
        let mut p = player.borrow_mut();
        p.shuffle = !p.shuffle;
    }
    let description = if player.borrow().shuffle {
        "Disable shuffling"
    } else {
        "Enable shuffling"
    };
    ui.description(description);
    ui.same_line(0., -1.);
    let id = {
        let p = player.borrow();
        let icons = p.icons.as_ref().unwrap();
        if p.repeat {
            icons.repeat_off.texture.id()
        } else {
            icons.repeat.texture.id()
        }
    };
    if ui.image_button(id, size, [0.; 2], [1.; 2], 0, [0.; 4], [1.; 4]) {
        let mut p = player.borrow_mut();
        p.repeat = !p.repeat;
    }
    let description = if player.borrow().repeat {
        "Disable loop"
    } else {
        "Enable loop"
    };
    ui.description(description);
    ui.same_line(0., -1.);
    ui.volume_slider(
        crate::music_player_volume_reference::VolumeReference {
            receiver: player.clone(),
        },
        "Volume",
        0.,
        1.,
        "%.3f",
        2.,
    );
    let id = {
        let p = player.borrow();
        let icons = p.icons.as_ref().unwrap();
        if p.paused {
            icons.play.texture.id()
        } else {
            icons.pause.texture.id()
        }
    };
    if ui.image_button(id, size, [0.; 2], [1.; 2], 0, [0.; 4], [1.; 4]) {
        let value = !player.borrow().paused;
        player.borrow_mut().set_paused(value);
    }
    let description = if player.borrow().paused {
        "Unpause music"
    } else {
        "Pause music"
    };
    ui.description(description);
    ui.same_line(0., -1.);
    let id = player
        .borrow()
        .icons
        .as_ref()
        .unwrap()
        .skip_next
        .texture
        .id();
    if ui.image_button(id, size, [0.; 2], [1.; 2], 0, [0.; 4], [1.; 4]) {
        player.borrow_mut().next_track();
    }
    ui.description("Skip track");
    ui.same_line(0., -1.);
    let max = player.borrow().max_progress();
    let format = player.borrow().current_track.clone();
    ui.progress_slider(
        crate::music_player_progress_reference::ProgressReference { receiver: player },
        "Playing",
        0.,
        max,
        &format,
        1.,
    );
}

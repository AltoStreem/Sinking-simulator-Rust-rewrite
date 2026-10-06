//! MusicPlayer$drawUI$1$1.java volume property reference.
#![allow(dead_code)]
use crate::music_player::{MusicNumber, SourceMusicPlayer};
use std::{cell::RefCell, rc::Rc};
pub(crate) struct VolumeReference {
    pub receiver: Rc<RefCell<SourceMusicPlayer>>,
}
impl VolumeReference {
    pub fn name(&self) -> &'static str {
        "volume"
    }
    pub fn signature(&self) -> &'static str {
        "getVolume()F"
    }
    pub fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.MusicPlayer"
    }
    pub fn get(&self) -> f32 {
        self.receiver.borrow().volume()
    }
    pub fn set(&self, value: Option<MusicNumber>) {
        self.receiver.borrow().set_volume(
            value
                .expect("volume property requires Number")
                .float_value(),
        );
    }
}

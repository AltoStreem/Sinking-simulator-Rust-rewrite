//! MusicPlayer$drawUI$1$2.java progress property reference.
#![allow(dead_code)]
use crate::music_player::{MusicNumber, SourceMusicPlayer};
use std::{cell::RefCell, rc::Rc};
pub(crate) struct ProgressReference {
    pub receiver: Rc<RefCell<SourceMusicPlayer>>,
}
impl ProgressReference {
    pub fn name(&self) -> &'static str {
        "progress"
    }
    pub fn signature(&self) -> &'static str {
        "getProgress()F"
    }
    pub fn owner(&self) -> &'static str {
        "com.wicpar.sinkingsimulator.MusicPlayer"
    }
    pub fn get(&self) -> f32 {
        self.receiver.borrow().progress()
    }
    pub fn set(&self, value: Option<MusicNumber>) {
        self.receiver.borrow().set_progress(
            value
                .expect("progress property requires Number")
                .float_value(),
        );
    }
}

//! MusicPlayer.java's retained icons and sequential constructor loads.
use crate::texture_2d::SourceTexture2D;
pub(crate) struct MusicIcons {
    pub play: SourceTexture2D,
    pub pause: SourceTexture2D,
    pub repeat: SourceTexture2D,
    pub repeat_off: SourceTexture2D,
    pub shuffle: SourceTexture2D,
    pub shuffle_disabled: SourceTexture2D,
    pub skip_next: SourceTexture2D,
}
impl MusicIcons {
    pub fn load(
        mut load: impl FnMut(&str) -> Result<SourceTexture2D, String>,
    ) -> Result<Self, String> {
        let play = load("play")?;
        let pause = load("pause")?;
        let repeat = load("repeat")?;
        let repeat_off = load("repeat-off")?;
        let shuffle = load("shuffle")?;
        let shuffle_disabled = load("shuffle-disabled")?;
        let skip_next = load("skip-next")?;
        Ok(Self {
            play,
            pause,
            repeat,
            repeat_off,
            shuffle,
            shuffle_disabled,
            skip_next,
        })
    }
}

use strum::Display;
use crate::models::song::SongList;

#[derive(Default, PartialEq, Display)]
pub enum InputMode {
    Nav,
    #[default]
    Input,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Focus {
    Help,
    #[default]
    Home,
    Lyrics,
    Queue,
    Search,
    Timer,
}

#[derive(Default, PartialEq)]
pub struct GlobalState {
    pub(crate) current_song: Option<String>,
    pub(crate) current_song_index: usize,
    pub(crate) songs: SongList,
    pub(crate) mode: InputMode,
    pub(crate) focus: Focus,
}

impl GlobalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            current_song: None,
            current_song_index: 0,
            songs: Vec::new(),
            mode: InputMode::Nav,
            focus: Focus::Home,
        }
    }
}

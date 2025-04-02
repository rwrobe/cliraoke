use std::time::Duration;
use strum::Display;
use crate::components::timer::Timer;
use crate::models::song::{Song, SongList};
use crate::state::SongState::Paused;

#[derive(Default, PartialEq, Display, Debug)]
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
    Queue,
    Search,
    Timer,
}

#[derive(Debug, Default, PartialEq)]
pub enum SongState{
    #[default]
    Playing,
    Paused,
}

#[derive(Default, PartialEq, Debug)]
pub struct GlobalState {
    // TODO this is too complicated
    pub(crate) song_state: SongState,
    pub(crate) current_song: Option<Song>,
    pub(crate) current_song_index: usize,
    pub(crate) current_song_elapsed: u64,
    pub(crate) current_lyric: String,
    pub(crate) songs: SongList,
    pub(crate) mode: InputMode,
    pub(crate) focus: Focus,
    pub(crate) session_time_elapsed: Duration,
}

impl GlobalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            song_state: Paused,
            current_song: None,
            current_song_index: 0,
            current_song_elapsed: 0,
            current_lyric: String::new(),
            songs: Vec::new(),
            mode: InputMode::Nav,
            focus: Focus::Home,
            session_time_elapsed: Duration::new(0, 0),
        }
    }
}

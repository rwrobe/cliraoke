use std::sync::{Arc, Mutex, MutexGuard};
use crate::models::song::{Song, SongList};
use crate::state::SongState::Paused;
use std::time::Duration;
use strum::Display;

#[derive(Default, Clone, PartialEq, Display, Debug)]
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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SongState{
    #[default]
    Playing,
    Paused,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct GlobalState {
    // current_lyrics will be a set of 3 lines of lyrics, where index 1 is the current lyric.
    pub(crate) current_lyrics: Vec<String>,
    pub(crate) current_song: Option<Song>,
    pub(crate) current_song_elapsed: u64,
    pub(crate) focus: Focus,
    pub(crate) mode: InputMode,
    pub(crate) session_time_elapsed: Duration,
    pub(crate) song_list: SongList,
    pub(crate) song_state: SongState,
}

impl GlobalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            song_state: Paused,
            current_song: None,
            current_song_elapsed: 0,
            current_lyrics: Vec::new(),
            song_list: Vec::new(),
            mode: InputMode::Nav,
            focus: Focus::Home,
            session_time_elapsed: Duration::new(0, 0),
        }
    }
}

// This is a global state that will be shared across the application.
pub type AMGlobalState = Arc<Mutex<GlobalState>>;

// -- Helper functions for working with global state.

pub fn get_state(state: &AMGlobalState) -> GlobalState {
    let guard = state.lock().unwrap();
    guard.clone()
}

pub fn get_guarded_state(state: &AMGlobalState) -> MutexGuard<GlobalState> {
    state.lock().unwrap()
}

// With closure that will be called with a mutable reference to the global state.
pub fn with_state<F, R>(state: &AMGlobalState, f: F) -> R
where
    F: FnOnce(&mut GlobalState) -> R,
{
    let mut guard = state.lock().unwrap();
    f(&mut guard)
}
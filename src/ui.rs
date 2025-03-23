use crate::app::{App, Song};
use ratatui::widgets::ListState;
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &App) {}

#[derive(Debug, Default, PartialEq)]
pub enum UIState {
    Lyrics,
    Queue,
    #[default]
    Search,
    SelectAudio,
    SelectLyrics,
}

#[derive(Debug, Default, PartialEq)]
pub enum UIMode {
    Edit,
    #[default]
    Navigation,
}

pub struct SearchState {
    pub query: String,
    pub results: Vec<Song>,
    pub stat: ListState,
}
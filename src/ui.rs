use crate::app::{App, Song};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{BLUE, GREEN, SLATE},
    },
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

pub fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [header_area, main_area, footer_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    Paragraph::new("Ratatui List Example")
        .bold()
        .centered()
        .render(area, buf);

    match app.ui_state {
        UIState::Lyrics => {
            // Draw lyrics
        }
        UIState::Queue => {
            // Draw queue
        }
        UIState::Search => {
            // Draw search
        }
        UIState::SelectAudio => {
            // Draw select audio
        }
        UIState::SelectLyrics => {
            // Draw select lyrics
        }
    }
}

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

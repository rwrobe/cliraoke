use ratatui::style::Stylize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use block::Title;
use super::RenderableComponent;
use crate::app::GlobalState;
use color_eyre::eyre::Result;
use ratatui::widgets::block;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::Block,
    Frame,
};
use serde_json::Value::String;
use crate::models::song::Song;

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
    SongRemainingTicker,
}

#[derive(Debug)]
pub struct Timer {
    global_state: Arc<Mutex<GlobalState>>,
}

impl Timer {
    pub fn new(state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            global_state: state
        }
    }
}

impl RenderableComponent for Timer {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
        state: Arc<Mutex<GlobalState>>,
    ) -> anyhow::Result<()> {
        let global_state = state.lock().unwrap();

        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect);

        let (left, right) = (rects[0], rects[1]);

        // Song remaining time as ms.
        let song_remaining_time = global_state.songs.get(global_state.current_song_index)
            .map(|song| song.duration_ms - global_state.current_song_elapsed)
            .unwrap_or(0);

        let s = format!(
            "Singing for {:02}:{:02}",
            global_state.session_time_elapsed.as_secs() / 60,
            global_state.session_time_elapsed.as_secs() % 60,
        );
        let time_singing = Block::default().title(Title::from(s.dim()).alignment(Alignment::Left));
        f.render_widget(time_singing, left);

        let next_song = global_state.songs.get(global_state.current_song_index + 1);

        if next_song.is_some() {
            let next_song_remaining = format!(
                ". {:02}:{:02} until {}",
                song_remaining_time / 60_000,
                (song_remaining_time % 60_000) / 1000,
                global_state.songs[global_state.current_song_index].title
            );

            let time_to_next = Block::default().title(Title::from(next_song_remaining.dim()).alignment(Alignment::Right));

            f.render_widget(time_to_next, right)
        }
        Ok(())
    }
}

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
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(rect);

        let song_remaining_time = global_state.songs.get(global_state.current_song_index)
            .map(|song| song.duration - global_state.song_time_elapsed)
            .unwrap_or(Duration::new(0, 0));

        let s = format!(
            "Time singing {:02}:{:02}. {:02}:{:02} until {}",
            global_state.session_time_elapsed.as_secs() / 60,
            global_state.session_time_elapsed.as_secs() % 60,
            song_remaining_time.as_secs() / 60,
            song_remaining_time.as_secs() % 60,
            global_state.current_song.unwrap_or(Song::default()).title
        );
        let block = Block::default().title(Title::from(s.dim()).alignment(Alignment::Right));
        f.render_widget(block, rects[0]);
        Ok(())
    }
}

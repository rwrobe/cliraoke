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

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
    SongRemainingTicker,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
    song_remaining_time: Duration, // TODO global state
}

impl Timer {
    pub fn new() -> Self {
        Self {
            song_remaining_time: Duration::from_secs(0),
        }
    }

    fn app_tick(&mut self) -> Result<()> {
        // TODO: song duration countdown
        Ok(())
    }
}

impl RenderableComponent for Timer {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
        state: Arc<Mutex<GlobalState>>,
    ) -> anyhow::Result<()> {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(rect);

        let rect = rects[0];

        let s = format!(
            "{:02}:{:02} until {}",
            self.song_remaining_time.as_secs() / 60,
            self.song_remaining_time.as_secs() % 60,
            "next song placeholder"
        );
        let block = Block::default().title(Title::from(s.dim()).alignment(Alignment::Right));
        f.render_widget(block, rect);
        Ok(())
    }
}

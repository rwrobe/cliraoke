use ratatui::style::Stylize;
use ratatui::widgets::block::Title;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use ratatui::widgets::block;
use super::{ RenderableComponent};
use crate::{action::Action};

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
    SongRemainingTicker,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
    song_remaining_time: Duration,
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
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
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
        let block = Block::default().title(block::Title::from(s.dim()).alignment(Alignment::Right));
        f.render_widget(block, rect);
        Ok(())
    }
}

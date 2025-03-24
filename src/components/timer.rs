use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, tui::Frame};

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
  SongRemainingTicker,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
  song_remaining_time: Duration,
}

impl Default for Timer {
  fn default() -> Self {
    Self::new()
  }
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

impl Component for Timer {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::Tick = action {
      self.app_tick()?
    };
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![
        Constraint::Length(1),
        Constraint::Min(0),
      ])
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

use super::{queue, timer, Component, Frame};
use crate::action::Action;
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use ratatui::{prelude::*, widgets::*};

pub struct Help;

impl Help {
  pub fn new() -> Self {
    Self
  }
}

impl Component for Help {
  fn update(&mut self, _action: Action) -> Result<Option<Action>> {
    Ok(None)
  }

  fn draw(&mut self, frame: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let help_text = Line::from(vec![
      "Press ".into(),
      Span::styled("u ", Style::default().fg(Color::Red)),
      "to see the ".into(),
      Span::styled("queue", Style::default().fg(Color::Yellow)),
      ", ".into(),
      Span::styled("/ ", Style::default().fg(Color::Red)),
      "to ".into(),
      Span::styled("search ", Style::default().fg(Color::Yellow)),
      "for a song, ".into(),
      "and ".into(),
      Span::styled("q ", Style::default().fg(Color::Red)),
      "to ".into(),
      Span::styled("quit", Style::default().fg(Color::Yellow)),
    ]);

    let help_text = Paragraph::new(help_text)
      .wrap(Wrap { trim: true })
      .block(Block::default().borders(Borders::ALL).title("Help"))
      .alignment(Alignment::Center);

    frame.render_widget(help_text, rect);

    Ok(())
  }
}

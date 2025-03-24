use super::{queue, timer, Component, Frame};
use crate::action::Action;
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tui_input::Input;

pub struct Search {
  pub query: Input,
}

impl Search {
  pub fn new() -> Self {
    Self { query: Input::default() }
  }
}

impl Component for Search {
  fn update(&mut self, _action: Action) -> Result<Option<Action>> {
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = self.query.visual_scroll(width as usize);

    let input = Paragraph::new(self.query.value())
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Yellow))
      .scroll((0, scroll as u16))
      .block(Block::default().borders(Borders::ALL).title_alignment(Alignment::Center).title(Line::from(vec![
        Span::raw("Search for a song "),
        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightRed)),
        Span::styled(" to submit)", Style::default().fg(Color::DarkGray)),
      ])));

    f.render_widget(input, rect);

    f.set_cursor(
      (rect.x + (rect.width / 2) - (self.query.cursor() as u16 / 2)).min(rect.x + rect.width - 2),
      rect.y + 1,
    );

    Ok(())
  }
}

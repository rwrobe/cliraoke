use super::RenderableComponent;
use ratatui::{prelude::*, widgets::*};
use ratatui::{prelude::*, widgets::*};

pub struct Help;

impl Help {
  pub fn new() -> Self {
    Self
  }
}

impl RenderableComponent for Help {
  fn render<B: Backend>(
    &self,
    f: &mut ratatui::Frame<B>,
    rect: Rect,
    focused: bool,
  ) -> anyhow::Result<()> {
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

    f.render_widget(help_text, rect);

    Ok(())
  }
}

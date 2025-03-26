use super::{queue, timer, Component, Frame};
use crate::action::Action;
use crate::tui::Event;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use log::error;
use ratatui::{prelude::*, widgets::*};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Default)]
pub struct Search {
    pub query: Input,
    pub action_tx: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            query: Input::default(),
            action_tx: None,
        }
    }
}

impl Component for Search {
    fn register_action_handler(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Esc => Action::GoHome,
            KeyCode::Enter => {
                if let Some(sender) = &mut self.action_tx {
                    if let Err(e) = sender.send(Action::SearchSong(self.query.value().to_string()))
                    {
                        error!("Failed to send action: {:?}", e);
                    }
                    sender.send(Action::SearchSong(self.query.value().to_string()))?;
                    sender.send(Action::ToggleSearch)?;
                    self.query.reset();
                }
                return Ok(None);
            }
            KeyCode::Char('/') => {
                self.query.reset();
                if let Some(sender) = &mut self.action_tx {
                    sender.send(Action::ToggleSearch)?;
                }
                return Ok(None);
            }
            _ => {
                self.query.handle_event(&crossterm::event::Event::Key(key));
                return Ok(None);
            }
        };
        Ok(None)
    }

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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center)
                    .title(Line::from(vec![
                        Span::raw("Search for a song "),
                        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "ENTER",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightRed),
                        ),
                        Span::styled(" to submit)", Style::default().fg(Color::DarkGray)),
                    ])),
            );

        f.render_widget(input, rect);

        f.set_cursor(
            (rect.x + (rect.width / 2) - (self.query.cursor() as u16 / 2))
                .min(rect.x + rect.width - 2),
            rect.y + 1,
        );

        Ok(())
    }
}

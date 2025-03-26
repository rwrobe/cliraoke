use super::{queue, timer, Frame, RenderableComponent};
use crate::{
    action::Action,
    events::{EventState, Key},
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use log::error;
use ratatui::{prelude::*, widgets::*};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use ratatui::{
    widgets::{List, ListState}
};
use crate::components::stateful_list::StatefulList;

#[derive(Default)]
pub struct Search<'search> {
    query: Input,
    audio_results: StatefulList<'search>,
    lyric_results: StatefulList<'search>,
}

impl<'a> Search<'a> {
    pub fn new() -> Self {
        Self {
            query: Input::default(),
            audio_results: StatefulList::default(),
            lyric_results: StatefulList::default(),
        }
    }

    fn add_to_query(&mut self, key: KeyEvent) {
        self.query.handle_event(&crossterm::event::Event::Key(key));
    }

    fn search(&mut self) {
        let query = self.query.value();
        if query.is_empty() {
            return;
        }

        // TODO: search for songs
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            k if k == Key::Enter => {
                self.search();
                return Ok(EventState::Consumed);
            }
            k if k == Key::Char('/') => {
                self.query.reset();
                return Ok(EventState::Consumed);
            }
            k if k == Key::Esc => {
                self.query.reset();
                return Ok(EventState::Consumed);
            }
            _ => {
                self.add_to_query(key.into());
            }
        }
        Ok(EventState::NotConsumed)
    }
}

impl<'search> RenderableComponent for Search<'search> {
    fn render<B: Backend>(
        &self,
        f: &mut ratatui::Frame<B>,
        rect: Rect,
        focused: bool,
    ) -> anyhow::Result<()> {
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

        Ok(())
    }
}

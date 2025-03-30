use super::{Frame, RenderableComponent};
use crate::app::GlobalState;
use crate::components::stateful_list::StatefulList;
use crate::events::{EventState, Key};
use crate::models::song::Song;
use crate::state::{Focus, InputMode};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::sync::{Arc, Mutex};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Default)]
pub struct Search<'a> {
    query: Input,
    audio_results: StatefulList<'a>,
    lyric_results: StatefulList<'a>,
    global_state: Arc<Mutex<GlobalState>>,
}

impl Search<'_> {
    pub fn new(state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            query: Input::default(),
            audio_results: StatefulList::default(),
            lyric_results: StatefulList::default(),
            global_state: state,
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

        {
            // Add it to the global state songs list.
            let mut global_state = self.global_state.lock().unwrap();
            global_state.songs.push(Song {
                lyric_id: "".to_string(),
                video_id: "".to_string(),
                title: query.to_string(),
                artist: "Unknown".to_string(),
                synced_lyrics: "".to_string(),
                lyric_map: None,
                message: (),
            })
        }

        self.query.reset()
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        if self.global_state.lock().unwrap().mode == InputMode::Nav {
            return Ok(EventState::NotConsumed);
        }

        match key {
            k if k == Key::Enter => {
                self.search();
                {
                    let mut global_state = self.global_state.lock().unwrap();
                    global_state.mode = InputMode::Nav;
                    global_state.focus = Focus::Queue;
                }
                return Ok(EventState::Consumed);
            }
            k if k == Key::Char('/') => {
                self.query.reset();
                self.global_state.lock().unwrap().mode = InputMode::Input;
                return Ok(EventState::Consumed);
            }
            k if k == Key::Esc => {
                self.query.reset();
                return Ok(EventState::Consumed);
            }
            Key::Char(v) => {
                self.add_to_query(KeyEvent::new(KeyCode::Char(v), KeyModifiers::NONE));
                return Ok(EventState::Consumed);
            }
            _ => {}
        }
        Ok(EventState::NotConsumed)
    }
}

impl RenderableComponent for Search<'_> {
    fn render<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
        state: Arc<Mutex<GlobalState>>,
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
                        Span::styled(
                            format!(" to submit) {}", self.global_state.lock().unwrap().mode),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])),
            );

        f.render_widget(input, rect);

        Ok(())
    }
}

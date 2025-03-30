use super::{Frame, RenderableComponent};
use crate::app::GlobalState;
use crate::audio::AudioService;
use crate::components::stateful_list::StatefulList;
use crate::events::{EventState, Key};
use crate::lyrics::LyricsService;
use crate::models::song::Song;
use crate::state::{Focus, InputMode};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::sync::{Arc, Mutex};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub struct Search<'a> {
    audio_service: &'a dyn AudioService,
    audio_results: StatefulList<'a>,
    lyric_results: StatefulList<'a>,
    lyrics_service: &'a dyn LyricsService,

    query: Input,
    global_state: Arc<Mutex<GlobalState>>,
}

impl<'b> Search<'b> {
    pub fn new(
        state: Arc<Mutex<GlobalState>>,
        lp: &'b (dyn LyricsService + 'b),
        ap: &'b (dyn AudioService + 'b),
    ) -> Self {
        Self {
            audio_service: ap,
            audio_results: StatefulList::default(),
            lyric_results: StatefulList::default(),
            lyrics_service: lp,

            query: Input::default(),
            global_state: state,
        }
    }

    fn add_to_query(&mut self, key: KeyEvent) {
        self.query.handle_event(&crossterm::event::Event::Key(key));
    }

    async fn search(&mut self) {
        let query = self.query.value();
        if query.is_empty() {
            return;
        }

        // Search audio.
        let audio_results = self.audio_service.search(query).await;
        match audio_results {
            Ok(results) => {
                self.audio_results = StatefulList::with_items(results.into_iter().map(|r| {
                    let item = ListItem::new(r.title);
                    item
                }).collect(), None);
            }
            Err(e) => {
                println!("Error searching audio: {}", e);
            }
        }

        // Search lyrics.
        let lyric_results = self.lyrics_service.search(query).await;
        match lyric_results {
            Ok(results) => {
                self.lyric_results = StatefulList::with_items(results.into_iter().map(|r| {
                    let item = ListItem::new(r.title);
                    item
                }).collect(), None);
            }
            Err(e) => {
                println!("Error searching lyrics: {}", e);
            }
        }

        self.query.reset()
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        if self.global_state.lock().unwrap().mode == InputMode::Nav {
            return Ok(EventState::NotConsumed);
        }

        match key {
            k if k == Key::Enter => {
                self.search().await;
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

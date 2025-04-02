use super::{Frame, RenderableComponent, ResettableComponent};
use crate::app::GlobalState;
use crate::audio::{AudioResult, AudioFetcher};
use crate::components::stateful_list::StatefulList;
use crate::events::{EventState, Key};
use crate::lyrics::{LyricsResult, LyricsFetcher};
use crate::models::song::Song;
use crate::state::{Focus, InputMode};
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, Default, PartialEq)]
enum SearchFocus {
    #[default]
    Input,
    Audio,
    Lyrics,
}

impl ResettableComponent for StatefulList<'_> {
    fn reset(&mut self) {
        self.items.clear();
        self.state.select(None);
    }
}

pub struct Search<'a> {
    audio_presentation_list: StatefulList<'a>,
    audio_results: Vec<AudioResult>,
    audio_fetcher: &'a dyn AudioFetcher,
    audio_state: ListState,

    lyrics_presentation_list: StatefulList<'a>,
    lyric_results: Vec<LyricsResult>,
    lyrics_service: &'a dyn LyricsFetcher,
    lyrics_state: ListState,

    focus: SearchFocus,
    global_state: Arc<Mutex<GlobalState>>,
    query: Input,
}

impl<'b> Search<'b> {
    pub fn new(
        state: Arc<Mutex<GlobalState>>,
        lp: &'b (dyn LyricsFetcher + 'b),
        ap: &'b (dyn AudioFetcher + 'b),
    ) -> Self {
        Self {
            audio_presentation_list: StatefulList::default(),
            audio_results: vec![],
            audio_fetcher: ap,
            audio_state: ListState::default(),

            lyrics_presentation_list: StatefulList::default(),
            lyric_results: vec![],
            lyrics_service: lp,
            lyrics_state: ListState::default(),

            focus: SearchFocus::Input,
            global_state: state,
            query: Input::default(),
        }
    }

    async fn search(&mut self) {
        let query = self.query.value();
        if query.is_empty() {
            return;
        }

        // Search audio.
        let audio_results = self.audio_fetcher.search(query).await;
        match audio_results {
            Ok(results) => {
                self.audio_results = results.clone();
                self.audio_presentation_list = StatefulList::with_items(
                    results
                        .into_iter()
                        .map(|r| {
                            let item = ListItem::new(r.title);
                            item
                        })
                        .collect(),
                    None,
                );
            }
            Err(e) => {
                println!("Error searching audio: {}", e);
            }
        }

        // Search lyrics.
        let lyric_results = self.lyrics_service.search(query).await;
        match lyric_results {
            Ok(results) => {
                self.lyric_results = results.clone();
                self.lyrics_presentation_list = StatefulList::with_items(
                    results
                        .into_iter()
                        .take(5)
                        .map(|r| {
                            let item = ListItem::new(format!("{} by {}", r.title, r.artist));
                            item
                        })
                        .collect(),
                    None,
                );
            }
            Err(e) => {
                println!("Error searching lyrics: {}", e);
            }
        }
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        // TODO: This should live elsewhere, so we don't create this for every keystroke.
        let mut song = Song::new();

        match (&self.focus, key) {
            // Component-level bindings.
            (SearchFocus::Input | SearchFocus::Audio | SearchFocus::Lyrics, Key::Tab) => {
                match self.focus {
                    SearchFocus::Input => {
                        self.focus = SearchFocus::Audio;
                    }
                    SearchFocus::Audio => {
                        self.focus = SearchFocus::Lyrics;
                    }
                    SearchFocus::Lyrics => {
                        self.focus = SearchFocus::Input;
                    }
                }
            }
            (SearchFocus::Audio | SearchFocus::Lyrics, Key::Char('/')) => {
                self.global_state.lock().unwrap().mode = InputMode::Input;
                self.focus = SearchFocus::Input;

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio | SearchFocus::Lyrics, Key::Esc) => {
                self.query.reset();
                self.audio_presentation_list.reset();
                self.lyrics_presentation_list.reset();
                self.focus = SearchFocus::Input;
                self.global_state.lock().unwrap().mode = InputMode::Input;

                return Ok(EventState::Consumed);
            }
            // Input bindings.
            (SearchFocus::Input, Key::Char('/')) => {
                self.query.reset();
                self.global_state.lock().unwrap().mode = InputMode::Input;

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Esc) => {
                if self.query.value().is_empty() {
                    self.audio_presentation_list.reset();
                    self.lyrics_presentation_list.reset();
                    self.global_state.lock().unwrap().focus = Focus::Home;

                    return Ok(EventState::Consumed);
                }

                self.query.reset();

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Backspace) => {
                self.query.handle_event(&Event::Key(KeyEvent::new(
                    KeyCode::Backspace,
                    KeyModifiers::NONE,
                )));
                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Char(v)) => {
                self.query.handle_event(&Event::Key(KeyEvent::new(
                    KeyCode::Char(v),
                    KeyModifiers::NONE,
                )));
                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Enter) => {
                self.search().await;
                self.global_state.lock().unwrap().mode = InputMode::Nav;
                self.focus = SearchFocus::Audio;

                // Select the first list items if none is selected.
                if self.audio_state.selected().is_none() {
                    self.audio_state.select(Some(0));
                }

                return Ok(EventState::Consumed);
            }
            // Audio bindings.
            (SearchFocus::Audio, Key::Up) => {
                if self.audio_state.selected().is_some() {
                    self.audio_state.select(Some(
                        self.audio_state.selected().unwrap_or(0).saturating_sub(1),
                    ));
                }

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio, Key::Down) => {
                if self.audio_state.selected().is_some() {
                    self.audio_state.select(Some(
                        self.audio_state.selected().unwrap_or(0).saturating_add(1),
                    ));
                }

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio, Key::Enter) => {
                if let Some(index) = self.audio_state.selected() {
                    song.video_id = self.audio_results[index].id.to_string();
                }

                self.focus = SearchFocus::Lyrics;
                if self.lyrics_state.selected().is_none() {
                    self.lyrics_state.select(Some(0));
                }

                return Ok(EventState::Consumed);
            }
            // Lyrics bindings.
            (SearchFocus::Lyrics, Key::Up) => {
                if self.lyrics_state.selected().is_some() {
                    self.lyrics_state.select(Some(
                        self.lyrics_state.selected().unwrap_or(0).saturating_sub(1),
                    ));
                }

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Lyrics, Key::Down) => {
                if self.lyrics_state.selected().is_some() {
                    self.lyrics_state.select(Some(
                        self.lyrics_state.selected().unwrap_or(0).saturating_add(1),
                    ));
                }

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Lyrics, Key::Enter) => {
                if let Some(index) = self.lyrics_state.selected() {
                    song.lyric_id = self.lyric_results[index].id.to_string();
                    song.title = self.lyric_results[index].title.to_string();
                    song.artist = self.lyric_results[index].artist.to_string();
                    song.synced_lyrics = self.lyric_results[index].synced_lyrics.to_string();
                    let map = self.lyrics_service.parse(self.lyric_results[index].synced_lyrics.clone()).await;

                    match map {
                        Ok(lyric_map) => {
                            song.lyric_map = lyric_map;
                        }
                        Err(e) => {
                            println!("Error parsing lyrics: {}", e);
                        }
                    }

                    // After selecting lyrics, push the song to the queue and return to Queue view.
                    {
                        let mut global_state = self.global_state.lock().unwrap();
                        global_state.songs.push(song.clone());
                        global_state.mode = InputMode::Nav;
                        global_state.focus = Focus::Queue;
                    }

                    // Clear component state.
                    self.query.reset();
                    self.audio_presentation_list.reset();
                    self.lyrics_presentation_list.reset();
                }

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
        f: &mut Frame,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.query.visual_scroll(width as usize);

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(100)])
            .split(rect);

        let (search, body) = (vert_chunks[0], vert_chunks[1]);

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

        f.render_widget(input, search);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body);

        let (audio, lyrics) = (chunks[0], chunks[1]);

        let audio_list = List::new(self.audio_presentation_list.items.clone())
            .block(
                Block::default()
                    .title("Audio Results")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
            .highlight_symbol(">> ");

        let lyrics_list = List::new(self.lyrics_presentation_list.items.clone())
            .block(
                Block::default()
                    .title("Lyrics Results")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
            .highlight_symbol(">> ");

        f.render_stateful_widget(audio_list, audio, &mut self.audio_state.clone());
        f.render_stateful_widget(lyrics_list, lyrics, &mut self.lyrics_state.clone());
        Ok(())
    }
}

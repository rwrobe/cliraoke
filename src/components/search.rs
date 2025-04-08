use super::{Frame, RenderableComponent, ResettableComponent};
use crate::app::GlobalState;
use crate::audio::{AudioFetcher, AudioResult};
use crate::components::search::NavDir::{Down, Up};
use crate::components::stateful_list::{StatefulList, get_list_items};
use crate::events::{EventState, Key};
use crate::lyrics::{LyricsFetcher, LyricsResult};
use crate::models::song::Song;
use crate::state::{AMGlobalState, Focus, InputMode, with_state};
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::sync::{Arc, Mutex};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Default, Clone, PartialEq)]
enum SearchFocus {
    #[default]
    Input,
    Audio,
    Lyrics,
}

#[derive(Debug, Clone)]
struct State<'a> {
    audio_presentation_list: StatefulList<'a>,
    audio_results: Vec<AudioResult>,
    audio_state: ListState,

    lyrics_presentation_list: StatefulList<'a>,
    lyric_results: Vec<LyricsResult>,
    lyrics_state: ListState,

    song: Song,
    focus: SearchFocus,
    query: Input,
}

enum NavDir {
    Up,
    Down,
}

impl State<'_> {
    fn new() -> Self {
        Self {
            audio_presentation_list: StatefulList::default(),
            audio_results: vec![],
            audio_state: ListState::default(),

            lyrics_presentation_list: StatefulList::default(),
            lyric_results: vec![],
            lyrics_state: ListState::default(),

            song: Song::new(),
            focus: SearchFocus::Input,
            query: Input::default(),
        }
    }

    fn query(&self) -> &str {
        self.query.value()
    }

    fn reset(&mut self) {
        self.audio_presentation_list = StatefulList::default();
        self.audio_results = vec![];
        self.audio_state = ListState::default();
        self.lyrics_presentation_list = StatefulList::default();
        self.lyric_results = vec![];
        self.lyrics_state = ListState::default();
        self.song = Song::new();
        self.focus = SearchFocus::Input;
        self.query = Input::default();
    }

    fn with_audio_results(&mut self, res: Vec<AudioResult>) {
        self.audio_results = res.clone();
        self.audio_presentation_list = StatefulList::with_items(
            res.into_iter()
                .map(|r| {
                    let item = ListItem::new(r.title);
                    item
                })
                .collect(),
            None,
        );
    }

    fn with_lyrics_results(&mut self, res: Vec<LyricsResult>) {
        self.lyric_results = res.clone();
        self.lyrics_presentation_list = StatefulList::with_items(
            res.into_iter()
                .take(5)
                .map(|r| {
                    let item = ListItem::new(format!("{} by {}", r.title, r.artist));
                    item
                })
                .collect(),
            None,
        );
    }

    fn with_state<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        f(self)
    }

    async fn with_async_state<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&State) -> R,
    {
        f(&self)
    }

    // TODO: There must be a way to do this with generics.
    fn navigate(&mut self, focus: SearchFocus, dir: NavDir) {
        let state = match focus {
            SearchFocus::Audio => &mut self.audio_state,
            SearchFocus::Lyrics => &mut self.lyrics_state,
            _ => return,
        };

        if state.selected().is_some() {
            let current = state.selected().unwrap_or(0);
            let new_index = match dir {
                NavDir::Up => current.saturating_sub(1),
                NavDir::Down => current.saturating_add(1),
            };
            state.select(Some(new_index));
        }
    }
}

pub struct Search<'a, AF, LF>
where
    AF: AudioFetcher + Send + Sync + 'static,
    LF: LyricsFetcher + Send + Sync + 'static,
{
    audio_fetcher: Arc<AF>,
    global_state: Arc<Mutex<GlobalState>>,
    lyrics_fetcher: Arc<LF>,
    state: State<'a>,
}

impl<AF, LF> Search<'_, AF, LF>
where
    AF: AudioFetcher + Send + Sync + 'static,
    LF: LyricsFetcher + Send + Sync + 'static,
{
    pub fn new(global_state: Arc<Mutex<GlobalState>>, af: Arc<AF>, lf: Arc<LF>) -> Self {
        Self {
            audio_fetcher: af,
            global_state,
            lyrics_fetcher: lf,
            state: State::new(),
        }
    }

    async fn search(&mut self) {
        // Search audio.
        let audio_results = self.audio_fetcher.search(self.state.query()).await;
        match audio_results {
            Ok(results) => self.state.with_audio_results(results),
            Err(e) => {
                println!("Error searching audio: {}", e);
            }
        }

        // Search lyrics.
        let lyric_results = self.lyrics_fetcher.search(self.state.query()).await;
        match lyric_results {
            Ok(results) => self.state.with_lyrics_results(results),
            Err(e) => {
                println!("Error searching lyrics: {}", e);
            }
        }
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        match (&self.state.focus, key) {
            // Component-level bindings.
            (SearchFocus::Input | SearchFocus::Audio | SearchFocus::Lyrics, Key::Tab) => {
                match self.state.focus {
                    SearchFocus::Input => self.state.with_state(|s| {
                        s.focus = SearchFocus::Audio;
                    }),
                    SearchFocus::Audio => self.state.with_state(|s| {
                        s.focus = SearchFocus::Lyrics;
                    }),
                    SearchFocus::Lyrics => self.state.with_state(|s| {
                        s.focus = SearchFocus::Input;
                    }),
                }
            }
            (SearchFocus::Audio | SearchFocus::Lyrics, Key::Char('/')) => {
                with_state(&self.global_state, |s| {
                    s.mode = InputMode::Input;
                });
                self.state.with_state(|s| {
                    s.focus = SearchFocus::Input;
                });

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio | SearchFocus::Lyrics, Key::Esc) => {
                self.state.reset();

                return Ok(EventState::Consumed);
            }
            // Input bindings.
            (SearchFocus::Input, Key::Char('/')) => {
                self.state.with_state(|s| {
                    s.query.reset();
                });
                with_state(&self.global_state, |s| {
                    s.mode = InputMode::Input;
                });

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Esc) => {
                if self.state.query.value().is_empty() {
                    self.state.with_state(|s| {
                        s.audio_presentation_list.reset();
                        s.lyrics_presentation_list.reset();
                    });
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Home;
                    });

                    return Ok(EventState::Consumed);
                }

                self.state.with_state(|s| {
                    s.query.reset();
                });

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Backspace) => {
                self.state.with_state(|s| {
                    s.query.handle_event(&Event::Key(KeyEvent::new(
                        KeyCode::Backspace,
                        KeyModifiers::NONE,
                    )));
                });

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Char(v)) => {
                self.state.with_state(|s| {
                    s.query.handle_event(&Event::Key(KeyEvent::new(
                        KeyCode::Char(v),
                        KeyModifiers::NONE,
                    )));
                });

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Input, Key::Enter) => {
                self.search().await;
                with_state(&self.global_state, |s| {
                    s.mode = InputMode::Nav;
                });
                self.state.with_state(|s| {
                    s.focus = SearchFocus::Audio;

                    // Select the first list items if none is selected.
                    if s.audio_state.selected().is_none() {
                        s.audio_state.select(Some(0));
                    }
                });

                return Ok(EventState::Consumed);
            }
            // Audio bindings.
            (SearchFocus::Audio, Key::Up) => {
                self.state.navigate(SearchFocus::Audio, Up);

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio, Key::Down) => {
                self.state.navigate(SearchFocus::Audio, Down);

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Audio, Key::Enter) => {
                self.state.with_state(|s| {
                    if let Some(index) = s.audio_state.selected() {
                        s.song = s.song.with_ar(s.audio_results[index].clone());
                    }

                    s.focus = SearchFocus::Lyrics;
                    if s.lyrics_state.selected().is_none() {
                        s.lyrics_state.select(Some(0));
                    }
                });

                return Ok(EventState::Consumed);
            }
            // Lyrics bindings.
            (SearchFocus::Lyrics, Key::Up) => {
                self.state.navigate(SearchFocus::Lyrics, Up);

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Lyrics, Key::Down) => {
                self.state.navigate(SearchFocus::Lyrics, Down);

                return Ok(EventState::Consumed);
            }
            (SearchFocus::Lyrics, Key::Enter) => {
                if let Some(index) = self.state.lyrics_state.selected() {
                    let this_lr = self.state.lyric_results[index].clone();
                    self.state.song = self.state.song.with_lr(
                        this_lr.clone(),
                        self.lyrics_fetcher
                            .parse(this_lr.synced_lyrics.to_owned())
                            .await
                            .unwrap_or_else(|e| {
                                println!("Error parsing lyrics: {}", e);
                                // Return an empty result instead of panicking
                                None
                            }),
                    );
                }

                with_state(&self.global_state, |s| {
                    s.song_list.push(self.state.song.clone());
                    s.mode = InputMode::Nav;

                    // Return to Home if we have more than one song or Queue to show that the
                    // next song was added to the queue.
                    match s.song_list.len() > 1 {
                        true => s.focus = Focus::Queue,
                        false => s.focus = Focus::Home,
                    }
                });

                // Clear component state.
                self.state.reset();

                return Ok(EventState::Consumed);
            }
            _ => {}
        }

        Ok(EventState::NotConsumed)
    }
}

impl<AF, LF> RenderableComponent for Search<'_, AF, LF>
where
    AF: AudioFetcher + Send + Sync + 'static,
    LF: LyricsFetcher + Send + Sync + 'static,
{
    fn render<B: Backend>(&self, f: &mut Frame, rect: Rect) -> anyhow::Result<()> {
        let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.state.query.visual_scroll(width as usize);

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(100)])
            .split(rect);

        let (search, body) = (vert_chunks[0], vert_chunks[1]);

        let input = Paragraph::new(self.state.query.value())
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

        f.render_widget(input, search);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body);

        let (audio, lyrics) = (chunks[0], chunks[1]);

        let audio_list = List::new(get_list_items(self.state.audio_presentation_list.clone()))
            .block(
                Block::default()
                    .title("Audio Results")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
            .highlight_symbol(">> ");

        let lyrics_list = List::new(get_list_items(self.state.lyrics_presentation_list.clone()))
            .block(
                Block::default()
                    .title("Lyrics Results")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
            .highlight_symbol(">> ");

        f.render_stateful_widget(audio_list, audio, &mut self.state.audio_state.clone());
        f.render_stateful_widget(lyrics_list, lyrics, &mut self.state.lyrics_state.clone());
        Ok(())
    }
}

use crate::components::RenderableComponent;
use crate::events::EventState;
use crate::models::song::SongList;
pub(crate) use crate::state::GlobalState;
use crate::state::{Focus, InputMode};
use crate::{
    components::{
        help::Help, lyrics::Lyrics, queue::Queue, search::Search, timer::Timer, title::Title,
    },
    events::Key,
};
use color_eyre::eyre::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::cmp::PartialEq;
use std::sync::{Arc, Mutex};
use strum::Display;
use crate::audio::AudioService;
use crate::lyrics::LyricsService;

pub struct AppComponent<'a> {
    lyrics_provider: &'a dyn LyricsService,
    audio_provider: &'a dyn AudioService,
    help: Help,
    lyrics: Lyrics,
    queue: Queue,
    search: Search<'a>,
    timer: Timer,
    state: Arc<Mutex<GlobalState>>,
}

impl AppComponent<'_> {
    pub fn new(lp: &dyn LyricsService, ap: &dyn AudioService) -> Self {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        Self {
            lyrics_provider: lp,
            audio_provider: ap,
            help: Help::new(),
            lyrics: Lyrics::new(),
            queue: Queue::new(global_state.clone()),
            search: Search::new(global_state.clone(), lp, ap),
            timer: Timer::new(),
            state: global_state.clone(),
        }
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        let focus = self.state.lock().unwrap().focus.clone();

        match focus {
            Focus::Queue => {
                if self.queue.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }

                match key {
                    Key::Char('u') => {
                        self.state.lock().unwrap().focus = Focus::Home;
                    }
                    _ => {}
                }
            }
            Focus::Search => {
                if self.search.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }

                match key {
                    Key::Char('/') | Key::Esc => {
                        self.state.lock().unwrap().focus = Focus::Home;
                    }
                    _ => {}
                }
            }
            Focus::Help => match key {
                Key::Esc | Key::Char('h') => {
                    self.state.lock().unwrap().focus = Focus::Home;
                }
                _ => {}
            },
            Focus::Lyrics => {
                // if self.lyrics.event(key).await?.is_consumed() {
                //     return Ok(EventState::Consumed);
                // }
            }
            _ => match key {
                Key::Esc => {
                    self.state.lock().unwrap().focus = Focus::Home;
                }
                Key::Char('h') => {
                    self.state.lock().unwrap().focus = Focus::Help;
                }
                Key::Char('u') => {
                    self.state.lock().unwrap().focus = Focus::Queue;
                }
                Key::Char('/') => {
                    let mut state = self.state.lock().unwrap();
                    state.mode = InputMode::Input;
                    state.focus = Focus::Search;
                }
                _ => {}
            },
        }

        Ok(EventState::NotConsumed)
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()> {
        let window = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Percentage(100),
                Constraint::Min(3),
            ])
            .split(rect);

        let (header, body, footer) = (chunks[0], chunks[1], chunks[2]);

        // Header
        const EMOJI_MARTINI: char = '\u{1F378}';
        const EMDASH: char = '\u{2014}';

        let app_title = Title::new(
            format!(
                " {} CLIraoke {} Karaoke for the Command Line {} ",
                EMOJI_MARTINI, EMDASH, EMOJI_MARTINI
            )
            .as_str(),
        );
        app_title.render(f, header, self.state.clone())?;

        // The layout of the body is determined by focus.
        let focus = self.state.lock().unwrap().focus.clone();
        match focus {
            Focus::Queue => {
                let inner_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[1]);

                let (left, right) = (inner_rects[0], inner_rects[1]);

                self.lyrics.render(f, left, self.state.clone())?;
                self.queue.render(f, right, self.state.clone())?;
            }
            Focus::Search => {
                self.search.render(f, body, self.state.clone())?;
            }
            _ => {
                self.lyrics.render(f, body, self.state.clone())?;
            }
        }

        // Footer.
        match focus {
            Focus::Help => {
                self.help.render(f, footer, self.state.clone())?;
            }
            _ => {
                self.timer.render(f, footer, self.state.clone())?;
            }
        }

        Ok(())
    }

    async fn components_event(&mut self, key: Key) -> Result<EventState> {
        let focus = self.state.lock().unwrap().focus.clone();
        match focus {
            // Focus::Lyrics => {
            //     if self.lyrics.event(key).await?.is_consumed() {
            //         return Ok(EventState::Consumed);
            //     }
            // }
            Focus::Queue => {
                if self.queue.event(key).await?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Search => {
                if self.search.event(key).await?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            _ => {}
        }
        Ok(EventState::NotConsumed)
    }

    // TODO
    async fn move_focus(&mut self, key: Key) -> Result<EventState> {
        let focus = self.state.lock().unwrap().focus.clone();
        match focus {
            // Focus::Lyrics => {
            //     if key == Key::Esc.down {
            //         self.focus = self.lyrics.active_focus();
            //     }
            // }
            Focus::Queue => {
                if key == Key::Char('u') {
                    self.state.lock().unwrap().focus = Focus::Queue
                }
            }
            Focus::Search => {
                if key == Key::Char('/') || key == Key::Enter {
                    self.state.lock().unwrap().focus = Focus::Search
                }
            }
            _ => {
                self.state.lock().unwrap().focus = Focus::Lyrics;
            }
        }
        Ok(EventState::NotConsumed)
    }
}

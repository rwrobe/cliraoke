use crate::audio::{AudioFetcher, AudioService};
use crate::components::RenderableComponent;
use crate::events::EventState;
use crate::lyrics::LyricsFetcher;
use crate::models::song::SongList;
pub(crate) use crate::state::GlobalState;
use crate::state::{Focus, InputMode, SongState};
use crate::{
    components::{
        help::Help, lyrics::Lyrics, queue::Queue, search::Search, timer::Timer, title::Title,
    },
    events::Key,
};
use color_eyre::eyre::Result;
use crossbeam;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::cmp::PartialEq;
use std::sync::{Arc, Mutex};
use strum::Display;

pub struct AppComponent<'a> {
    lyrics_fetcher: &'a dyn LyricsFetcher,
    audio_fetcher: &'a dyn AudioFetcher,
    audio_service: &'a dyn AudioService,
    help: Help,
    lyrics: Lyrics,
    queue: Queue,
    search: Search<'a>,
    timer: Timer,
    tick_accumulator: u64,
    state: Arc<Mutex<GlobalState>>,
}

impl<'a> AppComponent<'a> {
    pub fn new(
        lp: &'a (dyn LyricsFetcher + 'a),
        ap: &'a (dyn AudioFetcher + 'a),
        aus: &'a (dyn AudioService + 'a),
    ) -> Self {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        Self {
            // Injected services.
            lyrics_fetcher: lp,
            audio_fetcher: ap,
            audio_service: aus,

            // UI Components.
            help: Help::new(),
            lyrics: Lyrics::new(global_state.clone()),
            queue: Queue::new(global_state.clone()),
            search: Search::new(global_state.clone(), lp, ap),
            timer: Timer::new(global_state.clone()),

            // State.
            tick_accumulator: 0,
            state: global_state.clone(),
        }
    }

    // tick is called every second
    pub(crate) fn tick(&mut self, tick_rate_ms: u64) {
        // Our "tick" rate (refresh rate) is defined in ms.
        self.tick_accumulator += tick_rate_ms;

        // We want to convert this into seconds in a way that works for arbitrary ms values.
        // NB: The ms must be even divisors of 1000 for the second conversion to be accurate.
        // Use other values to dilate time.
        if self.tick_accumulator >= 1000 {
            let seconds = self.tick_accumulator / 1000;
            self.state.lock().unwrap().session_time_elapsed +=
                std::time::Duration::from_secs(seconds);
            self.tick_accumulator %= 1000;
        }

        // Move the next song in the queue to the current song if nothing is playing.
        {
            let mut state = self.state.lock().unwrap();
            if state.current_song.is_none() && !state.songs.is_empty() {
                let song = Some(state.songs.remove(0));
                state.current_song = song.clone();
                state.current_song_index = 0;
            }
        }
    }

    // play will start the song and set the SongState to Playing.
    // TODO: ffmpeg doesn't support pausing. For this we probably need soloud, which doesn't support
    // buffering from a URL.
    fn play(&mut self) {
        let mut state = self.state.lock().unwrap();
        if let Some(song) = &state.current_song {
            self.audio_service.play(song.video_id.as_str());

            let audio_service = self.audio_service.clone();
            let video_id = song.video_id.clone();

            let _play_thread = crossbeam::scope(|s| {
                s.spawn(move |_| audio_service.play(video_id.as_str()));
            });
            state.song_state = SongState::Playing;
        }
    }

    // event handles keystrokes and updates the state of the application.
    //
    // This is organized by "focus" (the component that is currently active). Child components
    // are given priority in handling events, so the event bubbles up the component hierarchy like
    // JS events in the DOM.
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
                    Key::Char('/') => {
                        let mut state = self.state.lock().unwrap();
                        state.mode = InputMode::Input;
                        state.focus = Focus::Search;
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
            Focus::Lyrics => match key {
                Key::Char(' ') => {
                    {
                        let mut state = self.state.lock().unwrap();
                        if state.song_state == SongState::Playing {
                            // TODO: pause with ffmpeg
                        } else {
                            state.song_state = SongState::Playing;
                        }
                    }
                    self.play();
                }
                _ => {}
            },
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

    pub fn render<B: Backend>(&self, f: &mut Frame, rect: Rect) -> anyhow::Result<()> {
        let window = f.area();

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
        app_title.render::<B>(f, header, self.state.clone())?;

        // The layout of the body is determined by focus.
        let focus = self.state.lock().unwrap().focus.clone();
        match focus {
            Focus::Queue => {
                let inner_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[1]);

                let (left, right) = (inner_rects[0], inner_rects[1]);

                self.lyrics.render::<B>(f, left, self.state.clone())?;
                self.queue.render::<B>(f, right, self.state.clone())?;
            }
            Focus::Search => {
                self.search.render::<B>(f, body, self.state.clone())?;
            }
            _ => {
                self.lyrics.render::<B>(f, body, self.state.clone())?;
            }
        }

        // Footer.
        match focus {
            Focus::Help => {
                self.help.render::<B>(f, footer, self.state.clone())?;
            }
            _ => {
                self.timer.render::<B>(f, footer, self.state.clone())?;
            }
        }

        Ok(())
    }
}

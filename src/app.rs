use crate::audio::{AudioFetcher, AudioService};
use crate::components::RenderableComponent;
use crate::events::EventState;
use crate::lyrics::{LyricsFetcher, LyricsService};
pub(crate) use crate::state::GlobalState;
use crate::state::{get_state, with_state, Focus, InputMode, SongState};
use crate::util::{EMDASH, EMOJI_MARTINI};
use crate::{
    components::{
        help::Help, lyrics::Lyrics, queue::Queue, search::Search, timer::Timer, title::Title,
    },
    events::Key,
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct AppComponent<'a, AF, AS, LF, LS>
where
    AF: AudioFetcher + Send + Sync + 'static,
    AS: AudioService + Send + Sync + 'static,
    LF: LyricsFetcher + Send + Sync + 'static,
    LS: LyricsService + Send + Sync + 'static,
{
    audio_service: Arc<AS>,
    lyrics_service: Arc<LS>,

    help: Help,
    lyrics: Lyrics<LS>,
    queue: Queue,
    search: Search<'a, AF, LF>,
    timer: Timer,

    global_state: Arc<Mutex<GlobalState>>,
    tick_accumulator: u64,
}

impl<AF, AS, LF, LS> AppComponent<'_, AF, AS, LF, LS>
where
    AF: AudioFetcher + Send + Sync + 'static,
    AS: AudioService + Send + Sync + 'static,
    LF: LyricsFetcher + Send + Sync + 'static,
    LS: LyricsService + Send + Sync + 'static,
{
    pub fn new(lf: Arc<LF>, ls: Arc<LS>, af: Arc<AF>, aus: Arc<AS>) -> Self {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        Self {
            // Injected services.
            audio_service: aus.clone(),
            lyrics_service: ls.clone(),

            // UI Components.
            help: Help::new(),
            lyrics: Lyrics::new(global_state.clone(), ls),
            queue: Queue::new(global_state.clone()),
            search: Search::new(global_state.clone(), af, lf),
            timer: Timer::new(global_state.clone()),

            // State.
            global_state: global_state.clone(),
            tick_accumulator: 0,
        }
    }

    pub(crate) async fn tick(&mut self, tick_rate_ms: u64) {
        // Our "tick" rate (refresh rate) is defined in ms.
        self.tick_accumulator += tick_rate_ms;

        // We want to convert this into seconds in a way that works for arbitrary ms values.
        // NB: The ms must be even divisors of 1000 for the second conversion to be accurate.
        // Use other values to dilate time.
        if self.tick_accumulator >= 1000 {
            let seconds = self.tick_accumulator / 1000;

            with_state(&self.global_state, |s| {
                s.session_time_elapsed += std::time::Duration::from_secs(seconds);
            });

            self.tick_accumulator %= 1000;
        }

        // Update global state.
        with_state(&self.global_state.clone(), |s| {
            // Move the next song in the queue to the current song if nothing is playing.
            if s.current_song.is_none() && !s.song_list.is_empty() {
                // play spawns two scoped threads for the music and lyrics.
                self.play()
            }
        });
    }

    fn play(&self) {
        let mut state = get_state(&self.global_state);

        let song = Some(state.song_list.remove(0));
        let mut lyrics = None;
        if let Some(song) = song.clone() {
            lyrics = song.lyric_map;
        }

        if let (Some(cs), Some(lyrics)) = (song.clone(), lyrics) {
            // Set initial state for a song.
            state.current_song = song.clone();
            state.current_song_elapsed = 0;
            state.song_state = SongState::Playing;

            // Spawn a thread for playing the audio.
            thread::scope(|s| {
                let aus = Arc::clone(&self.audio_service);
                let ls = Arc::clone(&self.lyrics_service);
                let id = cs.video_id.clone();

                let elapsed = state.current_song_elapsed;
                let lyrics_clone = lyrics.clone();
                let state_arc = Arc::clone(&self.global_state);

                s.spawn(async move || {
                    aus.play(&id).await;
                });

                s.spawn(move || {
                    if let Ok(lyric) = ls.play(elapsed, lyrics_clone) {
                        if !lyric.is_empty() {
                            let mut ret = vec![lyric];
                            if let Ok(mut state) = state_arc.lock() {
                                state.current_lyrics = ret;
                            }
                        }
                    }
                });
            });
        }
    }

    // event handles keystrokes and updates the state of the application.
    //
    // This is organized by "focus" (the component that is currently active). Child components
    // are given priority in handling events, so the event bubbles up the component hierarchy like
    // JS events in the DOM.
    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        let focus = self.global_state.lock().unwrap().focus.clone();

        match focus {
            Focus::Queue => {
                if self.queue.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }

                match key {
                    Key::Char('u') => {
                        with_state(&self.global_state, |s| {
                            s.focus = Focus::Home;
                        });
                    }
                    Key::Char('/') => {
                        with_state(&self.global_state, |s| {
                            s.focus = Focus::Search;
                            s.mode = InputMode::Input;
                        });
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
                        with_state(&self.global_state, |s| {
                            s.focus = Focus::Home;
                        });
                    }
                    _ => {}
                }
            }
            Focus::Help => match key {
                Key::Esc | Key::Char('h') => {
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Home;
                    });
                }
                _ => {}
            },
            _ => match key {
                Key::Esc => {
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Home;
                    });
                }
                Key::Char(' ') => {
                    // TODO: play/pause
                }
                Key::Char('h') => {
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Help;
                    });
                }
                Key::Char('u') => {
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Queue;
                    });
                }
                Key::Char('/') => {
                    with_state(&self.global_state, |s| {
                        s.focus = Focus::Search;
                        s.mode = InputMode::Input;
                    });
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

        let app_title = Title::new(
            format!(
                " {} CLIraoke {} Karaoke for the Command Line {} ",
                EMOJI_MARTINI, EMDASH, EMOJI_MARTINI
            )
            .as_str(),
        );
        app_title.render::<B>(f, header)?;

        // The layout of the body is determined by focus.
        let focus = get_state(&self.global_state).focus.clone();
        match focus {
            Focus::Queue => {
                let inner_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[1]);

                let (left, right) = (inner_rects[0], inner_rects[1]);

                self.lyrics.render::<B>(f, left)?;
                self.queue.render::<B>(f, right)?;
            }
            Focus::Search => {
                self.search.render::<B>(f, body)?;
            }
            _ => {
                self.lyrics.render::<B>(f, body)?;
            }
        }

        // Footer.
        match focus {
            Focus::Help => {
                self.help.render::<B>(f, footer)?;
            }
            _ => {
                self.timer.render::<B>(f, footer)?;
            }
        }

        Ok(())
    }
}

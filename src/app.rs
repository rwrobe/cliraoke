use crate::audio;
use crate::audio::{AudioFetcher, AudioService};
use crate::components::RenderableComponent;
use crate::events::EventState;
use crate::lyrics;
use crate::lyrics::{LyricsFetcher, LyricsService};
pub(crate) use crate::state::GlobalState;
use crate::state::{Focus, InputMode, SongState, get_state, with_async_state, with_state};
use crate::util::{EMDASH, EMOJI_MARTINI};
use crate::{
    components::{
        help::Help, lyrics::Lyrics, queue::Queue, search::Search, timer::Timer, title::Title,
    },
    events::Key,
};
use color_eyre::owo_colors::OwoColorize;
use crossbeam;
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
};
use std::sync::{Arc, Mutex};

pub struct AppComponent<'a, AF, AS, LF, LS>
where
    AF: AudioFetcher + 'a,
    AS: AudioService + 'a,
    LF: LyricsFetcher + 'a,
    LS: LyricsService + 'a,
{
    audio_fetcher: &'a AF,
    audio_service: &'a AS,
    lyrics_fetcher: &'a LF,
    lyrics_service: &'a LS,

    help: Help,
    lyrics: Lyrics<'a>,
    queue: Queue,
    search: Search<'a>,
    timer: Timer,

    global_state: Arc<Mutex<GlobalState>>,
    tick_accumulator: u64,
}

impl<
    'a,
    AF: AudioFetcher,
    AS: AudioService,
    LF: LyricsFetcher,
    LS: LyricsService,
> AppComponent<'a, AF, AS, LF, LS>
{
    pub fn new(lp: &'a LF, ls: &'a LS, ap: &'a AF, aus: &'a AS) -> Self {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        Self {
            // Injected services.
            audio_fetcher: ap,
            audio_service: aus,
            lyrics_fetcher: lp,
            lyrics_service: ls,

            // UI Components.
            help: Help::new(),
            lyrics: Lyrics::new(global_state.clone(), ls),
            queue: Queue::new(global_state.clone()),
            search: Search::new(global_state.clone(), lp, ap),
            timer: Timer::new(global_state.clone()),

            // State.
            global_state: global_state.clone(),
            tick_accumulator: 0,
        }
    }

    // tick is called every second
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
        let _ = with_state(&self.global_state.clone(), |s| {
            // If a song is playing, update the elapsed time and start lyrics.
            if s.song_state == SongState::Playing {
                s.current_song_elapsed += tick_rate_ms;

                // Update the lyrics.
                if let Some(song) = &s.current_song {
                    if let Some(lyric_map) = &song.lyric_map {
                        if let Ok(lyric) = self
                            .lyrics_service
                            .play(s.current_song_elapsed, lyric_map.clone())
                        {
                            // We don't want to replace the current lyric with an empty string.
                            // TODO we should probably let the lyrics fade eventually.
                            if lyric.is_empty() {
                                return;
                            }

                            // TODO
                            let mut ret = Vec::new();
                            ret.push(lyric);
                            s.current_lyrics = ret;
                        }
                    }
                }
            }
        });
    }

    async fn play(&self) {
        let mut state = get_state(&self.global_state);
        // Move the next song in the queue to the current song if nothing is playing.
        if state.current_song.is_none() && !state.song_list.is_empty() {
            let song = Some(state.song_list.remove(0));
            state.current_song = song.clone();
            state.current_song_elapsed = 0;
            state.song_state = SongState::Playing;

            match song {
                Some(cs) => {
                    self.audio_service
                        .play(cs.video_id.as_str())
                        .await
                        .expect("dawg shit");
                }
                None => {}
            }
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
                    self.play().await;
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

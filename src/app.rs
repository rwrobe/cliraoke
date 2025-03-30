use crate::components::RenderableComponent;
use crate::events::EventState;
use crate::{
    components::{
        help::Help, lyrics::Lyrics, queue::Queue, search::Search, timer::Timer, title::Title,
    },
    constants::Focus,
    events::Key,
};
use color_eyre::eyre::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::cmp::PartialEq;

pub struct AppComponent<'a> {
    help: Help,
    lyrics: Lyrics,
    queue: Queue,
    search: Search<'a>,
    timer: Timer,
    focus: Focus,
}

impl PartialEq for Focus {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl AppComponent<'_> {
    pub fn new() -> Self {
        Self {
            help: Help::new(),
            lyrics: Lyrics::new(),
            queue: Queue::new(),
            search: Search::new(),
            timer: Timer::new(),
            focus: Focus::Home,
        }
    }

    fn toggle_focus(&mut self, focus: Focus) {
        if self.focus == focus {
            self.focus = Focus::Home;
        }

        self.focus = focus;
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.focus.clone() {
            Focus::Queue => {
                if self.queue.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }

                match key {
                    Key::Char('u') => {
                        self.focus = Focus::Home;
                    }
                    _ => {}
                }
            }
            Focus::Search => {
                if self.search.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }

                match key {
                    Key::Char('/') => {
                        self.focus = Focus::Home;
                    }
                    _ => {}
                }
            }
            Focus::Help => match key {
                Key::Esc | Key::Char('h') => {
                    self.focus = Focus::Home;
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
                    self.focus = Focus::Home;
                }
                Key::Char('h') => {
                    self.focus = Focus::Help;
                }
                Key::Char('u') => {
                    self.focus = Focus::Queue;
                }
                Key::Char('/') => {
                    self.focus = Focus::Search;
                }
                _ => {}
            },
        }

        Ok(EventState::NotConsumed)
    }

    pub fn render<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
    ) -> anyhow::Result<()> {
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
        app_title.render(f, header, false)?;

        // The layout of the body is determined by focus.
        match self.focus {
            Focus::Queue => {
                let inner_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[1]);

                let (left, right) = (inner_rects[0], inner_rects[1]);

                self.lyrics.render(f, left, false)?;
                self.queue
                    .render(f, right, matches!(self.focus(), Focus::Queue))?;
            }
            Focus::Search => {
                self.search
                    .render(f, body, matches!(self.focus(), Focus::Queue))?;
            }
            _ => {
                self.lyrics.render(f, body, false)?;
            }
        }

        // Footer.
        match self.focus {
            Focus::Help => {
                self.help.render(f, footer, false)?;
            }
            _ => {
                self.timer.render(f, footer, false)?;
            }
        }

        Ok(())
    }

    fn focus(&self) -> Focus {
        self.focus.clone()
    }

    async fn components_event(&mut self, key: Key) -> Result<EventState> {
        match self.focus {
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
        match self.focus {
            // Focus::Lyrics => {
            //     if key == Key::Esc.down {
            //         self.focus = self.lyrics.active_focus();
            //     }
            // }
            Focus::Queue => {
                if key == Key::Char('u') {
                    self.focus = Focus::Queue
                }
            }
            Focus::Search => {
                if key == Key::Char('/') || key == Key::Enter {
                    self.focus = Focus::Search
                }
            }
            _ => {
                self.focus = Focus::Lyrics;
            }
        }
        Ok(EventState::NotConsumed)
    }
}

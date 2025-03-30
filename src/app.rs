use crate::events::EventState;
use crate::{
    action::Action,
    constants::Focus,
    components::{help::Help, queue::Queue, search::Search, timer, timer::Timer, title::Title},
    events::Key,
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::components::RenderableComponent;

pub struct AppComponent<'a> {
    help: Help,
    //lyrics: Lyrics,
    queue: Queue,
    search: Search<'a>,
    timer: Timer,
    focus: Focus,
}

impl AppComponent<'_> {
    pub fn new() -> Self {
        Self {
            help: Help::new(),
            //lyrics: Lyrics::new(),
            queue: Queue::new(),
            search: Search::new(),
            timer: Timer::new(),
            focus: Focus::Home,
        }
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

                self.queue.render(f,inner_rects[1], matches!(self.focus(), Focus::Queue))?;
            }
            Focus::SearchBar => {
                self.search.render(f,body, matches!(self.focus(), Focus::Queue))?;
            }
            _ => {
                let lyrics_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(lyrics_block, body);
            }
        }

        // Footer.
        match self.focus {
            Focus::Help => {
                self.help.render(f, footer,false)?;
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

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.focus {
            Focus::Home => {
                match key{
                    Key::Char('/') => {
                        self.focus = Focus::SearchBar
                    }
                    Key::Char('u') => {
                        self.focus = Focus::Queue
                    }
                    Key::Char('h') => {
                        self.focus = Focus::Help
                    }
                    _ => {}
                }
            }
            Focus::Queue => {
                if self.queue.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::SearchBar => {
                if self.search.event(key).await.unwrap().is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Lyrics => {
                // if self.lyrics.event(key).await?.is_consumed() {
                //     return Ok(EventState::Consumed);
                // }
            }
            _ => {}
        }
        //
        // if self.move_focus(key).await.unwrap().is_consumed() {
        //     return Ok(EventState::Consumed);
        // };

        Ok(EventState::NotConsumed)
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
            Focus::SearchBar => {
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
            Focus::SearchBar => {
                if key == Key::Char('/') || key == Key::Enter {
                    self.focus = Focus::SearchBar
                }
            }
            _ => {
                self.focus = Focus::Lyrics;
            }
        }
        Ok(EventState::NotConsumed)
    }
}

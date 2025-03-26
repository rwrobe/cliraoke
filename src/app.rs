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

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    WithQueue,
    Search,
    WithHelp,
    Processing,
}

pub struct AppComponent<'app> {
    help: Help,
    //lyrics: Lyrics,
    queue: Queue<'app>,
    search: Search<'app>,
    timer: Timer<'app>,
    mode: Mode,
    focus: Focus,
}

impl<'a> AppComponent<'a> {
    pub fn new() -> Self {
        Self {
            help: Help::new(),
            //lyrics: Lyrics::new(),
            queue: Queue::new(),
            search: Search::new(),
            timer: Timer::new(),
            mode: Mode::Normal,
            focus: Focus::Lyrics,
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

        // Body.
        let lyrics_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));


        // // The layout of the app is determined by the mode.
        // match self.mode {
        //     Mode::WithQueue => {
        //         let inner_rects = Layout::default()
        //             .direction(Direction::Horizontal)
        //             .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        //             .split(chunks[1]);
        //
        //         let mut queue = Queue::new();
        //         f.render_widget(lyrics_block, inner_rects[0]);
        //         f.render_stateful_widget(queue, inner_rects[1], &mut queue);
        //     }
        //     Mode::WithHelp => {
        //         f.render_widget(lyrics_block, chunks[1]);
        //
        //         let mut help = Help::new();
        //         help.draw(f, chunks[2])?;
        //     }
        //     Mode::Search => {
        //         let mut search = Search::new();
        //         search.draw(f, chunks[1])?;
        //
        //         let mut t = timer::Timer::new();
        //         t.draw(f, chunks[2])?;
        //     }
        //     _ => {
        //         f.render_widget(lyrics_block, chunks[1]);
        //
        //         // Add Timer to the footer.
        //         let mut t = timer::Timer::new();
        //         t.draw(f, chunks[2])?;
        //     }
        // }

        // Footer.
        self.timer.render(f, footer, focused)?;
    }

    fn focus(&self) -> Focus {
        self.focus.clone()
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        if self.move_focus(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

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

    async fn move_focus(&mut self, key: Key) -> Result<EventState> {
        match self.focus {
            // Focus::Lyrics => {
            //     if key == Key::Esc.down {
            //         self.focus = self.lyrics.active_focus();
            //     }
            // }
            Focus::Queue => {
                if key == Key::Char('u').up {
                    self.focus = Focus::Queue
                }
            }
            Focus::SearchBar => {
                if key == Key::Char('/').up || key == Key::Enter.up {
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

use clap::builder::Str;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{queue, timer, Component, Frame};
use crate::action::Action;
use crate::components::help::Help;
use crate::components::queue::Queue;
use crate::components::search::Search;
use crate::components::timer::Timer;
use crate::components::title::Title;
use crate::models::song::Song;
use crate::tui::Event;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    WithQueue,
    Search,
    WithHelp,
    Processing,
}

#[derive(Default)]
pub struct Container {
    pub current_song: Song,
    pub mode: Mode,
    pub queue: Queue,
    pub search: Search,
    pub timer: Timer,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl Container {
    pub fn new(queue: Queue, search: Search) -> Self {
        Self {
            current_song: Song::default(),
            mode: Mode::Normal,
            queue,
            search,
            timer: Timer::default(),
            action_tx: None,
        }
    }

    pub fn show_help(&mut self) {
        match self.mode {
            Mode::WithHelp => {
                self.mode = Mode::Normal;
            }
            _ => {
                self.mode = Mode::WithHelp;
            }
        }
    }
}

impl Component for Container {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        // Child components have first priority at handling events.
        let action = match self.mode {
            Mode::Normal => {
                if let Some(action) = self.queue.handle_events(event)? {
                    self.update(action)?
                } else {
                    None
                }
            }
            Mode::WithQueue => {
                if let Some(action) = self.queue.handle_events(event)? {
                    self.update(action)?
                } else {
                    None
                }
            }
            Mode::Search => {
                if let Some(action) = self.search.handle_events(event)? {
                    self.update(action)?
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(action)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            Mode::Search => self.search.handle_key_events(key)?,
            Mode::WithQueue => self.queue.handle_key_events(key)?,
            _ => Some(match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('h') => Action::ToggleHelp,
                KeyCode::Char('/') => Action::ToggleSearch,
                KeyCode::Char('u') => Action::ToggleQueue,
                KeyCode::Esc => Action::GoHome,
                KeyCode::Char('j') => match self.mode {
                    Mode::WithQueue | Mode::Search => Action::PreviousSong,
                    _ => Action::Noop,
                },
                KeyCode::Char('k') => match self.mode {
                    Mode::WithQueue | Mode::Search => Action::NextSong,
                    _ => Action::Noop,
                },
                _ => Action::Noop,
            })
        };

        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ToggleHelp => self.show_help(),
            Action::GoHome => {
                self.mode = Mode::Normal;
            }
            Action::ToggleSearch => match self.mode {
                Mode::Search => {
                    self.mode = Mode::Normal;
                }
                _ => {
                    self.mode = Mode::Search;
                }
            },
            Action::ToggleQueue => match self.mode {
                Mode::WithQueue => {
                    self.mode = Mode::Normal;
                }
                _ => {
                    self.mode = Mode::WithQueue;
                }
            },
            Action::EnterProcessing => {
                self.mode = Mode::Processing;
            }
            Action::ExitProcessing => {
                // TODO: Make this go to previous mode instead
                self.mode = Mode::Normal;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Percentage(100),
                Constraint::Min(3),
            ])
            .split(rect);

        // Header
        const EMOJI_MARTINI: char = '\u{1F378}';
        const EMDASH: char = '\u{2014}';

        Title::
        new(format!(" {} CLIraoke {} Karaoke for the Command Line {} ", EMOJI_MARTINI, EMDASH, EMOJI_MARTINI).as_str())
            .draw(f, rects[0])?;

        // Lyrics block.
        let lyrics_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        // If queue is visible, create a horizontal layout with the queue on the right.
        match self.mode {
            Mode::WithQueue => {
                let inner_rects = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ].as_ref())
                    .split(rects[1]);

                let mut queue = Queue::new();
                f.render_widget(
                    lyrics_block,
                    inner_rects[0],
                );
                queue.draw(f, inner_rects[1])?;
            }
            Mode::WithHelp => {
                f.render_widget(
                    lyrics_block,
                    rects[1],
                );

                let mut help = Help::new();
                help.draw(f, rects[2])?;
            }
            Mode::Search => {
                let mut search = Search::new();
                search.draw(f, rects[1])?;

                let mut t = timer::Timer::new();
                t.draw(f, rects[2])?;
            }
            _ => {
                f.render_widget(
                    lyrics_block,
                    rects[1],
                );

                // Add Timer to the footer.
                let mut t = timer::Timer::new();
                t.draw(f, rects[2])?;
            }
        }

        // Footer
        match self.mode {
            Mode::WithHelp => Help::new()
                .draw(f, rects[2])?,
            _ => self.timer.draw(f, rects[2])?,
        }

        Ok(())
    }
}

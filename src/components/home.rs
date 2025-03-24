use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{queue, timer, Component, Frame};
use crate::action::Action;
use crate::components::help::Help;
use crate::components::queue::Queue;
use crate::components::search::Search;
use crate::models::song::Song;

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
pub struct Home {
  pub counter: usize,
  pub current_song: Song,
  pub mode: Mode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
}

impl Home {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  pub fn add(&mut self, s: String) {
    self.input = Input::default();
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

impl Component for Home {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    let action = match self.mode {
      Mode::Normal | Mode::Processing | Mode::WithHelp => return Ok(None),
      Mode::Search => match key.code {
        KeyCode::Esc => Action::CancelSearch,
        KeyCode::Enter => {
          if let Some(sender) = &self.action_tx {
            if let Err(e) = sender.send(Action::SearchSong("test".to_string())) {
              error!("Failed to send action: {:?}", e);
            }
          }
          Action::CancelSearch
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          Action::Update
        },
      },
      _ => {
        if let Some(action) = self.keymap.get(&key) {
          action.clone()
        } else {
          Action::Tick
        }
      }
    };

    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::ToggleHelp => self.show_help(),
      Action::SearchSong(s) => self.add(s),
      Action::CancelSearch => {
        self.mode = Mode::Normal;
      },
      Action::ToggleSearch => {
        match self.mode {
          Mode::Search => {
            self.mode = Mode::Normal;
          },
          _ => {
            self.mode = Mode::Search;
          }
        }
      },
      Action::ToggleQueue => {
        match self.mode {
          Mode::WithQueue => {
            self.mode = Mode::Normal;
          },
          _ => {
            self.mode = Mode::WithQueue;
          }
        }
      },
      Action::EnterProcessing => {
        self.mode = Mode::Processing;
      },
      Action::ExitProcessing => {
        // TODO: Make this go to previous mode instead
        self.mode = Mode::Normal;
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Percentage(100), Constraint::Min(3)])
        .split(rect);

    let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = self.input.visual_scroll(width as usize);

    const EMOJI_MARTINI: char = '\u{1F378}';
    const EMDASH: char = '\u{2014}';

    // Title of the app.
    f.render_widget(
      Paragraph::new(format!(" {} CLIraoke {} Karaoke for the Command Line {} ", EMOJI_MARTINI, EMDASH, EMOJI_MARTINI))
          .style(Style::default().fg(Color::Yellow))
          .alignment(Alignment::Center),
      rects[0],
    );

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
      },
      Mode::WithHelp => {
        f.render_widget(
          lyrics_block,
          rects[1],
        );

        let mut help = Help::new();
        help.draw(f, rects[2])?;
      },
      Mode::Search => {
        let mut search = Search::new();
        search.draw(f, rects[1])?;

        let mut t = timer::Timer::new();
        t.draw(f, rects[2])?;
      },
      _ => {
        f.render_widget(
          lyrics_block,
          rects[1],
        );

        // Add Timer to the footer.
        let mut t = timer::Timer::new();
        t.draw(f, rects[2])?;
      },
    }

    // let input = Paragraph::new(self.input.value())
    //     .alignment(Alignment::Center)
    //     .style(match self.mode {
    //       Mode::Search => Style::default().fg(Color::Yellow),
    //       _ => Style::default(),
    //     })
    //     .scroll((0, scroll as u16))
    //     .block(Block::default().borders(Borders::ALL).title_alignment(Alignment::Center).title(match self.mode {
    //       Mode::Search => Line::from(vec![
    //         Span::raw("Search for a song "),
    //         Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
    //         Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightRed)),
    //         Span::styled(" to submit)", Style::default().fg(Color::DarkGray)),
    //       ]),
    //       Mode::Help => Line::from(vec![
    //         "Press ".into(),
    //         Span::styled("u ", Style::default().fg(Color::Red)),
    //         "to see the ".into(),
    //         Span::styled("queue", Style::default().fg(Color::Yellow)),
    //         ", ".into(),
    //         Span::styled("/ ", Style::default().fg(Color::Red)),
    //         "to ".into(),
    //         Span::styled("search ", Style::default().fg(Color::Yellow)),
    //         "for a song, ".into(),
    //         Span::styled("j ", Style::default().fg(Color::Red)),
    //         "or ".into(),
    //         Span::styled("k ", Style::default().fg(Color::Red)),
    //         "to move lyrics ".into(),
    //         Span::styled("forward", Style::default().fg(Color::Yellow)),
    //         " or ".into(),
    //         Span::styled("backward ", Style::default().fg(Color::Yellow)),
    //         "in time and ".into(),
    //         Span::styled("q ", Style::default().fg(Color::Red)),
    //         "to ".into(),
    //         Span::styled("quit", Style::default().fg(Color::Yellow)),
    //       ]),
    //       _ => Line::from("Press / to search for a song"),
    //     }));
    // f.render_widget(input, rects[2]);
    // if self.mode == Mode::Search {
    //   f.set_cursor(
    //     (rects[2].x + (rects[2].width / 2) - (self.input.cursor() as u16 / 2)).min(rects[2].x + rects[2].width - 2),
    //     rects[2].y + 1,
    //   )
    // }

    Ok(())
  }
}

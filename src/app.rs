use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  action::Action,
  components::{home::Home, Component},
  tui,
};
use crate::tui::Event;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
  #[default]
  Navigation,
  Editing,
}

pub struct App {
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_play: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let home = Home::new();
    let mode = Mode::Navigation;
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(home)],
      should_quit: false,
      should_play: false,
      mode,
      last_tick_key_events: Vec::new(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?;
    tui.tick_rate(self.tick_rate);
    tui.frame_rate(self.frame_rate);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init()?;
    }

    loop {
      if let Some(e) = tui.next().await {
        match e {
          Event::Quit => action_tx.send(Action::Quit)?,
          Event::Tick => action_tx.send(Action::Tick)?,
          Event::Render => action_tx.send(Action::Render)?,
          Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          Event::Key(key) if key.kind == KeyEventKind::Press && self.mode == Mode::Navigation => match key.code {
            KeyCode::Char('q') => action_tx.send(Action::Quit)?,
            KeyCode::Char(' ') => action_tx.send(Action::TogglePlay)?,
            _ => {
              for component in self.components.iter_mut() {
                if let Some(action) = component.handle_events(Some(e.clone()))? {
                  action_tx.send(action)?;
                }
              }
            },
          },
          _ => {},
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          log::debug!("{action:?}");
        }
        match action {
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::TogglePlay => self.should_play = !self.should_play,
          Action::ToggleSearch => {
            match self.mode {
              Mode::Editing => {
                self.mode = Mode::Navigation;
              },
              _ => {
                self.mode = Mode::Editing;
              }
            }
          },
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_play {
        tui.suspend()?;
        action_tx.send(Action::TogglePlay)?;
        tui = tui::Tui::new()?;
        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}

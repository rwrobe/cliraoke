use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::components::{
    help::Help,
    search::Search,
    title::Title,
    timer::Timer
};
use crate::tui::Event;
use crate::{
    action::Action,
    components::{container::Container, Component},
    tui,
};
use crate::components::queue::Queue;

pub struct App {
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub should_quit: bool,
    pub should_play: bool,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        Ok(Self {
            tick_rate,
            frame_rate,
            should_quit: false,
            should_play: false,
            last_tick_key_events: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?;
        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        tui.enter()?;

        const EMOJI_MARTINI: char = '\u{1F378}';
        const EMDASH: char = '\u{2014}';


        let mut container = Container::new(
            Queue::new(),
            Search::new(),
        );

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    Event::Quit => action_tx.send(Action::Quit)?,
                    Event::Tick => action_tx.send(Action::Tick)?,
                    Event::Render => action_tx.send(Action::Render)?,
                    Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                        _ => {
                            container.handle_events(Some(Event::Key(key)))?;
                        }
                    },
                  _ => {}
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::TogglePlay => self.should_play = !self.should_play,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            container.draw(f, f.size()).unwrap();
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            container.draw(f, f.size()).unwrap();
                        })?;
                    }
                    _ => {}
                }

                container.update(action.clone())?;
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

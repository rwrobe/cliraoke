use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::components::help::Help;
use crate::components::search::Search;
use crate::components::timer::Timer;
use crate::tui::Event;
use crate::{
    action::Action,
    components::{home::Home, Component},
    tui,
};

pub struct App {
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_play: bool,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let help = Help::new();
        let home = Home::new();
        //let lyrics = Lyrics::new();
        let search = Search::new();
        let timer = Timer::new();
        Ok(Self {
            tick_rate,
            frame_rate,
            // Component ordering is important for event propagation and rendering. Only the first
            // component is rendered, and the events propagate from the first to the last. Any event that
            // triggers an action stops the event propagation.
            // TODO: This is janky and should be fixed.
            components: vec![
                Box::new(home),
                Box::new(search),
                Box::new(help),
                Box::new(timer),
            ],
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
                    Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                        _ => {
                            for component in self.components.iter_mut() {
                                if let Some(action) = component.handle_events(Some(e.clone()))? {
                                    action_tx.send(action.clone())?;
                                    // Stop propagating the event if an action was triggered.
                                    if action != Action::Noop {
                                        break;
                                    }
                                }
                            }
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
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    Action::Render => {
                        if self.components.len() == 0 {
                            continue;
                        }
                        tui.draw(|f| {
                            self.components[0].draw(f, f.size()).unwrap();
                        })?;
                    }
                    _ => {}
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

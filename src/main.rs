#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// ANCHOR: all
pub mod cli;
pub mod components;
mod models;
mod util;
mod events;
mod app;
mod state;
mod audio;
mod lyrics;

use crate::audio::youtube::YouTube;
use crate::lyrics::lrclib::LRCLib;
use anyhow::Result;
use app::AppComponent;
use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::dotenv;
use events::{Event, Events, Key};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

// APP_TICK_RATE is the rate in ms at which the app will render. For timers, ensure it cleanly
// divides 1000.
const APP_TICK_RATE: u64 = 200;
const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<()> {
  dotenv().ok();
  setup_terminal()?;

  let stdout = io::stdout();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  let events = Events::new(APP_TICK_RATE);


  let api_key = dotenv::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");
  // Create lyrics provider.
  let lyrics = LRCLib::new();
  let audio = YouTube::new(api_key);

  let mut app = AppComponent::new(
    &lyrics,
    &lyrics,
    &audio,
    &audio,
  );
  terminal.clear()?;

  loop {
    terminal.draw(|f| {
      if let Err(err) = app.render::<CrosstermBackend<io::Stdout>>(f, f.area()) {
        println!("Error thrown: {:?}", err);
        std::process::exit(1);
      }
    })?;

    match events.next()? {
      Event::Input(key) => match app.event(key).await {
        Ok(state) => {
          if !state.is_consumed() && (key == Key::Char('q')) {
            break;
          }
        }
        Err(_) => unimplemented!(),
      },

      Event::Tick => app.tick(APP_TICK_RATE)
    }
  }

  shutdown_terminal()?;
  terminal.show_cursor()?;

  Ok(())
}

fn setup_terminal() -> Result<()> {
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen)?;
  Ok(())
}

fn shutdown_terminal() -> Result<()> {
  disable_raw_mode()?;

  let mut stdout = io::stdout();
  execute!(stdout, LeaveAlternateScreen)?;
  Ok(())
}
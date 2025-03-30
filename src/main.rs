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

use anyhow::Result;
use app::AppComponent;
use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{Event, Events, Key};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use dotenv::dotenv;
use crate::audio::youtube::YouTube;
use crate::lyrics::lrclib::LRCLib;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<()> {
  dotenv().ok();
  setup_terminal()?;

  let stdout = io::stdout();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  let events = Events::new(200);


  let api_key = dotenv::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");
  // Create lyrics provider.
  let lyrics = LRCLib::new();
  let audio = YouTube::new(api_key);

  let mut app = AppComponent::new(
    &lyrics,
    &audio,
  );
  terminal.clear()?;

  loop {
    terminal.draw(|f| {
      if let Err(err) = app.render(f, f.size()) {
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

      Event::Tick => {}
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
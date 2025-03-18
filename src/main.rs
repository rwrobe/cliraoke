mod app;
mod audio;
mod cli;
mod lyrics;

use crate::app::App;
use dotenv::dotenv;
use std::{env, io};
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");

    let mut app = App::new();
    let res = run(terminal, api_key.as_str()).await;

    ratatui::restore();

    res
}

async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {

}
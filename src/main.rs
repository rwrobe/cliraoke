mod app;
mod audio;
mod cli;
mod lyrics;

use crate::app::App;
use crate::cli::cli::CLIOption;
use crossterm::event;
use crossterm::event::Event;
use dotenv::dotenv;
use ratatui::DefaultTerminal;
use std::env;
use std::process::exit;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");

    let res = App::default().run(terminal, api_key.as_str()).await;

    ratatui::restore();

    res
}

mod app;
mod audio;
mod cli;
mod lyrics;
mod ui;

use crate::app::App;
use crate::ui::ui;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use dotenv::dotenv;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::{env, io};
use ui::{UIMode, UIState};
use ratatui::widgets::{Block, List, ListState};

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

    let mut app = App::new();
    let res = run(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");

    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.ui_state {
                UIState::Search => match key.code {
                    KeyCode::Insert | KeyCode::Char('i') => {
                        app.ui_mode = UIMode::Edit;
                    }
                    KeyCode::Esc => {
                        app.ui_mode = UIMode::Navigation;
                        app.ui_state = UIState::Queue;
                    }
                    KeyCode::Enter => {
                        if app.ui_mode == UIMode::Edit {
                            if app.ui_state == UIState::Search {
                                let res = app.search(&api_key).await;
                                match res {
                                    Ok((audio, lyrics)) => {
                                        app.audio_results = audio;
                                        app.lyric_results = lyrics;
                                        app.ui_state = UIState::SelectAudio;
                                    }
                                    Err(e) => {
                                        eprintln!("Error: {}", e);
                                    }
                                }
                            }
                            app.ui_mode = UIMode::Navigation;
                        }
                    }
                    KeyCode::Char('q') => {
                        if app.ui_mode == UIMode::Edit {
                            app.query.push('q');
                        }
                        app.exit = true;
                    }
                    KeyCode::Char(value) => {
                        if app.ui_mode == UIMode::Edit {
                            app.query.push(value);
                        }
                    }
                    _ => {}
                },
                UIState::SelectAudio => match key.code {
                    KeyCode::Enter => {
                        // Add the song to the queue and advance to the lyrics selection state.
                    }
                    KeyCode::Esc => {
                        app.ui_state = UIState::Search;
                    }
                    _ => {}
                },
                UIState::Queue => match key.code {
                    KeyCode::Tab => {
                        app.ui_mode = UIMode::Navigation;
                        app.ui_state = UIState::Search;
                    }
                    KeyCode::Char('q') => {
                        app.exit = true;
                    }
                    // TODO: list navigation
                    _ => {}
                },
                UIState::Lyrics => match key.code {
                    KeyCode::Tab => {
                        app.ui_state = UIState::Queue;
                    }
                    KeyCode::Right => {
                        app.advance_lyrics();
                    }
                    KeyCode::Left => {
                        app.retreat_lyrics();
                    }
                    KeyCode::Char('q') => {
                        app.exit = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

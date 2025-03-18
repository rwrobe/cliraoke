mod app;
mod audio;
mod cli;
mod lyrics;
mod ui;

use crate::app::{App, UIMode, WidgetState};
use crate::audio::Audio;
use crate::ui::ui;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, execute};
use dotenv::dotenv;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::{env, io};

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

    let yt = Audio::new(api_key.as_str());
    let mut app = App::new(yt);
    let res = run(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(()) = res {
        // what?
    } else if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.widget_state {
                WidgetState::SearchYT | WidgetState::SearchLyrics => match key.code {
                    KeyCode::Insert => {
                        app.ui_mode = UIMode::Edit;
                    }
                    KeyCode::Esc => {
                        app.ui_mode = UIMode::Navigation;
                    }
                    KeyCode::Enter => {
                        app.search()
                    }
                    KeyCode::Char('q') => {
                        if app.ui_mode == UIMode::Edit {
                            app.query.push('q');
                        }
                        app.exit = true;
                    }
                    KeyCode::Char(value) => {
                        if let Some(mode) = &app.ui_mode {
                            match mode {
                                UIMode::Edit => {
                                    app.query.push(value);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                WidgetState::Queue => match key.code {
                    KeyCode::Tab => {
                        app.ui_mode = UIMode::Navigation;
                        app.widget_state = WidgetState::SearchYT;
                    }
                    KeyCode::Char('q') => {
                        app.exit = true;
                    }
                    // TODO: list navigation
                    _ => {}
                },
                WidgetState::Lyrics => match key.code {
                    KeyCode::Tab => {
                        app.widget_state = WidgetState::Queue;
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
                }
                _ => {}
            }
        }
    }

}

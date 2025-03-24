use crate::action::Action;
use crate::components::Component;
use crate::models::song::Song;
use crate::tui::{Event, Frame};
use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use log::error;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use tokio::sync::mpsc::UnboundedSender;

pub struct Queue {
    pub songs: Vec<Song>,
    pub current_song: Option<Song>,
    pub current_song_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    fn default() -> Self {
        Self {
            songs: Vec::new(),
            current_song: None,
            current_song_index: 0,
            action_tx: None,
        }
    }

    fn add(&mut self, song: Song) {
        self.songs.push(song);
    }
}

impl Component for Queue {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        todo!()
    }

    fn init(&mut self) -> Result<()> {
        todo!()
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            _ => None,
        };
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        todo!()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                if self.songs.is_empty() {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) = sender.send(Action::ToggleQueue) {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                }

                Ok(None)
            }
            Action::SearchSong(s) => {
                self.songs.push(Song {
                    lyric_id: "".to_string(),
                    video_id: "".to_string(),
                    title: s,
                    artist: "Unknown".to_string(),
                    synced_lyrics: "".to_string(),
                    lyric_map: None,
                    message: (),
                });

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        f.render_widget(
            Block::default()
                .title(Span::styled(
                    format!("{} in Queue", self.songs.len()),
                    Style::default().fg(Color::Yellow),
                ))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
            rect,
        );

        Ok(())
    }
}

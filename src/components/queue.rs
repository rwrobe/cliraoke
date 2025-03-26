use clap::builder::styling::AnsiColor::White;
use crate::action::Action;
use crate::components::Component;
use crate::models::song::Song;
use crate::tui::{Event, Frame};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Color, Modifier, Style, Stylize,
    },
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use ratatui::layout::Alignment;
use ratatui::style::Color::{Black, Cyan};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
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
        self.action_tx = Some(tx);
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            _ => None,
        };
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            _ => {Ok(None)}
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
       match action {
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

                if let Some(sender) = &mut self.action_tx {
                    if let Err(e) = sender.send(Action::Render) {
                        error!("Failed to send action: {:?}", e);
                    }
                }
            }
            _ => {}
        };

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let block = Block::new()
            .title(Line::from(format!(" {} songs in the queue ", self.songs.len())))
            .title_alignment(Alignment::Center)
            .borders(Borders::TOP)
            .border_style(Style::new().fg(Cyan));

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .songs
            .iter()
            .enumerate()
            .map(|(i, song)| {
                ListItem::new(song.title.clone()).bg(Black)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::new().bg(Cyan).fg(Black))
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, rect, buf, &mut self.songs.state);

        Ok(())
    }
}

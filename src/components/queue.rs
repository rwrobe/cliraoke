use crate::action::Action;
use crate::components::{RenderableComponent};
use crate::models::song::{Song, SongList};
use crate::events::{Event};
use clap::builder::styling::AnsiColor::White;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::style::Color::{Black, Cyan};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;
use crate::events::{EventState, Key};

#[derive(Default)]
pub struct Queue {
    pub songs: SongList,
    pub current_song: Option<Song>,
    pub current_song_index: usize,
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
        }
    }

    fn add(&mut self, song: Song) {
        self.songs.push(song);
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            _ => {
                // TODO: song navigation
            }
        }
        Ok(EventState::NotConsumed)
    }
}

impl RenderableComponent for Queue {
    fn render<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
    ) -> anyhow::Result<()> {
        let block = Block::new()
            .title(Line::from(format!(
                " {} songs in the queue ",
                self.songs.len()
            )))
            .title_alignment(Alignment::Center)
            .borders(Borders::TOP)
            .border_style(Style::new().fg(Cyan));

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .songs
            .iter()
            .enumerate()
            .map(|(i, song)| ListItem::new(song.title.clone()).bg(Black))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::new().bg(Cyan).fg(Black))
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        f.render_widget(list, rect); // should be stateful

        Ok(())
    }
}

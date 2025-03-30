use crate::components::RenderableComponent;
use crate::events::{EventState, Key};
use crate::models::song::{Song, SongList};
use color_eyre::eyre::Result;
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::style::Color::{Black, Cyan};
use ratatui::widgets::BorderType;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem,
    },
    Frame,
};
use crate::app::GlobalState;

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
        Ok(EventState::NotConsumed)
    }
}

impl RenderableComponent for Queue {
    fn render<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
        state: GlobalState,
    ) -> anyhow::Result<()> {
        let block = Block::new()
            .title(Line::from(format!(
                " {} songs in the queue ",
                self.songs.len()
            )))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

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

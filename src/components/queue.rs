use crate::app::GlobalState;
use crate::components::RenderableComponent;
use crate::events::{EventState, Key};
use crate::state::{get_state, with_state, Focus};
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
use std::sync::{Arc, Mutex};
use crate::util::EMDASH;

#[derive(Default)]
pub struct Queue {
    pub global_state: Arc<Mutex<GlobalState>>,
}

impl Queue {
    pub fn new(state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            global_state: state,
        }
    }

    pub async fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char('/') => {
                with_state(&self.global_state, |s| {
                    s.focus = Focus::Search;
                });

                return Ok(EventState::Consumed)
            }
            _ => {}
        }

        Ok(EventState::NotConsumed)
    }
}

impl RenderableComponent for Queue {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let songs = get_state(&self.global_state).song_list.clone();
        let block = Block::new()
            .title(Line::from(format!(
                " {} songs in the queue ",
                songs.len(),
            )))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = songs
            .iter()
            .enumerate()
            .map(|(i, song)| ListItem::new(format!("{} {} {}", song.title, EMDASH, song.artist)).bg(Black))
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

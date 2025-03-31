use crate::app::GlobalState;
use crate::components::RenderableComponent;
use crate::models::song::Song;
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::prelude::Line;
use ratatui::widgets::BorderType;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{
        Block, Borders,
    },
    Frame,
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Lyrics {
    pub song: Option<Song>,
}

impl Lyrics {
    pub fn new() -> Self {
        Self::default()
    }

    fn default() -> Self {
        Self {
            song: None,
        }
    }
}

impl RenderableComponent for Lyrics {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
        state: Arc<Mutex<GlobalState>>,
    ) -> anyhow::Result<()> {
        let block = Block::default()
            .title(Line::from(" Song by Artist "))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        f.render_widget(block, rect); // should be stateful

        Ok(())
    }
}

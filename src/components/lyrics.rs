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
    widgets::{Block, Borders},
    Frame,
};
use std::sync::{Arc, Mutex};

pub struct Lyrics {
    pub global_state: Arc<Mutex<GlobalState>>,
}

impl Lyrics {
    pub fn new(state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            global_state: state,
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
        let current_song = self.global_state.lock().unwrap().current_song.clone();

        match current_song {
            Some(song) => {
                let block = Block::default()
                    .title(Line::from(format!(" {} by {} ", song.title, song.artist,)))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(block, rect);
                // Render the lyrics here
            }
            None => {
                let block = Block::default()
                    .title(Line::from(" Press / to search for your first song "))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(block, rect);
            }
        }

        Ok(())
    }
}

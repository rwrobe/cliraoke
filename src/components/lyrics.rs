use crate::app::GlobalState;
use crate::components::RenderableComponent;
use crate::models::song::Song;
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Margin};
use ratatui::widgets::{BorderType, Paragraph, Wrap};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders},
    Frame,
};
use std::sync::{Arc, Mutex};
use crate::lyrics::{LyricsFetcher, LyricsService};

pub struct Lyrics<'a> {
    ls: &'a dyn LyricsService,
    pub global_state: Arc<Mutex<GlobalState>>,
}

impl<'c> Lyrics<'c> {
    pub fn new(state: Arc<Mutex<GlobalState>>, ls: &'c (dyn LyricsService + 'c)) -> Self {
        Self {
            ls,
            global_state: state,
        }
    }
}

impl RenderableComponent for Lyrics<'_> {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let current_song = self.global_state.lock().unwrap().current_song.clone();
        let current_lyrics = self.global_state.lock().unwrap().current_lyric.clone();

        match current_song {
            Some(song) => {
                let block = Block::default()
                    .title(Line::from(format!(" {} by {} ", song.title, song.artist,)))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(block, rect);


                let line = Line::from(current_lyrics);

                let p = Paragraph::new(line)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                let padded = rect.inner(Margin {
                    vertical: 2,
                    horizontal: 2,
                });

                f.render_widget(p, padded);
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

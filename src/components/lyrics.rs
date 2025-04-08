use crate::app::GlobalState;
use crate::components::RenderableComponent;
use crate::lyrics::LyricsService;
use crate::state::{AMGlobalState, get_state};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Margin};
use ratatui::widgets::{BorderType, Paragraph, Wrap};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders},
};
use std::sync::{Arc, Mutex};

pub struct Lyrics<LS>
where
    LS: LyricsService + Send + Sync + 'static,
{
    ls: Arc<LS>,
    pub global_state: AMGlobalState,
}

impl<LS> Lyrics<LS>
where
    LS: LyricsService + Send + Sync + 'static,
{
    pub fn new(state: Arc<Mutex<GlobalState>>, ls: Arc<LS>) -> Self
    where
        LS: LyricsService + Send + Sync + 'static,
    {
        Self {
            ls,
            global_state: state,
        }
    }
}

impl<LS> RenderableComponent for Lyrics<LS>
where
    LS: LyricsService + Send + Sync + 'static,
{
    fn render<B: Backend>(&self, f: &mut Frame, rect: Rect) -> anyhow::Result<()> {
        let gs = get_state(&self.global_state);
        let current_song = gs.current_song;
        let current_lyrics = gs.current_lyrics.clone();

        match current_song {
            Some(song) => {
                let block = Block::default()
                    .title(Line::from(format!(" {} by {} ", song.title, song.artist,)))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(block, rect);

                let lines = current_lyrics
                    .iter()
                    .map(|line| {
                        let line = line.to_string();
                        let line = line.replace('\n', " ");
                        Line::from(line)
                    })
                    .collect::<Vec<_>>();

                let p = Paragraph::new(lines)
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

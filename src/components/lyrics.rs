use crate::app::GlobalState;
use crate::components::RenderableComponent;
use crate::lyrics::LyricsService;
use crate::state::{AMGlobalState, get_state};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::widgets::{BorderType, Paragraph, Wrap};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders},
};
use std::sync::{Arc, Mutex};
use color_eyre::owo_colors::OwoColorize;
use crate::components::title::Title;
use crate::util::{EMDASH, EMOJI_MARTINI};

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
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Percentage(100),
                    ].as_ref())
                    .split(rect);

                let (title, body) = (chunks[0], chunks[1]);

                let lyrics_title = Title::new(
                    format!("Now Playing: {} by {} ", song.title, song.artist).as_str(),
                );
                lyrics_title.render::<B>(f, title)?;

                // Lyrics vertically centered.
                let lines = current_lyrics
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        let line = line.to_string();
                        let line = line.replace('\n', " ");
                        if i == 1 {
                            Line::from(line).style(Style::default().fg(Color::Green))
                        } else {
                            Line::from(line).style(Style::default().fg(Color::DarkGray))
                        }
                    })
                    .collect::<Vec<_>>();

                let line_ct = lines.iter().len() as u16;
                let vertical_offset = (body.height.saturating_sub(line_ct)) / 2;

                let centered_body = Rect {
                    x: body.x,
                    y: body.y + vertical_offset,  // This centers the paragraph vertically
                    width: body.width,
                    height: line_ct,
                };

                let p = Paragraph::new(lines)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                f.render_widget(p, centered_body);
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

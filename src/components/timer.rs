use super::RenderableComponent;
use crate::app::GlobalState;
use block::Title;
use ratatui::style::Stylize;
use ratatui::widgets::block;
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::Block,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum Ticker {
    SongRemainingTicker,
}

#[derive(Debug)]
pub struct Timer {
    global_state: Arc<Mutex<GlobalState>>,
}

impl Timer {
    pub fn new(state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            global_state: state,
        }
    }
}

impl RenderableComponent for Timer {
    fn render<B: Backend>(&self, f: &mut Frame, rect: Rect) -> anyhow::Result<()> {
        let global_state = self.global_state.lock().unwrap();

        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect);

        let (left, right) = (rects[0], rects[1]);

        let s = format!(
            "Singing for {:02}:{:02}",
            global_state.session_time_elapsed.as_secs() / 60,
            global_state.session_time_elapsed.as_secs() % 60,
        );
        let time_singing = Block::default()
            .title(Title::from(s.dim()))
            .title_alignment(Alignment::Left);
        f.render_widget(time_singing, left);

        // If we have another song in the queue, show the time remaining.
        match global_state.song_list.get(0) {
            Some(song) => {
                if song.duration_ms == 0 {
                    return Ok(());
                }

                // Song remaining time as ms.
                let song_remaining_time = song.duration_ms - global_state.current_song_elapsed;

                if song_remaining_time == 0 {
                    return Ok(());
                }

                let time_remaining = format!(
                    " {:02}:{:02}",
                    song_remaining_time / 60_000,
                    (song_remaining_time % 60_000) / 1000,
                );

                let next_song = format!(
                    " until {}",
                    global_state
                        .song_list
                        .get(0)
                        .ok_or("Title Unknown")
                        .unwrap()
                        .title
                );

                let time_to_next = Block::default()
                    .title(Title::from(
                        format!("{}{}", time_remaining, next_song).dim(),
                    ))
                    .title_alignment(Alignment::Right);

                f.render_widget(time_to_next, right);
            }
            _ => {}
        };

        Ok(())
    }
}

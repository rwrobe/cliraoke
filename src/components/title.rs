use super::RenderableComponent;
use crate::app::GlobalState;
use ratatui::{prelude::*, widgets::*};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Title {
    pub text: String,
}

impl Title {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl RenderableComponent for Title {
    fn render<B: Backend>(
        &self,
        f: &mut Frame,
        rect: Rect,
        state: Arc<Mutex<GlobalState>>,
    ) -> anyhow::Result<()> {
        {
            f.render_widget(
                Paragraph::new(self.text.clone())
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center),
                rect,
            );

            Ok(())
        }
    }
}

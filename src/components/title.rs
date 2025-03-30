use ratatui::{prelude::*, widgets::*};

use super::RenderableComponent;

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
        f: &mut ratatui::Frame<B>,
        rect: Rect,
        focused: bool,
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

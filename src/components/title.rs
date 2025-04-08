use super::RenderableComponent;
use ratatui::{prelude::*, widgets::*};

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

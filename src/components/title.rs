use clap::builder::Str;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{queue, timer, Frame, RenderableComponent};
use crate::action::Action;
use crate::components::help::Help;
use crate::components::queue::Queue;
use crate::components::search::Search;
use crate::models::song::Song;

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

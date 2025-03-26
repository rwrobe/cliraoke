use std::{collections::HashMap, time::Duration};
use clap::builder::Str;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{queue, timer, Component, Frame};
use crate::action::Action;
use crate::components::help::Help;
use crate::components::queue::Queue;
use crate::components::search::Search;
use crate::models::song::Song;
use crate::tui::Event;

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

impl Component for Title {
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        f.render_widget(
            Paragraph::new(self.text.clone())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center),
            rect,
        );

        Ok(())
    }
}

use anyhow::Result;
use ratatui::{backend::Backend, layout::Rect, Frame};
use crate::app::GlobalState;

pub(crate) mod help;
pub(crate) mod queue;
pub(crate) mod search;
pub(crate) mod timer;
pub(crate) mod title;
pub(crate) mod lyrics;
mod stateful_list;

pub trait RenderableComponent {
  fn render<B: Backend>(&self, f: &mut Frame<B>, rect: Rect, state: GlobalState) -> Result<()>;
}
use anyhow::Result;
use tui::{backend::Backend, layout::Rect, Frame};

mod lyrics;
pub(crate) mod help;
pub(crate) mod queue;
pub(crate) mod search;
pub(crate) mod timer;
pub(crate) mod title;
mod stateful_list;

pub trait RenderableComponent {
  fn render<B: Backend>(&self, f: &mut Frame<B>, rect: Rect, focused: bool) -> Result<()>;
}
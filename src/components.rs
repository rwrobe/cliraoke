use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::Action,
  tui::{Event, Frame},
};

pub(crate) mod home;
pub(crate) mod timer;
pub(crate) mod queue;
mod lyrics;
pub(crate) mod help;
pub(crate) mod search;

//// ANCHOR: component
pub trait Component {
  #[allow(unused_variables)]
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    Ok(())
  }
  fn init(&mut self) -> Result<()> {
    Ok(())
  }
  fn name(&mut self) -> &'static str;
  fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
    let r = match event {
      Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
      _ => None,
    };
    Ok(r)
  }
  #[allow(unused_variables)]
  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    Ok(None)
  }
  #[allow(unused_variables)]
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    Ok(None)
  }
  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()>;
}
//// ANCHOR_END: component

use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

//// ANCHOR: action_enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
  Tick,
  Render,
  Resize(u16, u16),
  Play,
  Quit,
  Refresh,
  Error(String),
  Help,
  ScheduleIncrement,
  ScheduleDecrement,
  Increment(usize),
  Decrement(usize),
  SearchSong(String),
  CancelSearch,
  EnterProcessing,
  ExitProcessing,
  Update,
  ToggleHelp,
  TogglePlay,
  ToggleQueue,
  ToggleSearch,
}
//// ANCHOR_END: action_enum

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// ANCHOR: all
pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod tui;
mod models;
mod util;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use crate::{
  app::App,
};

async fn tokio_main() -> Result<()> {
  let args = Cli::parse();
  let mut app = App::new(args.tick_rate, args.frame_rate)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
// ANCHOR_END: all

mod cmd;
mod handle;

use crate::cmd::Args;
use clap::Parser;
use handle::EventHandler;
use releasy_core::event::Event;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let received_event = Event::try_from(args)?;
    received_event.handle()?;
    Ok(())
}

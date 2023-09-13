mod cmd;
mod event;

use clap::Parser;
use cmd::Args;
use event::Event;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let event = Event::try_from(args)?;
    println!("{event:?}");
    Ok(())
}

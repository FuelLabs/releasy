use std::path::PathBuf;

use clap::Parser;

pub(crate) const DEFAULT_MANIFEST_FILE_NAME: &str = "repo-plan.toml";

/// Command line tool to handle repo different repo dispatch events.
///
///
/// Event details can be provided via flags:
///
/// ```
/// releasy-emit --event "new-commit"
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Type of the event.
    ///
    /// Possible values: [new-commit, new-release]
    #[arg(long)]
    pub(crate) event: Option<String>,

    /// Path to the manifest file describing repo plan.
    ///
    /// By default `repo-plan.toml` expected to be in the current dir.
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,
}

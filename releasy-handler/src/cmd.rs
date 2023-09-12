use clap::Parser;

/// Command line tool to handle repo different repo dispatch events.
///
///
/// Event details can be provided via flags:
///
/// ```
/// releasy-handler --event "new-commit" --repo-name "sway" --repo-owner "FuelLabs"
/// ```
///
/// Or a JSON can be used to describe the event, which is more useful for CI applications.
///
/// ```
/// releasy-handler --json "{ event: "new-commit", repo-name: "sway", repo-owner: "FuelLabs" }"
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Type of the event.
    ///
    /// Possible values: [new-commit, new-release]
    #[arg(long)]
    pub(crate) event: Option<String>,

    /// Name of the repo creating this event.
    #[arg(long)]
    pub(crate) repo_name: Option<String>,

    /// Owner of the repo creating this event.
    #[arg(long)]
    pub(crate) repo_owner: Option<String>,

    /// JSON string descirbing the event.
    ///
    /// Cannot use any other flag if this parameter is provided.
    #[arg(long)]
    pub(crate) json: Option<String>,
}
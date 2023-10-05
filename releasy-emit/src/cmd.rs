use std::{env::current_dir, path::PathBuf, str::FromStr};

use clap::Parser;
use releasy_core::{
    default::DEFAULT_MANIFEST_FILE_NAME,
    event::{ClientPayload, Event, EventDetails, EventType},
    repo::Repo,
};
use releasy_graph::manifest::ManifestFile;

/// Command line tool to emit repo different repo dispatch events.
///
///
/// Event details can be provided via flags:
///
/// ```
/// releasy-emit --event "new-commit-to-dependency"
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Type of the event.
    ///
    /// Possible values: [new-commit-to-self, new-commit-to-dependency, new-release]
    #[arg(long)]
    pub(crate) event: Option<String>,

    /// Path to the manifest file describing repo plan.
    ///
    /// By default `repo-plan.toml` expected to be in the current dir.
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    /// Commit hash that triggered this event.
    #[arg(long)]
    pub(crate) event_commit_hash: Option<String>,

    /// Release tag that triggered this event.
    #[arg(long)]
    pub(crate) event_release_tag: Option<String>,
}

impl TryFrom<Args> for Event {
    type Error = anyhow::Error;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        /// Handles the case when event details are provided via parameters from the CLI.
        fn handle_event_params(
            event: &str,
            repo_name: &str,
            repo_owner: &str,
            event_commit_hash: Option<String>,
            event_release_tag: Option<String>,
        ) -> anyhow::Result<Event> {
            let event_type = EventType::from_str(event)?;
            let repo = Repo::new(repo_name.to_string(), repo_owner.to_string());
            let details = EventDetails::new(event_commit_hash, event_release_tag);
            let client_payload = ClientPayload::new(repo, details);
            let event = Event::new(event_type, client_payload);

            Ok(event)
        }
        let current_dir = current_dir()?;
        let manifest_path = value
            .path
            .unwrap_or_else(|| current_dir.join(DEFAULT_MANIFEST_FILE_NAME));
        let manifest_file = ManifestFile::from_file(&manifest_path)?.manifest();

        let current_repo = manifest_file.current_repo();

        let event = value
            .event
            .ok_or_else(|| anyhow::anyhow!("event should not be emtpy"))?;
        let repo_name = current_repo.name();
        let repo_owner = current_repo.owner();
        handle_event_params(
            &event,
            repo_name,
            repo_owner,
            value.event_commit_hash,
            value.event_release_tag,
        )
    }
}

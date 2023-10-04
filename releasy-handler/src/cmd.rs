use clap::Parser;
use releasy_core::{
    event::{ClientPayload, Event, EventDetails, EventType},
    repo::Repo,
};
use std::{path::PathBuf, str::FromStr};

/// Command line tool to handle repo different repo dispatch events.
///
///
/// Event details can be provided via flags:
///
/// ```
/// releasy-handler --event "new-commit-to-dependency" --repo-name "repo-name" --repo-owner "repo-owner"
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Type of the event.
    ///
    /// Possible values: [new-commit-to-dependency, new-release, new-commit-to-self]
    #[arg(long)]
    pub(crate) event: Option<String>,

    /// Name of the repo emitted this event.
    #[arg(long)]
    pub(crate) event_repo_name: Option<String>,

    /// Owner of the repo emitted this event.
    #[arg(long)]
    pub(crate) event_repo_owner: Option<String>,

    /// Commit hash that triggered this event.
    #[arg(long)]
    pub(crate) event_commit_hash: Option<String>,

    /// Release tag that triggered this event.
    #[arg(long)]
    pub(crate) event_release_tag: Option<String>,

    /// Path to the manifest file describing repo plan.
    ///
    /// By default `repo-plan.toml` expected to be in the current dir.
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,
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
        let event = value
            .event
            .ok_or_else(|| anyhow::anyhow!("event should not be emtpy"))?;
        let event_repo_name = value
            .event_repo_name
            .ok_or_else(|| anyhow::anyhow!("repo name should not be emtpy"))?;
        let event_repo_owner = value
            .event_repo_owner
            .ok_or_else(|| anyhow::anyhow!("repo owner should not be emtpy"))?;
        handle_event_params(
            &event,
            &event_repo_name,
            &event_repo_owner,
            value.event_commit_hash,
            value.event_release_tag,
        )
    }
}

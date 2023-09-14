use crate::cmd::{Args, DEFAULT_MANIFEST_FILE_NAME};
use anyhow::Context;
use releasy_graph::{manifest::ManifestFile, plan::Repo};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{env::current_dir, fmt::Display, str::FromStr};

/// An event to be emitted.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    event_type: EventType,
    client_payload: ClientPayload,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClientPayload {
    repo: Repo,
}

impl ClientPayload {
    pub fn new(repo: Repo) -> Self {
        Self { repo }
    }
}

impl Event {
    pub fn new(event_type: EventType, client_payload: ClientPayload) -> Self {
        Self {
            event_type,
            client_payload,
        }
    }

    /// Send this event to a target repo using github API.
    pub async fn send_to_repo(&self, target_repo: &Repo) -> anyhow::Result<()> {
        let github_token =
            std::env::var("DISPATCH_TOKEN").with_context(|| "missing DISPATCH_TOKEN variable")?;
        let client = reqwest::Client::builder().build()?;
        let request_url = format!(
            "https://api.github.com/repos/{}/{}/dispatches",
            target_repo.owner(),
            target_repo.name()
        );

        let bearer_token = format!("Bearer {github_token}");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(AUTHORIZATION, bearer_token.parse()?);
        headers.insert(ACCEPT, "application/vnd.github+json".parse()?);
        headers.insert(USER_AGENT, "releasy".parse()?);

        let json_str = serde_json::to_string(self)?;

        let request = client
            .request(reqwest::Method::POST, request_url)
            .headers(headers)
            .body(json_str);

        request.send().await?;
        Ok(())
    }
}

/// Possible event types.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    NewCommit,
    NewRelease,
}

impl FromStr for EventType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "new-commit" {
            Ok(Self::NewCommit)
        } else if s == "new-release" {
            Ok(Self::NewRelease)
        } else {
            anyhow::bail!("unexpected event type str")
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::NewCommit => write!(f, "new-commit"),
            EventType::NewRelease => write!(f, "new-release"),
        }
    }
}

impl TryFrom<Args> for Event {
    type Error = anyhow::Error;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        /// Handles the case when event details are provided via parameters from the CLI.
        fn handle_event_params(
            event: &str,
            repo_name: &str,
            repo_owner: &str,
        ) -> anyhow::Result<Event> {
            let event_type = EventType::from_str(event)?;
            let repo = Repo::new(repo_name.to_string(), repo_owner.to_string());
            let client_payload = ClientPayload::new(repo);
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
        handle_event_params(&event, repo_name, repo_owner)
    }
}

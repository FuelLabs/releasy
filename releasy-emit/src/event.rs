use crate::cmd::Args;
use releasy_graph::plan::Repo;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// An event received.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    event_type: EventType,
    repo: Repo,
}

impl Event {
    pub fn new(event_type: EventType, repo: Repo) -> Self {
        Self { event_type, repo }
    }
}

/// Possible event types.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
        /// Handles the case when a JSON is provided from the CLI.
        fn handle_json(json_str: &str) -> anyhow::Result<Event> {
            Ok(serde_json::from_str(json_str)?)
        }
        /// Handles the case when event details are provided via parameters from the CLI.
        fn handle_event_params(
            event: &str,
            repo_name: &str,
            repo_owner: &str,
        ) -> anyhow::Result<Event> {
            let event_type = EventType::from_str(event)?;
            let repo = Repo::new(repo_name.to_string(), repo_owner.to_string());
            let event = Event::new(event_type, repo);

            Ok(event)
        }

        let json_str = value.json;
        if let Some(json_str) = json_str {
            if value.repo_name.is_some() || value.repo_owner.is_some() || value.event.is_some() {
                anyhow::bail!("--json should be used without any other parameters")
            }
            handle_json(&json_str)
        } else {
            let event = value
                .event
                .ok_or_else(|| anyhow::anyhow!("event should not be emtpy"))?;
            let repo_name = value
                .repo_name
                .ok_or_else(|| anyhow::anyhow!("repo_name should not be emtpy"))?;
            let repo_owner = value
                .repo_owner
                .ok_or_else(|| anyhow::anyhow!("repo_owner should not be emtpy"))?;
            handle_event_params(&event, &repo_name, &repo_owner)
        }
    }
}

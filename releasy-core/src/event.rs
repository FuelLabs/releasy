use crate::{error::ReleasyCoreError, repo::Repo};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// An event to be emitted.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    event_type: EventType,
    client_payload: ClientPayload,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClientPayload {
    repo: Repo,
    details: EventDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EventDetails {
    commit_hash: Option<String>,
    release_tag: Option<String>,
}

impl EventDetails {
    pub fn new(commit_hash: Option<String>, release_tag: Option<String>) -> Self {
        Self {
            commit_hash,
            release_tag,
        }
    }

    pub fn commit_hash(&self) -> Option<&String> {
        self.commit_hash.as_ref()
    }

    pub fn release_tag(&self) -> Option<&String> {
        self.release_tag.as_ref()
    }
}

impl ClientPayload {
    pub fn new(repo: Repo, details: EventDetails) -> Self {
        Self { repo, details }
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn details(&self) -> &EventDetails {
        &self.details
    }
}

impl Event {
    const ACCEPT: &'static str = "application/vnd.github+json";
    const USER_AGENT: &'static str = "releasy";

    pub fn new(event_type: EventType, client_payload: ClientPayload) -> Self {
        Self {
            event_type,
            client_payload,
        }
    }

    /// Send this event to a target repo using github API.
    pub async fn send_to_repo(&self, target_repo: &Repo) -> Result<(), ReleasyCoreError> {
        let github_token = std::env::var("DISPATCH_TOKEN")
            .map_err(|_| ReleasyCoreError::MissingDispatchTokenEnvVariable)?;
        let client = reqwest::Client::builder()
            .build()
            .map_err(|_| ReleasyCoreError::FailedToBuildReqwestClient)?;
        let request_url = format!(
            "https://api.github.com/repos/{}/{}/dispatches",
            target_repo.owner(),
            target_repo.name()
        );

        let bearer_token = format!("Bearer {github_token}");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            bearer_token
                .parse()
                .map_err(|_| ReleasyCoreError::FailedToParseHeader(bearer_token))?,
        );
        headers.insert(
            ACCEPT,
            Self::ACCEPT
                .parse()
                .map_err(|_| ReleasyCoreError::FailedToParseHeader(Self::ACCEPT.to_string()))?,
        );
        headers.insert(
            USER_AGENT,
            Self::USER_AGENT
                .parse()
                .map_err(|_| ReleasyCoreError::FailedToParseHeader(Self::USER_AGENT.to_string()))?,
        );

        let json_str =
            serde_json::to_string(self).map_err(ReleasyCoreError::FailedToSerializeEventToJSON)?;

        let request = client
            .request(reqwest::Method::POST, request_url)
            .headers(headers)
            .body(json_str);

        request
            .send()
            .await
            .map_err(|e| ReleasyCoreError::FailedToSendDispatchRequest(target_repo.clone(), e))?;
        Ok(())
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn client_payload(&self) -> &ClientPayload {
        &self.client_payload
    }
}

/// Possible event types.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    NewCommitToDependency,
    NewCommitToSelf,
    NewRelease,
}

impl FromStr for EventType {
    type Err = ReleasyCoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "new-commit-to-dependency" {
            Ok(Self::NewCommitToDependency)
        } else if s == "new-commit-to-self" {
            Ok(Self::NewCommitToSelf)
        } else if s == "new-release" {
            Ok(Self::NewRelease)
        } else {
            return Err(ReleasyCoreError::FailedToConvertStrToEventType(
                s.to_string(),
            ));
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::NewCommitToDependency => write!(f, "new-commit-to-dependency"),
            EventType::NewCommitToSelf => write!(f, "new-commit-to-self"),
            EventType::NewRelease => write!(f, "new-release"),
        }
    }
}

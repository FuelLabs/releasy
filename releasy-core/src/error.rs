use thiserror::Error;

use crate::repo;

#[derive(Error, Debug)]
pub enum ReleasyCoreError {
    #[error("DISPATCH_TOKEN env variable is missing.\nPlease set the variable to a token that grants read and write access to repos in the dependency tree.")]
    MissingDispatchTokenEnvVariable,
    #[error("GITHUB_TOKEN env variable is missing.")]
    MissingGithubTokenEnvVariable,
    #[error("failed to build a reqwest client for dispatching events.")]
    FailedToBuildReqwestClient,
    #[error("failed to parse `{0}` as a request header.")]
    FailedToParseHeader(String),
    #[error("failed to serialize event to a JSON string, reason: `{0}`")]
    FailedToSerializeEventToJSON(serde_json::Error),
    #[error("failed to send dispatch request to {0}, reason: `{1}`")]
    FailedToSendDispatchRequest(repo::Repo, reqwest::Error),
    #[error("failed to convert str (`{0}`) to `EventType`, possible values are: [`new-commit`, `new-release`]")]
    FailedToConvertStrToEventType(String),
}

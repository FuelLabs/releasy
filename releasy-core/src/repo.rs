use serde::{Deserialize, Serialize};

use crate::error::ReleasyCoreError;

/// Represents a repository, a node in the dependency graph.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Repo {
    /// Name of the repository.
    name: String,
    /// Owner of the repository.
    owner: String,
}

impl Repo {
    pub fn new(name: String, owner: String) -> Self {
        Self { name, owner }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn github_url(&self) -> Result<String, ReleasyCoreError> {
        let github_token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| ReleasyCoreError::MissingGithubTokenEnvVariable)?;
        let github_actor = std::env::var("GITHUB_ACTOR")
            .map_err(|_| ReleasyCoreError::MissingGithubTokenEnvVariable)?;

        Ok(format!(
            "https://{}:{}@github.com/{}/{}.git",
            github_actor, github_token, self.owner, self.name
        ))
    }
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {} - owner: {}", self.name, self.owner)
    }
}

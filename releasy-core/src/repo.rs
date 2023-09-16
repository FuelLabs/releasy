use serde::{Deserialize, Serialize};

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

    pub fn github_url(&self) -> String {
        format!("git@github.com:{}/{}.git", self.owner, self.name)
    }
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {} - owner: {}", self.name, self.owner)
    }
}

use thiserror::Error;

use crate::plan::Repo;

#[derive(Error, Debug)]
pub enum BuildPlanError {
    #[error("`{0}` is depending on a project (`{1}`) that does not have a definition in the manifest file")]
    MissingProjectDefinition(String, String),
    #[error("provided repo `{0}` not found in dependency graph")]
    RepoNotFoundInGraph(Repo),
}

#[derive(Error, Debug)]
pub enum ManifestFileError {
    #[error("failed to read manifest file at `{0}`: {1}")]
    MissingManifestFile(String, std::io::Error),
    #[error("failed to parse manifest: {0}")]
    FailedToParseManifest(toml::de::Error),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildPlanError {
    #[error("Missing manifest file:`{0}`is not available")]
    MissingManifestFile(String),
}

#[derive(Error, Debug)]
pub enum ManifestFileError {
    #[error("Failed to read manifest file at `{0}`: {1}")]
    MissingManifestFile(String, std::io::Error),
    #[error("Failed to parse manifest: {0}")]
    FailedToParseManifest(toml::de::Error),
}

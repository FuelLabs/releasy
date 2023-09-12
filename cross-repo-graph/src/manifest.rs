use crate::plan::Repo;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

use crate::error::ManifestFileError;

/// A toml manifest file describing relations between different repositories.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub(crate) project: BTreeMap<String, Project>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Generated `Manifest` and possible warnings created during the process.
pub struct ManifestFile {
    warnings: Vec<String>,
    manifest: Manifest,
}

impl ManifestFile {
    /// Returns an iterator over warnings produced while generating the `Manifest`.
    pub fn warnings(&self) -> impl Iterator<Item = &String> {
        self.warnings.iter()
    }

    /// Takes ownership of this struct and returns underlying `Manifest`.
    pub fn manifest(self) -> Manifest {
        self.manifest
    }

    pub fn from_file(path: &Path) -> Result<ManifestFile, ManifestFileError> {
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| ManifestFileError::MissingManifestFile(format!("{path:?}"), e))?;
        ManifestFile::try_from(manifest_str)
    }
}

impl TryFrom<String> for ManifestFile {
    type Error = ManifestFileError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut warnings = vec![];
        let toml_de = toml::de::Deserializer::new(&value);
        let manifest: Manifest = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {path}");
            warnings.push(warning);
        })
        .map_err(ManifestFileError::FailedToParseManifest)?;

        let manifest_with_warnings = ManifestFile { warnings, manifest };
        Ok(manifest_with_warnings)
    }
}

/// A repository entry in the manifest, describing a repository and its dependencies.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub(crate) repo: Repo,
    /// Dependencies of this repo
    dependencies: Option<Vec<String>>,
}

impl Project {
    /// Returns a reference to underlying `Repo`.
    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    /// Returns an iterator over dependencies decribed in this `Project`.
    pub fn dependencies(&self) -> impl Iterator<Item = &String> {
        self.dependencies.iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::ManifestFile;

    #[test]
    fn parse_manifest_file_no_dependencies() {
        let manifest_str = r#"
[project.sway.repo]
name = "sway"
owner = "FuelLabs"
"#;

        let parsed = ManifestFile::try_from(manifest_str.to_string()).is_ok();
        assert!(parsed)
    }

    #[test]
    fn parse_manifest_file_two_projects_with_dependencies() {
        let manifest_str = r#"
[project.sway.repo]
name = "sway"
owner = "FuelLabs"

[project.sway]
dependencies = ["rust-sdk"]

[project.rust-sdk.repo]
name = "fuels-rs"
owner = "FuelLabs"
"#;

        let parsed = ManifestFile::try_from(manifest_str.to_string()).is_ok();
        assert!(parsed)
    }
}

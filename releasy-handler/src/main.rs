mod cmd;
mod handle;

use std::env::current_dir;

use crate::cmd::Args;
use clap::Parser;
use handle::EventHandler;
use releasy_core::{default::DEFAULT_MANIFEST_FILE_NAME, event::Event};
use releasy_graph::manifest::ManifestFile;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let current_dir = current_dir()?;
    let path = args
        .path
        .clone()
        .unwrap_or_else(|| current_dir.join(DEFAULT_MANIFEST_FILE_NAME));
    let received_event = Event::try_from(args)?;
    let manifest_file = ManifestFile::from_file(&path)?;
    for warning in manifest_file.warnings() {
        println!("WARNING: {warning}");
    }
    let manifest = manifest_file.manifest();
    received_event.handle(manifest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cmd::Args;
    use releasy_core::{
        event::{ClientPayload, Event, EventDetails, EventType},
        repo::Repo,
    };
    use releasy_graph::{manifest::ManifestFile, plan::Plan};

    const SWAY_WALLET_SDK_TEST_MANIFEST_FILE_NAME: &str = "repo-plan-sway-wallet-sdk.toml";

    #[test]
    fn parse_event_from_param_input() {
        let test_manifest_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(SWAY_WALLET_SDK_TEST_MANIFEST_FILE_NAME);

        let repo_name = "fuels-rs".to_string();
        let repo_owner = "FuelLabs".to_string();
        let event_type = "new-commit-to-dependency".to_string();

        let expected_commit_hash = "337d0eaa130dd18e9e347f83ab4fab76b3a6bd2a".to_string();
        let args = Args {
            event: Some(event_type),
            event_commit_hash: Some(expected_commit_hash.clone()),
            event_release_tag: None,
            event_repo_name: Some(repo_name.clone()),
            event_repo_owner: Some(repo_owner.clone()),
            path: Some(test_manifest_file),
        };

        let parsed_event = Event::try_from(args).unwrap();
        let sway_repo = Repo::new(repo_name, repo_owner);
        let details = EventDetails::new(Some(expected_commit_hash), None);
        let client_payload = ClientPayload::new(sway_repo, details);
        let expected_event = Event::new(EventType::NewCommitToDependency, client_payload);

        assert_eq!(parsed_event, expected_event)
    }

    /// In this test we have:
    ///  - forc-wallet
    ///  - sway
    ///  - fuels-rs
    /// repositories present. The dependency graph between them looks like:
    ///
    /// ```
    /// forc-wallet -> fuels-rs
    /// sway -> fuels-rs
    /// sway -> forc-wallet
    /// ```
    ///
    /// and the `current_repo` is `sway`.
    ///
    /// This is a simple example and the circular dependency between the sdk and sway is omitted.
    #[test]
    fn sway_wallet_sdk_example_test_event_order() {
        let test_manifest_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(SWAY_WALLET_SDK_TEST_MANIFEST_FILE_NAME);

        let manifest = ManifestFile::from_file(&test_manifest_file)
            .unwrap()
            .manifest();
        let current_repo = manifest.current_repo().clone();
        assert_eq!(current_repo.name(), "sway");

        let plan = Plan::try_from_manifest(manifest).unwrap();

        let upstream_repos = plan
            .upstream_repos(current_repo)
            .unwrap()
            .map(|repo| repo.name())
            .collect::<Vec<_>>();
        let expected_target_repos = vec!["forc-wallet", "fuels-rs"];

        assert_eq!(upstream_repos, expected_target_repos)
    }
}

mod cmd;
mod event;

use std::env::current_dir;

use clap::Parser;
use cmd::Args;
use event::Event;
use releasy_graph::{manifest::ManifestFile, plan::Plan};

const MANIFEST_FILE_NAME: &str = "repo-plan.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let current_dir = current_dir()?;
    let path = args
        .path
        .clone()
        .unwrap_or_else(|| current_dir.join(MANIFEST_FILE_NAME));
    let event = Event::try_from(args)?;

    let manifest_file = ManifestFile::from_file(&path)?;
    for warning in manifest_file.warnings() {
        println!("WARNING: {warning}");
    }
    let manifest = manifest_file.manifest();
    let current_repo = manifest.current_repo().clone();
    let plan = Plan::try_from_manifest(manifest)?;

    for target_repo in plan.neighbors(current_repo)? {
        event.send_to_repo(target_repo).await?;
        println!("Sending {event:?} to {target_repo:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use releasy_graph::{
        manifest::ManifestFile,
        plan::{Plan, Repo},
    };

    use crate::{
        cmd::Args,
        event::{ClientPayload, Event, EventType},
    };

    const SWAY_WALLET_SDK_TEST_MANIFEST_FILE_NAME: &str = "repo-plan-sway-wallet-sdk.toml";

    #[test]
    fn parse_event_from_json_input() {
        let json_str = r#"{ "event_type": "new-commit", "client_payload": { "repo": { "name": "sway", "owner": "FuelLabs"} } }"#;
        let args = Args {
            event: None,
            repo_name: None,
            repo_owner: None,
            json: Some(json_str.to_string()),
            path: None,
        };

        let parsed_event = Event::try_from(args).unwrap();
        let sway_name = "sway".to_string();
        let sway_owner = "FuelLabs".to_string();
        let sway_repo = Repo::new(sway_name, sway_owner);
        let payload = ClientPayload::new(sway_repo);
        let expected_event = Event::new(EventType::NewCommit, payload);

        assert_eq!(parsed_event, expected_event)
    }

    #[test]
    fn parse_event_from_param_input() {
        let repo_name = "sway".to_string();
        let repo_owner = "FuelLabs".to_string();
        let event_type = "new-commit".to_string();
        let args = Args {
            event: Some(event_type),
            repo_name: Some(repo_name.clone()),
            repo_owner: Some(repo_owner.clone()),
            json: None,
            path: None,
        };

        let parsed_event = Event::try_from(args).unwrap();
        let sway_repo = Repo::new(repo_name, repo_owner);
        let client_payload = ClientPayload::new(sway_repo);
        let expected_event = Event::new(EventType::NewCommit, client_payload);

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
        let plan = Plan::try_from_manifest(manifest).unwrap();

        let target_repos = plan
            .neighbors(current_repo)
            .unwrap()
            .map(|repo| repo.name())
            .collect::<Vec<_>>();
        let expected_target_repos = vec!["forc-wallet", "sway"];

        assert_eq!(target_repos, expected_target_repos)
    }
}

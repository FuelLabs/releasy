mod cmd;
mod handle;

use crate::cmd::Args;
use clap::Parser;
use handle::EventHandler;
use releasy_core::{event::Event, repo::Repo};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let current_repo_name = args.current_repo_name.clone();
    let current_repo_owner = args.current_repo_owner.clone();
    let received_event = Event::try_from(args)?;
    let current_repo = Repo::new(current_repo_name, current_repo_owner);

    received_event.handle(&current_repo)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cmd::Args;
    use releasy_core::{
        event::{ClientPayload, Event, EventDetails, EventType},
        repo::Repo,
    };

    #[test]
    fn parse_event_from_param_input() {
        let repo_name = "fuels-rs".to_string();
        let repo_owner = "FuelLabs".to_string();
        let event_type = "new-commit".to_string();

        let expected_commit_hash = "337d0eaa130dd18e9e347f83ab4fab76b3a6bd2a".to_string();
        let args = Args {
            event: Some(event_type),
            event_commit_hash: Some(expected_commit_hash.clone()),
            event_release_tag: None,
            event_repo_name: Some(repo_name.clone()),
            event_repo_owner: Some(repo_owner.clone()),
            current_repo_name: repo_name.clone(),
            current_repo_owner: repo_owner.clone(),
        };

        let parsed_event = Event::try_from(args).unwrap();
        let sway_repo = Repo::new(repo_name, repo_owner);
        let details = EventDetails::new(Some(expected_commit_hash), None);
        let client_payload = ClientPayload::new(sway_repo, details);
        let expected_event = Event::new(EventType::NewCommit, client_payload);

        assert_eq!(parsed_event, expected_event)
    }
}

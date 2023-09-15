mod cmd;
mod handle;

use crate::cmd::Args;
use clap::Parser;
use handle::EventHandler;
use releasy_core::event::Event;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let received_event = Event::try_from(args)?;
    received_event.handle()?;
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
            repo_name: Some(repo_name.clone()),
            repo_owner: Some(repo_owner.clone()),
        };

        let parsed_event = Event::try_from(args).unwrap();
        let sway_repo = Repo::new(repo_name, repo_owner);
        let details = EventDetails::new(Some(expected_commit_hash), None);
        let client_payload = ClientPayload::new(sway_repo, details);
        let expected_event = Event::new(EventType::NewCommit, client_payload);

        assert_eq!(parsed_event, expected_event)
    }
}

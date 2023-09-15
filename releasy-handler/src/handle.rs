use releasy_core::event::{Event, EventType};

pub trait EventHandler {
    fn handle(&self) -> anyhow::Result<()>;
}

impl EventHandler for Event {
    fn handle(&self) -> anyhow::Result<()> {
        match self.event_type() {
            EventType::NewCommit => handle_new_commit(self),
            EventType::NewRelease => handle_new_release(self),
        }
    }
}

fn handle_new_commit(event: &Event) -> anyhow::Result<()> {
    println!(
        "New commit event received from {}, commit hash: {:?}",
        event.client_payload().repo(),
        event.client_payload().details().commit_hash()
    );
    Ok(())
}

fn handle_new_release(event: &Event) -> anyhow::Result<()> {
    println!(
        "New release event received from {}, release_tag: {:?}",
        event.client_payload().repo(),
        event.client_payload().details().release_tag()
    );
    Ok(())
}

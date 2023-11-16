pub mod error;
pub mod event;
pub mod repo;

pub mod default {
    pub const DEFAULT_MANIFEST_FILE_NAME: &str = "repo-plan.toml";
    pub const DEFAULT_COMMIT_AUTHOR_EMAIL: &str = "releasy@fuel.sh";
    pub const DEFAULT_COMMIT_AUTHOR_NAME: &str = "releasy";
}

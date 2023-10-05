use std::{
    env::current_dir,
    path::Path,
    process::{Command, Stdio},
};

use releasy_core::{
    default::{DEFAULT_COMMIT_AUTHOR_EMAIL, DEFAULT_COMMIT_AUTHOR_NAME},
    event::{Event, EventType},
    repo::Repo,
};
use releasy_graph::{manifest::Manifest, plan::Plan};

pub trait EventHandler {
    fn handle(&self, plan: Manifest) -> anyhow::Result<()>;
}

impl EventHandler for Event {
    fn handle(&self, manifest: Manifest) -> anyhow::Result<()> {
        let current_repo = manifest.current_repo().clone();
        let plan = Plan::try_from_manifest(manifest)?;
        match self.event_type() {
            EventType::NewCommitToDependency => {
                handle_new_commit_to_dependency(self, &current_repo)
            }
            EventType::NewCommitToSelf => {
                let upstream_dependencies = plan
                    .upstream_repos(current_repo.clone())?
                    .cloned()
                    .collect::<Vec<_>>();
                handle_new_commit_to_self(self, upstream_dependencies, &current_repo)
            }
            EventType::NewRelease => handle_new_release(self),
        }
    }
}

/// Sets global git config to use releasy's dummy email and name for the commit author.
fn set_git_user() -> anyhow::Result<()> {
    // Set email.
    ReleasyHandlerCommand::new("git")
        .arg("config")
        .arg("--global")
        .arg("user.email")
        .arg(DEFAULT_COMMIT_AUTHOR_EMAIL)
        .execute()?;

    // Set name.
    ReleasyHandlerCommand::new("git")
        .arg("config")
        .arg("--global")
        .arg("user.name")
        .arg(DEFAULT_COMMIT_AUTHOR_NAME)
        .execute()
}

/// Rebase the current repository onto given branch.
fn rebase_repo(onto: &str, path: &Path) -> anyhow::Result<()> {
    ReleasyHandlerCommand::new("git")
        .arg("rebase")
        .arg("--onto")
        .arg(onto)
        .current_dir(path)
        .execute()
}

/// Get the default branch name from origin.
fn default_branch_name(path: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("remote")
        .arg("show")
        .arg("origin")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let name = stdout
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if line.starts_with("HEAD") {
                // We want the word after `HEAD branch:`, which is 2nd index.
                line.split(' ').nth(2).map(|word| word.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("cannot get default branch name"))?;

    Ok(name)
}

/// Checks if specified tracking branch is present in remote of the repo. If it is missing creates
/// a new branch from default branch.
///
/// After making sure that tracking branch is present, rebases is onto remote version of default
/// branch.
fn rebase_or_create_tracking_branch(
    tracking_branch_name: &str,
    default_branch: &str,
    repo_path: &Path,
) -> anyhow::Result<()> {
    // Checkout tracking branch.
    let checkout_status = ReleasyHandlerCommand::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(tracking_branch_name)
        .arg(format!("origin/{}", tracking_branch_name))
        .current_dir(repo_path)
        .execute();
    if checkout_status.is_err() {
        // To make sure we are creating the branch from default branch checkout default branch
        // first.
        ReleasyHandlerCommand::new("git")
            .arg("checkout")
            .arg(default_branch)
            .current_dir(repo_path)
            .execute()?;

        // Remote does not have the tracking branch yet. Create a new branch from default
        // branch.
        ReleasyHandlerCommand::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(tracking_branch_name)
            .current_dir(repo_path)
            .execute()?
    }

    // Pull remote changes.
    ReleasyHandlerCommand::new("git")
        .arg("pull")
        .arg("origin")
        .arg(tracking_branch_name)
        .current_dir(repo_path)
        .execute()?;

    // Rebase repo onto default branch of remote.
    rebase_repo(default_branch, repo_path)?;

    Ok(())
}

/// Handles the case when there is a new commit to an upstream repository.
///
/// For our needs, we want to make sure that our tracking branch (which contains patches in
/// `Cargo.toml`s that causes `master` version of upstream repos to be used, instead of the latest
/// released version) runs the CI again. To run the CI again new_commit handler, pushes a new commit
/// to the tracking branch.
///
/// By default we are expecting the tracking branch to be named as:
///
/// ```
/// upgrade/<source_repo_name>-master
/// ```
fn handle_new_commit_to_dependency(event: &Event, current_repo: &Repo) -> anyhow::Result<()> {
    println!(
        "New commit event received from {}, commit hash: {:?}",
        event.client_payload().repo(),
        event.client_payload().details().commit_hash()
    );

    let source_repo = event.client_payload().repo();
    let commit_hash = event
        .client_payload()
        .details()
        .commit_hash()
        .ok_or_else(|| anyhow::anyhow!("target commit hash missing"))?;
    let tracking_branch_name = format!("upgrade/{}-master", source_repo.name());

    with_repo(commit_hash, current_repo, |repo_path, default_branch| {
        rebase_or_create_tracking_branch(&tracking_branch_name, default_branch, repo_path)?;

        // Create an empty commit.
        let commit_message = format!(
            "re-run CI after {} commit merged to {}/{}",
            commit_hash,
            source_repo.owner(),
            source_repo.name()
        );
        ReleasyHandlerCommand::new("git")
            .arg("commit")
            .arg("--allow-empty")
            .arg("-m")
            .arg(format!("\"{}\"", commit_message))
            .current_dir(repo_path)
            .execute()?;

        // Push empty commit to remote.
        ReleasyHandlerCommand::new("git")
            .arg("push")
            .arg("origin")
            .arg("-f")
            .arg(&tracking_branch_name)
            .current_dir(repo_path)
            .execute()?;

        Ok(())
    })?;
    Ok(())
}

/// Handles the case when there is a new commit to the current repo.
///
/// All of the tracking branches should be rebased so that newest commit to master is taken into
/// account.
fn handle_new_commit_to_self(
    event: &Event,
    upstream_dependencies: Vec<Repo>,
    current_repo: &Repo,
) -> anyhow::Result<()> {
    let commit_hash = event
        .client_payload()
        .details()
        .commit_hash()
        .ok_or_else(|| anyhow::anyhow!("target commit hash missing"))?;

    println!(
        "New commit event received from this repo, commit hash: {:?}",
        commit_hash
    );

    with_repo(commit_hash, current_repo, |repo_path, default_branch| {
        for tracking_branch_name in upstream_dependencies
            .iter()
            .map(|repo| format!("upgrade/{}", repo.name()))
        {
            rebase_or_create_tracking_branch(&tracking_branch_name, default_branch, repo_path)?;
        }
        Ok(())
    })
}

fn handle_new_release(event: &Event) -> anyhow::Result<()> {
    println!(
        "New release event received from {}, release_tag: {:?}",
        event.client_payload().repo(),
        event.client_payload().details().release_tag()
    );
    println!("Not yet implemented!");
    Ok(())
}

/// Initializes a new temporary directory to fetch current repo into.
fn with_tmp_dir<F>(dir_name: &str, f: F) -> anyhow::Result<()>
where
    F: FnOnce(&Path) -> anyhow::Result<()>,
{
    // Clear existing temporary directory if it exists.
    let repo_dir = current_dir()?.join(".tmp").join(dir_name);
    if repo_dir.exists() {
        let _ = std::fs::remove_dir_all(&repo_dir);
    }

    // Create the tmp dir if it does not exists
    std::fs::create_dir_all(&repo_dir)?;

    // Call the user function.
    f(&repo_dir)?;

    // Clean up the temporary directory.
    let _ = std::fs::remove_dir_all(&repo_dir);
    Ok(())
}

/// Initializes a new temporary directory and clones the given repo into that directory.
/// Exact steps executed by this function can be listed as:
///
///  - git clone
///  - git remote set-url
///
/// Calls the user provided function with the cloned repo's absolute path.
fn with_repo<F>(tmp_dir_name: &str, repo: &Repo, f: F) -> anyhow::Result<()>
where
    F: FnOnce(&Path, &str) -> anyhow::Result<()>,
{
    set_git_user()?;
    with_tmp_dir(tmp_dir_name, |tmp_dir_path| {
        let absolute_path = tmp_dir_path.canonicalize()?;
        let repo_url = repo.github_url()?;

        // Clone the repo inside a tmp directory.
        ReleasyHandlerCommand::new("git")
            .arg("clone")
            .arg(&repo_url)
            .current_dir(&absolute_path)
            .execute()?;

        let repo_path = absolute_path.join(repo.name());

        // Set remote url to contain PAT.
        ReleasyHandlerCommand::new("git")
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg(&repo_url)
            .current_dir(&repo_path)
            .execute()?;

        // Get the default branch name from origin.
        let default_branch = default_branch_name(&repo_path)?;

        // Pull latest changes to default branch.
        ReleasyHandlerCommand::new("git")
            .arg("pull")
            .arg("origin")
            .arg(&default_branch)
            .current_dir(&repo_path)
            .execute()?;

        f(&repo_path, &default_branch)
    })
}

/// A wrapper around `std::process::Command` that provides easy to use error handling via
/// `execute()` and `output()` functions.
#[derive(Debug)]
struct ReleasyHandlerCommand {
    command: Command,
}

impl ReleasyHandlerCommand {
    /// Creates a new `CommandWrapper` with the specified command.
    fn new<S: AsRef<str>>(cmd: S) -> Self {
        Self {
            command: Command::new(cmd.as_ref()),
        }
    }

    /// Adds an argument to the command.
    fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self {
        self.command.arg(arg.as_ref());
        self
    }

    /// Executes the command and returns an `anyhow::Result<()>`.
    fn execute(&mut self) -> anyhow::Result<()> {
        let output = self.command.output()?;
        if output.status.success() {
            Ok(())
        } else {
            let error_message = format!(
                "Command {self:?} failed with exit code: {}",
                output.status.code().unwrap_or_default()
            );
            Err(anyhow::anyhow!(error_message))
        }
    }

    /// Sets the working directory for the command.
    fn current_dir<S: AsRef<std::path::Path>>(&mut self, dir: S) -> &mut Self {
        self.command.current_dir(dir);
        self
    }
}

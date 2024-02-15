//! Functions for interacting with GitHub via the `gh` CLI
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::OnceLock;

pub mod gh_cli;
pub mod gh_cli_fake;
pub mod util;

/// Get the GitHub CLI and initialize it with a default repository
/// If `fake` is true, a fake GitHub CLI is returned.
/// The fake GitHub CLI is used for testing and does not interact with GitHub
///
/// # Arguments
///
/// * `repo` - The default repository to use
/// * `fake` - If true, a fake GitHub CLI is returned
///
/// # Returns
///
/// [`Box<dyn GitHub>`](GitHub) - The GitHub CLI interface
///
/// # Example
///
/// ```
/// # use gh_workflow_parser::gh::init_github_cli;
/// let github_cli = init_github_cli("https://example.com/repo".to_string(), false);
/// ```
pub fn init_github_cli(repo: String, fake: bool) -> Box<dyn GitHub> {
    if fake {
        Box::new(gh_cli_fake::GitHubCliFake::new(repo))
    } else {
        Box::new(gh_cli::GitHubCli::new(repo))
    }
}

/// Trait describing the methods that the GitHub CLI should implement
pub trait GitHub {
    /// Get the summary of a run in a GitHub repository, if `repo` is `None` the default repository is used
    /// Returns the summary as a [String]
    fn run_summary(&self, repo: Option<&str>, run_id: &str) -> Result<String, Box<dyn Error>>;

    /// Get the log of a failed job in a GitHub repository, if `repo` is `None` the default repository is used
    /// Returns the log as a [String]
    fn failed_job_log(&self, repo: Option<&str>, job_id: &str) -> Result<String, Box<dyn Error>>;

    /// Create an issue in a GitHub repository, if `repo` is `None` the default repository is used
    fn create_issue(
        &self,
        repo: Option<&str>,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<(), Box<dyn Error>>;

    /// Get the bodies of open issues with a specific label in a GitHub repository, if `repo` is `None` the default repository is used
    /// Returns [`Vec<String>`](Vec) of issue bodies
    fn issue_bodies_open_with_label(
        &self,
        repo: Option<&str>,
        label: &str,
    ) -> Result<Vec<String>, Box<dyn Error>>;

    /// Get all labels in a GitHub repository, if `repo` is `None` the default repository is used
    /// Returns [`Vec<String>`](Vec) of GitHub labels
    fn all_labels(&self, repo: Option<&str>) -> Result<Vec<String>, Box<dyn Error>>;

    /// Create a label in a GitHub repository, if `repo` is `None` the default repository is used
    /// The color should be a 6 character hex code (e.g. "FF0000")
    /// if `force` is true and the label already exists, it will be overwritten
    fn create_label(
        &self,
        repo: Option<&str>,
        name: &str,
        color: &str,
        description: &str,
        force: bool,
    ) -> Result<(), Box<dyn Error>>;

    /// Get the default repository for the GitHub CLI
    fn default_repo(&self) -> &str;
}

include!(concat!(env!("OUT_DIR"), "/include_gh_cli.rs"));
pub static GITHUB_CLI: OnceLock<OsString> = OnceLock::new();
pub fn gh_cli() -> &'static OsStr {
    GITHUB_CLI.get_or_init(|| {
        let gh_cli_path = gh_cli_first_time_setup().unwrap();
        OsString::from(gh_cli_path)
    })
}

pub fn gh_cli_first_time_setup() -> Result<PathBuf, Box<dyn Error>> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("gh-workflow-parser-deps");

    if !path.exists() {
        std::fs::create_dir(&path)?;
    }

    let gh_cli_path = path.join("gh_cli");

    if !gh_cli_path.exists() {
        log::debug!("the gh_cli file at {gh_cli_path:?} doesn't exist. Creating it...");
        // first decompress the gh-cli binary blob
        let gh_cli_bytes = GH_CLI_BYTES;
        log::trace!("gh_cli_bytes size: {}", gh_cli_bytes.len());

        let decompressed_gh_cli = crate::util::bzip2_decompress(gh_cli_bytes)?;
        log::trace!("decompressed_gh_cli size: {}", decompressed_gh_cli.len());

        // Write the gh_cli file to the gh_cli_path
        std::fs::write(&gh_cli_path, decompressed_gh_cli)?;
        #[cfg(target_os = "linux")]
        crate::util::set_linux_file_permissions(&gh_cli_path, 0o755)?;
        log::debug!("gh_cli file written to {gh_cli_path:?}");
    } else {
        log::debug!(
            "the gh_cli file at {gh_cli_path:?} already exists. Skipping first time setup..."
        );
    }

    Ok(gh_cli_path)
}

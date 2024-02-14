//! Functions for interacting with GitHub via the `gh` CLI
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{error::Error, process::Command};

pub mod gh_cli;
pub mod gh_cli_fake;
pub mod util;

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

pub fn run_summary(repo: &str, run_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new(gh_cli())
        .arg("run")
        .arg(format!("--repo={repo}"))
        .arg("view")
        .arg(run_id)
        .output()?;

    assert!(
        output.status.success(),
        "Failed to get logs for repo={repo} run_id={run_id}. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn failed_job_log(repo: &str, job_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new(gh_cli())
        .arg("run")
        .arg("view")
        .arg("--repo")
        .arg(repo)
        .arg("--job")
        .arg(job_id)
        .arg("--log-failed")
        .output()?;

    assert!(
        output.status.success(),
        "Failed to get logs for job ID: {job_id}. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Create an issue in the GitHub repository
pub fn create_issue(
    repo: &str,
    title: &str,
    body: &str,
    labels: &[String],
) -> Result<(), Box<dyn Error>> {
    // First check if the labels exist on the repository
    let existing_labels = all_labels(repo)?;
    for label in labels {
        if !existing_labels.contains(label) {
            log::info!("Label {label} does not exist in the repository. Creating it...");
            create_label(repo, label, "FF0000", "", false)?;
        } else {
            log::debug!(
                "Label {label} already exists in the repository, continuing without creating it."
            )
        }
    }
    // format the labels into a single string separated by commas
    let labels = labels.join(",");
    let mut command = Command::new(gh_cli());
    command
        .arg("issue")
        .arg("create")
        .arg("--repo")
        .arg(repo)
        .arg("--title")
        .arg(title)
        .arg("--body")
        .arg(body)
        .arg("--label")
        .arg(labels);

    log::debug!("Debug view of command struct: {command:?}");
    // Run the command
    let output = command.output()?;

    assert!(
        output.status.success(),
        "Failed to create issue. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

/// Get the bodies of open issues with a specific label
pub fn issue_bodies_open_with_label(
    repo: &str,
    label: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new(gh_cli())
        .arg("issue")
        .arg("list")
        .arg("--repo")
        .arg(repo)
        .arg("--label")
        .arg(label)
        .arg("--json")
        .arg("body")
        .output()
        .expect("Failed to list issues");

    assert!(
        output.status.success(),
        "Failed to list issues. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    let output = String::from_utf8_lossy(&output.stdout);

    /// Helper struct to deserialize a JSON array of github issue bodies
    #[derive(Serialize, Deserialize)]
    struct GhIssueBody {
        pub body: String,
    }

    let parsed: Vec<GhIssueBody> = serde_json::from_str(&output)?;
    Ok(parsed.into_iter().map(|item| item.body).collect())
}

/// Get all labels in a GitHub repository
pub fn all_labels(repo: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new(gh_cli())
        .arg("--repo")
        .arg(repo)
        .arg("label")
        .arg("list")
        .arg("--json")
        .arg("name")
        .output()?;

    assert!(
        output.status.success(),
        "Failed to list labels. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    // Parse the received JSON vector of objects with a `name` field
    let output = String::from_utf8_lossy(&output.stdout);
    #[derive(Serialize, Deserialize)]
    struct Label {
        name: String,
    }
    let parsed: Vec<Label> = serde_json::from_str(&output)?;
    Ok(parsed.into_iter().map(|label| label.name).collect())
}

/// Create a label in the GitHub repository
/// The color should be a 6 character hex code
/// if `force` is true and the label already exists, it will be overwritten
pub fn create_label(
    repo: &str,
    name: &str,
    color: &str,
    description: &str,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new(gh_cli());
    cmd.arg("label")
        .arg("create")
        .arg(name)
        .arg("--repo")
        .arg(repo)
        .arg("--color")
        .arg(color)
        .arg("--description")
        .arg(description);

    if force {
        cmd.arg("--force");
    }

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "Failed to create label. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[ignore = "This test requires a GitHub repository"]
    fn test_issue_body_display() {
        let issue_bodies = issue_bodies_open_with_label(
            "https://github.com/luftkode/distro-template",
            "CI scheduled build",
        )
        .unwrap();
        for body in issue_bodies {
            println!("{body}");
        }
    }

    #[test]
    fn test_parse_json_body() {
        /// Helper struct to deserialize a JSON array of github issue bodies
        #[derive(Serialize, Deserialize)]
        struct GhIssueBody {
            pub body: String,
        }

        let data = r#"
    [
      {
        "body": "**Run ID**: 7858139663 [LINK TO RUN](github.com/luftkode/distro-template/actions/runs/7858139663)\\n\\n**1 job failed:**\\n- **`Test template xilinx`**\\n\\n### `Test template xilinx` (ID 21442749267)\\n**Step failed:** `📦 Build yocto image`\\n\\\\n**Log:** github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749267\\n\\\\n*Best effort error summary*:\\n``\\nERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')\\nERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616\\nERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'\\n\\n2024-02-11 00:09:04 - ERROR    - Command \"/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index\" failed with error 1\\n```"
      },
      {
        "body": "Build failed on xilinx. Check the logs at https://github.com/luftkode/distro-template/actions/runs/7858139663 for more details."
      },
      {
        "body": "Build failed on xilinx. Check the logs at https://github.com/luftkode/distro-template/actions/runs/7850874958 for more details."
      }
    ]
    "#;

        // Parse the JSON string to Vec<Item>
        let parsed: Vec<GhIssueBody> = serde_json::from_str(data).unwrap();

        // Extract the bodies into a Vec<String>
        let bodies: Vec<String> = parsed.into_iter().map(|item| item.body).collect();

        // Assert that the bodies are as expected
        assert_eq!(bodies.len(), 3);
        assert!(bodies[0].contains("**Run ID**: 7858139663 [LINK TO RUN]("));
        assert_eq!(
            bodies[1],
            "Build failed on xilinx. Check the logs at https://github.com/luftkode/distro-template/actions/runs/7858139663 for more details.");
        assert_eq!(
            bodies[2],
            "Build failed on xilinx. Check the logs at https://github.com/luftkode/distro-template/actions/runs/7850874958 for more details.");
    }
}

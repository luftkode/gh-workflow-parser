use std::{error::Error, process::Command};

use serde::{Deserialize, Serialize};

use crate::gh::gh_cli;

pub fn repo_url_to_job_url(repo_url: &str, run_id: &str, job_id: &str) -> String {
    let run_url = repo_url_to_run_url(repo_url, run_id);
    run_url_to_job_url(&run_url, job_id)
}

pub fn repo_url_to_run_url(repo_url: &str, run_id: &str) -> String {
    format!("{repo_url}/actions/runs/{run_id}")
}

pub fn run_url_to_job_url(run_url: &str, job_id: &str) -> String {
    format!("{run_url}/job/{job_id}")
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

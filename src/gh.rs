use serde::{Deserialize, Serialize};
use std::{error::Error, process::Command};

pub const GITHUB_CLI: &str = "gh";

pub fn run_summary(repo: &str, run_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new(GITHUB_CLI)
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

pub fn failed_job_log(job_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new(GITHUB_CLI)
        .arg("run")
        .arg("view")
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
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let output = Command::new(GITHUB_CLI)
        .arg("issue")
        .arg("create")
        .arg("--repo")
        .arg(repo)
        .arg("--title")
        .arg(title)
        .arg("--body")
        .arg(body)
        .arg("--label")
        .arg(label)
        .output()?;

    assert!(
        output.status.success(),
        "Failed to create issue. Failure: {stderr}",
        stderr = String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct GhIssueBody {
    pub body: String,
}

/// Get the bodies of open issues with a specific label
pub fn issue_bodies_open_with_label(
    repo: &str,
    label: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new(GITHUB_CLI)
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
    let parsed: Vec<GhIssueBody> = serde_json::from_str(&output)?;
    Ok(parsed.into_iter().map(|item| item.body).collect())
}

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

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

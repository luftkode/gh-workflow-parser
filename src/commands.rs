use std::error::Error;

/// The maximum Levenshtein distance for issues to be considered similar.
///
/// Determined in tests at the bottom of this file.
const LEVENSHTEIN_THRESHOLD: usize = 100;

use crate::{
    err_msg_parse,
    errlog::ErrorLog,
    gh,
    issue::{FailedJob, Issue},
    util,
};
use clap::{Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a GitHub issue from a failed workflow run
    CreateIssueFromRun {
        /// The GitHub workflow run ID
        #[arg(short = 'r', long)]
        run_id: String,
        /// The GitHub issue label
        #[arg(short, long)]
        label: String,
        /// The kind of workflow (e.g. Yocto)
        #[arg(short, long)]
        kind: WorkflowKind,
        /// Don't create the issue if a similar issue already exists
        #[arg(short, long, default_value_t = true)]
        no_duplicate: bool,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkflowKind {
    Yocto,
    Other,
}

fn parse_to_gh_issue(
    errlogs: Vec<ErrorLog>,
    repo: &str,
    run_id: String,
    label: String,
    kind: WorkflowKind,
) -> Result<Issue, Box<dyn Error>> {
    let failed_jobs: Vec<FailedJob> = errlogs
        .iter()
        .map(|errlog| {
            let err_summary = err_msg_parse::parse_error_message(errlog.no_prefix_log(), kind)?;
            Ok(FailedJob::new(
                errlog.failed_job().to_owned(),
                errlog.job_id().to_owned(),
                gh::util::repo_url_to_job_url(repo, &run_id, errlog.job_id()),
                errlog.failed_step().to_owned(),
                err_summary,
            ))
        })
        .collect::<Result<Vec<FailedJob>, Box<dyn Error>>>()?;

    let issue = Issue::new(
        run_id.to_string(),
        gh::util::repo_url_to_run_url(repo, &run_id),
        failed_jobs,
        label,
    );
    Ok(issue)
}

pub fn create_issue_from_failed_run(
    repo: String,
    run_id: &str,
    labels: &str,
    kind: WorkflowKind,
    dry_run: bool,
    no_duplicate: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Run the GitHub CLI to get the workflow run
    let run_summary = gh::run_summary(&repo, run_id)?;
    log::info!("Run summary: {run_summary}");

    let failed_jobs = util::take_lines_with_failed_jobs(run_summary);
    if failed_jobs.is_empty() {
        log::error!("No failed jobs found! Exiting...");
        std::process::exit(1);
    }

    log::info!("Failed jobs: {:?}", failed_jobs);
    let failed_job_ids = util::id_from_job_lines(&failed_jobs);
    let failed_job_logs: Vec<String> = failed_job_ids
        .iter()
        .map(|id| gh::failed_job_log(&repo, id))
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;
    log::info!("Got {} failed job log(s)", failed_job_logs.len());

    let failed_logs = failed_job_logs
        .iter()
        .zip(failed_job_ids.iter())
        .map(|(log, id)| ErrorLog::new(id.to_string(), log.to_string()))
        .collect::<Result<Vec<ErrorLog>, Box<dyn Error>>>()?;

    let gh_issue = parse_to_gh_issue(
        failed_logs,
        &repo,
        run_id.to_owned(),
        labels.to_string(),
        kind,
    )?;
    if no_duplicate {
        let similar_issues = gh::issue_bodies_open_with_label(&repo, labels)?;
        // Check how similar the issues are
        let smallest_distance = similar_issues
            .iter()
            .map(|issue| distance::levenshtein(issue, &gh_issue.body()))
            .min()
            .unwrap_or(usize::MAX);
        log::info!("Smallest levenshtein distance to similar issue: {smallest_distance} (Similarity threshold={LEVENSHTEIN_THRESHOLD})");
        match smallest_distance {
            0 => {
                log::warn!("An issue with the exact same body already exists. Exiting...");
                std::process::exit(0);
            },
            _ if smallest_distance < LEVENSHTEIN_THRESHOLD => {
                log::warn!("An issue with a similar body already exists. Exiting...");
                std::process::exit(0);
            },
            _ => log::info!("No similar issue found. Continuing..."),
        }
    }
    if dry_run {
        println!("####################################");
        println!("DRY RUN MODE! The following issue would be created:");
        println!("==== ISSUE TITLE ==== \n{}", gh_issue.title());
        println!("==== ISSUE LABEL(S) ==== \n{}", gh_issue.labels().join(","));
        println!("==== START OF ISSUE BODY ==== \n{}", gh_issue.body());
        println!("==== END OF ISSUE BODY ====");
    } else {
        log::debug!("Creating an issue in the remote repository with the following characteristics:\n==== ISSUE TITLE ==== \n{title}\n==== ISSUE LABEL(S) ==== \n{labels}\n==== START OF ISSUE BODY ==== \n{body}\n==== END OF ISSUE BODY ====", title = gh_issue.title(), labels = gh_issue.labels().join(","), body = gh_issue.body());

        gh::create_issue(&repo, gh_issue.title(), &gh_issue.body(), gh_issue.labels())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    const EXAMPLE_ISSUE_BODY_0: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    const EXAMPLE_ISSUE_BODY_1: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7858139663/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    #[test]
    fn test_issue_body_distance() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let distance = distance::levenshtein(&issue_0, &issue_1);
        assert_eq!(distance, 0);
    }

    /// Identical except for very similar run and job IDs
    #[test]
    fn test_issue_body_distance_edit_minimal_diff() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "7858139660";
        let new_job0_id = "21442749260";
        let new_job1_id = "21442749200";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);

        let distance = distance::levenshtein(&issue_0, &issue_1);
        assert_eq!(distance, 11);
    }

    /// Identical except for as different run and job IDs as possible
    #[test]
    fn test_issue_body_distance_edit_largest_similar() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "0000000000";
        let new_job0_id = "00000000000";
        let new_job1_id = "33333333333";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);

        let distance = distance::levenshtein(&issue_0, &issue_1);
        assert_eq!(distance, 74);
    }

    /// Smallest difference in job and run IDs but different in other ways and should be treated as different.
    #[test]
    fn test_issue_body_distance_edit_minimal_but_different() {
        let issue_0 = EXAMPLE_ISSUE_BODY_0.to_string();
        let issue_1 = EXAMPLE_ISSUE_BODY_1.to_string();
        let new_run_id = "7858139660";
        let new_job0_id = "21442749260";
        let new_job1_id = "21442749200";

        let issue_1 = issue_1.replace("7858139663", new_run_id);
        let issue_1 = issue_1.replace("21442749267", new_job0_id);
        let issue_1 = issue_1.replace("21442749166", new_job1_id);
        let issue_1 = issue_1.replace(
            "Yocto error: ERROR: No recipes available for: ...",
            "ERROR: fetcher failure. malformed url. Attempting to fetch from ${SOURCE_MIRROR_URL}",
        );

        let distance = distance::levenshtein(&issue_0, &issue_1);
        assert_eq!(distance, 153);
    }
}

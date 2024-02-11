use std::error::Error;

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
    /// Create a GitHub issuem from a failed workflow run
    #[command(arg_required_else_help = true)]
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
    let failed_jobs = errlogs
        .iter()
        .map(|errlog| {
            let err_summary = err_msg_parse::parse_error_message(errlog.no_prefix_log(), kind)?;
            Ok(FailedJob::new(
                errlog.failed_job().to_owned(),
                errlog.job_id().to_owned(),
                gh::repo_url_to_job_url(repo, &run_id, errlog.job_id()),
                errlog.failed_step().to_owned(),
                err_summary,
            ))
        })
        .collect::<Result<Vec<FailedJob>, Box<dyn Error>>>()?;

    let issue = Issue::new(
        run_id.to_string(),
        gh::repo_url_to_run_url(repo, &run_id),
        failed_jobs,
        label,
    );
    Ok(issue)
}

pub fn create_issue_from_failed_run(
    repo: String,
    run_id: &str,
    label: &str,
    kind: WorkflowKind,
    dry_run: bool,
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
        .map(|id| gh::failed_job_log(id))
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
        label.to_string(),
        kind,
    )?;
    if dry_run {
        println!("####################################");
        println!("DRY RUN MODE! The following issue would be created:");
        println!("==== ISSUE TITLE ==== \n{}", gh_issue.title());
        println!("==== ISSUE LABEL ==== \n{}", gh_issue.label());
        println!("==== ISSUE BODY ==== \n{}", gh_issue.body());
    } else {
        gh::create_issue(&repo, gh_issue.title(), &gh_issue.body(), gh_issue.label())?;
    }
    Ok(())
}

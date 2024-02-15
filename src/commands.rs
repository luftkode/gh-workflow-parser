//! The `commands` module contains the subcommands for the `gh-workflow-parser` CLI.

/// The maximum Levenshtein distance for issues to be considered similar.
///
/// Determined in tests at the bottom of this file.
pub const LEVENSHTEIN_THRESHOLD: usize = 100;

use clap::{Subcommand, ValueEnum};

pub mod create_issue_from_run;

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

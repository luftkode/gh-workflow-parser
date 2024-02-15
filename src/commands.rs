//! The `commands` module contains the subcommands for the `gh-workflow-parser` CLI.

/// The maximum Levenshtein distance for issues to be considered similar.
///
/// Determined in tests at the bottom of this file.
pub const LEVENSHTEIN_THRESHOLD: usize = 100;

use std::path::PathBuf;

use clap::*;
use clap::{Subcommand, ValueEnum};
use strum::{Display, EnumString};

pub mod create_issue_from_run;
pub mod locate_failure_log;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a GitHub issue from a failed workflow run
    CreateIssueFromRun {
        /// The GitHub repository to parse
        #[arg(long, value_hint = ValueHint::Url)]
        repo: String,
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

    /// Locate the specific failure log in a failed build/test/other
    LocateFailureLog {
        /// The kind of workflow (e.g. Yocto)
        #[arg(short, long)]
        kind: BuildKind,
        /// Log file to search for the failure log (e.g. log.txt or read from stdin)
        /// File to operate on (if not provided, reads from stdin)
        #[arg(short = 'f', long, value_hint = ValueHint::FilePath)]
        input_file: Option<PathBuf>,
    },
}

/// The kind of workflow (e.g. Yocto)
#[derive(ValueEnum, Display, Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkflowKind {
    Yocto,
    Other,
}

/// The kind of build (e.g. Yocto)
///
/// Could be extended to Python, Pytest, Vivado Synethesis, etc.
#[derive(ValueEnum, Display, EnumString, Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuildKind {
    Yocto,
    Other,
}

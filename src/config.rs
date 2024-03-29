//! CLI configuration and initialization
use crate::gh::gh_cli;
use crate::util::check_gh_cli_version;

use super::commands::Command;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::*;
use std::error::Error;
use which::which;

/// The minimum version of the GitHub CLI required for `gh-workflow-parser` to run as expected.
pub const GH_CLI_MIN_VERSION: semver::Version = semver::Version::new(2, 43, 1);

#[derive(Parser, Debug)]
#[command(version, about = "Parse GitHub CI workflows", author, styles = config_styles())]
pub struct Config {
    #[command(subcommand)]
    command: Option<Command>,
    /// Debug flag to run through a scenario without making changes
    #[arg(long, default_value_t = false, global = true)]
    dry_run: bool,
    /// Fake the GitHub CLI for testing
    #[arg(long, default_value_t = false, global = true)]
    fake_github_cli: bool,
    /// Verbosity level (0-4)
    #[arg(short, long, global = true, default_value_t = 2)]
    verbosity: u8,
    /// Generate completion scripts for the specified shell
    #[arg(long, global = true, value_hint = ValueHint::Other, name = "SHELL")]
    completions: Option<clap_complete::Shell>,
}

impl Config {
    /// Get the dry run flag
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    /// Get the fake GitHub CLI flag
    pub fn fake_github_cli(&self) -> bool {
        self.fake_github_cli
    }

    /// Get the subcommand
    pub fn subcmd(&self) -> &Command {
        if self.command.is_none() {
            log::error!("Subcommand required! use `--help` for more information");
            std::process::exit(1);
        }
        self.command.as_ref().expect("Subcommand not set")
    }

    /// Get the verbosity level
    pub fn verbosity(&self) -> u8 {
        self.verbosity
    }

    pub fn generate_completion_script(&self) -> bool {
        match self.completions {
            Some(shell) => {
                generate_completion_script(shell);
                true
            },
            None => false,
        }
    }
}

/// Initialize the CLI configuration
pub fn init() -> Result<Config, Box<dyn Error>> {
    let config = Config::parse();
    use stderrlog::LogLevelNum;
    let log_level = match config.verbosity() {
        0 => LogLevelNum::Error,
        1 => LogLevelNum::Warn,
        2 => LogLevelNum::Info,
        3 => LogLevelNum::Debug,
        4 => LogLevelNum::Trace,
        _ => {
            eprintln!("Invalid verbosity level: {}", config.verbosity());
            eprintln!("Using highest verbosity level: Trace");
            LogLevelNum::Trace
        },
    };
    stderrlog::new().verbosity(log_level).quiet(false).init()?;
    if config.dry_run() {
        log::warn!("Running in dry-run mode. No writes/changes will be made");
    }

    // Check that the GitHub CLI is installed
    if let Err(e) = which(gh_cli()) {
        log::error!("GitHub CLI not found: {e}");
        std::process::exit(1);
    }
    check_gh_cli_version(GH_CLI_MIN_VERSION)?;

    Ok(config)
}

// Styles for the help messages in the CLI
fn config_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Blue.on_default())
}

/// Generate completion scripts for the specified shell
fn generate_completion_script(shell: clap_complete::Shell) {
    log::info!("Generating completion script for {shell:?}");
    clap_complete::generate(
        shell,
        &mut <Config as clap::CommandFactory>::command(),
        "gh-workflow-parser",
        &mut std::io::stdout(),
    );
}

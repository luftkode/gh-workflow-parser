use super::commands::Command;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::*;
use std::error::Error;
use which::which;

#[derive(Parser, Debug)]
#[command(version, about = "Parse GitHub CI workflows", author, styles = config_styles())]
pub struct Config {
    #[command(subcommand)]
    command: Command,
    /// The GitHub repository to parse
    #[arg(long)]
    pub repo: String,
    /// Debug flag to run through a scenario without making changes
    #[arg(long, default_value_t = false, global = true)]
    dry_run: bool,
}

impl Config {
    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn subcmd(&self) -> &Command {
        &self.command
    }
}

pub fn init() -> Result<Config, Box<dyn Error>> {
    stderrlog::new().verbosity(2).quiet(false).init()?;
    // Check that the GitHub CLI is installed
    if which(crate::gh::GITHUB_CLI).is_err() {
        log::error!("GitHub CLI not found. Please install it from https://cli.github.com/");
        std::process::exit(1);
    }
    Ok(Config::parse())
}

// Styles for the help messages in the CLI
fn config_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Blue.on_default())
}

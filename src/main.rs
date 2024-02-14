use std::error::Error;

use gh_workflow_parser::{commands, config, gh};

fn main() -> Result<(), Box<dyn Error>> {
    let config = config::init()?;

    // Generate completion script and exit
    if config.generate_completion_script() {
        return Ok(());
    }

    log::info!("Parsing GitHub repository: {}", config.repo());

    let github_cli = gh::gh_cli::GitHubCli::new(config.repo().to_owned());

    use commands::Command::*;
    match config.subcmd() {
        CreateIssueFromRun {
            run_id,
            label,
            kind,
            no_duplicate,
        } => {
            log::info!("Creating issue from failed run: {run_id}");
            commands::create_issue_from_failed_run(
                github_cli,
                run_id,
                label,
                *kind,
                config.dry_run(),
                *no_duplicate,
            )?;
        },
    }

    Ok(())
}

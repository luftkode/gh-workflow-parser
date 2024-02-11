use std::error::Error;

mod commands;
pub mod config;
pub mod err_msg_parse;
pub mod errlog;
pub mod gh;
pub mod issue;
pub mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let config = config::init()?;

    // Generate completion script and exit
    if config.generate_completion_script() {
        return Ok(());
    }

    log::info!("Parsing GitHub repository: {}", config.repo());

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
                config.repo().to_owned(),
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

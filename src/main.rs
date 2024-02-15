use std::{error::Error, process::ExitCode};

use gh_workflow_parser::{commands, config, gh::init_github_cli};

fn main() -> ExitCode {
    match run() {
        // If the error is a broken pipe, we can just ignore it
        // First we need to downcast the error to an io::Error
        Err(err)
            if err
                .downcast_ref::<std::io::Error>()
                .map_or(false, |e| e.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            ExitCode::SUCCESS
        },
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        },
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = config::init()?;
    // Generate completion script and exit
    if config.generate_completion_script() {
        return Ok(());
    }

    use commands::Command::*;
    match config.subcmd() {
        CreateIssueFromRun {
            repo,
            run_id,
            label,
            kind,
            no_duplicate,
        } => {
            log::info!("Targeting GitHub repository: {repo}, run: {run_id}, label: {label}, kind: {kind}, no_duplicate: {no_duplicate}");
            let github_cli = init_github_cli(repo.to_owned(), config.fake_github_cli());
            commands::create_issue_from_run::create_issue_from_run(
                github_cli,
                run_id,
                label,
                *kind,
                config.dry_run(),
                *no_duplicate,
            )?;
        },
        LocateFailureLog { kind, input_file } => {
            log::info!("Locating failure log for kind: {kind}");
            commands::locate_failure_log::locate_failure_log(*kind, input_file.as_ref())?;
        },
    }

    Ok(())
}

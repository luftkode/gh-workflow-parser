pub use assert_cmd::prelude::*; // Add methods on commands
pub use assert_cmd::Command;
pub use assert_fs::fixture::ChildPath;
/// System test for the GitHub workflow parser.
use std::error::Error;
// Get the methods for the Commands struct
pub use assert_fs::prelude::*;
pub use assert_fs::TempDir;
#[allow(unused_imports)]
pub use predicates::prelude::*; // Used for writing assertions // Create temporary directories
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

#[test]
fn create_issue_from_failed_run_yocto() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("gh-workflow-parser")?;

    cmd.arg("--repo=https://github.com/luftkode/distro-template")
        .arg("create-issue-from-run")
        .arg("--run-id=7865472546")
        .arg("--label=\"CI scheduled build\"")
        .arg("--kind=yocto")
        .arg("--dry-run");

    let std::process::Output {
        status,
        stdout: _,
        stderr,
    } = cmd.output()?;

    let stderr = String::from_utf8(stderr)?;
    let stderr_contains_fn =
        predicate::str::contains("Logfile from error summary does not exist at");
    assert!(status.success());
    assert!(stderr_contains_fn.eval(&stderr));

    Ok(())
}
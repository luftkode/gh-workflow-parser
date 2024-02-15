//! Utility functions for parsing and working with GitHub CLI output and other utility functions.
use std::{error::Error, path::PathBuf, process::Command};

use crate::gh::gh_cli;
use bzip2::Compression;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::prelude::*;

/// Parse a path from a string
/// # Example
/// ```
/// # use gh_workflow_parser::util::first_path_from_str;
/// use std::path::PathBuf;
///
/// let haystack = r#"multi line
/// test string with/path/file.txt is
/// valid"#;
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(path, PathBuf::from("with/path/file.txt"));
///
/// // No path in string is an error
/// let haystack = "Random string with no path";
/// assert!(first_path_from_str(haystack).is_err());
///
/// // Path with no leading '/' and no file extension is OK
/// let haystack = "foo app/3-_2/t/3 bar";
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(path, PathBuf::from("app/3-_2/t/3"));
///
/// // More realistic example
/// let haystack = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
/// let path = first_path_from_str(haystack).unwrap();
/// assert_eq!(
///   path,
///  PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
/// );
/// ```
/// # Errors
/// This function returns an error if no valid path is found in the string
pub fn first_path_from_str(s: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"[a-zA-Z0-9-_.\/]+\/[a-zA-Z0-9-_.]+").unwrap());

    let path_str = RE.find(s).ok_or("No path found in string")?.as_str();
    Ok(PathBuf::from(path_str))
}

/// Take the lines with failed jobs from the output of `gh run view`
pub fn take_lines_with_failed_jobs(output: String) -> Vec<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"X.*ID [0-9]*\)").unwrap());

    RE.find_iter(&output)
        .map(|m| m.as_str().to_owned())
        .collect()
}

/// Extract the job IDs from the lines with job information
pub fn id_from_job_lines(lines: &[String]) -> Vec<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"ID (?<JOB_ID>[0-9]*)").unwrap());

    lines
        .iter()
        .map(|line| {
            RE.captures(line)
                .unwrap_or_else(|| {
                    panic!("Expected a line with a Job ID, but no ID found in line: {line}")
                })
                .name("JOB_ID")
                .expect("Expected a Job ID")
                .as_str()
                .to_owned()
        })
        .collect()
}

/// Parse an absolute path from a string. This assumes that the the first '/' found in the string is the start
/// of the path.
/// # Example
/// ```
/// # use gh_workflow_parser::util::first_abs_path_from_str;
/// use std::path::PathBuf;
///
/// let test_str = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
/// let path = first_abs_path_from_str(test_str).unwrap();
/// assert_eq!(
///    path,
///   PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
/// );
/// ```
///
/// # Errors
/// This function returns an error if no '/' is found in the string or
/// if the path is not a valid path.
pub fn first_abs_path_from_str(s: &str) -> Result<PathBuf, Box<dyn Error>> {
    let start = s.find('/').unwrap_or_else(|| {
        panic!("Expected a path in the string, but no '/' found in string: {s}")
    });
    let path = PathBuf::from(&s[start..]);
    Ok(path)
}

/// Retrieve the GitHub CLI version from the GitHub CLI binary and check that it meets version requirements.
pub fn check_gh_cli_version(min_required: semver::Version) -> Result<(), Box<dyn Error>> {
    let gh_cli_version = Command::new(gh_cli()).arg("--version").output()?;
    let version_str = String::from_utf8(gh_cli_version.stdout)?;
    check_gh_cli_version_str(min_required, &version_str)
}

/// Check that the GitHub CLI version meets version requirements from the string output of `gh --version`
///
/// # Example
/// ```
/// # use gh_workflow_parser::util::check_gh_cli_version_str;
/// let version_str = "gh version 2.43.1 (2024-01-31)";
/// let min_required = semver::Version::new(2, 43, 1);
/// let version = check_gh_cli_version_str(min_required, version_str);
/// assert!(version.is_ok());
/// ```
///
/// # Errors
/// Returns an error if the version string cannot be parsed as a semver version or
/// if the version is less than the minimum required version.
pub fn check_gh_cli_version_str(
    min_required: semver::Version,
    version_str: &str,
) -> Result<(), Box<dyn Error>> {
    static GH_CLI_VER_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"gh version (?P<version>[0-9]+\.[0-9]+\.[0-9]+)").unwrap());

    let version = GH_CLI_VER_RE
        .captures(version_str)
        .unwrap()
        .name("version")
        .unwrap()
        .as_str();

    let version = semver::Version::parse(version)?;
    if version < min_required {
        return Err(format!("GitHub CLI version {version} is not supported. Please install version {min_required} or higher")
        .into());
    }
    Ok(())
}

/// Set the file permissions for a file on Linux
#[cfg(target_os = "linux")]
pub fn set_linux_file_permissions(file: &std::path::Path, mode: u32) -> Result<(), Box<dyn Error>> {
    let metadata = std::fs::metadata(file).unwrap();
    let mut perms = metadata.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, mode);
    std::fs::set_permissions(file, perms).unwrap();
    Ok(())
}

pub fn bzip2_decompress(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut d = bzip2::bufread::BzDecoder::new(input);
    let mut out = Vec::new();
    d.read_to_end(&mut out)?;
    Ok(out)
}

pub fn bzip2_compress(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut e = bzip2::bufread::BzEncoder::new(input, Compression::new(9));
    let mut out = Vec::new();
    e.read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GH_CLI_MIN_VERSION;
    use pretty_assertions::assert_eq;
    use temp_dir::TempDir;

    // Output from `gh run --repo=github.com/luftkode/distro-template view 7858139663`
    const TEST_OUTPUT_VIEW_RUN: &str = r#"
    X master Use template and build image · 7858139663
    Triggered via schedule about 10 hours ago

    JOBS
    ✓ enable-ssh-agent in 5s (ID 21442747661)
    ✓ Test template raspberry in 19m20s (ID 21442749166)
    X Test template xilinx in 5m41s (ID 21442749267)
      ✓ Set up job
      ✓ Log in to the Container registry
      ✓ Cleanup build folder before start
      ✓ Run actions/checkout@v4
      ✓ Setup Rust and Just
      ✓ 🗻 Make a templated project
      ✓ ⚙️ Run new project setup steps
      ✓ ⚒️ Build docker image
      X 📦 Build yocto image
      - 📩 Deploy image artifacts
      ✓ Docker down
      ✓ Cleanup build folder after done
      ✓ Create issue on failure
      ✓ Post Run actions/checkout@v4
      ✓ Post Log in to the Container registry
      ✓ Complete job

    ANNOTATIONS
    X Process completed with exit code 2.
    Test template xilinx: .github#3839


    To see what failed, try: gh run view 7858139663 --log-failed
    View this run on GitHub: https://github.com/luftkode/distro-template/actions/runs/7858139663
"#;

    #[test]
    fn test_take_lines_with_failed_jobs() {
        let failed_jobs = take_lines_with_failed_jobs(TEST_OUTPUT_VIEW_RUN.to_string());
        assert_eq!(failed_jobs.len(), 1, "Failed jobs: {:?}", failed_jobs);
        assert_eq!(
            failed_jobs[0],
            "X Test template xilinx in 5m41s (ID 21442749267)"
        );
    }

    #[test]
    fn test_id_from_job_lines() {
        let job_lines = vec![
            "✓ Test template raspberry in 19m20s (ID 21442749166)".to_string(),
            "X Test template xilinx in 5m41s (ID 21442749267)".to_string(),
            "X Test template other in 5m1s (ID 01449267)".to_string(),
        ];
        let ids = id_from_job_lines(&job_lines);
        assert_eq!(ids.len(), 3, "Job IDs: {:?}", ids);
        assert_eq!(ids[0], "21442749166");
        assert_eq!(ids[1], "21442749267");
        assert_eq!(ids[2], "01449267");
    }

    #[test]
    fn test_absolute_path_from_str() {
        let test_str = r#" ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616"#;
        let path = first_abs_path_from_str(test_str).unwrap();
        assert_eq!(
            path,
            PathBuf::from("/app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616")
        );
    }

    const GH_CLI_VERSION_OK_STR: &str = r#"gh version 2.43.1 (2024-01-31)
https://github.com/cli/cli/releases/tag/v2.43.1"#;
    const GH_CLI_VERSION_BAD_STR: &str = r#"gh version 2.4.0 (2021-11-21)
https://github.com/cli/cli/releases/tag/v2.4.0"#;

    #[test]
    fn test_check_gh_cli_version_is_ok() {
        let version = check_gh_cli_version_str(GH_CLI_MIN_VERSION, GH_CLI_VERSION_OK_STR);
        assert!(version.is_ok());
    }

    #[test]
    fn test_check_gh_cli_version_bad() {
        let version = check_gh_cli_version_str(GH_CLI_MIN_VERSION, GH_CLI_VERSION_BAD_STR);
        assert!(version.is_err());
    }

    const GH_CLI_PATH: &str = "gh_cli/gh";

    #[test]
    pub fn test_compress_gh_cli_bz2() {
        /// Max upload size for crates.io is 10 MiB
        const MAX_CRATES_IO_UPLOAD_SIZE: usize = 1024 * 1024 * 10;
        let gh_cli_bytes = std::fs::read(GH_CLI_PATH).unwrap();
        let compressed = bzip2_compress(&gh_cli_bytes).unwrap();
        assert!(compressed.len() < gh_cli_bytes.len());
        assert!(compressed.len() < MAX_CRATES_IO_UPLOAD_SIZE); // Compressed size should be less than half the original size
    }

    #[test]
    pub fn test_decompress_gh_cli_bz2() {
        let gh_cli_bytes = std::fs::read(GH_CLI_PATH).unwrap();
        let compressed = bzip2_compress(&gh_cli_bytes).unwrap();
        let decompressed = bzip2_decompress(&compressed).unwrap();
        assert_eq!(gh_cli_bytes, decompressed);
    }

    #[test]
    pub fn test_compress_decompress_is_executable() {
        let gh_cli_bytes = std::fs::read(GH_CLI_PATH).unwrap();
        let compressed = bzip2_compress(&gh_cli_bytes).unwrap();
        let decompressed = bzip2_decompress(&compressed).unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("gh_cli");
        std::fs::write(&file, decompressed).unwrap();
        if cfg!(target_os = "linux") {
            set_linux_file_permissions(&file, 0o755).unwrap();
        }
        let output = std::process::Command::new(&file)
            .arg("--version")
            .output()
            .unwrap();
        assert!(output.status.success());
        println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    }
}

use std::{error::Error, path::PathBuf};

use regex::Regex;

/// Take the lines with failed jobs from the output of `gh run view`
pub fn take_lines_with_failed_jobs(output: String) -> Vec<String> {
    let re = Regex::new(r"X.*ID [0-9]*\)").unwrap();

    re.find_iter(&output)
        .map(|m| m.as_str().to_owned())
        .collect()
}

/// Extract the job IDs from the lines with job information
pub fn id_from_job_lines(lines: &[String]) -> Vec<String> {
    let re = Regex::new(r"ID (?<JOB_ID>[0-9]*)").unwrap();

    lines
        .iter()
        .map(|line| {
            re.captures(line)
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
}

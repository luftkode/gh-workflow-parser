use crate::{err_msg_parse::LOGFILE_MAX_LEN, util::first_abs_path_from_str};
use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YoctoError {
    summary: String,
    kind: YoctoFailureKind,
    logfile: Option<YoctoFailureLog>,
}

impl YoctoError {
    pub fn new(summary: String, kind: YoctoFailureKind, logfile: Option<YoctoFailureLog>) -> Self {
        YoctoError {
            summary,
            kind,
            logfile,
        }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
    pub fn kind(&self) -> YoctoFailureKind {
        self.kind
    }
    pub fn logfile(&self) -> Option<&YoctoFailureLog> {
        self.logfile.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YoctoFailureLog {
    pub name: String,
    pub contents: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum YoctoFailureKind {
    /// The 6 standard tasks in Yocto https://docs.yoctoproject.org/ref-manual/tasks.html
    DoBuild,
    DoCompile,
    DoCompilePtestBase,
    DoConfigure,
    DoConfigurePtestBase,
    DoDeploy,
    /// Other tasks
    DoFetch,
    /// If it's a type of failure we're not familiar with or parsing fails, default to this
    #[default]
    Misc,
}

impl Display for YoctoFailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            YoctoFailureKind::DoBuild => "do_build",
            YoctoFailureKind::DoCompile => "do_compile",
            YoctoFailureKind::DoCompilePtestBase => "do_compile_ptest_base",
            YoctoFailureKind::DoConfigure => "do_configure",
            YoctoFailureKind::DoConfigurePtestBase => "do_configure_ptest_base",
            YoctoFailureKind::DoDeploy => "do_deploy",
            YoctoFailureKind::DoFetch => "do_fetch",
            YoctoFailureKind::Misc => "misc",
        };
        write!(f, "{kind}")
    }
}

impl YoctoFailureKind {
    const TASK_NAMES: [&'static str; 7] = [
        "do_build",
        "do_compile",
        "do_compile_ptest_base",
        "do_configure",
        "do_configure_ptest_base",
        "do_deploy",
        "do_fetch",
    ];

    pub fn from_task(task: &str) -> Self {
        match task {
            "do_build" => YoctoFailureKind::DoBuild,
            "do_compile" => YoctoFailureKind::DoCompile,
            "do_compile_ptest_base" => YoctoFailureKind::DoCompilePtestBase,
            "do_configure" => YoctoFailureKind::DoConfigure,
            "do_configure_ptest_base" => YoctoFailureKind::DoConfigurePtestBase,
            "do_deploy" => YoctoFailureKind::DoDeploy,
            "do_fetch" => YoctoFailureKind::DoFetch,
            _ => YoctoFailureKind::Misc,
        }
    }
    /// Takes in a yocto logfile filename such as `log.do_fetch.21616` and attempts to determine the type
    /// of yocto task the the logfile is associated with.
    ///
    ///
    pub fn parse_from_logfilename(fname: &str) -> Result<Self, Box<dyn Error>> {
        for name in Self::TASK_NAMES {
            if fname.contains(name) {
                return Ok(Self::from_task(name));
            }
        }
        Err(format!("Could not determine task from input: {fname}").into())
    }
}

/// Find the `--- Error summary ---` section in the log and return the rest of the log until `bitbake -c build <string> failed with error 1`
pub fn parse_yocto_error(log: &str) -> Result<YoctoError, Box<dyn Error>> {
    const YOCTO_ERROR_SUMMARY_SIGNATURE: &str = "--- Error summary ---";
    let error_summary = log
        .split(YOCTO_ERROR_SUMMARY_SIGNATURE)
        .collect::<Vec<&str>>()
        .pop()
        .ok_or("No error summary found")?;
    // the prefix is like `Test template xilinx	📦 Build yocto image	2024-02-11T00:09:04.7119455Z`
    // Get the prefix `Test template xilinx	📦 Build yocto image` (remove trailing whitespace)

    // Trim the leading/trailing newlines and whitespace
    let error_summary = error_summary.trim().to_string();
    log::debug!("Yocto error before trimming: \n{}", error_summary);
    // Trim trailing Just recipes like
    // error: Recipe `in-container-build-ci-image` failed on line 31 with exit code 2
    // error: Recipe `run-in-docker` failed with exit code 2 .. etc.
    let error_summary = error_summary
        .lines()
        .rev()
        .skip(1) // Skip the last line, which is the github specific step failure line
        .skip_while(|line| line.starts_with("error: Recipe"))
        .collect::<Vec<&str>>() // Then reverse the iterator to get the original order
        .iter()
        .rev()
        .fold(String::with_capacity(error_summary.len()), |acc, line| {
            acc + line + "\n"
        });

    log::info!("Yocto error: \n{}", error_summary);

    // Find the kind of yocto failure in the string e.g. this would be `do_fetch`
    // ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616

    // Find the line with the `Logfile of failure stored in` and get the path
    let log_file_line = error_summary
        .lines()
        .find(|line| line.contains("Logfile of failure stored in"))
        .ok_or("No log file line found")?;
    let path = first_abs_path_from_str(log_file_line)?;
    let fname = path.file_stem().unwrap().to_str().unwrap();
    let yocto_failure_kind = match YoctoFailureKind::parse_from_logfilename(fname) {
        Ok(kind) => kind,
        Err(e) => {
            log::error!("{e}");
            log::warn!("Could not determine yocto failure kind, continuing with default kind");
            YoctoFailureKind::default()
        },
    };

    let logfile = if path.exists() {
        let contents = std::fs::read_to_string(&path)?;
        if contents.len() > LOGFILE_MAX_LEN {
            log::warn!("Logfile of yocto failure exceeds maximum length of {LOGFILE_MAX_LEN}. It will not be added to the issue body.");
            None
        } else {
            Some(YoctoFailureLog {
                name: fname.to_owned(),
                contents,
            })
        }
    } else {
        log::error!("Logfile from error summary does not exist at: {path:?}");
        log::warn!("Continuing without attempting to attach logfile to issue");
        None
    };

    let yocto_error = YoctoError {
        summary: error_summary,
        kind: yocto_failure_kind,
        logfile,
    };

    Ok(yocto_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const ERROR_SUMMARY_TEST_STR: &str = r#"ERROR: sqlite3-native-3_3.43.2-r0 do_fetch: Bitbake Fetcher Error: MalformedUrl('${SOURCE_MIRROR_URL}')
    ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616
    ERROR: Task (virtual:native:/app/yocto/build/../poky/meta/recipes-support/sqlite/sqlite3_3.43.2.bb:do_fetch) failed with exit code '1'

    2024-02-11 00:09:04 - ERROR    - Command "/app/yocto/poky/bitbake/bin/bitbake -c build test-template-ci-xilinx-image package-index" failed with error 1"#;

    #[test]
    fn test_determine_yocto_error_kind() {
        let task = "do_build";
        assert_eq!(YoctoFailureKind::from_task(task), YoctoFailureKind::DoBuild);
    }

    #[test]
    fn test_yocto_error_from_error_message() {
        // find the part of the string after
        let log_file_line = ERROR_SUMMARY_TEST_STR
            .lines()
            .find(|line| line.contains("Logfile of failure stored in"))
            .ok_or("No log file line found")
            .unwrap();
        dbg!(log_file_line);
        // Get the path stored after `Logfile of failure stored in: `
        let logfile_path = log_file_line
            .split("Logfile of failure stored in: ")
            .collect::<Vec<&str>>()
            .pop()
            .ok_or("No log file found");

        let path = std::path::PathBuf::from(logfile_path.unwrap());
        let fname = path.file_stem().unwrap().to_str().unwrap();

        let yocto_failure = YoctoFailureKind::parse_from_logfilename(fname).unwrap();
        assert_eq!(yocto_failure, YoctoFailureKind::DoFetch);
    }
}

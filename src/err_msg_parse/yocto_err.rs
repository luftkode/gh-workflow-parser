use crate::{err_msg_parse::LOGFILE_MAX_LEN, util::first_abs_path_from_str};
use std::error::Error;

use self::util::YoctoFailureKind;

pub mod util;

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

/// Find the `--- Error summary ---` section in the log and return the rest of the log until `bitbake -c build <string> failed with error 1`
pub fn parse_yocto_error(log: &str) -> Result<YoctoError, Box<dyn Error>> {
    let error_summary = util::yocto_error_summary(log)?;
    log::debug!(
        "Yocto error before trimming just recipe failures: \n{}",
        error_summary
    );

    let error_summary = util::trim_trailing_just_recipes(&error_summary)?;
    log::info!("Yocto error: \n{}", error_summary);

    // Find the kind of yocto failure in the string e.g. this would be `do_fetch`
    // ERROR: Logfile of failure stored in: /app/yocto/build/tmp/work/x86_64-linux/sqlite3-native/3.43.2/temp/log.do_fetch.21616

    // Find the line with the `Logfile of failure stored in` and get the path
    let log_file_line = util::find_yocto_failure_log_str(&error_summary)?;
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

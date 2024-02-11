#![allow(dead_code)]
use once_cell::sync::Lazy;
use std::error::Error;

use regex::Regex;

#[derive(Debug)]
pub struct ErrorLog {
    job_id: String,
    no_prefix_log: String,
    // Failed job/step can be retrieved from a failed job log by looking at the prefix
    prefix: ErrLogPrefix,
}

impl ErrorLog {
    pub fn new(job_id: String, raw_log: String) -> Result<Self, Box<dyn Error>> {
        static PREFIX_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?P<failed_job>.*)\t(?P<failed_step>.*)\t(?P<timestamp>[0-9]{4}-[0-9]{2}-[0-9]{2})T[0-9]{2}:[0-9]{2}:[0-9]{2}.*Z ")
                .expect("Failed to compile regex")
        });
        let first_line = raw_log
            .lines()
            .next()
            .expect("Expected raw log to have at least one line");
        let caps = PREFIX_RE.captures(first_line).map_or_else(
            || {
                panic!(
                    "Expected the first line of the raw log to match the prefix regex: {first_line}"
                )
            },
            |m| m,
        );
        let failed_job = caps.name("failed_job").unwrap().as_str().to_string();
        let failed_step = caps.name("failed_step").unwrap().as_str().to_string();
        let timestamp = caps.name("timestamp").unwrap().as_str().to_string();
        let prefix = ErrLogPrefix::new(failed_job, failed_step, timestamp);

        // Now trim the prefix from the log
        let no_prefix_log =
            raw_log
                .lines()
                .fold(String::with_capacity(raw_log.len() / 2), |mut acc, line| {
                    let mut s = PREFIX_RE.replace(line, "").to_string();
                    s.push('\n');
                    acc.push_str(&s);
                    acc
                });
        Ok(Self {
            job_id,
            no_prefix_log,
            prefix,
        })
    }

    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    pub fn no_prefix_log(&self) -> &str {
        &self.no_prefix_log
    }

    pub fn failed_job(&self) -> &str {
        self.prefix.failed_job()
    }

    pub fn failed_step(&self) -> &str {
        self.prefix.failed_step()
    }

    pub fn timestamp(&self) -> &str {
        self.prefix.timestamp()
    }
}

#[derive(Debug)]
pub struct ErrLogPrefix {
    failed_job: String,
    failed_step: String,
    // yyyy-mm-dd
    timestamp: String,
}

impl ErrLogPrefix {
    pub fn new(failed_job: String, failed_step: String, timestamp: String) -> Self {
        Self {
            failed_job,
            failed_step,
            timestamp,
        }
    }

    pub fn failed_job(&self) -> &str {
        &self.failed_job
    }

    pub fn failed_step(&self) -> &str {
        &self.failed_step
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TEST_LOG_STRING: &str = r#"Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5797561Z ##[group]Run just --yes build-ci-image
Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5799911Z [36;1mjust --yes build-ci-image[0m
Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5843410Z shell: /usr/bin/bash -e {0}
"#;

    const TEST_LOG_STRING_NO_PREFIX: &str = r#"##[group]Run just --yes build-ci-image
[36;1mjust --yes build-ci-image[0m
shell: /usr/bin/bash -e {0}
"#;

    #[test]
    fn test_errlog_prefix() {
        let err_log = ErrorLog::new("123".to_string(), TEST_LOG_STRING.to_owned()).unwrap();
        assert_eq!(err_log.failed_job(), "Test template xilinx");
        assert_eq!(err_log.failed_step(), "📦 Build yocto image");
        assert_eq!(err_log.timestamp(), "2024-02-10");

        assert_eq!(err_log.no_prefix_log(), TEST_LOG_STRING_NO_PREFIX);
    }
}

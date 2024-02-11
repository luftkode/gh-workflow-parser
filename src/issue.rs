//! Contains the Issue struct and its associated methods.
//!
//! The Issue struct is used to represent a GitHub issue that will be created
//! in a repository. It contains a title, label, and body. The body is a
//! collection of FailedJob structs, which contain information about the failed
//! jobs in a GitHub Actions workflow run.
use std::fmt::{self, Display, Formatter, Write};

#[derive(Debug)]
pub struct Issue {
    title: String,
    label: String,
    body: IssueBody,
}

impl Issue {
    pub fn new(
        run_id: String,
        run_link: String,
        failed_jobs: Vec<FailedJob>,
        label: String,
    ) -> Self {
        Self {
            title: "Scheduled run failed".to_string(),
            label,
            body: IssueBody::new(run_id, run_link, failed_jobs),
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn body(&self) -> String {
        self.body.to_string()
    }
}

#[derive(Debug)]
pub struct IssueBody {
    run_id: String,
    run_link: String,
    failed_jobs: Vec<FailedJob>,
}

impl IssueBody {
    pub fn new(run_id: String, run_link: String, failed_jobs: Vec<FailedJob>) -> Self {
        Self {
            run_id,
            run_link,
            failed_jobs,
        }
    }
}

impl Display for IssueBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "**Run ID**: {id} [LINK TO RUN]({run_url})

**{failed_jobs_list_title}**
{failed_jobs_name_list}",
            id = self.run_id,
            run_url = self.run_link,
            failed_jobs_list_title = format_args!(
                "{cnt} {job} failed:",
                cnt = self.failed_jobs.len(),
                job = if self.failed_jobs.len() == 1 {
                    "job"
                } else {
                    "jobs"
                }
            ),
            failed_jobs_name_list =
                self.failed_jobs
                    .iter()
                    .fold(String::new(), |mut s_out, job| {
                        let _ = writeln!(s_out, "- **`{}`**", job.name);
                        s_out
                    })
        )?;
        for job in &self.failed_jobs {
            write!(f, "{job}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FailedJob {
    name: String,
    id: String,
    url: String,
    failed_step: String,
    error_message: String,
}

impl FailedJob {
    pub fn new(
        name: String,
        id: String,
        url: String,
        failed_step: String,
        error_message: String,
    ) -> Self {
        Self {
            name,
            id,
            url,
            failed_step,
            error_message,
        }
    }
}

impl Display for FailedJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
### `{name}` (ID {id})
**Step failed:** `{failed_step}`
\\
**Log:** {url}
\\
*Best effort error summary*:
```
{error_message}```",
            name = self.name,
            id = self.id,
            failed_step = self.failed_step,
            url = self.url,
            error_message = self.error_message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const EXAMPLE_ISSUE_BODY: &str = r#"**Run ID**: 7858139663 [LINK TO RUN]( https://github.com/luftkode/distro-template/actions/runs/7850874958)

**2 jobs failed:**
- **`Test template xilinx`**
- **`Test template raspberry`**

### `Test template xilinx` (ID 21442749267)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```
### `Test template raspberry` (ID 21442749166)
**Step failed:** `📦 Build yocto image`
\
**Log:** https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166
\
*Best effort error summary*:
```
Yocto error: ERROR: No recipes available for: ...
```"#;

    #[test]
    fn test_issue_new() {
        let run_id = "7858139663".to_string();
        let run_link =
            "https://github.com/luftkode/distro-template/actions/runs/7850874958".to_string();
        let failed_jobs = vec![
            FailedJob::new(
                "Test template xilinx".to_string(),
                "21442749267".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267".to_string(),
                "📦 Build yocto image".to_string(),
                "Yocto error: ERROR: No recipes available for: ...
".to_string(),
            ),
            FailedJob::new(
                "Test template raspberry".to_string(),
                "21442749166".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166".to_string(),
                "📦 Build yocto image".to_string(),
                "Yocto error: ERROR: No recipes available for: ...
".to_string(),
            ),
        ];
        let label = "bug".to_string();
        let issue = Issue::new(run_id, run_link, failed_jobs, label);
        assert_eq!(issue.title, "Scheduled run failed");
        assert_eq!(issue.label, "bug");
        assert_eq!(issue.body.failed_jobs.len(), 2);
        assert_eq!(issue.body.failed_jobs[0].id, "21442749267");
    }

    #[test]
    fn test_issue_body_display() {
        let run_id = "7858139663".to_string();
        let run_link =
            " https://github.com/luftkode/distro-template/actions/runs/7850874958".to_string();
        let failed_jobs = vec![
            FailedJob::new(
                "Test template xilinx".to_string(),
                "21442749267".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749267".to_string(),
                "📦 Build yocto image".to_string(),
                "Yocto error: ERROR: No recipes available for: ...
".to_string(),
            ),
            FailedJob::new(
                "Test template raspberry".to_string(),
                "21442749166".to_string(),
                "https://github.com/luftkode/distro-template/actions/runs/7850874958/job/21442749166".to_string(),
                "📦 Build yocto image".to_string(),
                "Yocto error: ERROR: No recipes available for: ...
".to_string(),
            ),
            ];

        let issue_body = IssueBody::new(run_id, run_link, failed_jobs);
        assert_eq!(issue_body.to_string(), EXAMPLE_ISSUE_BODY);
    }
}

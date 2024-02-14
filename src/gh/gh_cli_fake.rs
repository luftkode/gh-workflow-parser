use super::GitHub;

#[derive(Debug, Default, Clone)]
pub struct GitHubCliFake {
    repo: String,
}

impl GitHubCliFake {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }
}

impl GitHub for GitHubCliFake {
    fn run_summary(
        &self,
        repo: Option<&str>,
        run_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        Ok(format!(
            "Fake run summary for repo={target_repo} and run_id={run_id}"
        ))
    }

    fn failed_job_log(
        &self,
        repo: Option<&str>,
        job_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        Ok(format!(
            "Fake failed job log for repo={target_repo} and job_id={job_id}"
        ))
    }

    fn create_issue(
        &self,
        repo: Option<&str>,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        log::info!(
            "Fake create_issue for repo={target_repo}, title={title}, body={body}, labels={labels:?}"
        );
        Ok(())
    }

    fn issue_bodies_open_with_label(
        &self,
        repo: Option<&str>,
        label: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        Ok(vec![format!(
            "Fake issue body for repo={target_repo} and label={label}"
        )])
    }

    fn all_labels(&self, repo: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        log::info!("Fake all_labels for repo={target_repo}");
        Ok(vec!["fake-label".to_string()])
    }

    fn create_label(
        &self,
        repo: Option<&str>,
        name: &str,
        color: &str,
        description: &str,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        log::info!(
            "Fake create_label for repo={target_repo}, name={name}, color={color}, description={description}, force={force}"
        );
        Ok(())
    }

    fn default_repo(&self) -> &str {
        &self.repo
    }
}

use super::GitHub;

#[derive(Debug, Default, Clone)]
pub struct GitHubCli {
    repo: String,
}

impl GitHubCli {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }
}

impl GitHub for GitHubCli {
    fn run_summary(
        &self,
        repo: Option<&str>,
        run_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        super::run_summary(target_repo, run_id)
    }

    fn failed_job_log(
        &self,
        repo: Option<&str>,
        job_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        super::failed_job_log(target_repo, job_id)
    }

    fn create_issue(
        &self,
        repo: Option<&str>,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        super::create_issue(target_repo, title, body, labels)
    }

    fn issue_bodies_open_with_label(
        &self,
        repo: Option<&str>,
        label: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        super::issue_bodies_open_with_label(target_repo, label)
    }

    fn all_labels(&self, repo: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        super::all_labels(target_repo)
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
        super::create_label(target_repo, name, color, description, force)
    }

    fn default_repo(&self) -> &str {
        &self.repo
    }
}

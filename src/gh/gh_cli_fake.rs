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
        log::info!("Fake run summary for repo={target_repo} and run_id={run_id}");

        // Return a fake run summary from an actual run output
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
        Ok(TEST_OUTPUT_VIEW_RUN.to_string())
    }

    fn failed_job_log(
        &self,
        repo: Option<&str>,
        job_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let target_repo = repo.unwrap_or(&self.repo);
        log::info!("Fake failed job log for repo={target_repo} and job_id={job_id}");
        // Return a fake log from an actual run output
        const TEST_LOG_STRING: &str = r#"Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5797561Z ##[group]Run just --yes build-ci-image
Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5799911Z [36;1mjust --yes build-ci-image[0m
Test template xilinx	📦 Build yocto image	2024-02-10T00:03:45.5843410Z shell: /usr/bin/bash -e {0}
"#;
        Ok(TEST_LOG_STRING.to_string())
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

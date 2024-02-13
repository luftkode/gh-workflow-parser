pub fn repo_url_to_job_url(repo_url: &str, run_id: &str, job_id: &str) -> String {
    let run_url = repo_url_to_run_url(repo_url, run_id);
    run_url_to_job_url(&run_url, job_id)
}

pub fn repo_url_to_run_url(repo_url: &str, run_id: &str) -> String {
    format!("{repo_url}/actions/runs/{run_id}")
}

pub fn run_url_to_job_url(run_url: &str, job_id: &str) -> String {
    format!("{run_url}/job/{job_id}")
}

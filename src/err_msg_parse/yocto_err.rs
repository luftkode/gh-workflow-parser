use std::error::Error;

/// Find the `--- Error summary ---` section in the log and return the rest of the log until `bitbake -c build <string> failed with error 1`
pub fn parse_yocto_error(log: &str) -> Result<String, Box<dyn Error>> {
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

    Ok(error_summary)
}

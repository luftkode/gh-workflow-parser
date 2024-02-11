use crate::commands::WorkflowKind;
use std::error::Error;

mod yocto_err;

pub fn parse_error_message(
    err_msg: &str,
    workflow: WorkflowKind,
) -> Result<String, Box<dyn Error>> {
    let err_msg = match workflow {
        WorkflowKind::Yocto => yocto_err::parse_yocto_error(err_msg).unwrap_or_else(|e| {
            log::warn!("Failed to parse Yocto error: {}", e);
            err_msg.to_string()
        }),
        WorkflowKind::Other => err_msg.to_string(),
    };
    Ok(err_msg)
}

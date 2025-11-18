use crate::core::refactor::types::ValidationResult;
use anyhow::Context;
use std::process::Command;

pub fn validate_with_cargo_check() -> anyhow::Result<ValidationResult> {
    let output = Command::new("cargo")
        .arg("check")
        .output()
        .context("Failed to execute cargo check")?;

    let success = output.status.success();
    let errors = if success {
        Vec::new()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        stderr.lines().map(|s| s.to_string()).collect()
    };

    Ok(ValidationResult {
        cargo_check_passed: success,
        errors,
    })
}

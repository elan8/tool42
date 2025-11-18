use super::cache;
use super::read::MAX_LINES;
use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct PaginatedResult {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub lines_returned: usize,
    pub exit_code: i32,
}

/// Internal helper function that executes cargo command in a specific working directory
fn execute_cargo_internal(
    args: Vec<String>,
    working_dir: PathBuf,
) -> anyhow::Result<(Vec<u8>, Vec<u8>, i32)> {
    let output = Command::new("cargo")
        .args(&args)
        .current_dir(&working_dir)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute cargo command in directory: {}",
                working_dir.display()
            )
        })?;

    let exit_code = output.status.code().unwrap_or(1);
    Ok((output.stdout, output.stderr, exit_code))
}

/// Execute cargo command with pagination support (for MCP mode)
pub fn execute_mcp_paginated(
    args: Vec<String>,
    working_dir: PathBuf,
    from: Option<usize>,
    to: Option<usize>,
) -> anyhow::Result<PaginatedResult> {
    let cache_key = cache::generate_cache_key(&working_dir, &args);

    // Check cache first
    if let Some((cached_lines, exit_code)) = cache::get_cached_output(&cache_key, &working_dir) {
        let total_lines = cached_lines.len();
        let (start_idx, end_idx) = determine_range(from, to, total_lines)?;

        // Ensure we don't exceed MAX_LINES
        let max_end_idx = start_idx + MAX_LINES;
        let actual_end_idx = std::cmp::min(max_end_idx, end_idx);
        let actual_end_idx = std::cmp::min(actual_end_idx, total_lines);

        let lines: Vec<String> = cached_lines[start_idx..actual_end_idx]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let lines_returned = lines.len();

        return Ok(PaginatedResult {
            lines,
            total_lines,
            lines_returned,
            exit_code,
        });
    }

    // Cache miss - execute command
    let (stdout, stderr, exit_code) = execute_cargo_internal(args.clone(), working_dir.clone())?;

    // Combine stdout and stderr into a single string
    let mut output = String::from_utf8_lossy(&stdout).to_string();
    if !stderr.is_empty() {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str("--- stderr ---\n");
        output.push_str(&String::from_utf8_lossy(&stderr));
    }

    // Store in cache
    cache::store_output(&cache_key, output.clone(), exit_code, &working_dir);

    // Split into lines and paginate
    let all_lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    let total_lines = all_lines.len();

    let (start_idx, end_idx) = determine_range(from, to, total_lines)?;

    // Ensure we don't exceed MAX_LINES
    let max_end_idx = start_idx + MAX_LINES;
    let actual_end_idx = std::cmp::min(max_end_idx, end_idx);
    let actual_end_idx = std::cmp::min(actual_end_idx, total_lines);

    let lines: Vec<String> = all_lines[start_idx..actual_end_idx]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let lines_returned = lines.len();

    Ok(PaginatedResult {
        lines,
        total_lines,
        lines_returned,
        exit_code,
    })
}

fn determine_range(
    from: Option<usize>,
    to: Option<usize>,
    total_lines: usize,
) -> anyhow::Result<(usize, usize)> {
    // Convert 1-based line numbers to 0-based indices
    let start_idx = match from {
        Some(f) => {
            if f == 0 {
                anyhow::bail!("Line numbers are 1-based; line 0 is invalid");
            }
            if f > total_lines {
                anyhow::bail!(
                    "Start line {} exceeds output length of {} lines",
                    f,
                    total_lines
                );
            }
            f - 1 // Convert to 0-based
        }
        None => 0,
    };

    let end_idx_1based = match to {
        Some(t) => {
            if t == 0 {
                anyhow::bail!("Line numbers are 1-based; line 0 is invalid");
            }
            // Clamp to total_lines if exceeds output length
            let clamped_t = std::cmp::min(t, total_lines);
            if let Some(f) = from {
                if clamped_t < f {
                    anyhow::bail!("End line {} is before start line {}", t, f);
                }
            }
            clamped_t // 1-based, inclusive, clamped to output length
        }
        None => {
            // Default: return first MAX_LINES lines
            std::cmp::min(MAX_LINES, total_lines)
        }
    };

    // Convert 1-based inclusive end to 0-based exclusive end
    let end_idx = end_idx_1based;

    // Validate range
    if start_idx >= end_idx {
        anyhow::bail!("Invalid range: start line must be less than end line");
    }

    Ok((start_idx, end_idx))
}

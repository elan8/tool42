use anyhow::Context;
use std::fs;
use std::path::PathBuf;

pub const MAX_LINES: usize = 500;

#[derive(Debug)]
pub struct ReadResult {
    pub content: Vec<String>,
    pub total_lines: usize,
    pub lines_returned: usize,
}

pub fn read_file(
    path: PathBuf,
    from: Option<usize>,
    to: Option<usize>,
) -> anyhow::Result<ReadResult> {
    // Read file content
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Determine the range to read
    let (start_idx, end_idx_1based_inclusive) = determine_range(from, to, total_lines)?;

    // Convert 1-based inclusive end to 0-based exclusive end
    let end_idx_exclusive = end_idx_1based_inclusive;

    // Ensure we don't exceed MAX_LINES
    let max_end_idx = start_idx + MAX_LINES;
    let actual_end_idx = std::cmp::min(max_end_idx, end_idx_exclusive);

    // Also ensure we don't exceed total_lines
    let actual_end_idx = std::cmp::min(actual_end_idx, total_lines);

    // Extract lines
    let content: Vec<String> = lines[start_idx..actual_end_idx]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let lines_returned = content.len();

    Ok(ReadResult {
        content,
        total_lines,
        lines_returned,
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
                    "Start line {} exceeds file length of {} lines",
                    f,
                    total_lines
                );
            }
            f - 1 // Convert to 0-based
        }
        None => 0,
    };

    let end_idx = match to {
        Some(t) => {
            if t == 0 {
                anyhow::bail!("Line numbers are 1-based; line 0 is invalid");
            }
            // Clamp to total_lines if exceeds file length
            let clamped_t = std::cmp::min(t, total_lines);
            if let Some(f) = from {
                if clamped_t < f {
                    anyhow::bail!("End line {} is before start line {}", t, f);
                }
            }
            clamped_t // 1-based, inclusive, clamped to file length
        }
        None => total_lines,
    };

    // Validate range
    if start_idx >= end_idx {
        anyhow::bail!("Invalid range: start line must be less than end line");
    }

    Ok((start_idx, end_idx))
}

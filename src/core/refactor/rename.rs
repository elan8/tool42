use crate::core::project;
use crate::core::refactor::types::{RefactorResult, RefactorStatus};
use crate::core::refactor::utils::{find_symbol_in_file, walk_and_find_symbol};
use crate::core::refactor::validation::validate_with_cargo_check;
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn rename_symbol(
    symbol: String,
    new_name: String,
    path: Option<PathBuf>,
    working_directory: Option<PathBuf>,
    preview: bool,
    apply: bool,
) -> anyhow::Result<RefactorResult> {
    let search_path = if let Some(p) = path {
        p
    } else if let Some(working_dir) = working_directory {
        // When path is None, use working_directory to find workspace root
        project::find_workspace_root(&working_dir).with_context(|| {
            format!(
                "Could not find project root (Cargo.toml) starting from working directory: {}",
                working_dir.display()
            )
        })?
    } else {
        PathBuf::from(".")
    };

    if !search_path.exists() {
        anyhow::bail!("Path does not exist: {}", search_path.display());
    }

    // Find all occurrences of the symbol
    let mut changes = Vec::new();
    let mut files_to_modify = HashMap::new();

    if search_path.is_file() {
        find_symbol_in_file(&search_path, &symbol, &mut changes, &mut files_to_modify)?;
    } else {
        walk_and_find_symbol(&search_path, &symbol, &mut changes, &mut files_to_modify)?;
    }

    if changes.is_empty() {
        anyhow::bail!("Symbol '{}' not found in the specified path", symbol);
    }

    // If preview mode, return preview with updated new names
    if preview && !apply {
        let mut preview_changes = changes;
        for change in &mut preview_changes {
            change.new = new_name.clone();
        }
        return Ok(RefactorResult {
            operation: "rename".to_string(),
            status: RefactorStatus::Preview,
            changes: preview_changes,
            validation: None,
            backup_files: Vec::new(),
        });
    }

    // Apply changes
    if !apply {
        anyhow::bail!("Must specify --apply to apply changes");
    }

    // Create backups
    let mut backup_files = Vec::new();
    for file_path in files_to_modify.keys() {
        let backup_path = format!("{}.backup", file_path.display());
        fs::copy(file_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
        backup_files.push(backup_path);
    }

    // Apply changes - replace old name with new name
    let mut applied_changes = Vec::new();
    for (file_path, replacements) in &mut files_to_modify {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let mut new_content = content.clone();
        let mut offset = 0i64;

        // Update replacements with new name
        for (_, _, new_text) in replacements.iter_mut() {
            *new_text = new_name.clone();
        }

        // Sort replacements by position (descending) to avoid offset issues
        let mut sorted_replacements: Vec<_> = replacements.iter().collect();
        sorted_replacements.sort_by(|a, b| b.0.cmp(&a.0));

        for (byte_pos, old_text, new_text) in sorted_replacements {
            let start = *byte_pos;
            let end = start + old_text.len();

            // Adjust position based on previous replacements
            let adjusted_start = (start as i64 + offset) as usize;
            let adjusted_end = (end as i64 + offset) as usize;

            if adjusted_end <= new_content.len() {
                new_content.replace_range(adjusted_start..adjusted_end, new_text);
                offset += (new_text.len() as i64) - (old_text.len() as i64);
            }
        }

        fs::write(file_path, &new_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        // Update changes with applied status and new name
        for mut change in changes.iter().cloned() {
            if change.file == file_path.display().to_string() {
                change.new = new_name.clone();
                applied_changes.push(change);
            }
        }
    }

    // Validate with cargo check
    let validation = validate_with_cargo_check()?;

    // If validation failed, rollback
    if !validation.cargo_check_passed {
        // Rollback changes
        for backup_file in &backup_files {
            if let Some(original_file) = backup_file.strip_suffix(".backup") {
                if let Err(e) = fs::copy(backup_file, original_file) {
                    eprintln!("Warning: Failed to rollback {}: {}", original_file, e);
                }
            }
        }

        return Ok(RefactorResult {
            operation: "rename".to_string(),
            status: RefactorStatus::Failed,
            changes: applied_changes,
            validation: Some(validation),
            backup_files,
        });
    }

    Ok(RefactorResult {
        operation: "rename".to_string(),
        status: RefactorStatus::Applied,
        changes: applied_changes,
        validation: Some(validation),
        backup_files,
    })
}

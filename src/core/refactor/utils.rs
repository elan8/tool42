use crate::core::refactor::ast_utils::find_symbol_usages_ast;
use crate::core::refactor::types::Change;
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Walk directory tree and find symbol occurrences
pub fn walk_and_find_symbol(
    dir: &Path,
    symbol: &str,
    changes: &mut Vec<Change>,
    files_to_modify: &mut HashMap<PathBuf, Vec<(usize, String, String)>>,
) -> anyhow::Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip hidden files and directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        // Skip target directory
        if path.file_name().and_then(|n| n.to_str()) == Some("target") {
            continue;
        }

        if path.is_file() {
            // Only process Rust source files
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    find_symbol_in_file(&path, symbol, changes, files_to_modify)?;
                }
            }
        } else if path.is_dir() {
            walk_and_find_symbol(&path, symbol, changes, files_to_modify)?;
        }
    }

    Ok(())
}

/// Find symbol occurrences in a single file using AST traversal
pub fn find_symbol_in_file(
    file_path: &Path,
    symbol: &str,
    changes: &mut Vec<Change>,
    files_to_modify: &mut HashMap<PathBuf, Vec<(usize, String, String)>>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Try AST-based search first
    match find_symbol_usages_ast(file_path, symbol, &content) {
        Ok(usages) => {
            // Convert AST usages to changes and replacements
            let file_path_buf = file_path.to_path_buf();
            let replacements = files_to_modify.entry(file_path_buf.clone()).or_default();

            for usage in usages {
                // Check if we already added this change (avoid duplicates)
                let already_added = changes
                    .iter()
                    .any(|c| c.file == file_path.display().to_string() && c.line == usage.line);

                if !already_added {
                    changes.push(Change {
                        file: file_path.display().to_string(),
                        line: usage.line,
                        old: symbol.to_string(),
                        new: symbol.to_string(),
                        context: usage.context,
                    });

                    // For replacements, we need byte positions
                    // Find the exact byte position of the symbol in the line
                    let line_map = build_line_map(&content);
                    let byte_pos = if usage.line > 0 && usage.line <= line_map.len() {
                        let line_start = line_map[usage.line - 1];
                        // Find the symbol in this line
                        if let Some(line_content) = content.lines().nth(usage.line - 1) {
                            if let Some(pos_in_line) = line_content.find(symbol) {
                                // Convert character position to byte position
                                let byte_offset_in_line: usize = line_content
                                    .char_indices()
                                    .nth(pos_in_line)
                                    .map(|(byte_pos, _)| byte_pos)
                                    .unwrap_or(0);
                                line_start + byte_offset_in_line
                            } else {
                                line_start
                            }
                        } else {
                            line_start
                        }
                    } else {
                        // Fallback: search for symbol in content
                        content.find(symbol).unwrap_or(0)
                    };

                    replacements.push((byte_pos, symbol.to_string(), symbol.to_string()));
                }
            }
        }
        Err(_) => {
            // Fallback to text-based search if AST parsing fails (might be incomplete code)
            find_symbol_text_search(file_path, symbol, &content, changes, files_to_modify)?;
        }
    }

    Ok(())
}

/// Find symbol using text-based search with word boundaries
fn find_symbol_text_search(
    file_path: &Path,
    symbol: &str,
    content: &str,
    changes: &mut Vec<Change>,
    files_to_modify: &mut HashMap<PathBuf, Vec<(usize, String, String)>>,
) -> anyhow::Result<()> {
    let line_map = build_line_map(content);
    let mut byte_pos = 0;

    // Find all occurrences of the symbol as a word boundary match
    while let Some(pos) = content[byte_pos..].find(symbol) {
        let absolute_pos = byte_pos + pos;

        // Check if it's a word boundary (not part of another identifier)
        let is_word_boundary = {
            let before = if absolute_pos > 0 {
                content.chars().nth(absolute_pos - 1)
            } else {
                None
            };
            let after = content.chars().nth(absolute_pos + symbol.len());

            let before_ok = before
                .map(|c| !c.is_alphanumeric() && c != '_')
                .unwrap_or(true);
            let after_ok = after
                .map(|c| !c.is_alphanumeric() && c != '_')
                .unwrap_or(true);
            before_ok && after_ok
        };

        if is_word_boundary {
            let line = byte_offset_to_line(absolute_pos, &line_map);
            let context = get_line_at_offset(content, absolute_pos);

            // Check if we already added this change (avoid duplicates)
            let already_added = changes
                .iter()
                .any(|c| c.file == file_path.display().to_string() && c.line == line);

            if !already_added {
                changes.push(Change {
                    file: file_path.display().to_string(),
                    line,
                    old: symbol.to_string(),
                    new: symbol.to_string(),
                    context: context.trim().to_string(),
                });

                let file_path_buf = file_path.to_path_buf();
                let replacements = files_to_modify.entry(file_path_buf).or_default();
                replacements.push((absolute_pos, symbol.to_string(), symbol.to_string()));
            }
        }

        byte_pos = absolute_pos + symbol.len();
        if byte_pos >= content.len() {
            break;
        }
    }

    Ok(())
}

/// Build a line map: maps line numbers to byte offsets
pub fn build_line_map(content: &str) -> Vec<usize> {
    let mut line_map = Vec::new();
    line_map.push(0); // Line 1 starts at byte 0

    for (i, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            line_map.push(i + 1);
        }
    }

    line_map
}

/// Convert byte offset to line number using line map
pub fn byte_offset_to_line(offset: usize, line_map: &[usize]) -> usize {
    match line_map.binary_search(&offset) {
        Ok(line) => line + 1, // 1-based line number
        Err(line) => line,    // 1-based line number
    }
}

/// Get the line content at a given byte offset
pub fn get_line_at_offset(content: &str, offset: usize) -> String {
    let start = content[..offset]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let end = content[offset..]
        .find('\n')
        .map(|pos| offset + pos)
        .unwrap_or(content.len());

    content[start..end].to_string()
}

/// Find line number of a text pattern in content (approximate)
pub fn find_line_in_content(content: &str, pattern: &str) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }

    // Extract a unique substring from pattern to search for
    let search_text = pattern.trim();
    if search_text.is_empty() {
        return None;
    }

    content
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(search_text))
        .map(|(idx, _)| idx + 1)
}

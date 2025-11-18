use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub file: String,
    pub line: usize,
    pub context: String,
}

pub fn search(query: String, path: Option<PathBuf>) -> anyhow::Result<SearchResults> {
    // Validate query is not empty
    if query.trim().is_empty() {
        anyhow::bail!("Query cannot be empty");
    }

    let search_path = path.unwrap_or_else(|| PathBuf::from("."));

    if !search_path.exists() {
        anyhow::bail!("Path does not exist: {}", search_path.display());
    }

    let mut matches = Vec::new();

    // Build regex from query (case-insensitive)
    let regex = Regex::new(&format!("(?i){}", regex::escape(&query))).context("Failed to create regex from query")?;

    // Walk directory tree
    if search_path.is_file() {
        search_file(&search_path, &regex, &mut matches)?;
    } else {
        walk_directory(&search_path, &regex, &mut matches)?;
    }

    Ok(SearchResults { query, matches })
}

fn walk_directory(dir: &Path, regex: &Regex, matches: &mut Vec<Match>) -> anyhow::Result<()> {
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
            // Only search Rust source files
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    search_file(&path, regex, matches)?;
                }
            }
        } else if path.is_dir() {
            walk_directory(&path, regex, matches)?;
        }
    }

    Ok(())
}

fn search_file(file_path: &Path, regex: &Regex, matches: &mut Vec<Match>) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    for (line_num, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            // Extract context (the matching line, trimmed)
            let context = line.trim().to_string();

            matches.push(Match {
                file: file_path.display().to_string(),
                line: line_num + 1, // 1-based line numbers
                context,
            });
        }
    }

    Ok(())
}




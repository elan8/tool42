use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<EntryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

pub fn list_directory(path: Option<PathBuf>) -> anyhow::Result<DirectoryListing> {
    let list_path = path.unwrap_or_else(|| PathBuf::from("."));

    if !list_path.exists() {
        anyhow::bail!("Path does not exist: {}", list_path.display());
    }

    if !list_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", list_path.display());
    }

    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(&list_path)
        .with_context(|| format!("Failed to read directory: {}", list_path.display()))?;

    for entry in dir_entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("Failed to read metadata for: {}", path.display()))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let entry_type = if metadata.is_file() {
            "file".to_string()
        } else if metadata.is_dir() {
            "directory".to_string()
        } else {
            "other".to_string()
        };

        let size = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };

        let modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs().to_string())
        });

        entries.push(EntryInfo {
            name,
            entry_type,
            size,
            modified,
        });
    }

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(
        |a, b| match (a.entry_type.as_str(), b.entry_type.as_str()) {
            ("directory", "file") => std::cmp::Ordering::Less,
            ("file", "directory") => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        },
    );

    Ok(DirectoryListing {
        path: list_path.display().to_string(),
        entries,
    })
}





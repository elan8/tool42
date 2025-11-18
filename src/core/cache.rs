use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Clone)]
struct CachedOutput {
    lines: Vec<String>,
    exit_code: i32,
    project_fingerprint: u64,
}

type Cache = Arc<Mutex<HashMap<String, CachedOutput>>>;

static OUTPUT_CACHE: LazyLock<Cache> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Generate a cache key from working directory and command arguments
pub fn generate_cache_key(working_dir: &Path, args: &[String]) -> String {
    format!("{}|{:?}", working_dir.display(), args)
}

/// Compute a fingerprint of the project by hashing file modification times
/// This includes all .rs files and Cargo.toml/Cargo.lock files
fn compute_project_fingerprint(working_dir: &Path) -> anyhow::Result<u64> {
    let project_root = crate::core::project::find_workspace_root(working_dir)
        .with_context(|| format!("Failed to find project root from: {}", working_dir.display()))?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    // Collect all relevant files
    let mut files_to_check = Vec::new();

    // Add Cargo.toml and Cargo.lock
    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() {
        files_to_check.push(cargo_toml);
    }
    let cargo_lock = project_root.join("Cargo.lock");
    if cargo_lock.exists() {
        files_to_check.push(cargo_lock);
    }

    // Walk directory to find all .rs files
    walk_for_rust_files(&project_root, &mut files_to_check)?;

    // Hash file paths and modification times
    for file_path in &files_to_check {
        // Hash the file path
        file_path.display().to_string().hash(&mut hasher);

        // Hash the modification time
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                // Convert SystemTime to a hashable representation
                modified.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
                    .hash(&mut hasher);
            }
        }
    }

    Ok(hasher.finish())
}

/// Walk directory tree to find all Rust source files
fn walk_for_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
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
            // Include all .rs files
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    files.push(path);
                }
            }
        } else if path.is_dir() {
            walk_for_rust_files(&path, files)?;
        }
    }

    Ok(())
}

/// Get cached output if available and project hasn't changed
pub fn get_cached_output(key: &str, working_dir: &Path) -> Option<(Vec<String>, i32)> {
    let cache = OUTPUT_CACHE.lock().unwrap();

    let cached = cache.get(key)?;

    // Check if project fingerprint matches
    match compute_project_fingerprint(working_dir) {
        Ok(current_fingerprint) => {
            if current_fingerprint != cached.project_fingerprint {
                // Project has changed, invalidate cache
                drop(cache);
                invalidate_cache_entry(key);
                return None;
            }
        }
        Err(_) => {
            // If we can't compute fingerprint, don't use cache to be safe
            return None;
        }
    }

    Some((cached.lines.clone(), cached.exit_code))
}

/// Store output in cache with current project fingerprint
pub fn store_output(key: &str, output: String, exit_code: i32, working_dir: &Path) {
    let lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();

    // Compute current project fingerprint
    let fingerprint = compute_project_fingerprint(working_dir).unwrap_or(0);

    let cached = CachedOutput {
        lines,
        exit_code,
        project_fingerprint: fingerprint,
    };

    let mut cache = OUTPUT_CACHE.lock().unwrap();
    cache.insert(key.to_string(), cached);
}

/// Invalidate a specific cache entry
fn invalidate_cache_entry(key: &str) {
    let mut cache = OUTPUT_CACHE.lock().unwrap();
    cache.remove(key);
}

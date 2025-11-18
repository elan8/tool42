use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Clone)]
struct CachedOutput {
    lines: Vec<String>,
    exit_code: i32,
}

type Cache = Arc<Mutex<HashMap<String, CachedOutput>>>;

static OUTPUT_CACHE: LazyLock<Cache> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Generate a cache key from working directory and command arguments
pub fn generate_cache_key(working_dir: &PathBuf, args: &[String]) -> String {
    format!("{}|{:?}", working_dir.display(), args)
}

/// Get cached output if available
pub fn get_cached_output(key: &str) -> Option<(Vec<String>, i32)> {
    let cache = OUTPUT_CACHE.lock().unwrap();
    cache
        .get(key)
        .map(|cached| (cached.lines.clone(), cached.exit_code))
}

/// Store output in cache
pub fn store_output(key: &str, output: String, exit_code: i32) {
    let lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    let cached = CachedOutput { lines, exit_code };
    let mut cache = OUTPUT_CACHE.lock().unwrap();
    cache.insert(key.to_string(), cached);
}

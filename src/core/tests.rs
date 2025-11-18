use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use syn::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub tests: Vec<TestInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInfo {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub module_path: String,
}

pub fn find_tests(path: Option<PathBuf>) -> anyhow::Result<TestResults> {
    let search_path = path.unwrap_or_else(|| PathBuf::from("."));

    if !search_path.exists() {
        anyhow::bail!("Path does not exist: {}", search_path.display());
    }

    let mut tests = Vec::new();

    // Walk directory tree to find Rust files
    if search_path.is_file() {
        find_tests_in_file(&search_path, "", &mut tests)?;
    } else {
        walk_directory_for_tests(&search_path, "", &mut tests)?;
    }

    Ok(TestResults { tests })
}

fn walk_directory_for_tests(
    dir: &Path,
    module_path: &str,
    tests: &mut Vec<TestInfo>,
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
            // Only search Rust source files
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    find_tests_in_file(&path, module_path, tests)?;
                }
            }
        } else if path.is_dir() {
            // Build new module path
            let new_module_path = if module_path.is_empty() {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                format!(
                    "{}::{}",
                    module_path,
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                )
            };
            walk_directory_for_tests(&path, &new_module_path, tests)?;
        }
    }

    Ok(())
}

fn find_tests_in_file(
    file_path: &Path,
    module_path: &str,
    tests: &mut Vec<TestInfo>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Parse Rust file
    let ast = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(_) => {
            // If parsing fails, skip this file (might be invalid Rust or have proc macros)
            return Ok(());
        }
    };

    // Build line map
    let line_map = build_line_map(&content);

    // Extract tests from items
    extract_tests_from_items(
        &ast.items,
        file_path,
        module_path,
        &line_map,
        &content,
        tests,
    );

    Ok(())
}

fn build_line_map(content: &str) -> Vec<usize> {
    let mut line_map = Vec::new();
    line_map.push(0); // Line 1 starts at byte 0

    for (i, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            line_map.push(i + 1);
        }
    }

    line_map
}

fn byte_offset_to_line(offset: usize, line_map: &[usize]) -> usize {
    match line_map.binary_search(&offset) {
        Ok(line) => line + 1, // 1-based line number
        Err(line) => line,    // 1-based line number
    }
}

fn extract_tests_from_items(
    items: &[Item],
    file_path: &Path,
    module_path: &str,
    line_map: &[usize],
    content: &str,
    tests: &mut Vec<TestInfo>,
) {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                // Check if function has #[test] attribute
                if has_test_attribute(&item_fn.attrs) {
                    // Use search-based approach to find line number (same as describe.rs)
                    let search_str = format!("fn {}", item_fn.sig.ident);
                    let line = if let Some(pos) = content.find(&search_str) {
                        byte_offset_to_line(pos, line_map)
                    } else {
                        1 // Fallback
                    };

                    let test_name = item_fn.sig.ident.to_string();
                    let full_module_path = if module_path.is_empty() {
                        "".to_string()
                    } else {
                        format!("{}::", module_path)
                    };

                    tests.push(TestInfo {
                        name: test_name,
                        file: file_path.display().to_string(),
                        line,
                        module_path: full_module_path,
                    });
                }
            }
            Item::Mod(item_mod) => {
                // Recursively search in module content
                if let Some((_, mod_items)) = &item_mod.content {
                    let new_module_path = if module_path.is_empty() {
                        item_mod.ident.to_string()
                    } else {
                        format!("{}::{}", module_path, item_mod.ident)
                    };
                    extract_tests_from_items(
                        mod_items,
                        file_path,
                        &new_module_path,
                        line_map,
                        content,
                        tests,
                    );
                }
            }
            _ => {}
        }
    }
}

fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("test") {
            return true;
        }
    }
    false
}

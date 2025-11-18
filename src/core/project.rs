use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use syn::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub workspace_root: String,
    pub packages: Vec<PackageStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStructure {
    pub name: String,
    pub path: String,
    pub modules: Vec<ModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ModuleInfo>>,
}

pub fn get_structure(path: Option<PathBuf>) -> anyhow::Result<ProjectStructure> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    if !project_path.exists() {
        anyhow::bail!("Path does not exist: {}", project_path.display());
    }

    // Find Cargo.toml to determine workspace root
    let workspace_root = find_workspace_root(&project_path)?;

    // Find all Cargo.toml files (packages)
    let mut packages = Vec::new();
    find_packages(&workspace_root, &mut packages)?;

    Ok(ProjectStructure {
        workspace_root: workspace_root.display().to_string(),
        packages,
    })
}

pub fn find_workspace_root(start_path: &Path) -> anyhow::Result<PathBuf> {
    let mut current = start_path.to_path_buf();

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!("No Cargo.toml found in directory tree"),
        }
    }
}

/// Resolve a working directory string to an absolute canonicalized path.
/// Handles both absolute and relative paths.
/// Relative paths are resolved from the current working directory.
pub fn resolve_working_directory(working_dir: &str) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(working_dir);

    let canonicalized = if path.is_absolute() {
        path.canonicalize()
            .with_context(|| format!("Failed to canonicalize absolute path: {}", working_dir))?
    } else {
        std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?
            .join(&path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "Failed to canonicalize relative path {}: {}",
                    working_dir,
                    path.display()
                )
            })?
    };

    // On Windows, canonicalize() adds the extended path prefix `\\?\`.
    // Remove it for cleaner paths in error messages and compatibility.
    Ok(normalize_path(canonicalized))
}

/// Normalize a path by removing Windows extended path prefix if present.
fn normalize_path(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        // Remove the extended path prefix
        PathBuf::from(stripped)
    } else {
        path
    }
}

/// Resolve a path relative to the project root (where Cargo.toml is).
/// If the path is already absolute, it's returned as-is.
/// If the path is relative, it's resolved relative to the project root.
///
/// Uses the provided working_directory to find the project root.
pub fn resolve_path(path: &Path, working_directory: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    // Find project root starting from the provided working directory
    let project_root = find_workspace_root(working_directory).with_context(|| {
        format!(
            "Could not find project root (Cargo.toml) starting from working directory: {}",
            working_directory.display()
        )
    })?;

    Ok(project_root.join(path))
}

fn find_packages(
    workspace_root: &Path,
    packages: &mut Vec<PackageStructure>,
) -> anyhow::Result<()> {
    // Check root Cargo.toml
    let root_cargo = workspace_root.join("Cargo.toml");
    if root_cargo.exists() {
        if let Ok(pkg) = parse_package(workspace_root, workspace_root) {
            packages.push(pkg);
        }
    }

    // Walk directory tree to find other Cargo.toml files
    walk_for_packages(workspace_root, workspace_root, packages)?;

    Ok(())
}

fn walk_for_packages(
    workspace_root: &Path,
    dir: &Path,
    packages: &mut Vec<PackageStructure>,
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

        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            // Found a Cargo.toml, parse the package
            if let Some(parent) = path.parent() {
                if parent != workspace_root {
                    // Only add if it's not the root package (already added)
                    if let Ok(pkg) = parse_package(parent, workspace_root) {
                        packages.push(pkg);
                    }
                }
            }
        } else if path.is_dir() {
            walk_for_packages(workspace_root, &path, packages)?;
        }
    }

    Ok(())
}

fn parse_package(package_dir: &Path, _workspace_root: &Path) -> anyhow::Result<PackageStructure> {
    let cargo_toml = package_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_toml.display()))?;

    // Extract package name from Cargo.toml (simple parsing)
    let name = extract_package_name(&content).unwrap_or_else(|| {
        package_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Find modules in src directory
    let src_dir = package_dir.join("src");
    let modules = if src_dir.exists() {
        find_modules_in_dir(&src_dir, package_dir)?
    } else {
        Vec::new()
    };

    Ok(PackageStructure {
        name,
        path: package_dir.display().to_string(),
        modules,
    })
}

fn extract_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn find_modules_in_dir(dir: &Path, package_root: &Path) -> anyhow::Result<Vec<ModuleInfo>> {
    let mut modules = Vec::new();

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip hidden files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    // Parse Rust file to find module declarations
                    if let Ok(module_info) = parse_rust_file_for_modules(&path, package_root) {
                        modules.push(module_info);
                    }
                }
            }
        } else if path.is_dir() {
            // Directory might be a module
            let module_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let mod_file = path.join("mod.rs");
            let mod_file_alt = dir.join(format!("{}.rs", module_name));

            if mod_file.exists() || mod_file_alt.exists() {
                let nested_modules = find_modules_in_dir(&path, package_root).unwrap_or_default();
                modules.push(ModuleInfo {
                    name: module_name,
                    path: path.display().to_string(),
                    modules: if nested_modules.is_empty() {
                        None
                    } else {
                        Some(nested_modules)
                    },
                });
            }
        }
    }

    Ok(modules)
}

fn parse_rust_file_for_modules(
    file_path: &Path,
    package_root: &Path,
) -> anyhow::Result<ModuleInfo> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Parse Rust file
    let ast = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(_) => {
            // If parsing fails, return basic module info
            return Ok(ModuleInfo {
                name: file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: file_path.display().to_string(),
                modules: None,
            });
        }
    };

    // Extract nested modules
    let mut nested_modules = Vec::new();
    for item in &ast.items {
        if let Item::Mod(item_mod) = item {
            if let Some((_, mod_items)) = &item_mod.content {
                // Inline module - extract nested modules
                for nested_item in mod_items {
                    if let Item::Mod(nested_mod) = nested_item {
                        nested_modules.push(ModuleInfo {
                            name: nested_mod.ident.to_string(),
                            path: file_path.display().to_string(),
                            modules: None,
                        });
                    }
                }
            } else {
                // External module - try to find the file
                let mod_name = item_mod.ident.to_string();
                let mod_file = file_path.parent().unwrap().join(format!("{}.rs", mod_name));
                let mod_dir = file_path.parent().unwrap().join(&mod_name);

                if mod_file.exists() {
                    if let Ok(nested_info) = parse_rust_file_for_modules(&mod_file, package_root) {
                        nested_modules.push(nested_info);
                    }
                } else if mod_dir.exists() {
                    if let Ok(nested_modules_list) = find_modules_in_dir(&mod_dir, package_root) {
                        nested_modules.push(ModuleInfo {
                            name: mod_name,
                            path: mod_dir.display().to_string(),
                            modules: if nested_modules_list.is_empty() {
                                None
                            } else {
                                Some(nested_modules_list)
                            },
                        });
                    }
                }
            }
        }
    }

    Ok(ModuleInfo {
        name: file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        path: file_path.display().to_string(),
        modules: if nested_modules.is_empty() {
            None
        } else {
            Some(nested_modules)
        },
    })
}

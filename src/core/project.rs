use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use syn::Item;
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub workspace_root: String,
    pub is_workspace: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_dependencies: Option<Vec<CargoDependency>>,
    pub packages: Vec<PackageStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStructure {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<CargoDependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_dependencies: Option<Vec<CargoDependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_dependencies: Option<Vec<CargoDependency>>,
    pub crates: Vec<CrateStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateStructure {
    pub name: String,
    pub crate_type: CrateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ModuleInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrateType {
    Lib,
    Bin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoDependency {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ModuleInfo>>,
}

pub fn get_structure(workspace_root: PathBuf) -> anyhow::Result<ProjectStructure> {
    if !workspace_root.exists() {
        anyhow::bail!("Path does not exist: {}", workspace_root.display());
    }

    // Check if this is a workspace and extract workspace dependencies and members
    let root_cargo_toml = workspace_root.join("Cargo.toml");
    let (is_workspace, workspace_dependencies, workspace_members) = if root_cargo_toml.exists() {
        let content = fs::read_to_string(&root_cargo_toml)
            .with_context(|| format!("Failed to read root Cargo.toml: {}", root_cargo_toml.display()))?;
        let toml: TomlValue = content.parse()
            .with_context(|| format!("Failed to parse root Cargo.toml: {}", root_cargo_toml.display()))?;
        let is_ws = toml.get("workspace").is_some();
        let ws_deps = if is_ws {
            extract_workspace_dependencies(&toml)
        } else {
            None
        };
        let ws_members = if is_ws {
            extract_workspace_members(&toml, &workspace_root)
        } else {
            None
        };
        (is_ws, ws_deps, ws_members)
    } else {
        (false, None, None)
    };

    // Find packages: for workspace, only include members; for non-workspace, only root package
    let mut packages = Vec::new();
    find_packages(&workspace_root, &mut packages, is_workspace, workspace_members.as_ref())?;

    Ok(ProjectStructure {
        workspace_root: workspace_root.display().to_string(),
        is_workspace,
        workspace_dependencies,
        packages,
    })
}

fn extract_workspace_members(toml: &TomlValue, workspace_root: &Path) -> Option<Vec<PathBuf>> {
    let workspace = toml.get("workspace")?.as_table()?;
    let members = workspace.get("members")?;
    
    let mut result = Vec::new();
    
    if let TomlValue::Array(arr) = members {
        for member in arr {
            if let Some(member_str) = member.as_str() {
                // Handle glob patterns (e.g., "crates/*")
                if member_str.contains('*') {
                    // Expand glob pattern - find all directories matching the pattern
                    let pattern = workspace_root.join(member_str);
                    if let Some(parent) = pattern.parent() {
                        if let Ok(entries) = fs::read_dir(parent) {
                            // Extract the prefix before the *
                            let pattern_str = pattern.to_string_lossy();
                            let prefix = if let Some(star_pos) = pattern_str.find('*') {
                                &pattern_str[..star_pos]
                            } else {
                                pattern_str.as_ref()
                            };
                            
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_dir() {
                                    let path_str = path.to_string_lossy();
                                    // Check if path matches the pattern prefix
                                    if path_str.starts_with(prefix) {
                                        let cargo_toml = path.join("Cargo.toml");
                                        if cargo_toml.exists() {
                                            result.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Direct path
                    let member_path = workspace_root.join(member_str);
                    let cargo_toml = member_path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        result.push(member_path);
                    }
                }
            }
        }
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
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
    is_workspace: bool,
    workspace_members: Option<&Vec<PathBuf>>,
) -> anyhow::Result<()> {
    if is_workspace {
        // For workspace: only include packages listed in workspace.members
        if let Some(members) = workspace_members {
            for member_path in members {
                if let Ok(pkg) = parse_package(member_path, workspace_root) {
                    packages.push(pkg);
                }
            }
        }
    } else {
        // For non-workspace: only include the root package
        let root_cargo = workspace_root.join("Cargo.toml");
        if root_cargo.exists() {
            if let Ok(pkg) = parse_package(workspace_root, workspace_root) {
                packages.push(pkg);
            }
        }
    }

    Ok(())
}


fn parse_package(
    package_dir: &Path,
    _workspace_root: &Path,
) -> anyhow::Result<PackageStructure> {
    let cargo_toml = package_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_toml.display()))?;

    // Parse Cargo.toml using proper TOML parser
    let toml: TomlValue = content.parse()
        .with_context(|| format!("Failed to parse Cargo.toml: {}", cargo_toml.display()))?;

    // Extract package metadata from [package] section
    let package_section = toml.get("package")
        .and_then(|p| p.as_table());

    let package_name = package_section
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            package_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    let version = package_section
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let edition = package_section
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());

    let description = package_section
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());

    let license = package_section
        .and_then(|p| p.get("license"))
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());

    // Extract dependencies
    let dependencies = extract_dependencies(&toml, "dependencies");
    let dev_dependencies = extract_dependencies(&toml, "dev-dependencies");
    let build_dependencies = extract_dependencies(&toml, "build-dependencies");

    let mut crates = Vec::new();

    // Check for library crate
    let lib_section = toml.get("lib").and_then(|l| l.as_table());
    let lib_name = lib_section
        .and_then(|l| l.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| package_name.clone());
    
    // Check if library exists (either explicit [lib] or default src/lib.rs)
    let src_lib = package_dir.join("src").join("lib.rs");
    if lib_section.is_some() || src_lib.exists() {
        crates.push(CrateStructure {
            name: lib_name,
            crate_type: CrateType::Lib,
            modules: None,
        });
    }

    // Check for binary crates
    // First check [[bin]] sections
    if let Some(bin_array) = toml.get("bin").and_then(|b| b.as_array()) {
        for bin_entry in bin_array {
            if let Some(bin_table) = bin_entry.as_table() {
                let bin_name = bin_table
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| package_name.clone());
                
                crates.push(CrateStructure {
                    name: bin_name,
                    crate_type: CrateType::Bin,
                    modules: None,
                });
            }
        }
    }

    // Check for default binary (src/main.rs)
    let src_main = package_dir.join("src").join("main.rs");
    if src_main.exists() {
        let bin_name = package_name.clone();
        // Only add if not already added via [[bin]] section
        if !crates.iter().any(|c| c.name == bin_name && matches!(c.crate_type, CrateType::Bin)) {
            crates.push(CrateStructure {
                name: bin_name,
                crate_type: CrateType::Bin,
                modules: None,
            });
        }
    }

    // Check for binaries in src/bin/*.rs
    let src_bin_dir = package_dir.join("src").join("bin");
    if src_bin_dir.exists() && src_bin_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&src_bin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext == "rs" {
                            if let Some(bin_name) = path.file_stem().and_then(|s| s.to_str()) {
                                // Only add if not already added via [[bin]] section
                                if !crates.iter().any(|c| c.name == bin_name && matches!(c.crate_type, CrateType::Bin)) {
                                    crates.push(CrateStructure {
                                        name: bin_name.to_string(),
                                        crate_type: CrateType::Bin,
                                        modules: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(PackageStructure {
        name: package_name,
        path: package_dir.display().to_string(),
        version,
        edition,
        description,
        license,
        dependencies: if dependencies.is_empty() { None } else { Some(dependencies) },
        dev_dependencies: if dev_dependencies.is_empty() { None } else { Some(dev_dependencies) },
        build_dependencies: if build_dependencies.is_empty() { None } else { Some(build_dependencies) },
        crates,
    })
}

fn extract_workspace_dependencies(toml: &TomlValue) -> Option<Vec<CargoDependency>> {
    let workspace = toml.get("workspace")?.as_table()?;
    let deps = workspace.get("dependencies")?.as_table()?;
    
    let mut result = Vec::new();
    for (name, value) in deps {
        if let Some(dep_info) = parse_dependency_value(name, value) {
            result.push(dep_info);
        }
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn extract_dependencies(toml: &TomlValue, section: &str) -> Vec<CargoDependency> {
    let deps_table = toml
        .get(section)
        .and_then(|d| d.as_table());
    
    let mut result = Vec::new();
    if let Some(deps) = deps_table {
        for (name, value) in deps {
            if let Some(dep_info) = parse_dependency_value(name, value) {
                result.push(dep_info);
            }
        }
    }
    result
}

fn parse_dependency_value(name: &str, value: &TomlValue) -> Option<CargoDependency> {
    match value {
        TomlValue::String(version) => {
            Some(CargoDependency {
                name: name.to_string(),
                version: Some(version.clone()),
                path: None,
                workspace: None,
                features: None,
            })
        }
        TomlValue::Table(table) => {
            let version = table.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let path = table.get("path")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let workspace = table.get("workspace")
                .and_then(|w| w.as_bool());
            let features = table.get("features")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });
            
            Some(CargoDependency {
                name: name.to_string(),
                version,
                path,
                workspace,
                features,
            })
        }
        _ => None,
    }
}


#[allow(dead_code)]
fn find_modules_in_dir(
    dir: &Path,
    package_root: &Path,
    max_depth: usize,
    current_depth: usize,
) -> anyhow::Result<Vec<ModuleInfo>> {
    // If we've exceeded max depth, return empty
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

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
                    if let Ok(module_info) = parse_rust_file_for_modules(&path, package_root, max_depth, current_depth + 1) {
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
                let nested_modules = if current_depth + 1 < max_depth {
                    find_modules_in_dir(&path, package_root, max_depth, current_depth + 1).unwrap_or_default()
                } else {
                    Vec::new()
                };
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

#[allow(dead_code)]
fn parse_rust_file_for_modules(
    file_path: &Path,
    package_root: &Path,
    max_depth: usize,
    current_depth: usize,
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

    // Extract nested modules only if we haven't exceeded max depth
    let mut nested_modules = Vec::new();
    if current_depth < max_depth {
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
                        if let Ok(nested_info) = parse_rust_file_for_modules(&mod_file, package_root, max_depth, current_depth + 1) {
                            nested_modules.push(nested_info);
                        }
                    } else if mod_dir.exists() {
                        if let Ok(nested_modules_list) = find_modules_in_dir(&mod_dir, package_root, max_depth, current_depth + 1) {
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

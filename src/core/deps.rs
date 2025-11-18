use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub workspace_root: String,
    pub packages: Vec<PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
}

pub fn get_dependencies(working_dir: PathBuf) -> anyhow::Result<DependencyInfo> {
    // Run cargo metadata in the specified working directory
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(&working_dir)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute cargo metadata command in directory: {}",
                working_dir.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo metadata failed: {}", stderr);
    }

    // Parse cargo metadata JSON
    let metadata_json =
        String::from_utf8(output.stdout).context("Failed to read cargo metadata output")?;

    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_json).context("Failed to parse cargo metadata JSON")?;

    // Extract relevant information
    let workspace_root = metadata["workspace_root"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let mut packages = Vec::new();

    if let Some(packages_array) = metadata["packages"].as_array() {
        for pkg in packages_array {
            let name = pkg["name"].as_str().unwrap_or("").to_string();
            let version = pkg["version"].as_str().unwrap_or("").to_string();
            let source = pkg["source"].as_str().map(|s| s.to_string());

            // Extract dependencies
            let mut dependencies = Vec::new();
            if let Some(deps_array) = pkg["dependencies"].as_array() {
                for dep in deps_array {
                    if let Some(dep_name) = dep["name"].as_str() {
                        dependencies.push(dep_name.to_string());
                    }
                }
            }

            // Extract features
            let features = pkg["features"]
                .as_object()
                .map(|f| f.keys().cloned().collect::<Vec<String>>());

            packages.push(PackageInfo {
                name,
                version,
                source,
                dependencies,
                features,
            });
        }
    }

    Ok(DependencyInfo {
        workspace_root,
        packages,
    })
}

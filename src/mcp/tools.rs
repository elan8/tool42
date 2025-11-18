use crate::core;
use crate::mcp::schemas::*;
use serde_json::Value;
use std::path::PathBuf;

pub async fn handle_cargo(params: Value) -> Result<Value, String> {
    let args: CargoArgs =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let working_dir_str = args.working_directory.clone();
    let cargo_args = args.args;
    let from = args.from;
    let to = args.to;

    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
        crate::core::cargo::execute_mcp_paginated(cargo_args, working_dir, from, to)
            .map_err(|e| format!("Cargo execution failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Cargo execution failed: {}", e))?;

    Ok(serde_json::json!({
        "output_lines": result.lines,
        "total_lines": result.total_lines,
        "lines_returned": result.lines_returned,
        "exit_code": result.exit_code,
    }))
}

pub async fn handle_clippy(params: Value) -> Result<Value, String> {
    let args: ClippyArgs =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let working_dir_str = args.working_directory.clone();
    let clippy_args = args.args;
    let from = args.from;
    let to = args.to;

    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
        crate::core::clippy::execute_mcp_paginated(clippy_args, working_dir, from, to)
            .map_err(|e| format!("Clippy execution failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Clippy execution failed: {}", e))?;

    Ok(serde_json::json!({
        "output_lines": result.lines,
        "total_lines": result.total_lines,
        "lines_returned": result.lines_returned,
        "exit_code": result.exit_code,
    }))
}

pub async fn handle_read(params: Value) -> Result<Value, String> {
    let read_params: ReadParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = read_params.path.clone();
    let working_dir_str = read_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
        let resolved_path = core::project::resolve_path(&PathBuf::from(path_str), &working_dir)
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        core::read::read_file(resolved_path, read_params.from, read_params.to)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Read failed: {}", e))?;

    Ok(serde_json::json!({
        "content": result.content,
        "total_lines": result.total_lines,
        "lines_returned": result.lines_returned,
    }))
}

pub async fn handle_describe(params: Value) -> Result<Value, String> {
    let desc_params: DescribeParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = desc_params.path.clone();
    let working_dir_str = desc_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
        let resolved_path = core::project::resolve_path(&PathBuf::from(path_str), &working_dir)
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        core::describe::describe_file(resolved_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Describe failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_search(params: Value) -> Result<Value, String> {
    let search_params: SearchParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = search_params.path.clone();
    let working_dir_str = search_params.working_directory.clone();
    let query = search_params.query.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        // Resolve working_directory to absolute path
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;

        let path = if let Some(p) = path_str {
            match core::project::resolve_path(&PathBuf::from(p), &working_dir) {
                Ok(resolved) => Some(resolved),
                Err(e) => return Err(format!("Failed to resolve path: {}", e)),
            }
        } else {
            match core::project::find_workspace_root(&working_dir) {
                Ok(root) => Some(root),
                Err(e) => return Err(format!("Failed to find project root: {}", e)),
            }
        };
        core::search::search(query, path).map_err(|e| format!("Search failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Search failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_deps(_params: Value) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(core::deps::get_dependencies)
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Deps failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_tests(params: Value) -> Result<Value, String> {
    let tests_params: TestsParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = tests_params.path.clone();
    let working_dir_str = tests_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        // Resolve working_directory to absolute path
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;

        let path = if let Some(p) = path_str {
            match core::project::resolve_path(&PathBuf::from(p), &working_dir) {
                Ok(resolved) => Some(resolved),
                Err(e) => return Err(format!("Failed to resolve path: {}", e)),
            }
        } else {
            match core::project::find_workspace_root(&working_dir) {
                Ok(root) => Some(root),
                Err(e) => return Err(format!("Failed to find project root: {}", e)),
            }
        };
        core::tests::find_tests(path).map_err(|e| format!("Tests failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Tests failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_project(params: Value) -> Result<Value, String> {
    let project_params: ProjectParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = project_params.path.clone();
    let working_dir_str = project_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        // Resolve working_directory to absolute path
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;

        let path = if let Some(p) = path_str {
            match core::project::resolve_path(&PathBuf::from(p), &working_dir) {
                Ok(resolved) => Some(resolved),
                Err(e) => return Err(format!("Failed to resolve path: {}", e)),
            }
        } else {
            match core::project::find_workspace_root(&working_dir) {
                Ok(root) => Some(root),
                Err(e) => return Err(format!("Failed to find project root: {}", e)),
            }
        };
        core::project::get_structure(path).map_err(|e| format!("Project failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Project failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_list(params: Value) -> Result<Value, String> {
    let list_params: ListParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = list_params.path.clone();
    let working_dir_str = list_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        // Resolve working_directory to absolute path
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;

        let path = if let Some(p) = path_str {
            match core::project::resolve_path(&PathBuf::from(p), &working_dir) {
                Ok(resolved) => Some(resolved),
                Err(e) => return Err(format!("Failed to resolve path: {}", e)),
            }
        } else {
            match core::project::find_workspace_root(&working_dir) {
                Ok(root) => Some(root),
                Err(e) => return Err(format!("Failed to find project root: {}", e)),
            }
        };
        core::list::list_directory(path).map_err(|e| format!("List failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("List failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_docs(params: Value) -> Result<Value, String> {
    let docs_params: DocsParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = docs_params.path.clone();
    let working_dir_str = docs_params.working_directory.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
        let resolved_path = core::project::resolve_path(&PathBuf::from(path_str), &working_dir)
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        core::docs::extract_docs(resolved_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Docs failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_refactor_rename(params: Value) -> Result<Value, String> {
    let rename_params: RefactorRenameParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let path_str = rename_params.path.clone();
    let working_dir_str = rename_params.working_directory.clone();
    let preview = rename_params.preview.unwrap_or(true);
    let apply = rename_params.apply.unwrap_or(false);

    let result = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let working_dir = core::project::resolve_working_directory(&working_dir_str)
            .map_err(|e| format!("Failed to resolve working directory: {}", e))?;

        let path = if let Some(p) = path_str {
            Some(
                core::project::resolve_path(&PathBuf::from(p), &working_dir)
                    .map_err(|e| format!("Failed to resolve path: {}", e))?,
            )
        } else {
            None
        };
        core::refactor::rename_symbol(
            rename_params.symbol,
            rename_params.to,
            path,
            Some(working_dir),
            preview,
            apply,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Refactor rename failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_refactor_extract(params: Value) -> Result<Value, String> {
    let extract_params: RefactorExtractParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let preview = extract_params.preview.unwrap_or(true);
    let apply = extract_params.apply.unwrap_or(false);
    let file_str = extract_params.file.clone();
    let working_dir = PathBuf::from(extract_params.working_directory);

    let result = tokio::task::spawn_blocking(move || {
        let resolved_path = core::project::resolve_path(&PathBuf::from(file_str), &working_dir)
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        core::refactor::extract_function(
            resolved_path,
            extract_params.from,
            extract_params.to,
            extract_params.name,
            preview,
            apply,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Refactor extract failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_refactor_move(params: Value) -> Result<Value, String> {
    let move_params: RefactorMoveParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let preview = move_params.preview.unwrap_or(true);
    let apply = move_params.apply.unwrap_or(false);
    let working_dir = PathBuf::from(move_params.working_directory);

    let result = tokio::task::spawn_blocking(move || {
        core::refactor::move_item(
            move_params.symbol,
            move_params.to,
            working_dir,
            preview,
            apply,
        )
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Refactor move failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

pub async fn handle_refactor_signature(params: Value) -> Result<Value, String> {
    let signature_params: RefactorSignatureParams =
        serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;

    let preview = signature_params.preview.unwrap_or(true);
    let apply = signature_params.apply.unwrap_or(false);
    let working_dir = PathBuf::from(signature_params.working_directory);

    let result = tokio::task::spawn_blocking(move || {
        core::refactor::change_signature(
            signature_params.function,
            signature_params.new_signature,
            working_dir,
            preview,
            apply,
        )
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Refactor signature failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

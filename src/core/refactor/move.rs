use crate::core::refactor::ast_utils::{
    build_new_import_path, find_impl_blocks_for_symbol, find_item_definition, item_matches_symbol,
    resolve_target_path, update_imports_in_ast,
};
use crate::core::refactor::types::{RefactorResult, RefactorStatus};
use crate::core::refactor::utils::walk_and_find_symbol;
use crate::core::refactor::validation::validate_with_cargo_check;
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::Item;

/// Move a function/struct/enum to a different module or file
pub fn move_item(
    symbol: String,
    target: String,
    working_directory: PathBuf,
    preview: bool,
    apply: bool,
) -> anyhow::Result<RefactorResult> {
    // Find the symbol in the codebase
    let mut changes = Vec::new();
    let mut files_to_modify = HashMap::new();

    walk_and_find_symbol(
        &working_directory,
        &symbol,
        &mut changes,
        &mut files_to_modify,
    )?;

    if changes.is_empty() {
        anyhow::bail!("Symbol '{}' not found in the codebase", symbol);
    }

    // Build preview changes
    let mut preview_changes = changes.clone();
    for change in &mut preview_changes {
        change.new = format!("Move to {}", target);
        change.context = format!("Move {} to {}", symbol, target);
    }

    // If preview mode, return preview
    if preview && !apply {
        return Ok(RefactorResult {
            operation: "move".to_string(),
            status: RefactorStatus::Preview,
            changes: preview_changes,
            validation: None,
            backup_files: Vec::new(),
        });
    }

    // Apply changes
    if !apply {
        anyhow::bail!("Must specify --apply to apply changes");
    }

    // Create backups
    let mut backup_files = Vec::new();
    for file_path in files_to_modify.keys() {
        let backup_path = format!("{}.backup", file_path.display());
        fs::copy(file_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
        backup_files.push(backup_path);
    }

    // Find the item definition using AST
    let source_file = find_item_definition(&symbol, &files_to_modify)?;

    // Determine target file path
    let target_path = resolve_target_path(&target)?;

    // If target file doesn't exist, create it
    if !target_path.exists() {
        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        // Create a basic module file
        fs::write(&target_path, format!("// Module: {}\n\n", target))
            .with_context(|| format!("Failed to create target file: {}", target_path.display()))?;
    }

    // Read target file
    let target_content = fs::read_to_string(&target_path)
        .with_context(|| format!("Failed to read target file: {}", target_path.display()))?;

    // Read source file
    let source_content = fs::read_to_string(&source_file)
        .with_context(|| format!("Failed to read source file: {}", source_file.display()))?;

    // Parse source file to extract item
    let source_ast = syn::parse_file(&source_content)
        .with_context(|| format!("Failed to parse source file: {}", source_file.display()))?;

    // Find and extract the item
    let item_code = extract_item_code(&source_ast, &symbol, &source_content)?;

    // Find and extract associated impl blocks
    let impl_blocks = find_impl_blocks_for_symbol(&source_ast, &symbol)?;
    let impl_code: Vec<String> = impl_blocks
        .iter()
        .map(|item| quote::quote!(#item).to_string())
        .collect();

    // Parse target file AST for better insertion
    let mut target_ast = if target_content.trim().is_empty() {
        syn::parse_file("// Module").unwrap_or_else(|_| syn::File {
            shebang: None,
            attrs: Vec::new(),
            items: Vec::new(),
        })
    } else {
        syn::parse_file(&target_content)
            .with_context(|| format!("Failed to parse target file: {}", target_path.display()))?
    };

    // Insert item into target AST
    insert_item_into_ast(&mut target_ast, &item_code)?;

    // Insert impl blocks after the item
    for impl_code_str in &impl_code {
        insert_item_into_ast(&mut target_ast, impl_code_str)?;
    }

    // Convert AST back to code
    let updated_target_content = quote::quote!(#target_ast).to_string();

    // Write target file
    fs::write(&target_path, &updated_target_content)
        .with_context(|| format!("Failed to write target file: {}", target_path.display()))?;

    // Remove item and impl blocks from source file
    let new_source_content =
        remove_item_and_impls_from_file(&source_ast, &symbol, &source_content)?;
    fs::write(&source_file, &new_source_content)
        .with_context(|| format!("Failed to update source file: {}", source_file.display()))?;

    // Update imports in all files that use this symbol
    update_imports_for_move(&symbol, &target, &working_directory, &mut backup_files)?;

    // Validate with cargo check
    let validation = validate_with_cargo_check()?;

    // If validation failed, rollback
    if !validation.cargo_check_passed {
        // Rollback changes
        for backup_file in &backup_files {
            if let Some(original_file) = backup_file.strip_suffix(".backup") {
                if let Err(e) = fs::copy(backup_file, original_file) {
                    eprintln!("Warning: Failed to rollback {}: {}", original_file, e);
                }
            }
        }

        return Ok(RefactorResult {
            operation: "move".to_string(),
            status: RefactorStatus::Failed,
            changes: preview_changes,
            validation: Some(validation),
            backup_files,
        });
    }

    Ok(RefactorResult {
        operation: "move".to_string(),
        status: RefactorStatus::Applied,
        changes: preview_changes,
        validation: Some(validation),
        backup_files,
    })
}

fn extract_item_code(ast: &syn::File, symbol: &str, _content: &str) -> anyhow::Result<String> {
    // Find the item and extract its code
    for item in &ast.items {
        if item_matches_symbol(item, symbol) {
            // Use quote to convert item back to code
            let item_code = quote::quote!(#item).to_string();
            // Format it nicely
            return Ok(item_code);
        }
    }

    anyhow::bail!("Item '{}' not found in AST", symbol);
}

/// Insert item code into AST (simplified - parses and adds to items)
fn insert_item_into_ast(ast: &mut syn::File, item_code: &str) -> anyhow::Result<()> {
    // Parse the item code
    let parsed: syn::File = syn::parse_str(item_code)
        .with_context(|| format!("Failed to parse item code: {}", item_code))?;

    // Add all items from parsed code to target AST
    ast.items.extend(parsed.items);

    Ok(())
}

/// Remove item and its impl blocks from file
fn remove_item_and_impls_from_file(
    ast: &syn::File,
    symbol: &str,
    _content: &str,
) -> anyhow::Result<String> {
    let mut new_items = Vec::new();

    for item in &ast.items {
        // Skip the item itself
        if item_matches_symbol(item, symbol) {
            continue;
        }

        // Skip impl blocks for this symbol
        if let Item::Impl(item_impl) = item {
            let should_skip = match &*item_impl.self_ty {
                syn::Type::Path(type_path) => type_path
                    .path
                    .get_ident()
                    .map(|ident| ident == symbol)
                    .unwrap_or(false),
                _ => false,
            };

            if should_skip {
                continue;
            }
        }

        new_items.push(item.clone());
    }

    let new_file = syn::File {
        shebang: ast.shebang.clone(),
        attrs: ast.attrs.clone(),
        items: new_items,
    };

    Ok(quote::quote!(#new_file).to_string())
}

fn update_imports_for_move(
    symbol: &str,
    target: &str,
    working_directory: &Path,
    backup_files: &mut Vec<String>,
) -> anyhow::Result<()> {
    // Find all files that import this symbol
    let mut files_to_update = Vec::new();

    // Walk the codebase to find all import statements
    walk_and_update_imports(working_directory, symbol, target, &mut files_to_update)?;

    // Update each file that imports the symbol
    for (file_path, new_import_path) in files_to_update {
        // Create backup if not already done
        let backup_path = format!("{}.backup", file_path.display());
        if !backup_files.contains(&backup_path) {
            fs::copy(&file_path, &backup_path)
                .with_context(|| format!("Failed to create backup: {}", backup_path))?;
            backup_files.push(backup_path.clone());
        }

        // Read file content
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Parse AST
        let mut ast = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;

        // Update import statements
        update_imports_in_ast(&mut ast, symbol, &new_import_path)?;

        // Write updated content
        let updated_content = quote::quote!(#ast).to_string();
        fs::write(&file_path, &updated_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    }

    Ok(())
}

/// Walk directory and find/update imports
fn walk_and_update_imports(
    dir: &Path,
    symbol: &str,
    target: &str,
    files_to_update: &mut Vec<(PathBuf, String)>,
) -> anyhow::Result<()> {
    use crate::core::refactor::ast_utils::find_import_statements;

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        if path.file_name().and_then(|n| n.to_str()) == Some("target") {
            continue;
        }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    // Check if this file imports the symbol
                    if let Ok(imports) = find_import_statements(&path, symbol) {
                        if !imports.is_empty() {
                            // Determine new import path
                            let new_path = build_new_import_path(target, symbol)?;
                            files_to_update.push((path, new_path));
                        }
                    }
                }
            }
        } else if path.is_dir() {
            walk_and_update_imports(&path, symbol, target, files_to_update)?;
        }
    }

    Ok(())
}

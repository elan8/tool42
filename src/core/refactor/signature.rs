use crate::core::refactor::ast_utils::{find_call_sites, find_item_definition, get_function_name};
use crate::core::refactor::types::{RefactorResult, RefactorStatus};
use crate::core::refactor::utils::walk_and_find_symbol;
use crate::core::refactor::validation::validate_with_cargo_check;
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    visit_mut::{self, VisitMut},
    Item,
};

/// Change a function signature and update all call sites
pub fn change_signature(
    function: String,
    new_signature: String,
    working_directory: PathBuf,
    preview: bool,
    apply: bool,
) -> anyhow::Result<RefactorResult> {
    // Find the function and all call sites
    let mut changes = Vec::new();
    let mut files_to_modify = HashMap::new();

    walk_and_find_symbol(
        &working_directory,
        &function,
        &mut changes,
        &mut files_to_modify,
    )?;

    if changes.is_empty() {
        anyhow::bail!("Function '{}' not found in the codebase", function);
    }

    // Build preview changes
    let mut preview_changes = changes.clone();
    for change in &mut preview_changes {
        change.old = format!("fn {}...", function);
        change.new = new_signature.clone();
        change.context = format!("Change signature of {}", function);
    }

    // If preview mode, return preview
    if preview && !apply {
        return Ok(RefactorResult {
            operation: "signature".to_string(),
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

    // Find the function definition
    let function_file = find_item_definition(&function, &files_to_modify)?;

    // Read function file
    let file_content = fs::read_to_string(&function_file)
        .with_context(|| format!("Failed to read file: {}", function_file.display()))?;

    // Parse file
    let mut ast = syn::parse_file(&file_content)
        .with_context(|| format!("Failed to parse file: {}", function_file.display()))?;

    // Find and update function signature
    let mut updated = false;
    for item in &mut ast.items {
        if let Item::Fn(ref mut item_fn) = item {
            if let Some(ident) = get_function_name(item_fn) {
                if ident == function {
                    // Parse new signature
                    let new_sig =
                        syn::parse_str::<syn::Signature>(&new_signature).with_context(|| {
                            format!("Failed to parse new signature: {}", new_signature)
                        })?;

                    // Update signature
                    item_fn.sig = new_sig;
                    updated = true;
                    break;
                }
            }
        }
    }

    if !updated {
        anyhow::bail!("Function '{}' definition not found", function);
    }

    // Convert AST back to code
    let updated_content = quote::quote!(#ast).to_string();

    // Write updated file
    fs::write(&function_file, &updated_content)
        .with_context(|| format!("Failed to write file: {}", function_file.display()))?;

    // Find and update all call sites
    update_call_sites_for_signature_change(
        &function,
        &new_signature,
        &working_directory,
        &backup_files,
    )?;

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
            operation: "signature".to_string(),
            status: RefactorStatus::Failed,
            changes: preview_changes,
            validation: Some(validation),
            backup_files,
        });
    }

    Ok(RefactorResult {
        operation: "signature".to_string(),
        status: RefactorStatus::Applied,
        changes: preview_changes,
        validation: Some(validation),
        backup_files,
    })
}

/// Update all call sites for a function signature change
fn update_call_sites_for_signature_change(
    function: &str,
    new_signature: &str,
    working_directory: &Path,
    backup_files: &[String],
) -> anyhow::Result<()> {
    // Parse new signature to extract parameter information
    let new_sig = syn::parse_str::<syn::Signature>(new_signature)
        .with_context(|| format!("Failed to parse new signature: {}", new_signature))?;

    // Find all call sites
    let call_sites = find_call_sites(function, working_directory)?;

    if call_sites.is_empty() {
        // No call sites to update
        return Ok(());
    }

    // Group call sites by file
    let mut files_to_update: HashMap<
        std::path::PathBuf,
        Vec<crate::core::refactor::types::CallSite>,
    > = HashMap::new();
    for call_site in call_sites {
        files_to_update
            .entry(call_site.file.clone())
            .or_default()
            .push(call_site);
    }

    // Update each file
    for (file_path, _call_sites_in_file) in files_to_update {
        // Create backup if not already done
        let backup_path = format!("{}.backup", file_path.display());
        if !backup_files.contains(&backup_path) {
            fs::copy(&file_path, &backup_path)
                .with_context(|| format!("Failed to create backup: {}", backup_path))?;
        }

        // Read file content
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Parse AST
        let mut ast = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;

        // Update call sites in AST
        update_call_sites_in_ast(&mut ast, function, &new_sig)?;

        // Write updated content
        let updated_content = quote::quote!(#ast).to_string();
        fs::write(&file_path, &updated_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    }

    Ok(())
}

/// Update call sites in AST
fn update_call_sites_in_ast(
    ast: &mut syn::File,
    function: &str,
    new_sig: &syn::Signature,
) -> anyhow::Result<()> {
    // We need the old signature to compare - find it first
    // For now, we'll use a simpler approach: update call sites based on new signature
    // If the number of parameters changed, we'll need to handle it

    // Use a visitor to update call sites
    let mut updater = CallSiteUpdater::new(function, new_sig);
    updater.visit_file_mut(ast);

    Ok(())
}

/// Visitor to update function call sites
struct CallSiteUpdater<'a> {
    function: &'a str,
    new_sig: &'a syn::Signature,
}

impl<'a> CallSiteUpdater<'a> {
    fn new(function: &'a str, new_sig: &'a syn::Signature) -> Self {
        Self { function, new_sig }
    }

    /// Get parameter count from new signature
    fn new_param_count(&self) -> usize {
        self.new_sig.inputs.len()
    }

    /// Update call site arguments to match new signature
    /// This is a simplified implementation that:
    /// - Keeps existing arguments if count matches
    /// - Adds default values (if possible) if new signature has more params
    /// - Removes extra arguments if new signature has fewer params
    fn update_call_arguments(&self, expr_call: &mut syn::ExprCall) {
        let old_arg_count = expr_call.args.len();
        let new_arg_count = self.new_param_count();

        if old_arg_count == new_arg_count {
            // Same number of arguments - no change needed
            // The types might have changed, but that will be caught by cargo check
        } else if old_arg_count < new_arg_count {
            // New signature has more parameters - we can't automatically add arguments
            // This would require knowing what values to use, which is complex
            // For now, we'll leave it and let cargo check catch the error
            // In a full implementation, we might try to infer default values
        } else {
            // New signature has fewer parameters - remove extra arguments
            // Keep only the first new_arg_count arguments
            // Create a new Punctuated with only the needed arguments
            let mut new_args = syn::punctuated::Punctuated::new();
            for (idx, pair) in expr_call.args.pairs().enumerate() {
                if idx < new_arg_count {
                    new_args.push((*pair.value()).clone());
                    if let Some(punct) = pair.punct() {
                        new_args.push_punct(**punct);
                    }
                } else {
                    break;
                }
            }
            expr_call.args = new_args;
        }
    }
}

impl<'a> VisitMut for CallSiteUpdater<'a> {
    fn visit_expr_call_mut(&mut self, expr_call: &mut syn::ExprCall) {
        // Check if this is a call to our function
        // Handle both simple identifiers and qualified paths
        let is_target_call = match &*expr_call.func {
            syn::Expr::Path(path_expr) => {
                // Check if the last segment matches our function name
                path_expr
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident == self.function)
                    .unwrap_or(false)
            }
            _ => false,
        };

        if is_target_call {
            // Update arguments based on new signature
            self.update_call_arguments(expr_call);
        }

        visit_mut::visit_expr_call_mut(self, expr_call);
    }
}

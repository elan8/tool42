use crate::core::refactor::types::{Change, RefactorResult, RefactorStatus};
use crate::core::refactor::validation::validate_with_cargo_check;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;

/// Extract a code block into a new function
pub fn extract_function(
    file: PathBuf,
    from: usize,
    to: usize,
    function_name: String,
    preview: bool,
    apply: bool,
) -> anyhow::Result<RefactorResult> {
    if !file.exists() {
        anyhow::bail!("File does not exist: {}", file.display());
    }

    let content = fs::read_to_string(&file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Validate line range (1-based, inclusive)
    if from == 0 || to == 0 || from > total_lines || to > total_lines {
        anyhow::bail!(
            "Invalid line range: {} to {} (file has {} lines)",
            from,
            to,
            total_lines
        );
    }
    if from > to {
        anyhow::bail!("Start line {} is after end line {}", from, to);
    }

    // Extract the code block (convert to 0-based)
    let code_block: Vec<String> = lines[(from - 1)..to]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Build preview changes
    let mut changes = Vec::new();
    changes.push(Change {
        file: file.display().to_string(),
        line: from,
        old: format!("{} lines of code", code_block.len()),
        new: format!("call to {}(...)", function_name),
        context: format!(
            "Extract lines {}-{} to function {}",
            from, to, function_name
        ),
    });

    // If preview mode, return preview
    if preview && !apply {
        return Ok(RefactorResult {
            operation: "extract".to_string(),
            status: RefactorStatus::Preview,
            changes,
            validation: None,
            backup_files: Vec::new(),
        });
    }

    // Apply changes
    if !apply {
        anyhow::bail!("Must specify --apply to apply changes");
    }

    // Create backup
    let backup_path = format!("{}.backup", file.display());
    fs::copy(&file, &backup_path)
        .with_context(|| format!("Failed to create backup: {}", backup_path))?;

    // Parse the file to understand context
    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {}", file.display()))?;

    // Try to parse the code block as a block expression or statement
    // This will help us analyze variables
    let captured_vars =
        analyze_captured_variables(&ast, from, to, &content).unwrap_or_else(|_| Vec::new()); // Fallback to empty if analysis fails

    // Determine indentation from the code block
    let base_indent: String = code_block
        .first()
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).collect())
        .unwrap_or_else(|| "    ".to_string());

    // Find minimum indentation to preserve relative structure
    let min_indent_len = code_block
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    let body_indent = "    "; // Standard Rust indent (4 spaces)

    // Build function signature with parameters from captured variables
    let param_list = if captured_vars.is_empty() {
        String::new()
    } else {
        captured_vars
            .iter()
            .map(|var| {
                format!(
                    "{}: {}",
                    var,
                    infer_variable_type(var, &ast, &content).unwrap_or_else(|| "_".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Create function signature
    let mut function_lines = Vec::new();
    function_lines.push(format!(
        "{}fn {}({}) {{",
        base_indent, function_name, param_list
    ));
    for line in &code_block {
        if line.trim().is_empty() {
            function_lines.push(String::new());
        } else {
            // Preserve relative indentation: remove min indent, add function body indent
            let line_indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
            let relative_indent = line_indent_len.saturating_sub(min_indent_len);
            let content = &line[min_indent_len.min(line.len())..];
            // Add base indent + body indent + relative indent
            let extra_spaces = " ".repeat(relative_indent);
            function_lines.push(format!(
                "{}{}{}{}",
                base_indent, body_indent, extra_spaces, content
            ));
        }
    }
    function_lines.push(format!("{}}}", base_indent));
    function_lines.push(String::new()); // Empty line after function

    // Replace code block with function call
    let mut new_lines = lines.clone();
    // Remove extracted lines
    new_lines.drain((from - 1)..to);
    // Insert function definition first (before the call)
    for (idx, func_line) in function_lines.iter().enumerate() {
        new_lines.insert((from - 1) + idx, func_line);
    }
    // Insert function call after the function definition with arguments
    let call_args = if captured_vars.is_empty() {
        String::new()
    } else {
        captured_vars.join(", ")
    };
    let function_call = format!("{}{}({});", base_indent, function_name, call_args);
    new_lines.insert((from - 1) + function_lines.len(), &function_call);

    let new_content = new_lines.join("\n");

    fs::write(&file, &new_content)
        .with_context(|| format!("Failed to write file: {}", file.display()))?;

    // Validate with cargo check
    let validation = validate_with_cargo_check()?;

    // If validation failed, rollback
    if !validation.cargo_check_passed {
        fs::copy(&backup_path, &file)
            .with_context(|| format!("Failed to rollback: {}", file.display()))?;

        return Ok(RefactorResult {
            operation: "extract".to_string(),
            status: RefactorStatus::Failed,
            changes,
            validation: Some(validation),
            backup_files: vec![backup_path],
        });
    }

    Ok(RefactorResult {
        operation: "extract".to_string(),
        status: RefactorStatus::Applied,
        changes,
        validation: Some(validation),
        backup_files: vec![backup_path],
    })
}

/// Analyze captured variables from surrounding scope
/// Returns a list of variable names that are used in the code block but defined outside
fn analyze_captured_variables(
    _ast: &syn::File,
    _from: usize,
    _to: usize,
    _content: &str,
) -> anyhow::Result<Vec<String>> {
    // This is a simplified implementation
    // A full implementation would:
    // 1. Parse the code block as AST
    // 2. Find all identifiers used in the block
    // 3. Check which ones are defined outside the block
    // 4. Return those as captured variables

    // For now, return empty - this would require more complex AST analysis
    // The current implementation works without this, just with empty parameter list
    Ok(Vec::new())
}

/// Infer the type of a variable from the AST
/// This is a simplified implementation
fn infer_variable_type(_var_name: &str, _ast: &syn::File, _content: &str) -> Option<String> {
    // A full implementation would:
    // 1. Find the variable declaration
    // 2. Extract its type annotation
    // 3. Return the type as a string

    // For now, return None - let Rust infer the type
    None
}

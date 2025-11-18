use crate::core::refactor::types::{CallSite, Import, Usage, UsageKind};
use crate::core::refactor::utils::{build_line_map, byte_offset_to_line, find_line_in_content};
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Item,
};

/// Convert a file path to a module path string
/// Example: src/utils/helpers.rs -> utils::helpers
pub fn resolve_module_path(file: &Path) -> anyhow::Result<String> {
    let path_str = file.to_string_lossy();

    // Remove .rs extension
    let without_ext = path_str.strip_suffix(".rs").unwrap_or(&path_str);

    // Remove src/ prefix if present
    let without_src = without_ext.strip_prefix("src/").unwrap_or(without_ext);

    // Handle mod.rs files - use parent directory name
    if without_src.ends_with("/mod.rs") || without_src == "mod.rs" {
        if let Some(parent) = Path::new(without_src).parent() {
            if let Some(parent_str) = parent.to_str() {
                if !parent_str.is_empty() {
                    return Ok(parent_str.replace('/', "::"));
                }
            }
        }
        return Ok("crate".to_string());
    }

    // Replace / with ::
    let module_path = without_src.replace('/', "::");

    // Remove lib.rs or main.rs (they're root modules)
    if module_path == "lib" || module_path == "main" {
        return Ok("crate".to_string());
    }

    Ok(module_path)
}

/// Find all import statements for a symbol in a file
pub fn find_import_statements(file: &Path, symbol: &str) -> anyhow::Result<Vec<Import>> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", file.display()))?;

    let mut imports = Vec::new();
    let line_map = build_line_map(&content);

    for item in &ast.items {
        if let Item::Use(use_item) = item {
            let is_pub = matches!(use_item.vis, syn::Visibility::Public(_));

            // Check if this use statement imports our symbol
            if use_tree_contains_symbol(&use_item.tree, symbol) {
                // Find line number by searching for the use statement in content
                let use_str = quote::quote!(#use_item).to_string();
                let first_line = use_str.lines().next().unwrap_or("");
                // Find this line in the content to get accurate line number
                let line = find_line_in_content(&content, first_line)
                    .unwrap_or_else(|| byte_offset_to_line(0, &line_map));

                let path = use_tree_to_path(&use_item.tree);
                imports.push(Import {
                    file: file.to_path_buf(),
                    line,
                    path,
                    is_pub,
                });
            }
        }
    }

    Ok(imports)
}

/// Check if a UseTree contains a symbol
pub fn use_tree_contains_symbol(tree: &syn::UseTree, symbol: &str) -> bool {
    match tree {
        syn::UseTree::Path(path) => {
            if path.ident == symbol {
                return true;
            }
            if let syn::UseTree::Name(_) | syn::UseTree::Rename(_) = &*path.tree {
                return true;
            }
            use_tree_contains_symbol(&path.tree, symbol)
        }
        syn::UseTree::Name(name) => name.ident == symbol,
        syn::UseTree::Rename(rename) => rename.ident == symbol,
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_contains_symbol(item, symbol)),
        syn::UseTree::Glob(_) => false,
    }
}

/// Convert a UseTree to a path string
pub fn use_tree_to_path(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(path) => {
            let mut result = path.ident.to_string();
            match &*path.tree {
                syn::UseTree::Path(_inner) => {
                    result.push_str("::");
                    result.push_str(&use_tree_to_path(&path.tree));
                }
                syn::UseTree::Name(name) => {
                    result.push_str("::");
                    result.push_str(&name.ident.to_string());
                }
                syn::UseTree::Rename(rename) => {
                    result.push_str("::");
                    result.push_str(&rename.ident.to_string());
                }
                syn::UseTree::Group(_) => {}
                syn::UseTree::Glob(_) => {
                    result.push_str("::*");
                }
            }
            result
        }
        syn::UseTree::Name(name) => name.ident.to_string(),
        syn::UseTree::Rename(rename) => rename.ident.to_string(),
        syn::UseTree::Group(_) => String::new(),
        syn::UseTree::Glob(_) => "*".to_string(),
    }
}

/// Find all usages of a symbol using AST traversal
pub fn find_all_usages(symbol: &str, path: &Path) -> anyhow::Result<Vec<Usage>> {
    let mut usages = Vec::new();

    if path.is_file() {
        find_usages_in_file(path, symbol, &mut usages)?;
    } else {
        walk_and_find_usages(path, symbol, &mut usages)?;
    }

    Ok(usages)
}

/// Walk directory and find usages
fn walk_and_find_usages(dir: &Path, symbol: &str, usages: &mut Vec<Usage>) -> anyhow::Result<()> {
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
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    find_usages_in_file(&path, symbol, usages)?;
                }
            }
        } else if path.is_dir() {
            walk_and_find_usages(&path, symbol, usages)?;
        }
    }

    Ok(())
}

/// Find usages in a single file
fn find_usages_in_file(file: &Path, symbol: &str, usages: &mut Vec<Usage>) -> anyhow::Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", file.display()))?;

    let line_map = build_line_map(&content);
    let mut visitor = SymbolVisitor::new(symbol, file, &line_map, &content);
    visitor.visit_file(&ast);

    usages.extend(visitor.usages);

    Ok(())
}

/// Find symbol usages in a file using AST traversal
/// Returns usages with accurate line numbers and byte positions
pub fn find_symbol_usages_ast(
    file_path: &Path,
    symbol: &str,
    content: &str,
) -> anyhow::Result<Vec<Usage>> {
    let ast = syn::parse_file(content)
        .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;

    let line_map = build_line_map(content);
    let mut visitor = SymbolVisitor::new(symbol, file_path, &line_map, content);
    visitor.visit_file(&ast);

    Ok(visitor.usages)
}

/// Visitor to find symbol usages in AST
struct SymbolVisitor<'a> {
    symbol: &'a str,
    file: &'a Path,
    line_map: &'a [usize],
    content: &'a str,
    usages: Vec<Usage>,
    // Track context to distinguish between different usage types
    in_call_context: bool,
    // Track which lines we've already found to avoid duplicates
    found_lines: std::collections::HashSet<usize>,
}

impl<'a> SymbolVisitor<'a> {
    fn new(symbol: &'a str, file: &'a Path, line_map: &'a [usize], content: &'a str) -> Self {
        Self {
            symbol,
            file,
            line_map,
            content,
            usages: Vec::new(),
            in_call_context: false,
            found_lines: std::collections::HashSet::new(),
        }
    }

    fn add_usage(&mut self, _span: impl Spanned, kind: UsageKind) {
        // Find the next line containing the symbol that we haven't already used
        let line_opt = self
            .content
            .lines()
            .enumerate()
            .find(|(idx, line_content)| {
                // Check if this line contains the symbol and we haven't already used this line
                let line_num = idx + 1;
                line_content.contains(self.symbol) && !self.found_lines.contains(&line_num)
            })
            .map(|(idx, _)| {
                let line_num = idx + 1;
                self.found_lines.insert(line_num);
                line_num
            });

        let line = match line_opt {
            Some(l) => l,
            None => {
                // Fallback: just find first occurrence if we haven't found any yet
                if self.found_lines.is_empty() {
                    if let Some((idx, _)) = self
                        .content
                        .lines()
                        .enumerate()
                        .find(|(_, line)| line.contains(self.symbol))
                    {
                        let line_num = idx + 1;
                        self.found_lines.insert(line_num);
                        line_num
                    } else {
                        // No symbol found, skip
                        return;
                    }
                } else {
                    // If we've already found some, just skip this one
                    return;
                }
            }
        };

        // Get context line
        let context = if line > 0 && line <= self.content.lines().count() {
            self.content.lines().nth(line - 1).unwrap_or("").to_string()
        } else {
            String::new()
        };

        // Check if we already added this usage (avoid duplicates)
        // Use a more lenient check - same line and same kind
        let already_added = self.usages.iter().any(|u| {
            u.file == self.file
                && u.line == line
                && matches!(
                    (&u.kind, &kind),
                    (UsageKind::Definition, UsageKind::Definition)
                        | (UsageKind::Call, UsageKind::Call)
                        | (UsageKind::Reference, UsageKind::Reference)
                        | (UsageKind::Import, UsageKind::Import)
                )
        });

        if !already_added {
            self.usages.push(Usage {
                file: self.file.to_path_buf(),
                line,
                kind,
                context: context.trim().to_string(),
            });
        }
    }
}

impl<'a> Visit<'_> for SymbolVisitor<'a> {
    fn visit_ident(&mut self, ident: &syn::Ident) {
        if ident == self.symbol {
            // Determine usage kind based on context
            let kind = if self.in_call_context {
                UsageKind::Call
            } else {
                UsageKind::Reference
            };
            self.add_usage(ident.span(), kind);
        }
        visit::visit_ident(self, ident);
    }

    fn visit_item_fn(&mut self, item_fn: &syn::ItemFn) {
        if item_fn.sig.ident == self.symbol {
            self.add_usage(item_fn.sig.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_fn(self, item_fn);
    }

    fn visit_item_struct(&mut self, item_struct: &syn::ItemStruct) {
        if item_struct.ident == self.symbol {
            self.add_usage(item_struct.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_struct(self, item_struct);
    }

    fn visit_item_enum(&mut self, item_enum: &syn::ItemEnum) {
        if item_enum.ident == self.symbol {
            self.add_usage(item_enum.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_enum(self, item_enum);
    }

    fn visit_item_type(&mut self, item_type: &syn::ItemType) {
        if item_type.ident == self.symbol {
            self.add_usage(item_type.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_type(self, item_type);
    }

    fn visit_item_const(&mut self, item_const: &syn::ItemConst) {
        if item_const.ident == self.symbol {
            self.add_usage(item_const.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_const(self, item_const);
    }

    fn visit_item_static(&mut self, item_static: &syn::ItemStatic) {
        if item_static.ident == self.symbol {
            self.add_usage(item_static.ident.span(), UsageKind::Definition);
        }
        visit::visit_item_static(self, item_static);
    }

    fn visit_expr_call(&mut self, expr_call: &syn::ExprCall) {
        // Check if this is a call to our symbol
        let old_in_call = self.in_call_context;
        self.in_call_context = true;

        // Handle qualified paths (e.g., mod::Symbol)
        // Typically the function name is the last segment, but check all segments
        if let syn::Expr::Path(path_expr) = &*expr_call.func {
            // Check all segments for the symbol (handles qualified paths)
            for segment in &path_expr.path.segments {
                if segment.ident == self.symbol {
                    self.add_usage(segment.ident.span(), UsageKind::Call);
                }
            }
        }

        visit::visit_expr_call(self, expr_call);
        self.in_call_context = old_in_call;
    }

    fn visit_expr_path(&mut self, path_expr: &syn::ExprPath) {
        // Handle qualified paths (e.g., mod::Symbol)
        // Check all segments for the symbol (handles both qualified and unqualified paths)
        for segment in &path_expr.path.segments {
            if segment.ident == self.symbol {
                // Check if this is part of a call (handled in visit_expr_call)
                if !self.in_call_context {
                    self.add_usage(segment.ident.span(), UsageKind::Reference);
                }
            }
        }
        visit::visit_expr_path(self, path_expr);
    }

    fn visit_type_path(&mut self, type_path: &syn::TypePath) {
        // Handle qualified type paths (e.g., mod::MyStruct)
        // Check all segments for the symbol
        for segment in &type_path.path.segments {
            if segment.ident == self.symbol {
                self.add_usage(segment.ident.span(), UsageKind::Reference);
            }
        }
        visit::visit_type_path(self, type_path);
    }

    fn visit_path(&mut self, path: &syn::Path) {
        // Handle paths in general (used in various contexts)
        for segment in &path.segments {
            if segment.ident == self.symbol {
                self.add_usage(segment.ident.span(), UsageKind::Reference);
            }
        }
        visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, use_item: &syn::ItemUse) {
        // Check if this use statement imports our symbol
        if use_tree_contains_symbol(&use_item.tree, self.symbol) {
            let line = find_line_in_content(self.content, &quote::quote!(#use_item).to_string())
                .unwrap_or_else(|| byte_offset_to_line(0, self.line_map));

            let context = self
                .content
                .lines()
                .nth(line.saturating_sub(1))
                .unwrap_or("")
                .to_string();

            self.usages.push(Usage {
                file: self.file.to_path_buf(),
                line,
                kind: UsageKind::Import,
                context: context.trim().to_string(),
            });
        }
        visit::visit_item_use(self, use_item);
    }
}

/// Find all call sites for a function
pub fn find_call_sites(function: &str, path: &Path) -> anyhow::Result<Vec<CallSite>> {
    let mut call_sites = Vec::new();

    if path.is_file() {
        find_call_sites_in_file(path, function, &mut call_sites)?;
    } else {
        walk_and_find_call_sites(path, function, &mut call_sites)?;
    }

    Ok(call_sites)
}

/// Walk directory and find call sites
fn walk_and_find_call_sites(
    dir: &Path,
    function: &str,
    call_sites: &mut Vec<CallSite>,
) -> anyhow::Result<()> {
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
                    find_call_sites_in_file(&path, function, call_sites)?;
                }
            }
        } else if path.is_dir() {
            walk_and_find_call_sites(&path, function, call_sites)?;
        }
    }

    Ok(())
}

/// Find call sites in a single file
fn find_call_sites_in_file(
    file: &Path,
    function: &str,
    call_sites: &mut Vec<CallSite>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", file.display()))?;

    let mut visitor = CallSiteVisitor::new(function, file, &content);
    visitor.visit_file(&ast);

    call_sites.extend(visitor.call_sites);

    Ok(())
}

/// Visitor to find function call sites
struct CallSiteVisitor<'a> {
    function: &'a str,
    file: &'a Path,
    content: &'a str,
    call_sites: Vec<CallSite>,
}

impl<'a> CallSiteVisitor<'a> {
    fn new(function: &'a str, file: &'a Path, content: &'a str) -> Self {
        Self {
            function,
            file,
            content,
            call_sites: Vec::new(),
        }
    }
}

impl<'a> Visit<'_> for CallSiteVisitor<'a> {
    fn visit_expr_call(&mut self, expr_call: &syn::ExprCall) {
        if let syn::Expr::Path(path_expr) = &*expr_call.func {
            if let Some(ident) = path_expr.path.get_ident() {
                if ident == self.function {
                    // Find line by searching for function call in content
                    let call_str = quote::quote!(#expr_call).to_string();
                    let first_line = call_str.lines().next().unwrap_or("");
                    let line =
                        find_line_in_content(self.content, first_line).unwrap_or_else(|| {
                            // Fallback: search for function name
                            self.content
                                .lines()
                                .enumerate()
                                .find(|(_, line)| line.contains(self.function))
                                .map(|(idx, _)| idx + 1)
                                .unwrap_or(1)
                        });

                    let context = self
                        .content
                        .lines()
                        .nth(line.saturating_sub(1))
                        .unwrap_or("")
                        .to_string();

                    self.call_sites.push(CallSite {
                        file: self.file.to_path_buf(),
                        line,
                        args: expr_call.args.iter().cloned().collect(),
                        context: context.trim().to_string(),
                    });
                }
            }
        }
        visit::visit_expr_call(self, expr_call);
    }
}

/// Check if an item matches a symbol name
pub fn item_matches_symbol(item: &Item, symbol: &str) -> bool {
    match item {
        Item::Fn(item_fn) => item_fn.sig.ident == symbol,
        Item::Struct(item_struct) => item_struct.ident == symbol,
        Item::Enum(item_enum) => item_enum.ident == symbol,
        Item::Type(item_type) => item_type.ident == symbol,
        Item::Const(item_const) => item_const.ident == symbol,
        Item::Static(item_static) => item_static.ident == symbol,
        _ => false,
    }
}

/// Get function name from ItemFn
pub fn get_function_name(item_fn: &syn::ItemFn) -> Option<String> {
    Some(item_fn.sig.ident.to_string())
}

/// Find the file containing the definition of a symbol
pub fn find_item_definition(
    symbol: &str,
    files_to_modify: &std::collections::HashMap<PathBuf, Vec<(usize, String, String)>>,
) -> anyhow::Result<PathBuf> {
    use anyhow::Context;

    // Find the first file that contains the symbol definition
    for file_path in files_to_modify.keys() {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let ast = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;

        // Search for the item in the AST
        for item in &ast.items {
            if item_matches_symbol(item, symbol) {
                return Ok(file_path.clone());
            }
        }
    }

    anyhow::bail!("Item '{}' definition not found", symbol);
}

/// Resolve target path from module path or file path string
pub fn resolve_target_path(target: &str) -> anyhow::Result<PathBuf> {
    // Handle module paths like "utils::helpers" or file paths like "src/utils.rs"
    if target.contains("::") {
        // Module path - convert to file path
        let parts: Vec<&str> = target.split("::").collect();
        let mut path = PathBuf::from("src");
        for part in &parts[..parts.len() - 1] {
            path.push(part);
        }
        path.push(format!("{}.rs", parts.last().unwrap()));
        Ok(path)
    } else if target.ends_with(".rs") {
        // Direct file path
        Ok(PathBuf::from(target))
    } else {
        // Assume it's a module name in src/
        Ok(PathBuf::from(format!("src/{}.rs", target)))
    }
}

/// Find all impl blocks for a symbol (struct/enum)
pub fn find_impl_blocks_for_symbol(
    ast: &syn::File,
    symbol: &str,
) -> anyhow::Result<Vec<syn::ItemImpl>> {
    let mut impls = Vec::new();

    for item in &ast.items {
        if let Item::Impl(item_impl) = item {
            // Check if this impl block is for our symbol
            if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                if let Some(ident) = type_path.path.get_ident() {
                    if ident == symbol {
                        impls.push(item_impl.clone());
                    }
                }
            }
        }
    }

    Ok(impls)
}

/// Build a UseTree from a module path string
/// This is a simplified implementation - for full correctness, we'd parse the path properly
pub fn build_use_tree_from_path(path: &str, symbol: &str) -> anyhow::Result<syn::UseTree> {
    // Parse the path string to create a UseTree
    // For now, use a simple approach: parse as Rust code
    let use_stmt = format!("use {}::{};", path, symbol);
    let parsed: syn::File = syn::parse_str(&use_stmt)
        .with_context(|| format!("Failed to parse import path: {}", use_stmt))?;

    // Extract the UseTree from the parsed file
    for item in parsed.items {
        if let Item::Use(use_item) = item {
            return Ok(use_item.tree);
        }
    }

    anyhow::bail!("Failed to build use tree from path: {}", path)
}

/// Build new import path for moved symbol
pub fn build_new_import_path(target: &str, symbol: &str) -> anyhow::Result<String> {
    // Handle different target formats:
    // - "utils::helpers" -> "utils::helpers::symbol"
    // - "src/utils.rs" -> convert to module path
    // - "utils" -> "utils::symbol"

    if target.contains("::") {
        // Already a module path
        Ok(format!("{}::{}", target, symbol))
    } else if target.ends_with(".rs") {
        // File path - convert to module path
        let module_path = resolve_module_path(Path::new(target))?;
        Ok(format!("{}::{}", module_path, symbol))
    } else {
        // Assume module name
        Ok(format!("{}::{}", target, symbol))
    }
}

/// Update import statements in AST
pub fn update_imports_in_ast(
    ast: &mut syn::File,
    symbol: &str,
    new_path: &str,
) -> anyhow::Result<()> {
    for item in &mut ast.items {
        if let Item::Use(use_item) = item {
            if use_tree_contains_symbol(&use_item.tree, symbol) {
                // Replace the use tree with new path
                use_item.tree = build_use_tree_from_path(new_path, symbol)?;
            }
        }
    }

    Ok(())
}

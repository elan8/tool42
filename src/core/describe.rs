use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use syn::{ImplItemFn, Item, TraitItemFn, Visibility};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDescription {
    pub file: String,
    pub items: Vec<ItemDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDescription {
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<ItemDescription>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldDescription>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDescription {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    pub start_line: usize,
}

pub fn describe_file(path: PathBuf) -> anyhow::Result<FileDescription> {
    // Read file content
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse Rust file
    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {}", path.display()))?;

    // Build line map for converting byte offsets to line numbers
    let line_map = build_line_map(&content);

    // Extract items
    let items = extract_items(&ast.items, &content, &line_map);

    // Build description
    Ok(FileDescription {
        file: path.display().to_string(),
        items,
    })
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

fn extract_visibility(vis: &Visibility) -> Option<String> {
    match vis {
        Visibility::Public(_) => Some("pub".to_string()),
        Visibility::Restricted(restricted) => {
            let path = quote::quote!(#restricted.path).to_string();
            Some(format!(
                "pub({})",
                path.trim_matches('"').trim_matches('\'')
            ))
        }
        Visibility::Inherited => None,
    }
}

fn extract_attributes(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            // Extract doc comments separately, so skip them here
            if attr.path().is_ident("doc") {
                return None;
            }
            Some(quote::quote!(#attr).to_string().trim().to_string())
        })
        .collect()
}

fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let doc_lines: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &meta.value
                    {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .collect();

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

fn find_item_start_position(item: &Item, content: &str, _line_map: &[usize]) -> Option<usize> {
    // Try to find the item by searching for its unique identifier
    // Use a more robust approach: search backwards from likely positions
    let search_patterns = match item {
        Item::Struct(s) => vec![
            format!("struct {}", s.ident),
            format!("pub struct {}", s.ident),
            format!("pub(crate) struct {}", s.ident),
        ],
        Item::Enum(e) => vec![
            format!("enum {}", e.ident),
            format!("pub enum {}", e.ident),
            format!("pub(crate) enum {}", e.ident),
        ],
        Item::Fn(f) => vec![
            format!("fn {}", f.sig.ident),
            format!("pub fn {}", f.sig.ident),
            format!("pub(crate) fn {}", f.sig.ident),
            format!("async fn {}", f.sig.ident),
            format!("pub async fn {}", f.sig.ident),
            format!("const fn {}", f.sig.ident),
            format!("pub const fn {}", f.sig.ident),
            format!("unsafe fn {}", f.sig.ident),
            format!("pub unsafe fn {}", f.sig.ident),
        ],
        Item::Impl(_) => vec!["impl".to_string(), "pub impl".to_string()],
        Item::Trait(t) => vec![
            format!("trait {}", t.ident),
            format!("pub trait {}", t.ident),
        ],
        Item::Mod(m) => vec![format!("mod {}", m.ident), format!("pub mod {}", m.ident)],
        Item::Type(t) => vec![format!("type {}", t.ident), format!("pub type {}", t.ident)],
        Item::Const(c) => vec![
            format!("const {}", c.ident),
            format!("pub const {}", c.ident),
        ],
        Item::Static(s) => vec![
            format!("static {}", s.ident),
            format!("pub static {}", s.ident),
        ],
        Item::Macro(m) => {
            if let Some(ident) = &m.ident {
                vec![
                    format!("macro_rules! {}", ident),
                    format!("pub macro_rules! {}", ident),
                ]
            } else {
                vec!["macro_rules!".to_string()]
            }
        }
        Item::Use(_) => vec!["use ".to_string()],
        Item::ExternCrate(_) => vec!["extern crate".to_string()],
        Item::Union(u) => vec![
            format!("union {}", u.ident),
            format!("pub union {}", u.ident),
        ],
        _ => return None,
    };

    // Find the first occurrence of any pattern
    for pattern in search_patterns {
        if let Some(pos) = content.find(&pattern) {
            return Some(pos);
        }
    }

    None
}

fn find_item_end_position(start_pos: usize, content: &str, _line_map: &[usize]) -> usize {
    // Find the end by matching braces/brackets
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut in_char = false;
    let mut in_comment = false;
    let mut comment_type = None; // None, Some('/'), Some('*')

    let mut pos = start_pos;

    while pos < content.len() {
        let ch = content.as_bytes()[pos];
        let next_ch = if pos + 1 < content.len() {
            Some(content.as_bytes()[pos + 1])
        } else {
            None
        };

        if escape_next {
            escape_next = false;
            pos += 1;
            continue;
        }

        // Handle comments
        if !in_string && !in_char {
            if ch == b'/' && next_ch == Some(b'/') {
                // Line comment - skip to end of line
                while pos < content.len() && content.as_bytes()[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            } else if ch == b'/' && next_ch == Some(b'*') {
                in_comment = true;
                comment_type = Some('*');
                pos += 2;
                continue;
            } else if in_comment && comment_type == Some('*') && ch == b'*' && next_ch == Some(b'/')
            {
                in_comment = false;
                comment_type = None;
                pos += 2;
                continue;
            }
        }

        if in_comment {
            pos += 1;
            continue;
        }

        // Handle strings and chars
        if ch == b'\\' && (in_string || in_char) {
            escape_next = true;
            pos += 1;
            continue;
        }

        if ch == b'"' && !in_char {
            in_string = !in_string;
        } else if ch == b'\'' && !in_string {
            in_char = !in_char;
        }

        if in_string || in_char {
            pos += 1;
            continue;
        }

        // Track braces for depth
        match ch {
            b'{' | b'(' | b'[' => {
                depth += 1;
            }
            b'}' | b')' | b']' => {
                depth -= 1;
                if depth < 0 {
                    // We've closed more than we opened, we're past the item
                    break;
                }
            }
            b'\n' => {
                // Track newlines for potential use
            }
            _ => {}
        }

        // If we're at depth 0 and hit a semicolon or new item, we might be done
        if depth == 0 {
            if ch == b';' {
                // Check if this looks like the end of an item
                let remaining = &content[pos + 1..];
                let trimmed = remaining.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('\n') || trimmed.starts_with("//") {
                    pos += 1;
                    break;
                }
            } else if ch == b'\n' {
                // Check if next non-whitespace looks like a new top-level item
                let remaining = &content[pos + 1..];
                let trimmed = remaining.trim_start();
                if trimmed.starts_with("pub ")
                    || trimmed.starts_with("fn ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("enum ")
                    || trimmed.starts_with("impl ")
                    || trimmed.starts_with("trait ")
                    || trimmed.starts_with("mod ")
                    || trimmed.starts_with("type ")
                    || trimmed.starts_with("const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("use ")
                    || trimmed.starts_with("macro_rules!")
                {
                    break;
                }
            }
        }

        pos += 1;
    }

    // Use the last position we found, or the start position if we didn't find anything
    if pos > start_pos {
        pos
    } else {
        start_pos + 100 // Fallback: estimate 100 bytes
    }
}

fn extract_items(items: &[Item], content: &str, line_map: &[usize]) -> Vec<ItemDescription> {
    let mut result = Vec::new();

    for item in items {
        if let Some(desc) = extract_item(item, content, line_map) {
            result.push(desc);
        }
    }

    result
}

fn find_item_lines(item: &Item, content: &str, line_map: &[usize]) -> (usize, usize) {
    if let Some(start_pos) = find_item_start_position(item, content, line_map) {
        let start_line = byte_offset_to_line(start_pos, line_map);
        let end_pos = find_item_end_position(start_pos, content, line_map);
        let end_line = byte_offset_to_line(end_pos, line_map);
        (start_line, end_line)
    } else {
        (1, 1) // Fallback
    }
}

fn find_item_lines_impl(item: &ImplItemFn, content: &str, line_map: &[usize]) -> (usize, usize) {
    let search_patterns = vec![
        format!("fn {}", item.sig.ident),
        format!("pub fn {}", item.sig.ident),
        format!("async fn {}", item.sig.ident),
        format!("pub async fn {}", item.sig.ident),
        format!("const fn {}", item.sig.ident),
        format!("pub const fn {}", item.sig.ident),
        format!("unsafe fn {}", item.sig.ident),
        format!("pub unsafe fn {}", item.sig.ident),
    ];

    for pattern in search_patterns {
        if let Some(pos) = content.find(&pattern) {
            let start_line = byte_offset_to_line(pos, line_map);
            let end_pos = find_item_end_position(pos, content, line_map);
            let end_line = byte_offset_to_line(end_pos, line_map);
            return (start_line, end_line);
        }
    }

    (1, 1) // Fallback
}

fn find_item_lines_trait_fn(
    item: &TraitItemFn,
    content: &str,
    line_map: &[usize],
) -> (usize, usize) {
    let search_patterns = vec![
        format!("fn {}", item.sig.ident),
        format!("async fn {}", item.sig.ident),
        format!("const fn {}", item.sig.ident),
        format!("unsafe fn {}", item.sig.ident),
    ];

    for pattern in search_patterns {
        if let Some(pos) = content.find(&pattern) {
            let start_line = byte_offset_to_line(pos, line_map);
            let end_pos = find_item_end_position(pos, content, line_map);
            let end_line = byte_offset_to_line(end_pos, line_map);
            return (start_line, end_line);
        }
    }

    (1, 1) // Fallback
}

fn extract_struct_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    content: &str,
    line_map: &[usize],
) -> Vec<FieldDescription> {
    let mut result = Vec::new();

    for field in fields.iter() {
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| quote::quote!(#field.ty).to_string());

        let field_type = Some(quote::quote!(#field.ty).to_string());

        // Try to find the field's line number
        let start_line = if let Some(ident) = &field.ident {
            let search_str = format!("{}:", ident);
            content
                .find(&search_str)
                .map(|pos| byte_offset_to_line(pos, line_map))
                .unwrap_or(1)
        } else {
            1
        };

        result.push(FieldDescription {
            name: field_name,
            field_type,
            start_line,
        });
    }

    result
}

fn extract_item(item: &Item, content: &str, line_map: &[usize]) -> Option<ItemDescription> {
    let (start_line, end_line) = find_item_lines(item, content, line_map);

    match item {
        Item::Struct(item_struct) => {
            let attrs = extract_attributes(&item_struct.attrs);
            let doc_comment = extract_doc_comment(&item_struct.attrs);
            let fields = match &item_struct.fields {
                syn::Fields::Named(fields) => {
                    let field_descs = extract_struct_fields(&fields.named, content, line_map);
                    if field_descs.is_empty() {
                        None
                    } else {
                        Some(field_descs)
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let field_descs = extract_struct_fields(&fields.unnamed, content, line_map);
                    if field_descs.is_empty() {
                        None
                    } else {
                        Some(field_descs)
                    }
                }
                syn::Fields::Unit => None,
            };

            Some(ItemDescription {
                item_type: "struct".to_string(),
                name: item_struct.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_struct.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields,
            })
        }
        Item::Enum(item_enum) => {
            let attrs = extract_attributes(&item_enum.attrs);
            let doc_comment = extract_doc_comment(&item_enum.attrs);
            let fields: Vec<FieldDescription> = item_enum
                .variants
                .iter()
                .map(|variant| {
                    let start_line = if let Some(pos) = content.find(&format!("{}", variant.ident))
                    {
                        byte_offset_to_line(pos, line_map)
                    } else {
                        1
                    };
                    FieldDescription {
                        name: variant.ident.to_string(),
                        field_type: None,
                        start_line,
                    }
                })
                .collect();

            Some(ItemDescription {
                item_type: "enum".to_string(),
                name: item_enum.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_enum.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                },
            })
        }
        Item::Fn(item_fn) => {
            let attrs = extract_attributes(&item_fn.attrs);
            let doc_comment = extract_doc_comment(&item_fn.attrs);
            let signature = Some(quote::quote!(#item_fn.sig).to_string());

            Some(ItemDescription {
                item_type: "function".to_string(),
                name: item_fn.sig.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_fn.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature,
                fields: None,
            })
        }
        Item::Impl(item_impl) => {
            let target = if let Some((_, path, _)) = &item_impl.trait_ {
                Some(path.segments.last().unwrap().ident.to_string())
            } else if let syn::Type::Path(type_path) = item_impl.self_ty.as_ref() {
                Some(type_path.path.segments.last().unwrap().ident.to_string())
            } else {
                None
            };

            let attrs = extract_attributes(&item_impl.attrs);
            let doc_comment = extract_doc_comment(&item_impl.attrs);

            let nested_items: Vec<ItemDescription> = item_impl
                .items
                .iter()
                .filter_map(|impl_item| {
                    match impl_item {
                        syn::ImplItem::Fn(impl_item_fn) => {
                            let (start_line, end_line) =
                                find_item_lines_impl(impl_item_fn, content, line_map);
                            let attrs = extract_attributes(&impl_item_fn.attrs);
                            let doc_comment = extract_doc_comment(&impl_item_fn.attrs);
                            let signature = Some(quote::quote!(#impl_item_fn.sig).to_string());

                            Some(ItemDescription {
                                item_type: "function".to_string(),
                                name: impl_item_fn.sig.ident.to_string(),
                                start_line,
                                end_line,
                                target: None,
                                items: None,
                                visibility: extract_visibility(&impl_item_fn.vis),
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature,
                                fields: None,
                            })
                        }
                        syn::ImplItem::Const(impl_const) => {
                            let attrs = extract_attributes(&impl_const.attrs);
                            let doc_comment = extract_doc_comment(&impl_const.attrs);

                            Some(ItemDescription {
                                item_type: "constant".to_string(),
                                name: impl_const.ident.to_string(),
                                start_line: 1, // TODO: improve
                                end_line: 1,
                                target: None,
                                items: None,
                                visibility: extract_visibility(&impl_const.vis),
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature: None,
                                fields: None,
                            })
                        }
                        syn::ImplItem::Type(impl_type) => {
                            let attrs = extract_attributes(&impl_type.attrs);
                            let doc_comment = extract_doc_comment(&impl_type.attrs);

                            Some(ItemDescription {
                                item_type: "type_alias".to_string(),
                                name: impl_type.ident.to_string(),
                                start_line: 1, // TODO: improve
                                end_line: 1,
                                target: None,
                                items: None,
                                visibility: extract_visibility(&impl_type.vis),
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature: None,
                                fields: None,
                            })
                        }
                        _ => None,
                    }
                })
                .collect();

            Some(ItemDescription {
                item_type: "impl".to_string(),
                name: target.clone().unwrap_or_else(|| "Unknown".to_string()),
                start_line,
                end_line,
                target,
                items: if nested_items.is_empty() {
                    None
                } else {
                    Some(nested_items)
                },
                visibility: None, // impl blocks don't have visibility
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        Item::Trait(item_trait) => {
            let attrs = extract_attributes(&item_trait.attrs);
            let doc_comment = extract_doc_comment(&item_trait.attrs);

            let nested_items: Vec<ItemDescription> = item_trait
                .items
                .iter()
                .filter_map(|trait_item| {
                    match trait_item {
                        syn::TraitItem::Fn(trait_item_fn) => {
                            let (start_line, end_line) =
                                find_item_lines_trait_fn(trait_item_fn, content, line_map);
                            let attrs = extract_attributes(&trait_item_fn.attrs);
                            let doc_comment = extract_doc_comment(&trait_item_fn.attrs);
                            let signature = Some(quote::quote!(#trait_item_fn.sig).to_string());

                            Some(ItemDescription {
                                item_type: "function".to_string(),
                                name: trait_item_fn.sig.ident.to_string(),
                                start_line,
                                end_line,
                                target: None,
                                items: None,
                                visibility: None, // Trait items don't have visibility
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature,
                                fields: None,
                            })
                        }
                        syn::TraitItem::Type(trait_type) => {
                            let attrs = extract_attributes(&trait_type.attrs);
                            let doc_comment = extract_doc_comment(&trait_type.attrs);

                            Some(ItemDescription {
                                item_type: "type_alias".to_string(),
                                name: trait_type.ident.to_string(),
                                start_line: 1, // TODO: improve
                                end_line: 1,
                                target: None,
                                items: None,
                                visibility: None,
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature: None,
                                fields: None,
                            })
                        }
                        syn::TraitItem::Const(trait_const) => {
                            let attrs = extract_attributes(&trait_const.attrs);
                            let doc_comment = extract_doc_comment(&trait_const.attrs);

                            Some(ItemDescription {
                                item_type: "constant".to_string(),
                                name: trait_const.ident.to_string(),
                                start_line: 1, // TODO: improve
                                end_line: 1,
                                target: None,
                                items: None,
                                visibility: None,
                                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                                doc_comment,
                                signature: None,
                                fields: None,
                            })
                        }
                        _ => None,
                    }
                })
                .collect();

            Some(ItemDescription {
                item_type: "trait".to_string(),
                name: item_trait.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: if nested_items.is_empty() {
                    None
                } else {
                    Some(nested_items)
                },
                visibility: extract_visibility(&item_trait.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        Item::Mod(item_mod) => {
            let nested_items = if let Some((_, items)) = &item_mod.content {
                extract_items(items, content, line_map)
            } else {
                Vec::new()
            };

            let attrs = extract_attributes(&item_mod.attrs);
            let doc_comment = extract_doc_comment(&item_mod.attrs);

            Some(ItemDescription {
                item_type: "module".to_string(),
                name: item_mod.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: if nested_items.is_empty() {
                    None
                } else {
                    Some(nested_items)
                },
                visibility: extract_visibility(&item_mod.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        Item::Type(item_type) => {
            let attrs = extract_attributes(&item_type.attrs);
            let doc_comment = extract_doc_comment(&item_type.attrs);
            let signature = Some(quote::quote!(#item_type.ty).to_string());

            Some(ItemDescription {
                item_type: "type_alias".to_string(),
                name: item_type.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_type.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature,
                fields: None,
            })
        }
        Item::Const(item_const) => {
            let attrs = extract_attributes(&item_const.attrs);
            let doc_comment = extract_doc_comment(&item_const.attrs);
            let signature = Some(quote::quote!(#item_const.ty).to_string());

            Some(ItemDescription {
                item_type: "constant".to_string(),
                name: item_const.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_const.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature,
                fields: None,
            })
        }
        Item::Static(item_static) => {
            let attrs = extract_attributes(&item_static.attrs);
            let doc_comment = extract_doc_comment(&item_static.attrs);
            let signature = Some(quote::quote!(#item_static.ty).to_string());

            Some(ItemDescription {
                item_type: "static".to_string(),
                name: item_static.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_static.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature,
                fields: None,
            })
        }
        Item::Macro(item_macro) => {
            let attrs = extract_attributes(&item_macro.attrs);
            let doc_comment = extract_doc_comment(&item_macro.attrs);
            let name = item_macro
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "macro_rules!".to_string());

            Some(ItemDescription {
                item_type: "macro".to_string(),
                name,
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: None, // macros don't have visibility in syn
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        Item::Use(item_use) => {
            let attrs = extract_attributes(&item_use.attrs);
            let doc_comment = extract_doc_comment(&item_use.attrs);
            let use_path = quote::quote!(#item_use.tree).to_string();

            Some(ItemDescription {
                item_type: "use".to_string(),
                name: use_path,
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_use.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        Item::Union(item_union) => {
            let attrs = extract_attributes(&item_union.attrs);
            let doc_comment = extract_doc_comment(&item_union.attrs);
            // Unions only have named fields
            let field_descs = extract_struct_fields(&item_union.fields.named, content, line_map);
            let fields = if field_descs.is_empty() {
                None
            } else {
                Some(field_descs)
            };

            Some(ItemDescription {
                item_type: "union".to_string(),
                name: item_union.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_union.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields,
            })
        }
        Item::ExternCrate(item_extern) => {
            let attrs = extract_attributes(&item_extern.attrs);
            let doc_comment = extract_doc_comment(&item_extern.attrs);

            Some(ItemDescription {
                item_type: "extern_crate".to_string(),
                name: item_extern.ident.to_string(),
                start_line,
                end_line,
                target: None,
                items: None,
                visibility: extract_visibility(&item_extern.vis),
                attributes: if attrs.is_empty() { None } else { Some(attrs) },
                doc_comment,
                signature: None,
                fields: None,
            })
        }
        _ => None, // Skip other item types
    }
}

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use syn::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationResults {
    pub file: String,
    pub items: Vec<DocItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    pub line: usize,
}

pub fn extract_docs(path: PathBuf) -> anyhow::Result<DocumentationResults> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if !path.is_file() {
        anyhow::bail!("Path is not a file: {}", path.display());
    }

    // Read file content
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse Rust file
    let ast = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {}", path.display()))?;

    // Build line map
    let line_map = build_line_map(&content);

    // Extract documentation
    let items = extract_docs_from_items(&ast.items, &content, &line_map);

    Ok(DocumentationResults {
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

fn extract_docs_from_items(items: &[Item], content: &str, line_map: &[usize]) -> Vec<DocItem> {
    let mut result = Vec::new();

    for item in items {
        if let Some(doc_item) = extract_item_docs(item, content, line_map) {
            result.push(doc_item);
        }
    }

    result
}

fn extract_item_docs(item: &Item, content: &str, line_map: &[usize]) -> Option<DocItem> {
    // Use search-based approach to find line number (same as describe.rs)
    let search_str = match item {
        Item::Fn(f) => format!("fn {}", f.sig.ident),
        Item::Struct(s) => format!("struct {}", s.ident),
        Item::Enum(e) => format!("enum {}", e.ident),
        Item::Trait(t) => format!("trait {}", t.ident),
        Item::Impl(_) => "impl".to_string(),
        Item::Mod(m) => format!("mod {}", m.ident),
        _ => return None,
    };

    let line = if let Some(pos) = content.find(&search_str) {
        byte_offset_to_line(pos, line_map)
    } else {
        1 // Fallback
    };

    let (item_type, name, attrs) = match item {
        Item::Fn(item_fn) => ("function", item_fn.sig.ident.to_string(), &item_fn.attrs),
        Item::Struct(item_struct) => ("struct", item_struct.ident.to_string(), &item_struct.attrs),
        Item::Enum(item_enum) => ("enum", item_enum.ident.to_string(), &item_enum.attrs),
        Item::Trait(item_trait) => ("trait", item_trait.ident.to_string(), &item_trait.attrs),
        Item::Impl(item_impl) => {
            // For impl blocks, we'll extract docs from the impl itself
            let target = if let Some((_, path, _)) = &item_impl.trait_ {
                path.segments.last().unwrap().ident.to_string()
            } else if let syn::Type::Path(type_path) = item_impl.self_ty.as_ref() {
                type_path.path.segments.last().unwrap().ident.to_string()
            } else {
                "Unknown".to_string()
            };
            ("impl", target, &item_impl.attrs)
        }
        Item::Mod(item_mod) => ("module", item_mod.ident.to_string(), &item_mod.attrs),
        _ => return None,
    };

    // Extract doc comments
    let (docs, examples) = extract_doc_attributes(attrs, content);

    Some(DocItem {
        item_type: item_type.to_string(),
        name,
        docs: if docs.is_empty() { None } else { Some(docs) },
        examples: if examples.is_empty() {
            None
        } else {
            Some(examples)
        },
        line,
    })
}

fn extract_doc_attributes(attrs: &[syn::Attribute], _content: &str) -> (String, Vec<String>) {
    let mut docs = String::new();
    let mut examples = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            // Extract doc comment content
            if let Ok(meta) = attr.parse_args::<syn::LitStr>() {
                let doc_line = meta.value();
                if doc_line.trim().starts_with("Example") || doc_line.trim().starts_with("example")
                {
                    examples.push(doc_line.trim().to_string());
                } else {
                    if !docs.is_empty() {
                        docs.push('\n');
                    }
                    docs.push_str(&doc_line);
                }
            }
        }
    }

    (docs, examples)
}




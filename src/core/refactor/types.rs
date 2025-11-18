use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorResult {
    pub operation: String,
    pub status: RefactorStatus,
    pub changes: Vec<Change>,
    pub validation: Option<ValidationResult>,
    pub backup_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefactorStatus {
    Preview,
    Applied,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub file: String,
    pub line: usize,
    pub old: String,
    pub new: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub cargo_check_passed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageKind {
    Definition,
    Call,
    Reference,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub file: PathBuf,
    pub line: usize,
    pub kind: UsageKind,
    pub context: String,
}

#[derive(Clone)]
pub struct CallSite {
    pub file: PathBuf,
    pub line: usize,
    pub args: Vec<syn::Expr>,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub file: PathBuf,
    pub line: usize,
    pub path: String,
    pub is_pub: bool,
}




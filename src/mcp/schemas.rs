use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CargoArgs {
    pub args: Vec<String>,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadParams {
    pub path: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescribeParams {
    pub path: String,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TestsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsParams {
    pub path: String,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorRenameParams {
    pub symbol: String,
    pub to: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorExtractParams {
    pub file: String,
    pub working_directory: String,
    pub from: usize,
    pub to: usize,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorMoveParams {
    pub symbol: String,
    pub to: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorSignatureParams {
    pub function: String,
    pub new_signature: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClippyArgs {
    pub args: Vec<String>,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<usize>,
}

use crate::mcp::schemas::*;
use crate::mcp::tools::*;
use rmcp::model::InitializeResult;
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::io::{stdin, stdout};

pub struct Tool42Server;

impl Default for Tool42Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool42Server {
    pub fn new() -> Self {
        Self
    }
}

impl ServerHandler for Tool42Server {
    fn get_info(&self) -> InitializeResult {
        use rmcp::model::ProtocolVersion;
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability {
                    list_changed: None,
                }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "tool42".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Tool42".into()),
                website_url: None,
                icons: None,
            },
            instructions: Some("Tool42 provides Rust development tools for AI agents including cargo and clippy command execution (with pagination), file reading, code analysis, searching, dependency management, test discovery, project structure analysis, directory listing, documentation extraction, and refactoring operations.".into()),
        }
    }

    async fn list_tools(
        &self,
        _paginated: Option<rmcp::model::PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        use rmcp::model::Tool;

        // Helper function to convert schema Value to Arc<Map>
        fn schema_to_map(schema: Value) -> Arc<Map<String, Value>> {
            match schema {
                Value::Object(map) => {
                    // Ensure it's a valid JSON Schema object
                    if map.is_empty() {
                        // Return empty object schema if map is empty
                        let mut empty_map = Map::new();
                        empty_map.insert("type".to_string(), Value::String("object".to_string()));
                        empty_map.insert("properties".to_string(), Value::Object(Map::new()));
                        Arc::new(empty_map)
                    } else {
                        Arc::new(map)
                    }
                }
                _ => {
                    // Fallback to empty object schema if conversion failed
                    let mut map = Map::new();
                    map.insert("type".to_string(), Value::String("object".to_string()));
                    map.insert("properties".to_string(), Value::Object(Map::new()));
                    Arc::new(map)
                }
            }
        }

        // Helper function to create empty object schema
        fn empty_schema() -> Arc<Map<String, Value>> {
            let mut map = Map::new();
            map.insert("type".to_string(), Value::String("object".to_string()));
            map.insert("properties".to_string(), Value::Object(Map::new()));
            Arc::new(map)
        }

        let tools = vec![
            Tool {
                name: "tool42_cargo".into(),
                title: Some("Execute Cargo Command".into()),
                description: Some("Execute a Cargo command with pagination support. Returns output in chunks (default: first 500 lines). Use from/to parameters for range requests. Requires working_directory parameter to specify the directory containing Cargo.toml.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(CargoArgs)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_clippy".into(),
                title: Some("Execute Clippy Command".into()),
                description: Some("Execute cargo clippy with pagination support. Returns output in chunks (default: first 500 lines). Use from/to parameters for range requests. Requires working_directory parameter to specify the directory containing Cargo.toml. Clippy must be installed (rustup component add clippy).".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ClippyArgs)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_read".into(),
                title: Some("Read File".into()),
                description: Some("Read a file or specific section with line number limits (max 500 lines)".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ReadParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_describe".into(),
                title: Some("Describe Rust File".into()),
                description: Some("Extract structural information from a Rust source file".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(DescribeParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_search".into(),
                title: Some("Search Codebase".into()),
                description: Some("Search for text patterns across the codebase".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(SearchParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_deps".into(),
                title: Some("Get Dependencies".into()),
                description: Some("Extract dependency information from Cargo.toml".into()),
                // tool42_deps takes no parameters, so empty schema is correct
                input_schema: empty_schema(),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_tests".into(),
                title: Some("List Tests".into()),
                description: Some("List all tests in the project".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(TestsParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_project".into(),
                title: Some("Get Project Structure".into()),
                description: Some("Get overview of project structure. By default returns a lightweight summary with package metadata (name, version, edition, etc.) but no module details. Use `detailed_package` to specify a single package that should include detailed module information. Use `max_depth` to control module traversal depth for the detailed package (default: 2). This prevents large outputs that require file writing.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ProjectParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_list".into(),
                title: Some("List Directory".into()),
                description: Some("List directory contents with metadata".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ListParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_docs".into(),
                title: Some("Extract Documentation".into()),
                description: Some("Extract doc comments from Rust code".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(DocsParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_rename".into(),
                title: Some("Rename Symbol".into()),
                description: Some("Rename a symbol (struct, enum, function, type, etc.) across the entire codebase".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorRenameParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_extract".into(),
                title: Some("Extract Function".into()),
                description: Some("Extract a code block into a new function".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorExtractParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_move".into(),
                title: Some("Move Item".into()),
                description: Some("Move a function/struct/enum to a different module or file".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorMoveParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_signature".into(),
                title: Some("Change Function Signature".into()),
                description: Some("Change a function signature and update all call sites".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorSignatureParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
        ];

        Ok(rmcp::model::ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let name = &params.name;
        let arguments_map = params.arguments.unwrap_or_default();
        let arguments = serde_json::Value::Object(arguments_map);

        let result = match name.as_ref() {
            "tool42_cargo" => handle_cargo(arguments).await,
            "tool42_clippy" => handle_clippy(arguments).await,
            "tool42_read" => handle_read(arguments).await,
            "tool42_describe" => handle_describe(arguments).await,
            "tool42_search" => handle_search(arguments).await,
            "tool42_deps" => handle_deps(arguments).await,
            "tool42_tests" => handle_tests(arguments).await,
            "tool42_project" => handle_project(arguments).await,
            "tool42_list" => handle_list(arguments).await,
            "tool42_docs" => handle_docs(arguments).await,
            "tool42_refactor_rename" => handle_refactor_rename(arguments).await,
            "tool42_refactor_extract" => handle_refactor_extract(arguments).await,
            "tool42_refactor_move" => handle_refactor_move(arguments).await,
            "tool42_refactor_signature" => handle_refactor_signature(arguments).await,
            _ => Err(format!("Unknown tool: {}", name)),
        };

        match result {
            Ok(content) => {
                let content_str = serde_json::to_string(&content).unwrap_or_default();
                Ok(rmcp::model::CallToolResult {
                    content: vec![rmcp::model::Annotated {
                        raw: rmcp::model::RawContent::Text(rmcp::model::RawTextContent {
                            text: content_str,
                            meta: None,
                        }),
                        annotations: None,
                    }],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                })
            }
            Err(e) => {
                // Return error as a result with is_error flag set to true
                // This format is better recognized by test frameworks that expect error: true
                let error_message = e.to_string();
                let error_content = serde_json::json!({
                    "error": {
                        "code": -32603,
                        "message": error_message
                    }
                });
                let error_str = serde_json::to_string(&error_content).unwrap_or(error_message);
                Ok(rmcp::model::CallToolResult {
                    content: vec![rmcp::model::Annotated {
                        raw: rmcp::model::RawContent::Text(rmcp::model::RawTextContent {
                            text: error_str,
                            meta: None,
                        }),
                        annotations: None,
                    }],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                })
            }
        }
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let transport = (stdin(), stdout());
    let service = Tool42Server::new();
    let server = service.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}

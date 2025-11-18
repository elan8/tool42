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
                description: Some("Parse a Rust source file and extract comprehensive structural information in JSON format. Returns a hierarchical tree of all code items including structs (with fields), enums (with variants), functions (with signatures), impl blocks (with nested methods), traits (with associated items), modules (with nested items), type aliases, constants, static items, macros, use statements, unions, and extern crates. For each item, provides name, type, start/end line numbers, visibility, attributes, doc comments, signatures, and fields. Useful for understanding file structure before reading or analyzing code.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(DescribeParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_search".into(),
                title: Some("Search Codebase".into()),
                description: Some("Search for text patterns in Rust source files (.rs) across the codebase. The query is treated as a literal string (regex-escaped) and matched case-insensitively. Returns matches with file path, line number, and the matching line as context. Automatically skips hidden files/directories and the target directory. If no path is specified, searches from the workspace root. Query cannot be empty.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(SearchParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_deps".into(),
                title: Some("Get Dependencies".into()),
                description: Some("Extract comprehensive dependency information from a Rust project using cargo metadata. Returns workspace root path and a list of all packages (including workspace members) with their names, versions, sources, direct dependencies, and available features. Works for both single-package projects and Cargo workspaces. The output includes all transitive dependencies and provides a complete dependency graph structure in JSON format.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(DepsParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_tests".into(),
                title: Some("List Tests".into()),
                description: Some("Discover and list all test functions in a Rust project. Scans the workspace root (found from working_directory) for all Rust source files (.rs), parses them to identify functions with the #[test] attribute, and returns a structured list with test names, file paths, line numbers, and module paths. Automatically skips hidden files/directories and the target directory. Useful for understanding the test suite structure and locating specific tests.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(TestsParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_project".into(),
                title: Some("Get Project Structure".into()),
                description: Some("Get a high-level overview of the Rust project structure. Returns workspace information (if applicable), workspace-level dependencies, and a list of packages with their crates. For workspaces, only includes packages listed in workspace.members. For non-workspace projects, includes only the root package. Each package includes metadata (name, version, edition, description, license) and package-level dependencies (dependencies, dev-dependencies, build-dependencies). Each crate is listed with its name and type (lib or bin). Uses Cargo.toml to determine project structure, not directory scanning.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ProjectParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_list".into(),
                title: Some("List Directory".into()),
                description: Some("List the contents of a directory with metadata. Returns all entries (files and subdirectories) with their names, types (file/directory/other), file sizes (for files), and modification timestamps. Entries are sorted with directories first, then files, both alphabetically. If no path is specified, lists the workspace root. Useful for exploring project structure, locating source files, understanding directory organization, and navigating the codebase. Returns structured JSON with entry metadata for easy programmatic access.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(ListParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_docs".into(),
                title: Some("Extract Documentation".into()),
                description: Some("Extract documentation comments from a specific Rust source file. Parses the file to identify all documented items (functions, structs, enums, traits, impl blocks, modules) and returns their doc comments and examples in a structured format. Useful for quickly understanding API documentation, usage examples, and public interfaces without reading the full source code. Requires a path parameter to specify the target file. Returns documentation organized by item type with line numbers for easy reference.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(DocsParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_rename".into(),
                title: Some("Rename Symbol".into()),
                description: Some("Rename a symbol (struct, enum, function, type alias, constant, etc.) across the entire codebase. Searches for all occurrences of the symbol including definitions, usages, imports, and references, then updates them to the new name. Can optionally scope the search to a specific file or directory path. Supports preview mode (default) to review changes before applying, and apply mode to execute the refactoring. Validates changes with cargo check after applying. Returns a detailed list of all changes with file paths, line numbers, and context. Automatically creates backup files before making changes.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorRenameParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_extract".into(),
                title: Some("Extract Function".into()),
                description: Some("Extract a code block (specified by line range) from a Rust source file into a new function. Takes a file path, start/end line numbers (1-based, inclusive), and a function name. Creates a new function containing the extracted code and replaces the original code block with a function call. Handles variable scoping and ensures the extracted function receives necessary parameters. Supports preview mode (default) to review changes before applying, and apply mode to execute the refactoring. Validates changes with cargo check after applying. Returns detailed changes showing the extracted code and the replacement function call. Useful for breaking down large functions, improving code reusability, and enhancing readability.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorExtractParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_move".into(),
                title: Some("Move Item".into()),
                description: Some("Move a function, struct, enum, or other item to a different module or file. Searches the codebase to find the item definition and all its usages, then moves it to the target location (specified as a module path or file path). Automatically updates all imports, references, and usages throughout the codebase to reflect the new location. Supports preview mode (default) to review changes before applying, and apply mode to execute the refactoring. Validates changes with cargo check after applying. Returns a detailed list of all files modified, showing what was moved and where. Useful for reorganizing code structure, improving module organization, and separating concerns.".into()),
                input_schema: schema_to_map(serde_json::to_value(schemars::schema_for!(RefactorMoveParams)).unwrap_or_default()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "tool42_refactor_signature".into(),
                title: Some("Change Function Signature".into()),
                description: Some("Change a function's signature (parameters, return type, visibility, etc.) and automatically update all call sites throughout the codebase. Searches for the function definition and all places where it's called, then updates them to match the new signature. The new_signature parameter should be the complete function signature as it should appear (e.g., \"pub fn my_function(x: i32, y: String) -> bool\"). Supports preview mode (default) to review changes before applying, and apply mode to execute the refactoring. Validates changes with cargo check after applying. Returns detailed changes showing the old and new signatures at the definition and all call sites. Useful for refactoring APIs, adding or removing parameters, changing return types, and updating function visibility.".into()),
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

//! # Tool42
//!
//! **AI assisted Rust development**
//!
//! Tool42 is an MCP (Model Context Protocol) server that provides Rust development tools for AI agents.
//! It enables AI agents to perform various tasks for Rust software development, including executing
//! Cargo commands, reading and analyzing source files, searching codebases, managing dependencies,
//! discovering tests, analyzing project structure, extracting documentation, and performing
//! refactoring operations.
//!
//! ## Why Tool42?
//!
//! AI assistants in Cursor and other MCP-compatible environments often fail or hang when executing
//! cargo commands directly due to serialization errors. Tool42 solves this by providing a stable
//! MCP interface that handles command execution with proper pagination and error handling.
//!
//! Tool42 also helps make file operations cross-platform, eliminating the need for separate
//! PowerShell vs Bash scripts.
//!
//! ## Quick Start
//!
//! ### Installation
//!
//! ```bash
//! cargo install tool42
//! ```
//!
//! Or from source:
//!
//! ```bash
//! git clone https://github.com/elan8/tool42.git
//! cd tool42
//! cargo install --path .
//! ```
//!
//! ### MCP Client Configuration
//!
//! Configure your MCP client (e.g., Cursor) to use Tool42:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "tool42": {
//!       "command": "tool42"
//!     }
//!   }
//! }
//! ```
//!
//! The server starts automatically when invoked. Use `tool42 --version` or `tool42 --help` for
//! basic information.
//!
//! ## Architecture
//!
//! Tool42 is organized into two main modules:
//!
//! - **[`core`]**: Core functionality for Rust development tools, including:
//!   - Cargo command execution with pagination
//!   - Clippy linting with pagination
//!   - File reading and analysis
//!   - Code structure parsing
//!   - Dependency management
//!   - Test discovery
//!   - Project structure analysis
//!   - Code search
//!   - Documentation extraction
//!   - Refactoring operations (rename, extract, move, signature changes)
//!
//! - **[`mcp`]**: MCP server implementation that exposes the core tools as MCP
//!   resources and tools, handling protocol communication, request routing, and response formatting.
//!
//! ## Available Tools
//!
//! Tool42 provides a comprehensive set of MCP tools. For detailed documentation on all available
//! tools, including parameters, return values, and usage notes, see the
//! [MCP Tools Reference](https://github.com/elan8/tool42/blob/main/MCP_TOOLS.md).
//!
//! **Core Tools:**
//! - `tool42_cargo` - Execute Cargo commands with pagination support
//! - `tool42_clippy` - Execute cargo clippy with pagination support
//! - `tool42_read` - Read files with line number limits
//! - `tool42_describe` - Extract structural information from Rust source files
//! - `tool42_search` - Search for text patterns in Rust source files
//! - `tool42_deps` - Extract dependency information from Rust projects
//! - `tool42_tests` - Discover and list all test functions
//! - `tool42_project` - Get project structure overview
//! - `tool42_list` - List directory contents with metadata
//! - `tool42_docs` - Extract documentation comments from Rust files
//!
//! **Refactoring Tools:**
//! - `tool42_refactor_rename` - Rename symbols across the codebase
//! - `tool42_refactor_extract` - Extract code blocks into new functions
//! - `tool42_refactor_move` - Move items to different modules or files
//! - `tool42_refactor_signature` - Change function signatures and update call sites
//!
//! ## Usage Examples
//!
//! ### Executing Cargo Commands
//!
//! Tool42 can execute any cargo command with automatic pagination for large outputs:
//!
//! ```json
//! {
//!   "tool": "tool42_cargo",
//!   "arguments": {
//!     "args": ["build", "--release"],
//!     "working_directory": "/path/to/rust/project"
//!   }
//! }
//! ```
//!
//! ### Reading and Analyzing Code
//!
//! Read specific sections of files or analyze code structure:
//!
//! ```json
//! {
//!   "tool": "tool42_read",
//!   "arguments": {
//!     "path": "src/main.rs",
//!     "working_directory": "/path/to/project",
//!     "from": 1,
//!     "to": 50
//!   }
//! }
//! ```
//!
//! ### Refactoring Operations
//!
//! All refactoring tools support preview mode (default) and require explicit `apply: true`:
//!
//! ```json
//! {
//!   "tool": "tool42_refactor_rename",
//!   "arguments": {
//!     "symbol": "old_name",
//!     "to": "new_name",
//!     "working_directory": "/path/to/project",
//!     "preview": true
//!   }
//! }
//! ```
//!
//! ## Common Notes
//!
//! - All tools return structured JSON responses according to the MCP specification
//! - The `working_directory` parameter must always be an absolute path
//! - All refactoring tools support preview mode (default) and require explicit `apply: true` to make changes
//! - Refactoring tools automatically validate changes with `cargo check` after applying
//! - Maximum 500 lines per request for paginated operations
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the [LICENSE](https://github.com/elan8/tool42/blob/main/LICENSE) file for details.

pub mod core;
pub mod mcp;

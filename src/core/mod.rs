//! Core functionality for Rust development tools.
//!
//! This module provides the core implementations for all Tool42 MCP tools, organized by
//! functionality:
//!
//! ## Command Execution
//! - [`cargo`] - Execute Cargo commands with pagination
//! - [`clippy`] - Execute cargo clippy with pagination
//!
//! ## File Operations
//! - [`read`] - Read files with line number limits
//! - [`list`] - List directory contents with metadata
//!
//! ## Code Analysis
//! - [`describe`] - Extract structural information from Rust source files
//! - [`docs`] - Extract documentation comments from Rust files
//! - [`search`](mod@crate::core::search) - Search for text patterns in Rust source files
//!
//! ## Project Management
//! - [`project`] - Get project structure overview
//! - [`deps`] - Extract dependency information
//! - [`tests`] - Discover and list all test functions
//!
//! ## Refactoring
//! - [`refactor`] - Refactoring operations (rename, extract, move, signature changes)
//!
//! ## Utilities
//! - [`cache`] - Caching utilities for command outputs

pub mod cache;
pub mod cargo;
pub mod clippy;
pub mod deps;
pub mod describe;
pub mod docs;
pub mod list;
pub mod project;
pub mod read;
pub mod refactor;
pub mod search;
pub mod tests;

pub use cache::*;
// Don't use glob re-exports for cargo and clippy to avoid ambiguous exports
// They both export PaginatedResult and execute_mcp_paginated
pub use deps::*;
pub use describe::*;
pub use docs::*;
pub use list::*;
pub use project::*;
pub use read::*;
pub use refactor::*;
pub use search::*;
pub use tests::*;

//! MCP server implementation.
//!
//! This module provides the MCP (Model Context Protocol) server implementation that exposes
//! the core tools as MCP resources and tools.
//!
//! ## Modules
//! - [`server`] - Main MCP server implementation
//! - [`tools`] - Tool handlers that bridge MCP requests to core functionality
//! - [`schemas`] - JSON schemas for MCP tool parameters and responses

pub mod schemas;
pub mod server;
pub mod tools;

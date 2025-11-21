//! # Tool42 Binary
//!
//! The `tool42` binary is the entry point for the Tool42 MCP server. When invoked, it starts
//! an MCP server that provides Rust development tools for AI agents.
//!
//! ## Usage
//!
//! ```bash
//! tool42              # Start the MCP server
//! tool42 --version    # Show version information
//! tool42 --help       # Show help information
//! ```
//!
//! ## MCP Server
//!
//! When run without flags, the binary starts an MCP server that communicates via stdin/stdout
//! following the Model Context Protocol specification. The server provides various tools for
//! Rust development, including:
//!
//! - Cargo command execution with pagination
//! - Code analysis and structure extraction
//! - Dependency management
//! - Test discovery
//! - Refactoring operations
//!
//! For detailed information about available tools, see the [crate documentation](tool42).

use clap::Parser;

#[derive(Parser)]
#[command(name = "tool42")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Tool42 MCP Server - Rust development tools for AI agents")]
#[command(
    long_about = "Tool42 is an MCP (Model Context Protocol) server that provides Rust development tools for AI agents, including cargo command execution, file reading, code analysis, searching, dependency management, test discovery, project structure analysis, directory listing, documentation extraction, and refactoring operations."
)]
struct Args {
    // No subcommands - just version and help flags
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse args to handle --version and --help
    // If these flags are present, clap will print and exit automatically
    let _args = Args::parse();

    // If we get here, no --version or --help was used, so start MCP server
    tool42::mcp::server::run().await?;
    Ok(())
}

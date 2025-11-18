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

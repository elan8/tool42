# Tool42
## AI assisted Rust development

Tool42 is an MCP (Model Context Protocol) server that provides Rust development tools for AI agents. It enables AI agents to perform various tasks for Rust software development, including executing Cargo commands, reading and analyzing source files, searching codebases, managing dependencies, discovering tests, analyzing project structure, extracting documentation, and performing refactoring operations.

## Quick Start

**MCP Client Configuration:**

```json
{
  "mcpServers": {
    "tool42": {
      "command": "tool42"
    }
  }
}
```

The server starts automatically when invoked. Use `tool42 --version` or `tool42 --help` for basic information.

## Available MCP Tools

- `tool42_cargo` - Execute Cargo commands with pagination support (default: first 500 lines, use `from`/`to` for ranges)
- `tool42_clippy` - Execute cargo clippy with pagination support (default: first 500 lines, use `from`/`to` for ranges). Requires clippy to be installed (`rustup component add clippy`)
- `tool42_read` - Read files with line limits (max 500 lines)
- `tool42_describe` - Parse Rust source file and extract comprehensive structural information (structs, enums, functions, impl blocks, traits, modules, etc.) with line numbers, visibility, attributes, doc comments, and signatures in JSON format
- `tool42_search` - Search codebase for symbols, functions, types, or text patterns
- `tool42_deps` - Get dependency information from Cargo.toml
- `tool42_tests` - List all tests in the project with their locations
- `tool42_project` - Get an overview of the entire project structure
- `tool42_list` - List directory contents with metadata
- `tool42_docs` - Extract documentation from Rust source files
- `tool42_refactor_rename` - Rename symbols across the codebase
- `tool42_refactor_extract` - Extract code blocks into new functions
- `tool42_refactor_move` - Move functions/structs/enums to different modules
- `tool42_refactor_signature` - Change function signatures and update call sites

Each tool returns structured JSON responses according to the MCP specification.

## For AI Agents

**Important**: The `working_directory` input parameter for tool42 MCP tools must be an absolute path.

**Quick reference**: Always use tool42 MCP tools:
- Use `tool42_cargo` instead of direct cargo commands (supports pagination for large outputs)
- Use `tool42_clippy` for clippy linting (supports pagination for large outputs)
- Use `tool42_read` for file reading (max 500 lines)
- Use `tool42_describe` to understand file structure
- Use `tool42_search` to find code patterns

**Pagination**: Both `tool42_cargo` and `tool42_clippy` support pagination to handle large outputs:
- Default: Returns first 500 lines
- Use `from` and `to` parameters (1-based, inclusive) to request specific line ranges
- Output is cached after first execution for efficient range requests

For Cursor users, the `.cursorrules` file contains condensed rules that will automatically guide AI assistance.

## Testing

### Unit and Integration Tests

```bash
cargo test
```

### Mandrel MCP Test Harness

Tool42 uses the [Mandrel MCP Test Harness](https://rustic-ai.github.io/codeprism/docs/test-harness/) for comprehensive MCP protocol testing.

**Quick Start:**
```powershell
cd tests\mandrel
.\run-tests.ps1
```

For detailed information, see [tests/mandrel/README.md](tests/mandrel/README.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
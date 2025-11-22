# Tool42
## AI assisted Rust development

Tool42 is an MCP (Model Context Protocol) server that provides Rust development tools for AI agents. It enables AI agents to perform various tasks for Rust software development, including executing Cargo commands, reading and analyzing source files, searching codebases, managing dependencies, discovering tests, analyzing project structure, extracting documentation, and performing refactoring operations.

## Why use Tool42 instead of seperate CLI tools such as cargo?
I was requesting AI assistents to use cargo for things like "cargo check" and "cargo check". AI assistents in Cursor would often fail/hang on cargo output due to serialization errors. So I first used a wrapper script to write the output of cargo to a textfile , but then figured I might as well make a proper solution. I now force my AI assistents to use tool42 instead of cargo and I don't have serialization errors anymore. 

Tool42 can also help in making file operations cross platform: not longer Powershell vs Bash script, but use tool42 on any platform. But this is still work-in-progress.

## TODO

Future features planned for Tool42:

- [ ] **Cross-platform file rename/move tool**: Add a tool for renaming and moving files across different platforms (Windows, Unix, etc.)
- [ ] **Code formatting tool**: Integrate `rustfmt` to format Rust code with configurable options
- [ ] **Security audit tool**: Add `cargo audit` integration to check for known security vulnerabilities in dependencies
- [ ] **Dependency update tool**: Provide tools to check and update dependencies (cargo update, cargo upgrade)
- [ ] **Code metrics tool**: Calculate code statistics (lines of code, cyclomatic complexity, function counts, etc.)


## Installation

### From crates.io (Recommended)

Tool42 is available on crates.io and can be easily installed:

```bash
cargo install tool42
```

This will install the `tool42` binary to your Cargo bin directory (typically `~/.cargo/bin` on Unix systems or `%USERPROFILE%\.cargo\bin` on Windows).

### From Source

Alternatively, you can install Tool42 from source:

```bash
git clone https://github.com/elan8/tool42.git
cd tool42
cargo install --path .
```

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

Tool42 provides a comprehensive set of MCP tools for Rust development. For detailed documentation on all available tools, including parameters, return values, and usage notes, see [MCP_TOOLS.md](MCP_TOOLS.md).

**Available Tools:**
- `tool42_cargo` - Execute Cargo commands with pagination support
- `tool42_clippy` - Execute cargo clippy with pagination support
- `tool42_read` - Read files with line number limits
- `tool42_describe` - Extract structural information from Rust source files
- `tool42_search` - Search for text patterns in Rust source files
- `tool42_deps` - Extract dependency information from Rust projects
- `tool42_tests` - Discover and list all test functions
- `tool42_project` - Get project structure overview
- `tool42_list` - List directory contents with metadata
- `tool42_docs` - Extract documentation comments from Rust files
- `tool42_refactor_rename` - Rename symbols across the codebase
- `tool42_refactor_extract` - Extract code blocks into new functions
- `tool42_refactor_move` - Move items to different modules or files
- `tool42_refactor_signature` - Change function signatures and update call sites


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
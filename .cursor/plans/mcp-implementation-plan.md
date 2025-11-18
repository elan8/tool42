# MCP Implementation Plan for Tool42

## Overview

This plan outlines the implementation of a Model Context Protocol (MCP) server interface for tool42, exposing all existing CLI functionalities as MCP tools. This will enable direct integration with AI agents that support MCP, providing better performance and structured communication.

## Goals

1. **Refactor core logic** - Separate command implementations from CLI parsing
2. **Implement MCP server** - Add MCP server capability using Rust MCP library
3. **Expose all commands** - Map all 9 existing commands to MCP tools
4. **Maintain compatibility** - Keep CLI interface fully functional
5. **Feature flag** - Make MCP server optional via Cargo feature flag

## Current Functionalities to Expose

1. **`cargo`** - Execute cargo commands, write output to temp file
2. **`read`** - Read file or file section (with line limits)
3. **`describe`** - Extract Rust file structure (JSON to temp file)
4. **`search`** - Search codebase for patterns (JSON to temp file)
5. **`deps`** - Extract dependency information (JSON to temp file)
6. **`tests`** - List all tests in project (JSON to temp file)
7. **`project`** - Get project structure overview (JSON to temp file)
8. **`list`** - List directory contents (JSON to temp file)
9. **`docs`** - Extract doc comments (JSON to temp file)

## Architecture Changes

### Phase 1: Refactoring Core Logic

**Goal**: Separate business logic from CLI interface

**Changes**:
1. Create `src/core/` module with pure function implementations
2. Move command logic to return structured data instead of writing files/printing
3. Keep CLI commands as thin wrappers that call core functions and handle I/O

**Structure**:
```
src/
  core/
    mod.rs
    cargo.rs      - Returns output path and exit code
    read.rs       - Returns file content lines
    describe.rs   - Returns FileDescription struct
    search.rs     - Returns SearchResults struct
    deps.rs       - Returns DependencyInfo struct
    tests.rs      - Returns TestResults struct
    project.rs    - Returns ProjectStructure struct
    list.rs       - Returns DirectoryListing struct
    docs.rs       - Returns DocumentationResults struct
  commands/
    mod.rs        - CLI wrappers (call core functions)
    cargo.rs      - CLI wrapper
    read.rs       - CLI wrapper
    ...
  mcp/
    mod.rs        - MCP server setup
    server.rs     - MCP server implementation
    tools.rs      - Tool handlers
    schemas.rs    - JSON schemas for tools
  lib.rs
  main.rs
```

### Phase 2: MCP Server Implementation

**Dependencies**:
- Use **`rmcp`** crate - The official Rust SDK for Model Context Protocol ([GitHub](https://github.com/modelcontextprotocol/rust-sdk))
- Add `tokio` for async runtime (required by rmcp)
- `rmcp` version: `0.8.0` or latest from crates.io

**Why rmcp?**
- Official SDK maintained by Model Context Protocol organization
- Actively maintained (2.6k+ stars, 103 contributors)
- Well-documented with examples
- Uses `ServerHandler` pattern for clean tool registration
- Supports stdio transport natively
- Stable API (v0.8.0+)

**MCP Server Structure**:
- Server runs on stdio (standard MCP transport)
- Each command becomes an MCP tool using `ServerHandler`
- Tools return structured JSON responses
- For commands that write temp files, return file path in response

### Phase 3: Tool Definitions

Each tool needs:
- **Name**: Descriptive tool name
- **Description**: What the tool does
- **Input schema**: JSON schema for parameters
- **Output schema**: JSON schema for results
- **Handler**: Function that executes the tool

## Detailed Tool Specifications

### 1. `tool42_cargo`

**MCP Tool Name**: `tool42_cargo`

**Description**: Execute a Cargo command and write output to a temporary file

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "args": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Cargo subcommand and arguments"
    }
  },
  "required": ["args"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "output_file": {
      "type": "string",
      "description": "Path to temporary file containing cargo output"
    },
    "exit_code": {
      "type": "integer",
      "description": "Exit code from cargo command (0 = success)"
    }
  },
  "required": ["output_file", "exit_code"]
}
```

**Handler**: Calls `core::cargo::execute(args)` → returns `(PathBuf, i32)`

---

### 2. `tool42_read`

**MCP Tool Name**: `tool42_read`

**Description**: Read a file or specific section with line number limits

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "File path to read"
    },
    "from": {
      "type": "integer",
      "minimum": 1,
      "description": "Start line number (1-based, inclusive)"
    },
    "to": {
      "type": "integer",
      "minimum": 1,
      "description": "End line number (1-based, inclusive)"
    }
  },
  "required": ["path"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": { "type": "string" },
      "description": "File lines (max 500)"
    },
    "total_lines": {
      "type": "integer",
      "description": "Total lines in file"
    },
    "lines_returned": {
      "type": "integer",
      "description": "Number of lines returned"
    }
  },
  "required": ["content", "total_lines", "lines_returned"]
}
```

**Handler**: Calls `core::read::read_file(path, from, to)` → returns `ReadResult`

---

### 3. `tool42_describe`

**MCP Tool Name**: `tool42_describe`

**Description**: Extract structural information from a Rust source file

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Rust source file path"
    }
  },
  "required": ["path"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "file": { "type": "string" },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string" },
          "name": { "type": "string" },
          "start_line": { "type": "integer" },
          "end_line": { "type": "integer" },
          "target": { "type": "string" },
          "items": { "type": "array" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::describe::describe_file(path)` → returns `FileDescription`

---

### 4. `tool42_search`

**MCP Tool Name**: `tool42_search`

**Description**: Search for text patterns across the codebase

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Text pattern to search for"
    },
    "path": {
      "type": "string",
      "description": "Optional path to search in (defaults to current directory)"
    }
  },
  "required": ["query"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string" },
    "matches": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "file": { "type": "string" },
          "line": { "type": "integer" },
          "context": { "type": "string" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::search::search(query, path)` → returns `SearchResults`

---

### 5. `tool42_deps`

**MCP Tool Name**: `tool42_deps`

**Description**: Extract dependency information from Cargo.toml

**Input Schema**:
```json
{
  "type": "object",
  "properties": {}
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "workspace_root": { "type": "string" },
    "packages": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "version": { "type": "string" },
          "source": { "type": "string" },
          "dependencies": { "type": "array", "items": { "type": "string" } },
          "features": { "type": "array", "items": { "type": "string" } }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::deps::get_dependencies()` → returns `DependencyInfo`

---

### 6. `tool42_tests`

**MCP Tool Name**: `tool42_tests`

**Description**: List all tests in the project

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Optional path to search for tests (defaults to current directory)"
    }
  }
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "tests": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "file": { "type": "string" },
          "line": { "type": "integer" },
          "module_path": { "type": "string" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::tests::find_tests(path)` → returns `TestResults`

---

### 7. `tool42_project`

**MCP Tool Name**: `tool42_project`

**Description**: Get overview of project structure

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Optional path to project root (defaults to current directory)"
    }
  }
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "workspace_root": { "type": "string" },
    "packages": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "path": { "type": "string" },
          "modules": { "type": "array" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::project::get_structure(path)` → returns `ProjectStructure`

---

### 8. `tool42_list`

**MCP Tool Name**: `tool42_list`

**Description**: List directory contents with metadata

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Optional path to list (defaults to current directory)"
    }
  }
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": { "type": "string" },
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "type": { "type": "string", "enum": ["file", "directory", "other"] },
          "size": { "type": "integer" },
          "modified": { "type": "string" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::list::list_directory(path)` → returns `DirectoryListing`

---

### 9. `tool42_docs`

**MCP Tool Name**: `tool42_docs`

**Description**: Extract doc comments from Rust code

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Rust source file path"
    }
  },
  "required": ["path"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "file": { "type": "string" },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string" },
          "name": { "type": "string" },
          "docs": { "type": "string" },
          "examples": { "type": "array", "items": { "type": "string" } },
          "line": { "type": "integer" }
        }
      }
    }
  }
}
```

**Handler**: Calls `core::docs::extract_docs(path)` → returns `DocumentationResults`

---

## Implementation Steps

### Step 1: Set Up rmcp Dependency

**Tasks**:
- Add `rmcp` crate to `Cargo.toml` with `server` feature
- Add `tokio` dependency (required by rmcp)
- Review rmcp documentation and examples: https://github.com/modelcontextprotocol/rust-sdk
- Study example implementations in rmcp repository

**Library Choice**: **rmcp** (Official Rust SDK)
- ✅ Official SDK from Model Context Protocol organization
- ✅ Actively maintained (2.6k+ stars, recent releases)
- ✅ Well-documented with examples
- ✅ Supports stdio transport
- ✅ Clean `ServerHandler` API
- ✅ Stable version available (0.8.0+)

**Estimated Time**: 1-2 hours (mostly reading docs/examples)

---

### Step 2: Refactor Core Logic

**Tasks**:

1. **Create `src/core/mod.rs`**:
   - Define module structure
   - Re-export all core functions

2. **Refactor each command module**:
   - Extract pure functions that return data structures
   - Keep file I/O separate (for temp file writing)
   - Example for `read.rs`:
     ```rust
     // src/core/read.rs
     pub struct ReadResult {
         pub content: Vec<String>,
         pub total_lines: usize,
         pub lines_returned: usize,
     }
     
     pub fn read_file(path: PathBuf, from: Option<usize>, to: Option<usize>) -> anyhow::Result<ReadResult> {
         // Implementation that returns data instead of printing
     }
     
     // src/commands/read.rs
     pub fn run(path: PathBuf, from: Option<usize>, to: Option<usize>) -> anyhow::Result<()> {
         let result = core::read::read_file(path, from, to)?;
         // Print to stdout
         for line in result.content {
             println!("{}", line);
         }
         Ok(())
     }
     ```

3. **Update all 9 command modules**:
   - `cargo.rs` - Return `(PathBuf, i32)` instead of exiting
   - `read.rs` - Return `ReadResult` struct
   - `describe.rs` - Return `FileDescription` (already exists)
   - `search.rs` - Return `SearchResults` (already exists)
   - `deps.rs` - Return `DependencyInfo` (already exists)
   - `tests.rs` - Return `TestResults` (already exists)
   - `project.rs` - Return `ProjectStructure` (already exists)
   - `list.rs` - Return `DirectoryListing` (already exists)
   - `docs.rs` - Return `DocumentationResults` (already exists)

**Estimated Time**: 8-12 hours

---

### Step 3: Create MCP Server Module

**Tasks**:

1. **Create `src/mcp/mod.rs`**:
   - Module declarations
   - Re-export server types

2. **Create `src/mcp/server.rs`**:
   - Implement `ServerHandler` trait from rmcp
   - Define tool list with schemas
   - Set up stdio transport
   - Server initialization and run loop

3. **Create `src/mcp/tools.rs`**:
   - Tool handler functions (async)
   - Map tool names to handlers
   - Error handling and conversion
   - Response formatting

4. **Create `src/mcp/schemas.rs`**:
   - JSON schema definitions for each tool
   - Input/output type definitions
   - Schema validation helpers

**rmcp Server Pattern**:
```rust
use rmcp::ServerHandler;
use tokio::io::{stdin, stdout};

struct Tool42Server {
    // Tool handlers
}

impl ServerHandler for Tool42Server {
    // Implement tool registration and handling
}

// Server startup
let transport = (stdin(), stdout());
let server = service.serve(transport).await?;
```

**Estimated Time**: 6-8 hours

---

### Step 4: Implement Tool Handlers

**Tasks**:

For each of the 9 tools:
1. Create handler function
2. Map MCP parameters to core function parameters
3. Call core function
4. Format response according to output schema
5. Handle errors appropriately

**Example Handler** (using rmcp):
```rust
// src/mcp/tools.rs
use rmcp::types::Tool;
use serde_json::Value;

pub async fn handle_read(params: Value) -> Result<Value, rmcp::Error> {
    let path = params["path"]
        .as_str()
        .ok_or_else(|| rmcp::Error::InvalidParams("path required".to_string()))?;
    
    let from = params["from"].as_u64().map(|v| v as usize);
    let to = params["to"].as_u64().map(|v| v as usize);
    
    // Call core function (may need to spawn_blocking for sync code)
    let result = tokio::task::spawn_blocking(move || {
        core::read::read_file(PathBuf::from(path), from, to)
    }).await??;
    
    Ok(serde_json::json!({
        "content": result.content,
        "total_lines": result.total_lines,
        "lines_returned": result.lines_returned,
    }))
}

// Tool registration
pub fn register_tools(handler: &mut impl ServerHandler) {
    handler.register_tool(Tool {
        name: "tool42_read".to_string(),
        description: Some("Read a file or file section".to_string()),
        input_schema: read_schema(),
    });
}
```

**Note**: Since core functions are synchronous and rmcp handlers are async, we'll use `tokio::task::spawn_blocking` to run sync code in async context.

**Estimated Time**: 12-16 hours (1.5-2 hours per tool)

---

### Step 5: Add MCP Server Entry Point

**Tasks**:

1. **Update `src/main.rs`**:
   - Add auto-detection logic using `stdin().is_terminal()`
   - When stdin is NOT a TTY, run MCP server (stdio mode)
   - When stdin IS a TTY, run CLI (normal mode)
   - Optionally support `--mcp-server` flag as explicit override

2. **Update `Cargo.toml`**:
   - Add MCP dependencies with feature flag
   - Add `tokio` if needed for async

**Design Decision: Auto-detect Mode**

Instead of requiring a `--mcp-server` flag, we'll auto-detect the execution context:
- **If stdin is NOT a TTY** (pipe/stdio) → MCP server mode (MCP clients communicate via stdio)
- **If stdin IS a TTY** (interactive terminal) → CLI mode (normal command execution)

This allows `tool42` to work seamlessly in both contexts:
- MCP clients: `tool42` (no args, communicates via stdio)
- CLI users: `tool42 read src/lib.rs` (normal CLI usage)

**Example**:
```rust
// src/main.rs
#[cfg(feature = "mcp-server")]
use tokio;
use std::io::IsTerminal;

#[cfg(feature = "mcp-server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    // Check for explicit MCP flag
    let force_mcp = args.len() > 1 && args[1] == "--mcp-server";
    
    // If there are CLI arguments (subcommands), always use CLI mode
    // MCP mode only if: explicit flag OR (no args AND stdin is not a TTY)
    let has_cli_args = args.len() > 1 && !force_mcp;
    let is_tty = std::io::stdin().is_terminal();
    let should_run_mcp = force_mcp || (!has_cli_args && !is_tty);
    
    if should_run_mcp {
        // MCP server mode
        // MCP clients communicate via stdio (not a TTY)
        mcp::server::run().await?;
        Ok(())
    } else {
        // CLI mode - parse normally (clap handles --mcp-server if present)
        let cli = tool42::Cli::parse();
        if let Err(e) = tool42::run(cli) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        Ok(())
    }
}

#[cfg(not(feature = "mcp-server"))]
fn main() {
    let cli = tool42::Cli::parse();
    if let Err(e) = tool42::run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

**Benefits of Auto-detection**:
- ✅ No flags needed - works automatically
- ✅ MCP clients can just run `tool42` directly
- ✅ CLI users see no change in behavior
- ✅ Follows common pattern for dual-mode tools
- ✅ Backward compatible

**Logic Summary**:
- **Has CLI arguments** (e.g., `tool42 read src/lib.rs`) → Always CLI mode
- **No arguments + stdin is TTY** → CLI mode (clap will show help/error)
- **No arguments + stdin is NOT TTY** → MCP server mode (MCP client context)
- **`--mcp-server` flag** → Force MCP server mode (explicit override)

This ensures:
- ✅ CLI users see no change: `tool42 read src/lib.rs` works as before
- ✅ MCP clients work seamlessly: `tool42` (no args, stdio) → MCP server
- ✅ Explicit control: `tool42 --mcp-server` forces MCP mode

**Note**: When `mcp-server` feature is enabled, main becomes async. When disabled, it remains sync.

**Estimated Time**: 2-3 hours

---

### Step 6: Testing

**Tasks**:

1. **Unit Tests**:
   - Test core functions independently
   - Test MCP tool handlers
   - Test schema validation

2. **Integration Tests**:
   - Test MCP server with sample requests
   - Test all 9 tools via MCP
   - Verify responses match schemas

3. **CLI Compatibility Tests**:
   - Ensure CLI still works after refactoring
   - Test all commands still function correctly

**Estimated Time**: 8-12 hours

---

### Step 7: Documentation

**Tasks**:

1. **Update README.md**:
   - Add MCP server section
   - Document how to use MCP interface
   - Provide example MCP client configuration

2. **Add MCP-specific docs**:
   - Tool reference documentation
   - Schema documentation
   - Example requests/responses

**Estimated Time**: 4-6 hours

---

## Cargo.toml Changes

```toml
[features]
default = []
mcp-server = []

[dependencies]
# Existing dependencies...
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
syn = { version = "2.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quote = "1.0"
regex = "1.10"

# MCP dependencies (only when mcp-server feature is enabled)
[target.'cfg(feature = "mcp-server")'.dependencies]
rmcp = { version = "0.8", features = ["server"] }
tokio = { version = "1.0", features = ["full"] }
```

**Note**: `rmcp` requires the `server` feature to enable server functionality. The `tokio` dependency is required for async runtime.

## Project Structure After Implementation

```
src/
  core/
    mod.rs
    cargo.rs
    read.rs
    describe.rs
    search.rs
    deps.rs
    tests.rs
    project.rs
    list.rs
    docs.rs
  commands/
    mod.rs
    cargo.rs      # Thin wrapper calling core::cargo
    read.rs       # Thin wrapper calling core::read
    describe.rs   # Thin wrapper calling core::describe
    search.rs     # Thin wrapper calling core::search
    deps.rs       # Thin wrapper calling core::deps
    tests.rs      # Thin wrapper calling core::tests
    project.rs    # Thin wrapper calling core::project
    list.rs       # Thin wrapper calling core::list
    docs.rs       # Thin wrapper calling core::docs
  mcp/
    mod.rs
    server.rs
    tools.rs
    schemas.rs
  lib.rs
  main.rs
```

## Usage Examples

### CLI Usage (unchanged)
```bash
tool42 read src/lib.rs --from 10 --to 20
tool42 cargo check
tool42 describe src/lib.rs
```

### MCP Server Usage
```bash
# MCP server auto-starts when stdin is not a TTY (stdio/pipe)
# MCP clients just run tool42 directly:
tool42

# Or build with feature flag
cargo build --features mcp-server
```

### MCP Client Configuration
```json
{
  "mcpServers": {
    "tool42": {
      "command": "tool42"
      // No args needed! Auto-detects MCP mode via stdio
    }
  }
}
```

**Note**: The MCP server automatically starts when `tool42` is invoked with stdin connected to a pipe (non-TTY), which is how MCP clients communicate. CLI usage remains unchanged.

## Benefits of This Approach

1. **Separation of Concerns**: Core logic separated from I/O
2. **Dual Interface**: Both CLI and MCP available
3. **Optional Feature**: MCP doesn't bloat CLI-only users
4. **Maintainability**: Single source of truth for each command
5. **Testability**: Core functions easily testable without I/O
6. **Future-proof**: Easy to add more interfaces (HTTP API, etc.)

## Risks and Mitigations

### Risk 1: MCP Library Maturity
- **Risk**: ~~Chosen library may be unstable or poorly maintained~~ ✅ **Mitigated**: Using official `rmcp` SDK
- **Mitigation**: Official SDK from MCP organization, actively maintained with 2.6k+ stars

### Risk 2: Breaking Changes During Refactoring
- **Risk**: Refactoring may break existing CLI functionality
- **Mitigation**: Comprehensive test suite, incremental refactoring

### Risk 3: Performance Overhead
- **Risk**: MCP server may add overhead
- **Mitigation**: Feature flag ensures no overhead for CLI users, profile and optimize

### Risk 4: Async Complexity
- **Risk**: MCP may require async, adding complexity
- **Mitigation**: Use async only where needed, keep sync core functions

## Timeline Estimate

- **Step 1**: 1-2 hours (reduced - using official SDK)
- **Step 2**: 8-12 hours
- **Step 3**: 6-8 hours
- **Step 4**: 12-16 hours
- **Step 5**: 2-3 hours
- **Step 6**: 8-12 hours
- **Step 7**: 4-6 hours

**Total**: 41-59 hours (~1-1.5 weeks of focused development)

## Success Criteria

1. ✅ All 9 commands work via MCP interface
2. ✅ All 9 commands still work via CLI interface
3. ✅ MCP server can be built as optional feature
4. ✅ All tools have proper JSON schemas
5. ✅ Comprehensive test coverage
6. ✅ Documentation updated
7. ✅ No performance regression for CLI users

## Next Steps

1. ✅ Review and approve this plan
2. ✅ **Library chosen**: `rmcp` (official Rust SDK)
3. Start with Step 1 (set up rmcp dependency)
4. Proceed incrementally through steps
5. Test thoroughly at each step

## References

- **rmcp SDK**: https://github.com/modelcontextprotocol/rust-sdk
- **MCP Specification**: https://modelcontextprotocol.io
- **rmcp Examples**: https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples


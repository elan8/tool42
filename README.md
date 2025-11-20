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

### From Source (Current)

Currently, Tool42 must be installed from source:

```bash
git clone https://github.com/elan8/tool42.git
cd tool42
cargo install --path .
```

This will install the `tool42` binary to your Cargo bin directory (typically `~/.cargo/bin` on Unix systems or `%USERPROFILE%\.cargo\bin` on Windows).

### From crates.io (Coming Soon)

Tool42 will be available on crates.io soon, making installation easier:

```bash
cargo install tool42
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

### `tool42_cargo`

Execute Cargo commands with pagination support for handling large outputs.

**Parameters:**
- `args` (required): Array of strings representing the Cargo command and its arguments (e.g., `["build"]`, `["test", "--release"]`)
- `working_directory` (required): Absolute path to the directory containing `Cargo.toml`
- `from` (optional): Starting line number (1-based, inclusive) for pagination. If omitted, defaults to line 1
- `to` (optional): Ending line number (1-based, inclusive) for pagination. If omitted, defaults to line 500

**Returns:**
- `output_lines`: Array of output lines (strings)
- `total_lines`: Total number of lines in the complete output
- `lines_returned`: Number of lines returned in this response
- `exit_code`: Exit code from the Cargo command

**Notes:**
- Default behavior: Returns first 500 lines if no pagination parameters are specified
- Output is cached after first execution for efficient range requests
- Combines stdout and stderr into a single output stream
- Maximum 500 lines per request (even if `to` exceeds this limit)

---

### `tool42_clippy`

Execute `cargo clippy` with pagination support for handling large lint outputs.

**Parameters:**
- `args` (required): Array of strings representing additional Clippy arguments (e.g., `["--", "-W", "clippy::all"]`)
- `working_directory` (required): Absolute path to the directory containing `Cargo.toml`
- `from` (optional): Starting line number (1-based, inclusive) for pagination. If omitted, defaults to line 1
- `to` (optional): Ending line number (1-based, inclusive) for pagination. If omitted, defaults to line 500

**Returns:**
- `output_lines`: Array of output lines (strings)
- `total_lines`: Total number of lines in the complete output
- `lines_returned`: Number of lines returned in this response
- `exit_code`: Exit code from the Clippy command

**Notes:**
- **Requires Clippy to be installed**: Run `rustup component add clippy` if not already installed
- Default behavior: Returns first 500 lines if no pagination parameters are specified
- Output is cached after first execution for efficient range requests
- Maximum 500 lines per request (even if `to` exceeds this limit)

---

### `tool42_read`

Read a file or specific section with line number limits.

**Parameters:**
- `path` (required): Relative or absolute path to the file to read
- `working_directory` (required): Absolute path used to resolve relative paths
- `from` (optional): Starting line number (1-based, inclusive). If omitted, starts from line 1
- `to` (optional): Ending line number (1-based, inclusive). If omitted, reads up to line 500 or end of file

**Returns:**
- `content`: Array of file lines (strings)
- `total_lines`: Total number of lines in the file
- `lines_returned`: Number of lines returned in this response

**Notes:**
- Maximum 500 lines per request (even if `to` exceeds this limit)
- Line numbers are 1-based (first line is line 1, not line 0)
- Automatically resolves relative paths based on `working_directory`

---

### `tool42_describe`

Parse a Rust source file and extract comprehensive structural information in JSON format.

**Parameters:**
- `path` (required): Relative or absolute path to the Rust source file (`.rs`)
- `working_directory` (required): Absolute path used to resolve relative paths

**Returns:**
A hierarchical JSON structure containing:
- **Structs**: With all fields, their types, visibility, attributes, and doc comments
- **Enums**: With all variants and their associated data
- **Functions**: With complete signatures (parameters, return types, visibility, attributes)
- **Impl blocks**: With nested methods and associated items
- **Traits**: With associated items and method signatures
- **Modules**: With nested items
- **Type aliases**: With their definitions
- **Constants and static items**: With their types and values
- **Macros**: With their definitions
- **Use statements**: Import information
- **Unions**: With their fields
- **Extern crates**: External dependencies

For each item, provides:
- Name and type
- Start and end line numbers
- Visibility (pub, pub(crate), etc.)
- Attributes (e.g., `#[derive(...)]`, `#[cfg(...)]`)
- Doc comments
- Signatures (for functions/methods)
- Fields (for structs/unions)

**Notes:**
- Useful for understanding file structure before reading or analyzing code
- Returns structured JSON that can be programmatically processed
- Provides complete code organization information in a single call

---

### `tool42_search`

Search for text patterns in Rust source files (`.rs`) across the codebase.

**Parameters:**
- `query` (required): Text pattern to search for (treated as a literal string, regex-escaped, case-insensitive)
- `path` (optional): Specific directory or file to search within. If omitted, searches from workspace root
- `working_directory` (required): Absolute path used to resolve relative paths and find workspace root

**Returns:**
Array of matches, each containing:
- `file`: Path to the file containing the match
- `line`: Line number where the match was found
- `content`: The matching line with context

**Notes:**
- Query is matched case-insensitively
- Automatically skips hidden files/directories and the `target` directory
- If no `path` is specified, searches from the workspace root (found from `working_directory`)
- Query cannot be empty

---

### `tool42_deps`

Extract comprehensive dependency information from a Rust project using `cargo metadata`.

**Parameters:**
- `working_directory` (required): Absolute path to the directory containing `Cargo.toml`

**Returns:**
- `workspace_root`: Absolute path to the workspace root
- `packages`: Array of all packages, each containing:
  - `name`: Package name
  - `version`: Package version
  - `source`: Dependency source (registry, path, git, etc.)
  - `dependencies`: Direct dependencies with versions and features
  - `features`: Available feature flags

**Notes:**
- Works for both single-package projects and Cargo workspaces
- Includes all transitive dependencies
- Provides complete dependency graph structure
- Uses `cargo metadata` for accurate dependency resolution

---

### `tool42_tests`

Discover and list all test functions in a Rust project.

**Parameters:**
- `working_directory` (required): Absolute path used to find the workspace root

**Returns:**
Array of tests, each containing:
- `name`: Test function name
- `file`: Path to the file containing the test
- `line`: Line number where the test is defined
- `module`: Module path where the test is located

**Notes:**
- Scans the workspace root (found from `working_directory`) for all Rust source files
- Identifies functions with the `#[test]` attribute
- Automatically skips hidden files/directories and the `target` directory
- Useful for understanding test suite structure and locating specific tests

---

### `tool42_project`

Get a high-level overview of the Rust project structure.

**Parameters:**
- `working_directory` (required): Absolute path used to find the workspace root

**Returns:**
- `workspace`: Workspace information (if applicable), including:
  - `root`: Workspace root path
  - `members`: List of workspace member packages
  - `dependencies`: Workspace-level dependencies
- `packages`: Array of packages, each containing:
  - `name`: Package name
  - `version`: Package version
  - `edition`: Rust edition (e.g., "2021")
  - `description`: Package description
  - `license`: License information
  - `dependencies`: Package-level dependencies
  - `dev_dependencies`: Development dependencies
  - `build_dependencies`: Build dependencies
  - `crates`: Array of crates in the package, each with:
    - `name`: Crate name
    - `type`: Crate type ("lib" or "bin")

**Notes:**
- For workspaces, only includes packages listed in `workspace.members`
- For non-workspace projects, includes only the root package
- Uses `Cargo.toml` to determine project structure (not directory scanning)
- Provides metadata and dependency information at the package level

---

### `tool42_list`

List the contents of a directory with metadata.

**Parameters:**
- `path` (optional): Specific directory to list. If omitted, lists the workspace root
- `working_directory` (required): Absolute path used to resolve relative paths and find workspace root

**Returns:**
Array of directory entries, each containing:
- `name`: Entry name
- `type`: Entry type ("file", "directory", or "other")
- `size`: File size in bytes (for files only)
- `modified`: Modification timestamp (ISO 8601 format)

**Notes:**
- Entries are sorted with directories first, then files, both alphabetically
- Returns structured JSON with entry metadata for easy programmatic access
- Useful for exploring project structure, locating source files, and understanding directory organization

---

### `tool42_docs`

Extract documentation comments from a specific Rust source file.

**Parameters:**
- `path` (required): Relative or absolute path to the Rust source file (`.rs`)
- `working_directory` (required): Absolute path used to resolve relative paths

**Returns:**
Structured documentation organized by item type, each containing:
- Item name and type
- Doc comments (/// and //! style)
- Examples (if present in doc comments)
- Line numbers for easy reference

**Notes:**
- Parses the file to identify all documented items (functions, structs, enums, traits, impl blocks, modules)
- Returns documentation in a structured format
- Useful for quickly understanding API documentation, usage examples, and public interfaces without reading the full source code

---

### `tool42_refactor_rename`

Rename a symbol (struct, enum, function, type alias, constant, etc.) across the entire codebase.

**Parameters:**
- `symbol` (required): Current name of the symbol to rename
- `to` (required): New name for the symbol
- `working_directory` (required): Absolute path to the project root
- `path` (optional): Specific file or directory path to scope the search. If omitted, searches entire codebase
- `preview` (optional): If `true` (default), shows changes without applying them
- `apply` (optional): If `true`, executes the refactoring. Defaults to `false`

**Returns:**
Detailed list of all changes, including:
- File paths modified
- Line numbers where changes occurred
- Context showing old and new code
- Validation status (if `apply` is true)

**Notes:**
- Searches for all occurrences including definitions, usages, imports, and references
- Supports preview mode (default) to review changes before applying
- Automatically creates backup files before making changes
- Validates changes with `cargo check` after applying
- Can scope search to a specific file or directory

---

### `tool42_refactor_extract`

Extract a code block (specified by line range) from a Rust source file into a new function.

**Parameters:**
- `file` (required): Path to the Rust source file containing the code to extract
- `working_directory` (required): Absolute path used to resolve relative paths
- `from` (required): Starting line number (1-based, inclusive) of the code block to extract
- `to` (required): Ending line number (1-based, inclusive) of the code block to extract
- `name` (required): Name for the new function
- `preview` (optional): If `true` (default), shows changes without applying them
- `apply` (optional): If `true`, executes the refactoring. Defaults to `false`

**Returns:**
Detailed changes showing:
- The extracted code that will become the new function
- The replacement function call
- File path and line numbers affected
- Validation status (if `apply` is true)

**Notes:**
- Creates a new function containing the extracted code
- Replaces the original code block with a function call
- Handles variable scoping and ensures the extracted function receives necessary parameters
- Supports preview mode (default) to review changes before applying
- Validates changes with `cargo check` after applying
- Useful for breaking down large functions, improving code reusability, and enhancing readability

---

### `tool42_refactor_move`

Move a function, struct, enum, or other item to a different module or file.

**Parameters:**
- `symbol` (required): Name of the item to move (function, struct, enum, etc.)
- `to` (required): Target location as a module path (e.g., `"crate::utils::helpers"`) or file path
- `working_directory` (required): Absolute path to the project root
- `preview` (optional): If `true` (default), shows changes without applying them
- `apply` (optional): If `true`, executes the refactoring. Defaults to `false`

**Returns:**
Detailed list of all files modified, showing:
- What was moved and where
- Updated imports and references
- File paths and line numbers affected
- Validation status (if `apply` is true)

**Notes:**
- Searches the codebase to find the item definition and all its usages
- Automatically updates all imports, references, and usages throughout the codebase
- Supports preview mode (default) to review changes before applying
- Validates changes with `cargo check` after applying
- Useful for reorganizing code structure, improving module organization, and separating concerns

---

### `tool42_refactor_signature`

Change a function's signature (parameters, return type, visibility, etc.) and automatically update all call sites.

**Parameters:**
- `function` (required): Name of the function to modify
- `new_signature` (required): Complete function signature as it should appear (e.g., `"pub fn my_function(x: i32, y: String) -> bool"`)
- `working_directory` (required): Absolute path to the project root
- `preview` (optional): If `true` (default), shows changes without applying them
- `apply` (optional): If `true`, executes the refactoring. Defaults to `false`

**Returns:**
Detailed changes showing:
- Old and new signatures at the definition
- All call sites with their updates
- File paths and line numbers affected
- Validation status (if `apply` is true)

**Notes:**
- Searches for the function definition and all places where it's called
- Updates all call sites to match the new signature
- Supports preview mode (default) to review changes before applying
- Validates changes with `cargo check` after applying
- Useful for refactoring APIs, adding or removing parameters, changing return types, and updating function visibility

---

**Common Notes for All Tools:**
- All tools return structured JSON responses according to the MCP specification
- The `working_directory` parameter must always be an absolute path
- All refactoring tools support preview mode (default) and require explicit `apply: true` to make changes
- Refactoring tools automatically validate changes with `cargo check` after applying

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
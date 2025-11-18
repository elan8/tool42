<!-- 219b602e-e775-4715-b086-f944f441a45b 380ec835-73fb-41ae-8837-73d057f0b067 -->
# Tool42 Functionality Expansion Suggestions (Revised)

## Key Insight

The main value of tool42 is **avoiding serialization crashes** with large outputs (writing to temp files), not parsing complexity. AI agents can already parse plain text effectively, including compiler errors and lint output.

**JSON is valuable when:**

- Data requires complex parsing (like Rust AST with `syn` in `describe`)
- Structure is hard to extract from plain text
- Relationships/hierarchy matter (like project structure)

**JSON is NOT needed when:**

- Plain text is already well-structured (errors, lint output)
- AI agents can parse it effectively
- The main issue is just output size (solved by temp files)

## High Priority Additions

### 1. `tool42 search` - Code Search

Search for symbols, functions, types, or text patterns across the codebase. Output structured JSON with file paths and line numbers.

**Why JSON?** Finding all matches across multiple files and organizing by file/line requires structured data.

**Use cases:**

- Find all usages of a function/type
- Search for specific patterns or text
- Locate where symbols are defined/used

**Output:** JSON file with matches (file, line, context snippet)

### 2. `tool42 deps` - Dependency Analysis

Extract and structure dependency information from Cargo.toml and Cargo.lock. Output JSON with dependency tree, versions, features, etc.

**Why JSON?** Dependency relationships form a tree/graph structure that's easier to navigate in JSON.

**Use cases:**

- Understand project dependencies
- Check for version conflicts
- Analyze feature flags
- Understand dependency tree

**Output:** JSON file with structured dependency data

**Note:** `cargo metadata` exists but outputs large JSON. This could provide a filtered/structured view.

### 3. `tool42 tests` - Test Discovery

List all tests in the project with their locations, names, and module paths. Output structured JSON.

**Why JSON?** Test discovery across multiple files/modules benefits from structured organization.

**Use cases:**

- Discover available tests
- Run specific tests
- Understand test structure
- Find tests related to specific modules

**Output:** JSON file with test information (file, line, name, module path, attributes)

## Medium Priority Additions

### 4. `tool42 project` - Project Structure Overview

Get an overview of the entire project structure: modules, files, their relationships. Output JSON.

**Why JSON?** Project structure is hierarchical (modules contain modules, files contain items) - JSON represents this well.

**Use cases:**

- Understand project layout
- Navigate large codebases
- Find related files
- Understand module hierarchy

**Output:** JSON file with project structure tree

### 5. `tool42 list` - Directory Listing

List directory contents with metadata (file types, sizes, modification times). Output structured JSON.

**Why JSON?** Filtering/sorting by metadata is easier with structured data.

**Use cases:**

- Explore project structure
- Find files by type
- Understand file organization
- Filter by size/modification time

**Output:** JSON file with directory contents

### 6. `tool42 docs` - Documentation Extraction

Extract doc comments from Rust code and output structured format. Include item names, doc strings, examples.

**Why JSON?** Organizing docs by item type/name and extracting examples benefits from structure.

**Use cases:**

- Understand code documentation
- Extract examples
- Generate documentation summaries
- Find undocumented items

**Output:** JSON file with doc comments organized by item

## Lower Priority / Advanced Features

### 7. `tool42 git` - Git Operations Wrapper

Wrapper around common git commands (status, diff, log) with structured JSON output.

**Why JSON?** Git output can be parsed, but structured data makes it easier to filter/analyze changes.

**Use cases:**

- Check git status (structured)
- Get diffs in structured format
- Analyze commit history
- Filter changes by file type

**Output:** JSON file with git information

**Note:** May be less valuable since `git status --porcelain` and `git diff --name-status` already provide structured output.

### 8. `tool42 metrics` - Code Metrics

Calculate code metrics (lines of code, complexity, etc.) and output JSON.

**Use cases:**

- Code analysis
- Project health checks
- Complexity analysis

**Output:** JSON file with metrics

### 9. `tool42 find` - File Finding

Find files by name patterns, extensions, or other criteria. Output JSON list.

**Use cases:**

- Locate files in large projects
- Find files by type/pattern

**Output:** JSON file with matching file paths

## Design Principles

All new commands should follow existing patterns:

- Output to temporary files (avoid serialization issues)
- Use JSON for structured data **only when extraction is non-trivial**
- Print file path to stdout
- Handle errors gracefully
- Support common AI agent workflows

## Implementation Priority Recommendation

**Phase 3 (Next):**

1. `tool42 search` - Most versatile, high utility for finding code across codebase
2. `tool42 deps` - Extract dependency info (though `cargo metadata` exists, structured extraction could help)
3. `tool42 tests` - Discover tests with locations (useful for test workflows)

**Phase 4:**

4. `tool42 project` - Helps navigate large codebases (structured overview)
5. `tool42 list` - Basic file system operations (structured directory listing)
6. `tool42 docs` - Extract doc comments (if doc parsing adds value beyond plain text)

**Phase 5+:**
Remaining features based on user feedback and needs

## Implementation Notes

### `tool42 search`

- **Implementation**: Custom Rust code using regex or string matching
- Parse Rust source files directly (similar to `describe`)
- Can use `syn` for Rust-aware symbol searching (find struct names, function names, etc.)
- For text search, use Rust's regex crate or simple string matching
- Walk directory tree using `std::fs` to find files

### `tool42 deps`

- **Implementation**: Use `cargo metadata --format-version 1` command
- `cargo metadata` is a built-in cargo command (no separate installation needed)
- Parse and filter the JSON output to extract relevant dependency information
- Build dependency tree from the metadata output
- Extract versions, features, workspace structure, and relationships
- Write filtered/structured JSON to temp file (cargo metadata output can be very large)

### `tool42 tests`

- **Implementation**: Parse Rust source files directly using `syn` (like `describe`)
- Find all `#[test]` attributes and test function definitions
- Extract test names, locations, and module paths from AST
- No need to compile or run cargo commands

### `tool42 project`

- **Implementation**: Parse Cargo.toml for workspace structure
- Walk filesystem to discover source files
- Parse module declarations using `syn` to understand module hierarchy
- Build project structure tree from parsed data

### `tool42 list`

- **Implementation**: Use `std::fs::read_dir` and file metadata
- Pure Rust standard library implementation
- No external dependencies needed

### `tool42 docs`

- **Implementation**: Parse Rust source files using `syn`
- Extract doc comments from AST (similar to `describe`)
- Parse doc comment attributes and examples
- Organize by item type and name

### To-dos

- [x] Implement tool42 search - search for symbols/patterns across codebase, output JSON
- [x] Implement tool42 deps - extract dependency info using cargo metadata, output JSON
- [x] Implement tool42 tests - discover tests using syn parsing, output JSON
- [x] Implement tool42 project - project structure overview, output JSON
- [x] Implement tool42 list - directory listing with metadata, output JSON
- [x] Implement tool42 docs - extract doc comments using syn, output JSON
- [x] Update src/commands/mod.rs to add new command modules and CLI definitions
- [x] Update src/lib.rs to wire up new command handlers
- [x] Update Cargo.toml with any new dependencies (regex, toml, etc.)
- [x] Test compilation with cargo check
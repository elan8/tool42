<!-- 743fef58-14b0-4fe0-a64d-8754cd063b0f 55a4f462-4475-4c1c-a841-146744a4fc15 -->
# Phase 2 Implementation Plan

## Overview

Implement two new commands for Phase 2:

- `tool42 read`: Read files with optional line range and default 500-line limit, outputting directly to stdout with strict limits
- `tool42 describe`: Parse Rust source files and extract structural information (structs, functions, impl blocks, etc.) with line numbers

The `read` command outputs directly to stdout (with limits), while `describe` writes to temporary files (like `tool42 cargo`) to prevent AI agent serialization issues.

## Implementation Details

### 1. `tool42 read` Command

**File**: `src/commands/read.rs`

**Functionality**:

- Read a file from the filesystem
- Support optional `--from` and `--to` line number arguments (1-based, inclusive)
- Default limit of 500 lines if no range specified and file exceeds 500 lines
- Write output directly to stdout (not temp file) to keep it simple for file reading
- Apply strict output limits (max 500 lines) even when custom ranges are specified to prevent AI agent serialization issues
- Handle errors gracefully (file not found, invalid line numbers, etc.)

**Implementation steps**:

- Create `read.rs` module with `run()` function
- Parse file path and optional line range arguments
- Read file line by line, applying range filter if specified
- Enforce maximum 500-line limit regardless of range specified (clamp ranges if needed)
- Write filtered content directly to stdout
- Return appropriate exit codes (0 for success, non-zero for errors)

**CLI arguments** (already defined in `commands/mod.rs`):

- `path: PathBuf` - file to read
- `from: Option<usize>` - start line (1-based, inclusive)
- `to: Option<usize>` - end line (1-based, inclusive)

**Range behavior**:

- Line numbers are 1-based (first line is line 1, not line 0)
- Range is inclusive on both ends: `--from 10 --to 20` includes lines 10 and 20
- Examples:
  - `--from 10 --to 20` reads lines 10, 11, 12, ..., 20 (11 lines total)
  - `--from 1 --to 5` reads lines 1, 2, 3, 4, 5 (5 lines total)
  - `--from 10` (no `--to`) reads from line 10 to end of file (up to 500 line limit)
  - `--to 20` (no `--from`) reads from start of file to line 20 (up to 500 line limit)

**Line limit logic**:

- If no range specified: read first 500 lines
- If range specified but exceeds 500 lines: clamp to 500 lines (from start of range)
- If range is within 500 lines: respect the range exactly (inclusive on both ends)

### 2. `tool42 describe` Command

**File**: `src/commands/describe.rs`

**Functionality**:

- Parse Rust source file using `syn` crate
- Extract structural elements: structs, enums, functions, impl blocks, traits, modules, type aliases, constants, static items
- Parse nested items recursively (e.g., methods inside impl blocks, items inside modules, nested structs/enums)
- For each item, record: name, type (struct/enum/fn/etc.), start line, end line
- Maintain hierarchy in JSON output (e.g., methods nested under impl blocks, items nested under modules)
- Output in JSON format for easy AI agent parsing
- Write output to temp file (pattern: `tool42_describe_<timestamp>_<random>.txt`)
- Print temp file path to stdout

**Dependencies**:

- Add `syn` crate to `Cargo.toml` for Rust code parsing
- Add `quote` crate (likely already available transitively, but may need explicit dependency)

**Implementation steps**:

- Add `syn` dependency to `Cargo.toml`
- Create `describe.rs` module with `run()` function
- Parse Rust file using `syn::parse_file()`
- Traverse AST to extract items with line number tracking
- Format output as structured text (e.g., "struct MyStruct (lines 10-25)")
- Write to temp file and print path to stdout
- Handle parse errors gracefully (invalid Rust code, file not found, etc.)

**Line number tracking**:

- Use `syn::spanned::Spanned` trait to get line numbers
- Convert byte offsets to line numbers by counting newlines in source

**Output format** (example):

```
File: src/lib.rs

structs:
  MyStruct (lines 10-25)
  AnotherStruct (lines 30-45)

functions:
  my_function (lines 50-60)
  helper_function (lines 65-75)

impl blocks:
  impl MyStruct (lines 80-120)
    method1 (lines 85-90)
    method2 (lines 95-110)

...
```

### 3. Integration

**Files to modify**:

- `src/commands/mod.rs`: Add `pub mod read;` and `pub mod describe;`
- `src/lib.rs`: Update `run()` function to call `read::run()` and `describe::run()`
- `Cargo.toml`: Add `syn` dependency

**Error handling**:

- Use `anyhow::Context` for error messages (consistent with `cargo.rs`)
- Return appropriate exit codes
- Provide clear error messages for common failures

### 4. Testing

**Test files**:

- `tests/read_command.rs`: Tests for `tool42 read`
- `tests/describe_command.rs`: Tests for `tool42 describe`

**Test cases for `read`**:

- Read entire file (should limit to 500 lines if file is larger)
- Read with line range (should respect range but clamp to 500 lines max)
- Read file exceeding 500 lines (should truncate to 500 lines)
- Read non-existent file (should error)
- Read with invalid line numbers (should error)
- Verify stdout output format and line limits

**Test cases for `describe`**:

- Describe simple Rust file with structs and functions
- Describe file with nested items
- Describe invalid Rust file (should handle gracefully)
- Describe non-existent file (should error)
- Verify output format and line numbers
- Verify temp file creation and format

## Files to Create/Modify

**New files**:

- `src/commands/read.rs`
- `src/commands/describe.rs`
- `tests/read_command.rs`
- `tests/describe_command.rs`

**Modified files**:

- `src/commands/mod.rs` - Add module declarations
- `src/lib.rs` - Wire up command handlers
- `Cargo.toml` - Add `syn` dependency

## Dependencies

- `syn` - Rust code parsing (add to `Cargo.toml`)
- `serde_json` - JSON serialization (add to `Cargo.toml`)
- `serde` - Serialization framework (add to `Cargo.toml`, needed by `serde_json`)
- Existing: `clap`, `anyhow`, `std::fs`, `std::path`, `std::env`
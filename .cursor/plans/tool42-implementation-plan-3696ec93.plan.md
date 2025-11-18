<!-- 3696ec93-8c63-4905-b33b-b92493e4a1be 65dd867f-f564-4267-add5-747b5cf74194 -->
# Tool42 Implementation Plan (Test-Driven Development)

## Project Structure

Create a standard Rust CLI project structure:

- `Cargo.toml` - Project manifest with dependencies (clap, anyhow, etc.)
- `src/main.rs` - Entry point and CLI setup
- `src/lib.rs` - Library code for testability
- `src/commands/` - Module for subcommands
                - `src/commands/mod.rs` - Commands module
                - `src/commands/cargo.rs` - Cargo wrapper implementation
                - `src/commands/read.rs` - File reading implementation (Phase 2)
                - `src/commands/describe.rs` - Rust source analysis implementation (Phase 3)
- `tests/` - Integration tests directory
                - `tests/common/` - Test utilities and helpers
                                - `tests/common/mod.rs` - Test utilities module
                                - `tests/common/test_project.rs` - Helper for creating temporary Rust projects
                - `tests/cargo_command.rs` - Integration tests for `tool42 cargo`

## Phase 1: Core Infrastructure and `tool42 cargo` (TDD Approach)

### 1. Project Setup

- Initialize Rust project with `cargo init --lib` (to enable library mode for testing)
- Add dependencies to `Cargo.toml`:
                - `clap` with `derive` feature for CLI parsing
                - `anyhow` for error handling
                - Use `std::env::temp_dir()` for temp directory access
                - `uuid` feature `v4` for generating unique filenames (or use timestamp + random)
- Set up test infrastructure:
                - Create `tests/` directory structure
                - Create test utilities for temporary project creation
                - Create test helpers for file operations and assertions

### 2. Test-Driven Development: Write Tests First

#### 2.1 Integration Tests (`tests/cargo_command.rs`)

**Happy Path Tests:**

- Test `tool42 cargo --version` (simple command that always works)
- Test `tool42 cargo check` on a valid Rust project
- Test `tool42 cargo build` on a valid Rust project
- Test `tool42 cargo test` on a project with tests
- Verify output file is created in temp directory
- Verify output file contains combined stdout/stderr
- Verify output file path is printed to stdout (exactly once)
- Verify exit code matches cargo's exit code (0 for success)
- Verify output file persists after command completes
- Test with various cargo flags (`--release`, `--verbose`, etc.)

**Error Handling Tests:**

- Test with invalid cargo command (e.g., `tool42 cargo nonexistent`)
- Test with cargo command that fails (e.g., `tool42 cargo build` on broken code)
- Test when temp directory is not accessible (simulate permissions issue)
- Test when temp directory doesn't exist (edge case)
- Test when file cannot be written (simulate disk full scenario)
- Test when cargo binary is not found in PATH
- Test with empty arguments (`tool42 cargo`)
- Test with special characters in cargo arguments
- Test with very long argument lists

**Edge Cases:**

- Test with very large cargo output (stress test with verbose output)
- Test with empty cargo output (some commands produce no output)
- Test with binary output (should handle gracefully, not corrupt)
- Test concurrent executions (multiple tool42 cargo calls simultaneously)
- Test with cargo commands that produce no output
- Test with cargo commands that only produce stderr (errors)
- Test with cargo commands that only produce stdout
- Test file path uniqueness (multiple runs don't overwrite)
- Test filename format (contains expected prefix, is valid path)
- Test with cargo subcommands that don't exist

**CLI Tests:**

- Test argument parsing (all cargo flags pass through correctly)
- Test help output (`tool42 --help`, `tool42 cargo --help`)
- Test version output (`tool42 --version`)
- Test invalid CLI usage (missing subcommand)
- Test trailing arguments with hyphens (e.g., `tool42 cargo -- --test-flag`)

**Output Format Tests:**

- Verify stdout and stderr are both captured
- Verify output order (stdout first, then stderr, or interleaved appropriately)
- Verify no extra formatting is added to output file
- Verify output file is plain text (not binary)

### 3. Implementation (After Tests)

#### 3.1 Core Functionality (`src/commands/cargo.rs`)

- Create subcommand that accepts arbitrary cargo arguments (pass-through all args to cargo)
- Execute cargo command via `std::process::Command`
- Capture both stdout and stderr using `Command::output()`
- Combine stdout and stderr into a single stream (stdout first, then stderr, or interleave)
- Write combined output to a temporary file in OS temp directory
- Generate unique filename using format: `tool42_cargo_<timestamp>_<random>.txt`
                - Use unique filenames to support concurrent executions and multiple runs
                - Format: `tool42_cargo_<unix_timestamp>_<random_hex>.txt` (e.g., `tool42_cargo_1704067200_a3f2.txt`)
- Print the output file path to stdout after execution (always, regardless of success/failure)
- Preserve cargo's exit code and propagate it via `std::process::exit()`
- Implement automatic cleanup of old output files (see File Cleanup section below)

#### 3.2 Robust Error Handling

- Handle cases where cargo command fails to execute (command not found) - return clear error
- Handle cases where cargo command returns non-zero exit code (preserve and propagate)
- Handle file I/O errors:
                - Temp directory access failures (permissions, doesn't exist) - return error with context
                - File creation failures - return error with file path
                - File writing failures (disk full, permissions) - return error with details
- Handle cases where temp directory path is invalid - return error
- Handle cases where output is too large for memory - use streaming if needed, or handle gracefully
- Provide clear, actionable error messages using anyhow with context
- Ensure no panics - all errors should be Result-based
- Log errors appropriately (consider using `eprintln!` for errors)

#### 3.3 Edge Case Handling

- Handle empty output gracefully (create file even if empty)
- Handle binary output (detect and handle or skip, ensure text mode)
- Ensure thread-safe file naming for concurrent executions (use atomic operations or locks)
- Handle very long file paths (OS limits) - truncate or use shorter names
- Handle special characters in file paths (OS-specific, sanitize if needed)
- Handle case where temp directory is full
- Handle case where multiple processes generate same filename (very unlikely but possible)

### 4. Test Utilities (`tests/common/`)

#### 4.1 Test Project Helper (`tests/common/test_project.rs`)

- Function to create temporary Rust project with valid `Cargo.toml`
- Function to create Rust project with compilation errors (for error testing)
- Function to create Rust project with tests
- Cleanup utilities for test projects

#### 4.2 Test Assertion Helpers (`tests/common/mod.rs`)

- Helper to verify output file exists and is readable
- Helper to verify output file contents match expected cargo output
- Helper to verify file path format
- Helper to extract file path from stdout
- Helper to run tool42 command and capture output

## Implementation Details

### CLI Structure

```rust
// Pseudo-structure
#[derive(Parser)]
#[command(name = "tool42")]
#[command(about = "A CLI tool designed to help AI agents perform Rust software development tasks")]
#[command(long_about = "Tool42 is a CLI tool designed to help AI agents perform various tasks for Rust software development, such as editing source files, checking and/or compiling code, checking log files, and much more.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wrapper around Cargo that writes output to a temporary file
    /// 
    /// This command executes any cargo subcommand and writes the combined stdout/stderr
    /// output to a temporary file in the OS temp directory. The file path is printed
    /// to stdout after execution, allowing AI agents to read the output without
    /// dealing with large serialized CLI output directly.
    /// 
    /// The output file is located in the standard temporary directory of the operating
    /// system and has a unique filename to prevent collisions.
    /// 
    /// Examples:
    ///   tool42 cargo check
    ///   tool42 cargo build --release
    ///   tool42 cargo test --verbose
    ///   tool42 cargo --version
    /// 
    /// Output:
    ///   The command prints the output file path to stdout (e.g., /tmp/tool42_cargo_1234567890_abc.txt)
    ///   The exit code matches the cargo command's exit code (0 for success, non-zero for failure)
    Cargo {
        /// Cargo subcommand and arguments (all arguments are passed through to cargo)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Read { path: PathBuf, from: Option<usize>, to: Option<usize> },
    Describe { path: PathBuf },
}
```

### CLI Help Documentation for AI Agents

#### 1. Comprehensive Help Text Requirements

**Main Command Help (`tool42 --help`):**

- Clear description of what tool42 does
- List all available subcommands with brief descriptions
- Show usage examples
- Explain that tool42 is designed for AI agent use cases
- Include version information
- Show how to get help for specific subcommands

**Subcommand Help (`tool42 cargo --help`):**

- Detailed description of what the command does
- Explain the purpose: wrapping cargo output to files for AI agent consumption
- Document the output file location (OS temp directory)
- Document the output file naming convention
- Explain that stdout/stderr are combined
- Explain that the file path is printed to stdout
- Explain exit code behavior (matches cargo)
- Provide multiple usage examples
- Document all supported cargo subcommands (or note that all are supported)
- Explain error handling and what happens in failure cases

#### 2. Help Text Content Structure

**For `tool42 cargo`:**

```
tool42 cargo - Wrapper around Cargo that writes output to a temporary file

DESCRIPTION:
    Executes any cargo subcommand and writes the combined stdout/stderr output
    to a temporary file. This is designed for AI agents that need to process
    large CLI output without direct serialization issues.

OUTPUT FILE:
  - Location: OS standard temporary directory (e.g., /tmp on Linux, %TEMP% on Windows)
  - Filename format: tool42_cargo_<timestamp>_<random>.txt
  - Contents: Combined stdout and stderr from cargo command
  - Persistence: File persists after command completion for reading

BEHAVIOR:
  - All arguments are passed through to cargo unchanged
  - Exit code matches cargo's exit code (0 = success, non-zero = failure)
  - Output file path is printed to stdout after execution (always)
  - File is created even if cargo produces no output (empty file)

EXAMPLES:
    # Check code
    tool42 cargo check
    
    # Build release
    tool42 cargo build --release
    
    # Run tests with verbose output
    tool42 cargo test --verbose
    
    # Get cargo version
    tool42 cargo --version
    
    # Build with specific target
    tool42 cargo build --target x86_64-unknown-linux-gnu

ERROR HANDLING:
  - If cargo command fails, exit code is preserved and error output is in file
  - If temp directory is inaccessible, error is printed to stderr
  - If file cannot be written, error is printed to stderr
  - All errors include context for debugging

USAGE:
    tool42 cargo [CARGO_ARGS]...
```

#### 3. Additional Documentation Features

**Version Information:**

- `tool42 --version` should show version, commit hash (if available), and build date
- Include in help text that version info is available

**Examples in Help:**

- Include 5-10 practical examples showing common use cases
- Show examples with different cargo subcommands
- Show examples with flags and options
- Show examples of error scenarios

**Error Message Documentation:**

- Help text should explain what errors can occur
- Error messages should be self-documenting (include context)
- Error messages should suggest solutions when possible

**Output Format Documentation:**

- Explain that output file is plain text
- Explain stdout/stderr combination order (stdout first, then stderr)
- Explain that file path is the only thing printed to stdout (for parsing)
- Explain that errors are printed to stderr (not in output file)

#### 4. Implementation Requirements

**Using clap's Documentation Features:**

- Use `#[command(about = "...")]` and `#[command(long_about = "...")]` for descriptions
- Use `///` doc comments for subcommands (clap uses these for help)
- Use `#[arg(help = "...")]` for argument descriptions
- Use `#[command(example = "...")]` for examples (clap 4.x feature)
- Use `#[command(visible_alias = "...")]` for aliases if needed

**Help Text Testing:**

- Test that help text is comprehensive and readable
- Test that examples are valid and work
- Test that help text is properly formatted
- Verify all subcommands have adequate documentation

**AI Agent Considerations:**

- Help text should be parseable and structured
- Use consistent formatting that's easy to parse
- Include all necessary information for programmatic use
- Avoid ambiguous language
- Include machine-readable information where possible (e.g., file path format)

### Key Files to Create (Phase 1)

- `Cargo.toml` - Project configuration with dev dependencies for testing
- `src/lib.rs` - Library entry point exposing public API for testing
- `src/main.rs` - CLI entry point (thin wrapper calling lib)
- `src/commands/mod.rs` - Commands module
- `src/commands/cargo.rs` - Cargo wrapper implementation
- `tests/common/mod.rs` - Test utilities module
- `tests/common/test_project.rs` - Test project helper
- `tests/cargo_command.rs` - Integration tests

### Implementation Notes

- Use `Command::new("cargo")` and pass through all arguments
- Use `Command::output()` to capture stdout/stderr (or `Command::spawn()` + `wait_with_output()` for more control)
- Combine stdout and stderr: write stdout first, then stderr, or interleave with markers
- Use `std::fs::write()` to write output to temp file (or `BufWriter` for large outputs)
- Generate filename: `tool42_cargo_{timestamp}_{random}.txt` using `SystemTime` and `rand` or `uuid`
- Use `std::env::temp_dir()` and `PathBuf::join()` for path construction
- Print file path using `println!()` after file is written
- Use `std::process::exit()` to preserve cargo's exit code

### Testing Strategy

- Write tests before implementation (TDD red-green-refactor cycle)
- Run tests frequently during development (`cargo test`)
- Ensure all tests pass before considering feature complete
- Test on multiple platforms if possible (Windows, Linux, macOS)
- Use `#[ignore]` for tests that require specific environment setup
- Document test requirements and assumptions

## GitHub Actions CI/CD Setup

### 1. Build Workflow (`.github/workflows/build.yml`)

Create a GitHub Actions workflow that:

**Triggers:**

- On push to `main` branch
- On pull requests
- On tags (for releases)
- Manual workflow dispatch

**Jobs:**

#### 1.1 Build and Test (Matrix Strategy)

- Use matrix strategy to build on multiple platforms:
                - `ubuntu-latest` (Linux x86_64)
                - `windows-latest` (Windows x86_64)
                - `macos-latest` (macOS x86_64)
                - `macos-13` (macOS ARM64/Apple Silicon)
- For each platform:
                - Set up Rust toolchain (latest stable)
                - Cache cargo dependencies
                - Run `cargo test` to execute all tests
                - Run `cargo build --release` to create release binaries
                - Run `cargo clippy -- -D warnings` for linting (optional)
                - Run `cargo fmt --check` for code formatting (optional)

#### 1.2 Build Artifacts

- Upload build artifacts for each platform:
                - Linux: `target/release/tool42`
                - Windows: `target/release/tool42.exe`
                - macOS x86_64: `target/release/tool42`
                - macOS ARM64: `target/aarch64-apple-darwin/release/tool42` (if cross-compiling)
- Artifact naming: `tool42-<platform>-<arch>-<version>`
- Store artifacts for later download or release

#### 1.3 Release Workflow (Optional)

- On tag push (e.g., `v1.0.0`):
                - Build binaries for all platforms
                - Create GitHub Release
                - Upload binaries as release assets
                - Generate checksums (SHA256) for each binary
                - Upload checksums file

### 2. Workflow File Structure

Create `.github/workflows/build.yml` with:

**Key Components:**

- Matrix strategy for multiple OS/architecture combinations
- Rust setup action (`actions-rs/toolchain` or `dtolnay/rust-toolchain`)
- Cargo cache action for faster builds
- Build steps for each platform
- Artifact upload steps
- Conditional release steps (only on tags)

**Platform-Specific Considerations:**

- Windows: Handle `.exe` extension
- macOS: May need separate jobs for x86_64 and ARM64, or use cross-compilation
- Linux: Standard x86_64 build

### 3. Additional Workflows (Optional)

#### 3.1 Lint and Format Check (`.github/workflows/lint.yml`)

- Run on every PR
- Check code formatting with `cargo fmt --check`
- Run clippy lints
- Fail if code doesn't meet standards

#### 3.2 Security Audit (`.github/workflows/audit.yml`)

- Run `cargo audit` to check for known vulnerabilities
- Run on schedule (weekly) and on PRs

### 4. Files to Create

- `.github/workflows/build.yml` - Main build and test workflow
- `.github/workflows/lint.yml` - Linting workflow (optional)
- `.github/workflows/audit.yml` - Security audit workflow (optional)
- `.github/dependabot.yml` - Dependabot configuration for Rust dependencies (optional)

### 5. Build Configuration

**Cargo.toml additions:**

- Ensure `[package]` section has proper metadata
- Add `[profile.release]` optimizations if needed
- Consider adding `[target.*]` sections for platform-specific configs

**Cross-compilation (if needed):**

- For macOS ARM64 on GitHub Actions, may need to use `macos-13` runner or cross-compile
- Consider using `cross` tool for easier cross-compilation if needed
- Or use separate jobs for each architecture

### 6. Release Process

**Manual Release:**

1. Update version in `Cargo.toml`
2. Create git tag: `git tag v1.0.0`
3. Push tag: `git push origin v1.0.0`
4. GitHub Actions will automatically build and create release

**Automated Release:**

- Use `cargo-release` or similar tool to automate version bumping
- Or use GitHub Actions to create releases automatically on tag push

## Phase 2: `tool42 read` (Future)

- TDD approach with comprehensive tests
- Accept file path argument
- Optional `--from` and `--to` line number arguments
- Default 500 line limit
- Output file contents to stdout

## Phase 3: `tool42 describe` (Future)

- TDD approach with comprehensive tests
- Parse Rust source files
- Extract structs, functions, impl blocks, etc.
- Use `syn` crate for Rust parsing
- Output structured information with line numbers
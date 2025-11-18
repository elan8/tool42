# Mandrel Test Harness for tool42

This directory contains comprehensive test specifications for the tool42 MCP server using the [Mandrel MCP Test Harness](https://rustic-ai.github.io/codeprism/docs/test-harness/) (moth).

## Overview

The Mandrel test harness provides enterprise-grade testing capabilities for MCP servers, including:

- Protocol compliance validation (MCP 2024-11-05)
- Tool functionality testing
- Error handling validation
- Performance monitoring
- JSONPath-based response validation

## Prerequisites

1. **Mandrel Test Harness (moth)**: The test harness is cloned in `3rdparty/codeprism` and must be built:
   ```powershell
   cd 3rdparty\codeprism
   cargo build --release --bin moth
   ```

2. **tool42**: The tool42 binary must be available in your PATH or built:
   ```powershell
   cargo build --release
   ```

## Test Configuration

The main test specification is in `tool42-server.yaml`, which includes:

- **13 Tool Tests**: Comprehensive tests for all tool42 MCP tools
  - `tool42_cargo` - Cargo command execution
  - `tool42_read` - File reading with line limits
  - `tool42_describe` - Rust file structure extraction
  - `tool42_search` - Codebase search
  - `tool42_deps` - Dependency information
  - `tool42_tests` - Test discovery
  - `tool42_project` - Project structure
  - `tool42_list` - Directory listing
  - `tool42_docs` - Documentation extraction
  - `tool42_refactor_rename` - Symbol renaming
  - `tool42_refactor_extract` - Function extraction
  - `tool42_refactor_move` - Item moving
  - `tool42_refactor_signature` - Signature changes

- **Error Handling Tests**: Validation of proper MCP error codes and messages
- **Performance Tests**: Response time validation for various operations

## Running Tests

### Quick Start

Use the provided PowerShell script:

```powershell
cd tests\mandrel
.\run-tests.ps1
```

### Manual Execution

1. **Validate the test configuration**:
   ```powershell
   ..\..\3rdparty\codeprism\target\release\moth.exe validate tool42-server.yaml
   ```

2. **Run the test suite**:
   ```powershell
   ..\..\3rdparty\codeprism\target\release\moth.exe run tool42-server.yaml
   ```

### Options

The `run-tests.ps1` script supports:

- `-MothPath <path>`: Specify custom path to moth binary (default: `..\..\3rdparty\codeprism\target\release\moth.exe`)
- `-TestConfig <file>`: Specify test configuration file (default: `tool42-server.yaml`)
- `-BuildTool42`: Build tool42 before running tests
- `-ValidateOnly`: Only validate the configuration, don't run tests

Example:
```powershell
.\run-tests.ps1 -BuildTool42 -ValidateOnly
```

## Test Results

The test harness generates detailed reports including:

- Test pass/fail status
- Response time metrics
- Error details
- JSONPath validation results

Results are displayed in the console and can be exported to JSON, HTML, or JUnit XML formats using moth's reporting features.

## Adding New Tests

To add new tests, edit `tool42-server.yaml`:

1. **Add a new tool test**: Add an entry under the `tools:` section
2. **Add error tests**: Add entries under the `error_tests:` section
3. **Configure validation**: Use JSONPath expressions in the `expected.fields` section

Example test structure:
```yaml
- name: "tool42_read"
  description: "Read files with line limits"
  tests:
    - name: "read_example_file"
      description: "Read a specific file"
      input:
        path: "example.rs"
        working_directory: "."
      expected:
        error: false
        fields:
          - path: "$.content[0].text"
            field_type: "string"
            required: true
            contains: "expected_content"
      tags: ["read", "example"]
```

## Troubleshooting

### moth binary not found
- Ensure you've built moth: `cd 3rdparty\codeprism && cargo build --release --bin moth`
- Check the path in `run-tests.ps1` matches your setup

### tool42 not found
- Install tool42: `cargo install --path .`
- Or use `-BuildTool42` flag to build before testing

### Test failures
- Check that tool42 is working: `tool42 --help`
- Verify the working directory in test inputs is correct
- Review error messages in test output for specific issues

### Validation errors
- Check YAML syntax
- Verify all required fields are present
- Ensure JSONPath expressions are correct

## References

- [Mandrel Test Harness Documentation](https://rustic-ai.github.io/codeprism/docs/test-harness/)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [tool42 Documentation](../README.md)


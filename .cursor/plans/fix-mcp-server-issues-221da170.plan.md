<!-- 221da170-b164-4377-adcd-17e82be01400 49bb5800-521b-4f5d-a572-b22b5aebf9fd -->
# Fix MCP Server "No Tools" Issue in Cursor

## Critical Issue

**Problem**: When adding tool42 MCP server to Cursor, it shows "No tools, prompts or resources" in the description.

**Root Cause**: The MCP server's `capabilities` field uses `Default::default()` which doesn't explicitly enable the tools capability. MCP clients need explicit capability declarations to discover tools.

## Problems Identified

1. **CRITICAL: Missing Tool Capabilities** - `capabilities: Default::default()` in `get_info()` doesn't enable tools capability
2. **Schema Generation Issues** - Schema conversion might fail silently, resulting in invalid tool definitions  
3. **Test Coverage Gap**: Only 4/13 tools are tested
4. **Test Mismatch**: Test expects 9 tools but server has 13 tools
5. **Missing Error Tests**: No tests for error handling

## Implementation Plan

### Phase 1: Fix Critical Capabilities Issue (PRIORITY)

**File**: `src/mcp/server.rs`

- Fix `get_info()` method to explicitly enable tools capability
- Set `capabilities.tools` to indicate server supports tools
- Verify proper capability structure for rmcp library
- Test: Verify Cursor can now see all 13 tools after fix

**Code Change**:

```rust
capabilities: rmcp::model::ServerCapabilities {
    tools: Some(rmcp::model::ToolsCapability {
        list_changed: false,
    }),
    ..Default::default()
}
```

### Phase 2: Fix Schema Generation

**File**: `src/mcp/server.rs`

- Review schema conversion in `schema_to_map()` function
- Add error handling/logging for schema generation failures
- Ensure all schemas are valid JSON Schema format
- Fix `tool42_deps` to use proper schema instead of empty schema

### Phase 3: Fix Test Mismatch

**File**: `tests/mcp_integration_test.rs`

- Update `test_mcp_list_tools` to expect all 13 tools (including 4 refactor tools)
- Verify tool names match exactly between server and tests

### Phase 4: Add Missing Tool Tests

**File**: `tests/mcp_integration_test.rs`

Add integration tests for:

- `tool42_cargo` - Test cargo command execution
- `tool42_search` - Test codebase search  
- `tool42_tests` - Test test discovery
- `tool42_project` - Test project structure
- `tool42_list` - Test directory listing
- `tool42_docs` - Test documentation extraction
- All 4 refactor tools (rename, extract, move, signature)

### Phase 5: Add Error Handling Tests

**File**: `tests/mcp_integration_test.rs`

Create tests for:

- Invalid tool names
- Missing required parameters
- Invalid file paths
- Invalid parameter types
- Tool execution failures

## Files to Modify

1. **`src/mcp/server.rs`** - CRITICAL: Fix capabilities to enable tools
2. `tests/mcp_integration_test.rs` - Add comprehensive tests
3. `src/mcp/schemas.rs` - Review and fix schema definitions (if needed)

## Testing Strategy

- **Immediate**: Test that Cursor can see tools after capabilities fix
- Each tool should have at least one success test
- Each tool should have at least one error test
- Test tool discovery (list_tools)
- Test initialization handshake
- Test error response format

### To-dos

- [ ] Fix capabilities in get_info() to explicitly enable tools capability
- [ ] Test that Cursor can now see all 13 tools after capabilities fix
- [ ] Review and improve schema generation with proper error handling
- [ ] Update test_mcp_list_tools to expect all 13 tools including refactor tools
- [ ] Add integration tests for cargo, search, tests, project, list, docs, and all 4 refactor tools
- [ ] Add error handling tests for invalid parameters, missing files, and tool failures
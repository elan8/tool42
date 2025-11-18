<!-- 5928f349-f491-47cb-86ca-e26caf30492f e3c4610f-7fa5-4f56-9f02-b05c53bfe229 -->
# Improve Refactor Functionality

## Current State Analysis

### Strengths

- Basic refactor operations (rename, extract, move, signature) are implemented
- Preview mode and validation with `cargo check` are working
- Backup and rollback mechanisms are in place
- MCP integration is complete

### Limitations Identified

#### 1. Rename Operation (`rename.rs`)

- Uses text-based search with word boundaries instead of AST-aware symbol resolution
- May incorrectly match symbols that are part of larger identifiers
- Doesn't distinguish between different symbol types (struct vs function vs variable)
- No handling of qualified paths (e.g., `mod::Symbol`)

#### 2. Extract Operation (`extract.rs`)

- Doesn't analyze captured variables from surrounding scope
- Doesn't automatically determine function parameters
- Doesn't infer return types
- Simple text extraction without AST analysis
- Doesn't handle error propagation or control flow properly

#### 3. Move Operation (`move.rs`)

- Import updating implementation exists but may have edge cases
- Doesn't handle module declarations (`mod.rs` files, `mod` statements)
- May not preserve visibility modifiers correctly
- Code formatting may be lost when using `quote::quote!`

#### 4. Signature Operation (`signature.rs`)

- Call site updater (`CallSiteUpdater`) doesn't actually modify arguments
- No parameter mapping/reordering support
- Doesn't handle generic parameters properly
- No validation of signature compatibility before applying

#### 5. General Issues

- Line number detection uses approximate text search instead of span information
- Error messages could be more specific
- Preview output could show more detail (imports, call sites, etc.)

## Improvement Plan

### Phase 1: AST-Aware Symbol Resolution

**Files**: `src/core/refactor/rename.rs`, `src/core/refactor/utils.rs`

**Changes**:

1. Replace text-based search in `find_symbol_in_file` with AST traversal
2. Use `syn::visit::Visit` to find all symbol usages with proper context
3. Distinguish between definition, reference, call, and import usages
4. Handle qualified paths (e.g., `mod::Symbol`)
5. Preserve span information for accurate line numbers

**Benefits**: More accurate symbol finding, fewer false positives, better handling of edge cases

### Phase 2: Enhanced Extract Function

**Files**: `src/core/refactor/extract.rs`, `src/core/refactor/ast_utils.rs`

**Changes**:

1. Parse code block with AST to identify captured variables
2. Analyze surrounding scope to determine which variables are used
3. Infer function parameters from captured variables
4. Detect return statements and infer return type
5. Handle control flow (early returns, breaks, continues)
6. Preserve comments and formatting better

**New Functions**:

- `analyze_captured_variables(ast, block_span) -> Vec<Variable>`
- `infer_return_type(block_ast) -> Option<Type>`
- `build_function_signature(vars, return_type) -> Signature`

**Benefits**: More intelligent function extraction, fewer manual edits needed

### Phase 3: Complete Move Operation

**Files**: `src/core/refactor/move.rs`, `src/core/refactor/ast_utils.rs`

**Changes**:

1. Improve `update_imports_for_move` to handle all import patterns
2. Add support for module declarations (`mod` statements)
3. Preserve visibility modifiers when moving items
4. Handle re-exports properly
5. Use AST-based code insertion to preserve formatting
6. Handle associated items (impl blocks, trait implementations) more robustly

**New Functions**:

- `find_module_declarations(path) -> Vec<ModuleDecl>`
- `update_module_declarations(ast, old_path, new_path)`
- `preserve_formatting_when_inserting(ast, item)`

**Benefits**: More reliable move operations, better import handling

### Phase 4: Complete Signature Change

**Files**: `src/core/refactor/signature.rs`, `src/core/refactor/ast_utils.rs`

**Changes**:

1. Implement actual call site argument updates in `CallSiteUpdater`
2. Add parameter mapping support (by name and position)
3. Handle parameter additions/removals
4. Support generic parameters in signatures
5. Validate signature compatibility before applying
6. Provide detailed preview showing all call site changes

**New Functions**:

- `map_parameters(old_sig, new_sig) -> ParameterMapping`
- `update_call_arguments(call_site, mapping) -> ExprCall`
- `validate_signature_compatibility(old_sig, new_sig) -> Result<()>`

**Benefits**: Automatic call site updates, fewer manual fixes needed

### Phase 5: Enhanced Preview and Error Reporting

**Files**: `src/core/refactor/types.rs`, all refactor operation files

**Changes**:

1. Add more detailed change information to `Change` struct:

   - Change type (rename, import_update, call_site_update, etc.)
   - Affected files list
   - Import changes separately
   - Call site changes with context

2. Improve error messages with specific failure reasons
3. Add warnings for potential breaking changes
4. Show import dependency chains in preview

**New Types**:

```rust
enum ChangeType {
    SymbolRename,
    ImportUpdate { old_path: String, new_path: String },
    CallSiteUpdate { old_args: String, new_args: String },
    ItemMove { from: String, to: String },
}
```

**Benefits**: Better user understanding of changes, easier debugging

### Phase 6: Testing and Edge Cases

**Files**: `tests/refactor_command.rs`

**Changes**:

1. Add tests for AST-aware rename with qualified paths
2. Add tests for extract with variable capture analysis
3. Add tests for move with complex import scenarios
4. Add tests for signature change with parameter mapping
5. Add edge case tests (nested modules, private items, generics, etc.)

**Benefits**: More reliable refactoring, catches regressions

## Implementation Priority

1. **High Priority**: Phase 1 (AST-aware rename) - Most impactful, fixes accuracy issues
2. **High Priority**: Phase 4 (Complete signature change) - Currently incomplete
3. **Medium Priority**: Phase 2 (Enhanced extract) - Improves usability significantly
4. **Medium Priority**: Phase 3 (Complete move) - Fixes edge cases
5. **Low Priority**: Phase 5 (Enhanced preview) - Nice to have, improves UX
6. **Low Priority**: Phase 6 (Testing) - Should be done alongside each phase

## Success Criteria

- Rename operation uses AST traversal instead of text search
- Extract operation analyzes captured variables and infers parameters
- Move operation correctly handles all import patterns and module declarations
- Signature change operation updates all call sites automatically
- All operations provide detailed, accurate previews
- Error messages are specific and actionable
- Existing tests pass, new tests cover improved functionality

### To-dos

- [ ] Replace text-based search in rename operation with AST traversal using syn::visit::Visit
- [ ] Add support for qualified paths (e.g., mod::Symbol) in symbol resolution
- [ ] Implement captured variable analysis for extract operation to determine function parameters
- [ ] Add return type inference for extract operation based on return statements
- [ ] Improve import updating in move operation to handle all import patterns and edge cases
- [ ] Add support for module declarations (mod.rs files, mod statements) in move operation
- [ ] Implement actual call site argument updates in signature change operation
- [ ] Add parameter mapping support (by name and position) for signature changes
- [ ] Enhance preview output with detailed change types, import changes, and call site changes
- [ ] Improve error messages with specific failure reasons and actionable suggestions
<!-- 530bfd3b-7b03-4fb5-b11b-2b398f01d77d 820d377f-b19e-460f-aee6-9aa924636b00 -->
# Improve Refactor Functionality

## Current Limitations

### Move Operation (`move_item`)

- `update_imports_for_move` is a stub (does nothing)
- Doesn't handle module declarations (`mod.rs` files, `mod` statements)
- Doesn't preserve visibility modifiers (`pub`, `pub(crate)`, etc.)
- Doesn't handle associated items (impl blocks, trait implementations)
- Simple text insertion doesn't preserve code formatting
- Doesn't handle re-exports

### Signature Change (`change_signature`)

- Doesn't update call sites (relies entirely on `cargo check` to catch errors)
- No parameter mapping/reordering support
- Doesn't handle generic parameters properly
- No validation of signature compatibility before applying

## Implementation Plan

### Phase 1: AST Utilities and Symbol Resolution

**File: `src/core/refactor.rs`**

1. **Add helper functions for AST traversal:**

   - `find_all_usages(symbol: &str, path: &Path) -> Vec<Usage>` - Find all usages of a symbol using AST
   - `find_call_sites(function: &str, path: &Path) -> Vec<CallSite>` - Find all function call sites
   - `resolve_module_path(file: &Path) -> String` - Convert file path to module path
   - `find_import_statements(file: &Path, symbol: &str) -> Vec<Import>` - Find import statements for a symbol

2. **Add data structures:**
   ```rust
   struct Usage {
       file: PathBuf,
       line: usize,
       kind: UsageKind, // Definition, Call, Reference, Import
   }
   
   struct CallSite {
       file: PathBuf,
       line: usize,
       args: Vec<Expr>, // AST representation of arguments
   }
   
   struct Import {
       file: PathBuf,
       line: usize,
       path: String, // Current import path
   }
   ```


### Phase 2: Improve Move Operation

**File: `src/core/refactor.rs`**

1. **Implement `update_imports_for_move`:**

   - Parse all Rust files in the project
   - Find all `use` statements that import the moved symbol
   - Update import paths to point to new location
   - Handle both absolute and relative imports
   - Preserve import grouping and formatting

2. **Handle module structure:**

   - Detect if target is a new module (create `mod.rs` if needed)
   - Update `mod` declarations in parent modules
   - Handle `pub use` re-exports

3. **Handle associated items:**

   - Find all `impl` blocks for moved structs/enums
   - Move impl blocks along with the type
   - Handle trait implementations

4. **Improve code insertion:**

   - Use AST-based insertion instead of text manipulation
   - Preserve code formatting and comments
   - Insert items in appropriate location (after other items, maintain order)

5. **Handle visibility:**

   - Preserve original visibility modifiers
   - Update visibility if moving to different module scope (e.g., `pub` to `pub(crate)`)

### Phase 3: Improve Signature Change Operation

**File: `src/core/refactor.rs`**

1. **Implement call site updates:**

   - Find all call sites using AST traversal
   - Parse function call expressions
   - Map old parameters to new parameters
   - Update argument lists based on signature changes

2. **Parameter mapping:**

   - Support parameter reordering (by name matching)
   - Handle removed parameters (remove from call sites)
   - Handle added parameters (add default values or require user input)
   - Validate parameter types match

3. **Generic parameters:**

   - Preserve generic parameters in signature
   - Update call sites with appropriate type parameters
   - Handle where clauses

4. **Return type changes:**

   - Update call sites that use return values
   - Handle `Result` type changes
   - Update error handling if return type changes

5. **Validation:**

   - Check signature compatibility before applying
   - Detect breaking changes (parameter type mismatches, etc.)
   - Provide detailed error messages

### Phase 4: Enhanced Preview and Error Reporting

**File: `src/core/refactor.rs`**

1. **Improve preview output:**

   - Show all files that will be modified
   - Show import changes separately
   - Show call site changes with context
   - Indicate potential breaking changes

2. **Better error messages:**

   - Specific errors for each type of failure
   - Suggest fixes for common issues
   - Show which files/modules are affected

### Phase 5: Testing and Edge Cases

**File: `tests/refactor_command.rs`**

1. **Add comprehensive tests:**

   - Move operation with imports
   - Move operation with impl blocks
   - Signature change with multiple call sites
   - Signature change with parameter reordering
   - Edge cases (nested modules, private items, etc.)

## Implementation Details

### Key Functions to Implement

1. **`find_all_usages`** - Recursively traverse AST to find all symbol usages
2. **`update_imports_for_move`** - Full implementation with AST-based import updating
3. **`find_and_update_call_sites`** - Find and update all function call sites
4. **`map_parameters`** - Map old parameters to new parameters for signature changes
5. **`insert_item_ast`** - Insert item into file using AST (preserve formatting)

### Dependencies

No new dependencies needed - use existing `syn` and `quote` crates for AST manipulation.

## Success Criteria

1. Move operation correctly updates all imports across the codebase
2. Move operation handles impl blocks and associated items
3. Signature change updates all call sites automatically
4. Both operations provide detailed previews showing all changes
5. Better error messages help users understand and fix issues
6. All existing tests pass, new tests added for improved functionality

### To-dos

- [ ] Implement AST traversal utilities (find_all_usages, find_call_sites, resolve_module_path, find_import_statements) with proper data structures
- [ ] Implement update_imports_for_move to find and update all import statements using AST traversal
- [ ] Handle module structure (mod.rs files, mod declarations) when moving items
- [ ] Handle associated items (impl blocks, trait implementations) when moving types
- [ ] Improve code insertion using AST-based approach to preserve formatting
- [ ] Implement find_and_update_call_sites to find and update all function call sites using AST
- [ ] Implement parameter mapping logic for handling parameter reordering, additions, and removals
- [ ] Handle generic parameters and where clauses in signature changes
- [ ] Add validation for signature compatibility before applying changes
- [ ] Enhance preview output to show import changes, call site changes, and potential breaking changes
- [ ] Improve error messages with specific details and suggestions for fixes
- [ ] Add comprehensive tests for move with imports, signature change with call sites, and edge cases
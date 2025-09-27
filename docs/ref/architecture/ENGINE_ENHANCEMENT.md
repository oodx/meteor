# Meteor Engine Enhancement Plan

## Objectives
- Provide higher-level iteration and query helpers so CLI/REPL consumers no longer need to reach directly into `StorageData`.
- Expose safer cursor manipulation primitives to reduce manual `switch_*` calls and prevent inconsistent state.
- Package namespace/context slices and fully-qualified meteor views as first-class returns to prepare for future `Meteor` rearchitecture.
- Tighten parser integration by surfacing the validation behavior the engine relies on and by eliminating brittle fallback paths (`find` substring matching, unstructured path parsing).
- Support document/script virtualization described in `DOC_VIRTUALIZATION_MODEL.md` (export/import helpers, ordered sections, metadata management).
- Introduce an internal workspace layer so ordering metadata, caches, and scratch buffers do not leak into canonical storage.

## Proposed Additions

### 1. Iteration & Query Helpers (ENG-10 Complete)

**Status**: ✅ Implemented in ENG-10

- ✅ `MeteorEngine::iter_entries()` → returns `EntriesIterator<'_>` over `(String, String, String, String)` tuples representing `(context, namespace, key, value)`; used by CLI `parse` output and REPL `list/dump` commands. Leverages workspace `key_order` for deterministic iteration.
- ✅ `MeteorEngine::contexts_iter()` → returns `impl Iterator<Item = String>` wrapper to replace repeated `storage.contexts()` clones; contexts sorted alphabetically.
- ✅ `MeteorEngine::namespaces_iter(context)` → returns `impl Iterator<Item = String>` for namespace iteration within a context; namespaces sorted alphabetically.
- ⚪ `MeteorEngine::entries_in(context: &str, namespace: &str)` → typed view for a single namespace (deferred to ENG-11: NamespaceView).
- ⚪ `MeteorEngine::namespace_view(context, namespace)` → returns struct with `(context, namespace, tokens)` (deferred to ENG-11).

### 2. Cursor Guards & Scoped Helpers
- `CursorGuard` (RAII) or `MeteorEngine::with_cursor(context, namespace, |cursor| ...)` to temporarily switch cursor and automatically restore previous state. REPL scratch commands (`mem set/get/edit`) become single-call closures instead of manual save/restore.
- `MeteorEngine::cursor()` returning lightweight struct exposing current `Context`/`Namespace` plus mutators (`set_context`, `set_namespace`, `reset`). Make CLI status output/REPL prompts read from this object.

### 3. Meteor Construction Primitives
- `MeteorEngine::meteors()` → iterator yielding future `Meteor` structs grouped by `(context, namespace)`. Pre-compute the grouping inside engine using storage index; foundation for CLI `parse` output and for a future `Meteor` API that enforces shared context/namespace.
- `MeteorEngine::meteor_for(context, namespace)` → returns aggregated meteor (context, namespace, Vec<Token>) or `None`. CLI JSON output and REPL `list` can rely on this instead of manual loops.

### 4. Command/History Utilities
- `MeteorEngine::clear_context(context)` / `clear_namespace(context, namespace)` convenience commands that record into audit trail (current CLI/REPL issue direct `set/delete` calls that bypass history).
- `MeteorEngine::command_history_iter()` / `failed_commands_iter()` to support paginated or filtered CLI output (future `history` command).
- Macro helper `engine_exec!(engine, cmd => target)` for REPL built-ins to guarantee history recording and consistent error reporting.

### 5. Internal Workspace & Scratch Memory
- Add `EngineWorkspace` inside `MeteorEngine` that tracks:
  - per-namespace ordering tables (`Vec<KeyOrder>` keyed by `(context, namespace)`) so section/part iteration is deterministic without recomputing sort orders.
  - optional query caches (glob/prefix results, resolved namespace views) with invalidation hooks triggered on mutations.
  - scratch buffers for multi-step operations (concatenating sections into `full`, staging hashes, temporary import/export state).
- Expose limited public helpers (`engine.workspace_status()` or debug dumps) while keeping mutation APIs internal to engine modules/macros.
- Provide a dedicated scratch context facade for REPL (`engine.scratch_slot(name)`) that maps to workspace memory instead of polluting canonical contexts.

### 6. Parsing & Validation Tightening
- Move path parsing into dedicated module (`meteor::path`) with richer diagnostics; `parse_meteor_path` should return structured errors (invalid colon count, empty context, etc.) used by CLI/REPL to display hints.
- Replace `find()` substring fallback with pattern-aware search (glob/prefix). Expose a `QueryPattern` struct that both parsers and REPL commands can re-use.
- Ensure `TokenStreamParser::process`/`MeteorStreamParser::process` rely on the same quoting/semicolon logic exposed via engine helper (e.g., centralize the `smart_split` variant in a shared utility to avoid divergence).
- Add validation hook for engine setters (`set/store_token`) that optionally receives parser context (e.g., `EngineSetOptions` with `permit_append`, `enforce_namespace_depth` flags) so CLI and REPL can opt into strict behavior.

## Workspace Inspection & Debugging (ME-1 Complete)

**Status**: ✅ Implemented in ENG-01/ENG-02/ENG-03

The `EngineWorkspace` layer includes debug-only inspection capabilities for monitoring workspace state and performance:

### workspace_status() Method

Available in debug builds (`#[cfg(debug_assertions)]`), returns a `WorkspaceStatus` struct with:

```rust
pub struct WorkspaceStatus {
    pub namespace_count: usize,           // Total namespaces with workspace data
    pub scratch_slot_count: usize,        // Active scratch slots
    pub total_cached_queries: usize,      // Cached query result count
    pub total_ordered_keys: usize,        // Total keys tracked for ordering

    // Optional instrumentation (requires `workspace-instrumentation` feature)
    #[cfg(feature = "workspace-instrumentation")]
    pub total_cache_hits: u64,            // Aggregate cache hits across namespaces
    #[cfg(feature = "workspace-instrumentation")]
    pub total_cache_misses: u64,          // Aggregate cache misses
    #[cfg(feature = "workspace-instrumentation")]
    pub overall_cache_hit_ratio: f64,     // Global hit ratio (0.0-1.0)
}
```

### Usage Example

```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
engine.set("app:main:key1", "value1").unwrap();
engine.set("app:ui:button", "click").unwrap();

#[cfg(debug_assertions)]
{
    let status = engine.workspace_status();
    println!("Namespaces: {}", status.namespace_count);
    println!("Ordered keys: {}", status.total_ordered_keys);
    println!("Cached queries: {}", status.total_cached_queries);

    #[cfg(feature = "workspace-instrumentation")]
    {
        println!("Cache hits: {}", status.total_cache_hits);
        println!("Cache misses: {}", status.total_cache_misses);
        println!("Hit ratio: {:.2}%", status.overall_cache_hit_ratio * 100.0);
    }
}
```

### Guard Rails

1. **Debug-Only Access**: `workspace_status()` is only available in debug builds to prevent production overhead
2. **Read-Only**: Returns immutable snapshot; workspace cannot be mutated via status inspection
3. **Feature Flag**: Cache instrumentation requires `workspace-instrumentation` feature flag in `Cargo.toml`
4. **Atomic Invalidation**: Counters automatically reset on cache invalidation (set/delete/reset operations)
5. **No Public Workspace Access**: Direct workspace access (`workspace()`, `workspace_mut()`) remains `pub(crate)` to maintain encapsulation

### Cache Invalidation Semantics

All mutations trigger automatic cache invalidation:
- **Insert/Update** (`store_token`, `set`): Invalidates namespace cache + resets instrumentation counters
- **Delete Key**: Invalidates namespace cache for the affected namespace
- **Delete Namespace/Context**: Removes entire workspace entry
- **Clear Storage**: Clears all workspace data (namespaces + scratch slots)

### Performance Monitoring

Enable instrumentation for ME-2 iteration testing:

```bash
# Build with instrumentation
cargo build --features workspace-instrumentation

# Run tests with instrumentation
cargo test --features workspace-instrumentation
```

Instrumentation adds per-namespace `cache_hits` and `cache_misses` counters for monitoring query cache effectiveness during ME-2 iterator implementation.

## Iterator Implementation (ENG-10 Complete)

**Status**: ✅ Implemented and tested

### API Surface

#### contexts_iter()
```rust
pub fn contexts_iter(&self) -> impl Iterator<Item = String>
```
Returns an iterator over all context names in sorted order. Replaces manual `storage.contexts()` clones.

**Example:**
```rust
for context in engine.contexts_iter() {
    println!("Context: {}", context);
}
```

#### namespaces_iter(context)
```rust
pub fn namespaces_iter(&self, context: &str) -> impl Iterator<Item = String>
```
Returns an iterator over namespace names within a context, sorted alphabetically.

**Example:**
```rust
for namespace in engine.namespaces_iter("app") {
    println!("Namespace: {}", namespace);
}
```

#### iter_entries()
```rust
pub fn iter_entries(&self) -> EntriesIterator<'_>
```
Returns an iterator over all entries with workspace ordering. Yields `(String, String, String, String)` tuples representing `(context, namespace, key, value)`.

**Key Features:**
- **Workspace Ordering**: Keys within each namespace are returned in workspace insertion order (from `key_order` Vec)
- **Hybrid Storage**: Falls back to storage keys if workspace data unavailable
- **Lifetime Safety**: Iterator borrows engine immutably with explicit `'_` lifetime
- **Lazy Evaluation**: Contexts and namespaces loaded progressively as iteration advances

**Example:**
```rust
for (context, namespace, key, value) in engine.iter_entries() {
    println!("{}:{}:{} = {}", context, namespace, key, value);
}
```

### Implementation Details

#### EntriesIterator Structure
```rust
pub struct EntriesIterator<'a> {
    engine: &'a MeteorEngine,
    contexts: Vec<String>,
    current_context_idx: usize,
    current_namespaces: Vec<String>,
    current_namespace_idx: usize,
    current_keys: Vec<String>,
    current_key_idx: usize,
}
```

**Iteration Algorithm:**
1. Load all contexts sorted (once at creation)
2. For each context, load namespaces sorted
3. For each namespace:
   - Try to get `key_order` from workspace (insertion order)
   - Fall back to `storage.find_keys()` if workspace unavailable
4. Yield `(context, namespace, key, value)` for each key
5. Advance to next namespace/context when current keys exhausted

#### Workspace Integration

The iterator leverages workspace ordering via:
```rust
if let Some(ws) = self.engine.workspace.get_namespace(context, namespace) {
    self.current_keys = ws.key_order.clone();
} else {
    self.current_keys = self.engine.storage.find_keys(context, namespace, "*");
}
```

This ensures:
- **Deterministic ordering** when workspace data exists
- **Graceful fallback** for namespaces without workspace (e.g., created via storage_data directly)
- **Insertion order preservation** reflecting user's data entry sequence

### Instrumentation (Optional)

When compiled with `workspace-instrumentation` feature flag, workspace tracks iteration metrics using `Cell<u64>` for interior mutability (allows updating through immutable references):

```rust
pub(crate) iteration_count: Cell<u64>,     // Total iterations over namespace
pub(crate) keys_iterated: Cell<u64>,       // Total keys returned from namespace
pub(crate) fn record_iteration(&self, key_count: usize)  // Note: &self (immutable)
pub(crate) fn avg_keys_per_iteration(&self) -> f64
```

Added to `WorkspaceStatus`:
```rust
#[cfg(feature = "workspace-instrumentation")]
pub total_iterations: u64,                 // Aggregate iteration count
#[cfg(feature = "workspace-instrumentation")]
pub total_keys_iterated: u64,              // Aggregate keys iterated
#[cfg(feature = "workspace-instrumentation")]
pub avg_keys_per_iteration: f64,           // Global average
```

**Key Design Decisions:**
- **Interior Mutability**: Uses `Cell<u64>` so `EntriesIterator` (which holds `&MeteorEngine`) can update counters
- **Lifetime Statistics**: Iteration metrics persist across cache invalidations (track cumulative usage)
- **Automatic Recording**: `EntriesIterator` calls `record_iteration()` when using workspace `key_order`
- **No Overhead without Feature**: All instrumentation code removed when feature flag disabled

**Usage:**
```bash
# Build with instrumentation
cargo build --features workspace-instrumentation

# Run tests with instrumentation
cargo test --features workspace-instrumentation

# Check iteration metrics in debug builds
#[cfg(debug_assertions)]
{
    let status = engine.workspace_status();
    println!("Total iterations: {}", status.total_iterations);
    println!("Avg keys/iteration: {:.2}", status.avg_keys_per_iteration);
}
```

### Test Coverage

**Test Files**:
- `tests/test_engine_iterators.rs` (17 tests, 249 LOC) - Core iterator functionality
- `tests/test_iteration_instrumentation.rs` (6 tests, 137 LOC) - Instrumentation validation (requires `workspace-instrumentation` feature)

- `test_contexts_iter_empty/single/multiple` - Context iteration edge cases
- `test_contexts_iter_is_sorted` - Alphabetical ordering verification
- `test_namespaces_iter_empty/single/multiple` - Namespace iteration edge cases
- `test_iter_entries_empty/single` - Entry iteration base cases
- `test_iter_entries_multiple_same_namespace` - Single namespace with multiple keys
- `test_iter_entries_multiple_namespaces` - Multiple namespaces in single context
- `test_iter_entries_multiple_contexts` - Cross-context iteration
- `test_iter_entries_workspace_ordering` - Insertion order preservation (zebra→apple→banana→aardvark, NOT alphabetical)
- `test_iter_entries_complex_data` - Complex multi-context/multi-namespace scenarios
- `test_iter_entries_values_correct` - Value integrity validation
- `test_iter_entries_after_delete` - Iterator reflects deletions
- `test_iter_entries_preserves_workspace_order_after_updates` - Update doesn't reorder keys

**All 17 tests passing** in default profile. Compatible with all 4 configuration profiles (default, enterprise, embedded, strict).

### Performance Characteristics

- **Time Complexity**:
  - `contexts_iter()`: O(C log C) where C = contexts (sort once)
  - `namespaces_iter()`: O(N log N) where N = namespaces in context (sort once)
  - `iter_entries()`: O(C log C + Σ(N_i log N_i) + K) where K = total keys (sorts + iteration)

- **Space Complexity**: O(C + N + K) - stores context list, namespace list per context, key list per namespace

- **Workspace Advantage**: Eliminates per-namespace key sorting when `key_order` exists (O(K) vs O(K log K))

### Lifetimes and Borrowing

**Explicit Lifetime Annotation** (`'_`):
```rust
pub fn iter_entries(&self) -> EntriesIterator<'_>
```

The `'_` makes the lifetime relationship explicit, avoiding warnings about elided lifetimes. The iterator borrows the engine immutably for its entire lifetime:

```rust
let mut engine = MeteorEngine::new();
engine.set("app:main:key", "value").unwrap();

// Iterator borrows engine immutably
let iter = engine.iter_entries();

// Cannot mutate engine while iterator exists
// engine.set("app:main:key2", "value2").unwrap();  // ERROR: cannot borrow mutably

// Consuming iterator releases borrow
for entry in iter {
    // Process entry
}

// Now can mutate again
engine.set("app:main:key2", "value2").unwrap();  // OK
```

## CLI/REPL Integration
- CLI `parse_command`: swap manual storage walk with `engine.meteors()`; JSON/text outputs reuse shared formatting functions built on the new view structs.
- CLI `validate_command`: add `--explain` flag that leverages richer parser errors; cross-link to new path diagnostics.
- REPL: new commands `history` (show command history), `cursor` (inspect/modify cursor using guard), `meteor <context> <namespace>` (dump aggregated meteor). Update existing `list`, `contexts`, `namespaces`, `mem` helpers to use engine iterators/guards.
- Shared formatting module (`src/bin/common/format.rs`) to render engine output in text/json/debug modes, backed by the new API.

## Follow-up Considerations
- Once `meteor_for`/`meteors` exist, refactor `Meteor` to include explicit `(Context, Namespace)` fields and enforce invariants during construction.
- Provide serialization utilities (`Meteor::to_text`, `Meteor::to_json`) reused by CLI/REPL/SDK consumers.
- If higher-level orchestration (live sync daemons, collaborative editing, remote APIs) proves valuable, consider building a thin wrapper crate that composes `MeteorEngine` rather than bloating the core library. The engine remains focused on Meteor/TokenStream semantics; the wrapper can own watchers, schedulers, or network services.

## Test & Documentation Tasks
- Add integration tests covering new iterators/guards via CLI smoke tests (`tests/cli.rs`); ensure REPL commands behave with cursor guard.
- Update `docs/ref/architecture/METEORSHOWER_ENGINE.md` and `docs/CONFIGURATION.md` to describe new helpers.
- Provide code samples in `docs/ref/guides/TOKEN_NAMESPACE_CONCEPT.md` illustrating meteor aggregation APIs.

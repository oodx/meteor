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
- ✅ `MeteorEngine::namespace_view(context, namespace)` → returns `NamespaceView<'_>` with metadata (entry_count, has_default) and ordered iterators (entries, keys, values) plus access methods (get, has_key, find_keys). Implemented in ENG-11.

### 2. Cursor Guards & Scoped Helpers (ENG-12 Complete)

**Status**: ✅ Implemented in ENG-12

- ✅ `MeteorEngine::cursor()` - returns `Cursor<'_>` accessor with context/namespace read/write methods (`context()`, `namespace()`, `set_context()`, `set_namespace()`, `reset()`, `position()`). Enables ergonomic cursor inspection and modification.
- ✅ `MeteorEngine::cursor_guard()` - returns `CursorGuard` (RAII) that saves cursor state and automatically restores on drop, even on panic. Eliminates manual save/restore boilerplate in REPL scratch commands and parsers.
- ✅ Added `From<&str>` and `From<String>` traits for `Context` and `Namespace` - enables `cursor.set_context("user")` without explicit type construction.

### 3. Meteor Construction Primitives (ENG-20 Complete)

**Status**: ✅ Implemented in ENG-20

- ✅ `MeteorEngine::meteors()` - returns `MeteorsIterator<'_>` that yields `Meteor` instances grouped by (context, namespace). Each meteor contains all tokens for that namespace in workspace insertion order. Foundation for CLI `parse` output and future meteor aggregation features.
- ✅ `MeteorEngine::meteor_for(context, namespace)` - returns `Option<Meteor>` for a specific (context, namespace) pair. CLI JSON output and REPL `list` commands can use this instead of manual storage loops.

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

## NamespaceView (ENG-11 Complete)

**Status**: ✅ Implemented and tested

### Overview

`NamespaceView` provides a lightweight, read-only view into a single namespace with ordered access to entries and metadata. Designed to support document/script virtualization (DOC_VIRTUALIZATION_MODEL.md) and namespace inspection without copying all data.

### API Surface

#### namespace_view(context, namespace)
```rust
pub fn namespace_view(&self, context: &str, namespace: &str) -> Option<NamespaceView<'_>>
```

Returns a view into a specific namespace, or `None` if the namespace doesn't exist.

**Example:**
```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
engine.set("doc:guides.install:intro", "Welcome").unwrap();
engine.set("doc:guides.install:setup", "Step 1...").unwrap();

if let Some(view) = engine.namespace_view("doc", "guides.install") {
    println!("Namespace has {} entries", view.entry_count);
    for (key, value) in view.entries() {
        println!("{} = {}", key, value);
    }
}
```

### NamespaceView Structure

```rust
pub struct NamespaceView<'a> {
    pub context: String,           // Context this namespace belongs to
    pub namespace: String,          // Namespace path
    pub entry_count: usize,         // Number of entries
    pub has_default: bool,          // Whether .index key exists

    // Private fields
    engine: &'a MeteorEngine,
    keys: Vec<String>,
}
```

**Public Fields:**
- `context` - The context name (e.g., "doc", "kb", "shell")
- `namespace` - Full namespace path (e.g., "guides.install", "lib.utils.hashing")
- `entry_count` - Total number of keys in namespace
- `has_default` - `true` if `.index` key exists (default value detection for directory-style namespaces)

### Iterator Methods

#### entries()
```rust
pub fn entries(&self) -> impl Iterator<Item = (String, String)> + '_
```
Returns iterator over `(key, value)` pairs in workspace insertion order.

**Example:**
```rust
for (key, value) in view.entries() {
    println!("{} = {}", key, value);
}
```

#### keys()
```rust
pub fn keys(&self) -> impl Iterator<Item = &str>
```
Returns iterator over keys in workspace insertion order.

**Example:**
```rust
let keys: Vec<&str> = view.keys().collect();
println!("Keys: {:?}", keys);
```

#### values()
```rust
pub fn values(&self) -> impl Iterator<Item = String> + '_
```
Returns iterator over values in workspace insertion order.

**Example:**
```rust
for value in view.values() {
    process_value(value);
}
```

### Access Methods

#### get(key)
```rust
pub fn get(&self, key: &str) -> Option<String>
```
Get a single value by key without iterating.

**Example:**
```rust
if let Some(intro) = view.get("intro") {
    println!("Intro: {}", intro);
}
```

#### has_key(key)
```rust
pub fn has_key(&self, key: &str) -> bool
```
Check if a key exists in this namespace.

**Example:**
```rust
if view.has_key(".index") {
    println!("Has default value");
}
```

#### find_keys(pattern)
```rust
pub fn find_keys(&self, pattern: &str) -> Vec<String>
```
Get all keys matching a pattern (supports `*` wildcard).

**Example:**
```rust
let section_keys = view.find_keys("sections*");
for key in section_keys {
    println!("Section: {}", key);
}
```

### Implementation Details

#### Workspace Ordering Integration

Like `EntriesIterator`, `NamespaceView` uses workspace `key_order` for deterministic iteration:

```rust
let keys = if let Some(ws) = self.workspace.get_namespace(context, namespace) {
    ws.key_order.clone()  // Insertion order
} else {
    let keys = self.storage.find_keys(context, namespace, "*");
    if keys.is_empty() {
        return None;  // Namespace doesn't exist
    }
    keys  // Sorted order (fallback)
};
```

**Benefits:**
- **Insertion Order**: Keys returned in the order they were added (e.g., zebra→apple→banana, NOT alphabetical)
- **Hybrid Storage**: Falls back to storage keys if workspace unavailable
- **Efficient**: Keys cloned once at view creation, reused for all iterations

#### Default Value Detection

The `.index` key convention (from DOC_VIRTUALIZATION_MODEL.md) indicates a namespace has a default value:

```rust
let has_default = keys.iter().any(|k| k == ".index");
```

Used for directory-style namespaces where `doc:guides.install:.index` provides default content when accessing `doc:guides.install` without a specific key.

#### Lazy Value Access

Unlike copying all values upfront, `NamespaceView` stores only keys and accesses values on-demand:

```rust
pub fn entries(&self) -> impl Iterator<Item = (String, String)> + '_ {
    self.keys.iter().filter_map(move |key| {
        self.engine
            .get(&format!("{}:{}:{}", self.context, self.namespace, key))
            .map(|v| (key.clone(), v.to_string()))
    })
}
```

**Advantages:**
- **Memory Efficient**: Only keys stored in view struct
- **Always Current**: Values fetched from engine (reflects concurrent updates)
- **Flexible**: Can iterate multiple times without copying

### Test Coverage

**Test File**: `tests/test_namespace_view.rs` (18 tests, 372 LOC)

**Core Functionality:**
- `test_namespace_view_basic_metadata` - Context, namespace, entry_count validation
- `test_namespace_view_with_default_value` - `.index` detection
- `test_namespace_view_nonexistent_namespace` - Returns `None` for missing namespaces
- `test_namespace_view_entries_iterator` - Entry iteration correctness
- `test_namespace_view_workspace_ordering` - Insertion order preservation (zebra→apple→banana)
- `test_namespace_view_keys_iterator` - Key iteration
- `test_namespace_view_values_iterator` - Value iteration

**Access Methods:**
- `test_namespace_view_get_method` - Single key access
- `test_namespace_view_has_key_method` - Key existence check
- `test_namespace_view_find_keys_wildcard` - Pattern matching (`section*`)

**Edge Cases:**
- `test_namespace_view_empty_namespace` - Empty namespace handling
- `test_namespace_view_multiple_contexts` - Context isolation
- `test_namespace_view_after_deletion` - Reflects deletions
- `test_namespace_view_ordering_preserved_across_updates` - Update doesn't reorder

**Integration:**
- `test_namespace_view_bracket_notation_keys` - Bracket keys (`sections[intro]`)
- `test_namespace_view_complex_namespace_paths` - Deep namespaces (`guides.install.linux.ubuntu`)
- `test_namespace_view_default_detection_with_multiple_keys` - `.index` with other keys
- `test_namespace_view_entries_values_consistency` - Iterator consistency

**All 18 tests passing** across all configuration profiles.

### Performance Characteristics

- **View Creation**: O(K) where K = keys in namespace (clone key vector)
- **Metadata Access**: O(1) for `entry_count`, `has_default`, `context`, `namespace`
- **Single Key Access**: O(1) via `get(key)` - direct engine lookup
- **Iteration**: O(K) for keys/values/entries - linear scan
- **Pattern Search**: O(K × P) where P = pattern complexity - delegates to storage `find_keys()`

**Memory**: O(K) - stores only key names, values fetched on-demand

### Lifetimes and Borrowing

`NamespaceView` borrows `MeteorEngine` immutably:

```rust
pub struct NamespaceView<'a> {
    engine: &'a MeteorEngine,
    // ...
}
```

Same borrowing rules as `EntriesIterator`:
- Cannot mutate engine while view exists
- View lifetime tied to engine lifetime
- Multiple views can coexist (all immutable borrows)

**Example:**
```rust
let view1 = engine.namespace_view("doc", "guides.install").unwrap();
let view2 = engine.namespace_view("doc", "guides.quickstart").unwrap();  // OK - both immutable

// Cannot mutate while views exist
// engine.set("doc:guides.install:new", "value").unwrap();  // ERROR

// Drop views to mutate
drop(view1);
drop(view2);
engine.set("doc:guides.install:new", "value").unwrap();  // OK
```

### Use Cases

**Document Export (DOC_VIRTUALIZATION_MODEL.md):**
```rust
if let Some(view) = engine.namespace_view("doc", "guides.install") {
    let full = view.get("full").unwrap_or_default();
    let sections: Vec<_> = view.find_keys("sections*")
        .iter()
        .filter_map(|k| view.get(k).map(|v| (k, v)))
        .collect();

    export_document(&view.namespace, &full, sections);
}
```

**Namespace Inspection:**
```rust
for context in engine.contexts_iter() {
    for namespace in engine.namespaces_iter(&context) {
        if let Some(view) = engine.namespace_view(&context, &namespace) {
            println!("{}: {} entries (default: {})",
                view.namespace, view.entry_count, view.has_default);
        }
    }
}
```

**CLI `list` Command Enhancement:**
```rust
if let Some(view) = engine.namespace_view(&context, &namespace) {
    println!("Namespace: {}:{}", view.context, view.namespace);
    println!("Entries: {}", view.entry_count);
    if view.has_default {
        println!("Default: {}", view.get(".index").unwrap());
    }
    for (key, value) in view.entries() {
        println!("  {} = {}", key, value);
    }
}
```

## Cursor Guards & Accessors (ENG-12 Complete)

**Status**: ✅ Implemented and tested

### Overview

Cursor guards and accessors provide safe, ergonomic cursor state management for temporary context/namespace switches. The RAII guard pattern ensures cursor restoration even on panic or early return, eliminating manual save/restore boilerplate.

### API Surface

#### cursor()
```rust
pub fn cursor(&mut self) -> Cursor<'_>
```

Returns a lightweight cursor accessor for reading and modifying cursor state with convenient methods.

**Example:**
```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
{
    let mut cursor = engine.cursor();
    assert_eq!(cursor.context().name(), "app");
    assert_eq!(cursor.namespace().to_string(), "main");
    assert_eq!(cursor.position(), "app:main");

    cursor.set_context("user");
    cursor.set_namespace("settings");
}
assert_eq!(engine.current_context.name(), "user");
```

#### cursor_guard()
```rust
pub fn cursor_guard(&mut self) -> CursorGuard
```

Creates an RAII guard that saves current cursor position and automatically restores it when dropped.

**Example:**
```rust
use meteor::types::{MeteorEngine, Context};

let mut engine = MeteorEngine::new();
{
    let _guard = engine.cursor_guard();
    engine.switch_context(Context::user());
    engine.switch_namespace("temp".into());
    engine.store_token("temp_key", "temp_value");
    // Cursor is user:temp here
} // Guard drops, cursor restored to app:main

assert_eq!(engine.current_context.name(), "app");
assert_eq!(engine.current_namespace.to_string(), "main");
```

### Cursor Structure

```rust
pub struct Cursor<'a> {
    engine: &'a mut MeteorEngine,
}

impl<'a> Cursor<'a> {
    pub fn context(&self) -> &Context
    pub fn namespace(&self) -> &Namespace
    pub fn set_context(&mut self, context: impl Into<Context>)
    pub fn set_namespace(&mut self, namespace: impl Into<Namespace>)
    pub fn reset(&mut self)
    pub fn position(&self) -> String  // Returns "context:namespace"
}
```

**Features:**
- **Mutable Borrow**: Borrows engine mutably, preventing data mutations while cursor is accessed
- **Flexible Input**: `set_context()` and `set_namespace()` accept `&str`, `String`, `Context`, or `Namespace`
- **Position Helper**: `position()` returns formatted "context:namespace" string for display
- **Reset Convenience**: `reset()` restores cursor to defaults (app:main)

### CursorGuard Structure

```rust
pub struct CursorGuard {
    saved_context: Context,
    saved_namespace: Namespace,
    engine_ptr: *mut MeteorEngine,
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        // Restores cursor state, even on panic
    }
}
```

**RAII Guarantees:**
- **Automatic Restoration**: Cursor restored when guard goes out of scope
- **Panic Safety**: Works even if panic occurs during guarded scope
- **Early Return Safety**: Works with `return`, `break`, `continue`, `?` operator
- **Nested Guards**: Multiple guards can be nested, each restoring its saved state

### Implementation Details

#### Flexible Type Conversions

The cursor API uses `Into<Context>` and `Into<Namespace>` bounds for ergonomic usage:

```rust
// All of these work:
cursor.set_context("user");                    // &str
cursor.set_context("system".to_string());      // String
cursor.set_context(Context::user());           // Context

cursor.set_namespace("settings");              // &str
cursor.set_namespace("config.db".to_string()); // String
cursor.set_namespace(Namespace::from_string("temp")); // Namespace
```

Implemented via `From` traits on `Context` and `Namespace`:
```rust
impl From<&str> for Context {
    fn from(s: &str) -> Self {
        Context::new(s)
    }
}

impl From<&str> for Namespace {
    fn from(s: &str) -> Self {
        Namespace::from_string(s)
    }
}
```

#### Guard Safety Pattern

`CursorGuard` uses raw pointer (`*mut MeteorEngine`) to enable restoration in `Drop`:

```rust
impl CursorGuard {
    fn new(engine: &mut MeteorEngine) -> Self {
        Self {
            saved_context: engine.current_context.clone(),
            saved_namespace: engine.current_namespace.clone(),
            engine_ptr: engine as *mut MeteorEngine,
        }
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        unsafe {
            let engine = &mut *self.engine_ptr;
            engine.current_context = self.saved_context.clone();
            engine.current_namespace = self.saved_namespace.clone();
        }
    }
}
```

**Safety Invariants:**
- Guard lifetime tied to engine borrow (Rust's lifetime system enforces this)
- Engine cannot be moved or destroyed while guard exists
- Pointer remains valid for guard's entire lifetime
- Drop always runs (except on `std::mem::forget`, which is explicitly unsafe)

### Test Coverage

**Test File**: `tests/test_cursor_guard.rs` (18 tests, 307 LOC)

**Cursor Accessor Tests:**
- `test_cursor_accessor_basic` - Read context/namespace/position
- `test_cursor_accessor_set_context` - Set context via Context type
- `test_cursor_accessor_set_context_str` - Set context via &str
- `test_cursor_accessor_set_namespace` - Set namespace via Namespace type
- `test_cursor_accessor_set_namespace_str` - Set namespace via &str
- `test_cursor_accessor_reset` - Reset to defaults
- `test_cursor_accessor_position` - Position string formatting

**Guard RAII Tests:**
- `test_cursor_guard_basic_restoration` - Basic drop restoration
- `test_cursor_guard_nested_guards` - Multiple nested guards
- `test_cursor_guard_with_data_operations` - Guard with data mutations
- `test_cursor_guard_restoration_on_early_return` - Early return via Result<>
- `test_cursor_guard_restoration_on_panic` - Panic safety with `catch_unwind`
- `test_cursor_guard_multiple_sequential` - Sequential guards
- `test_cursor_guard_preserves_modified_state` - Restores to saved state, not defaults

**Integration Tests:**
- `test_cursor_guard_with_control_commands` - Guard with control command execution
- `test_cursor_guard_complex_namespace_paths` - Deep namespace paths
- `test_cursor_accessor_and_guard_together` - Combined cursor + guard usage
- `test_cursor_guard_drop_order_verification` - Drop order validation with nested guards

**All 18 tests passing** across all configuration profiles.

### Performance Characteristics

**Cursor Accessor:**
- **Creation**: O(1) - zero-cost abstraction over mutable reference
- **Operations**: O(1) - direct field access/modification
- **Memory**: 0 bytes overhead (just a reference)

**CursorGuard:**
- **Creation**: O(1) - clones Context and Namespace (small structs)
- **Drop**: O(1) - restores two fields
- **Memory**: 2 × sizeof(Context + Namespace) + sizeof(*mut) ≈ 48-64 bytes

### Use Cases

**REPL Scratch Commands:**
```rust
// REPL 'mem set' command implementation
fn repl_mem_set(engine: &mut MeteorEngine, key: &str, value: &str) -> Result<(), String> {
    let _guard = engine.cursor_guard();
    engine.switch_context(Context::system());
    engine.switch_namespace("scratch".into());
    engine.store_token(key, value);
    Ok(())
} // Cursor automatically restored to user's original position
```

**Parser Temporary Context:**
```rust
// Import operation that temporarily switches context
fn import_namespace(engine: &mut MeteorEngine, data: &ImportData) -> Result<(), String> {
    let _guard = engine.cursor_guard();
    engine.switch_context(data.target_context.clone());

    for (ns, entries) in &data.namespaces {
        engine.switch_namespace(ns.clone());
        for (key, value) in entries {
            engine.store_token(key, value);
        }
    }

    Ok(())
} // Original cursor position restored
```

**CLI Status Display:**
```rust
// Display current cursor position without modifying state
fn cli_status_command(engine: &mut MeteorEngine) {
    let cursor = engine.cursor();
    println!("Current position: {}", cursor.position());
    println!("Context: {}", cursor.context().name());
    println!("Namespace: {}", cursor.namespace().to_string());
}
```

**Panic-Safe Operations:**
```rust
fn risky_operation(engine: &mut MeteorEngine) -> Result<(), String> {
    let _guard = engine.cursor_guard();
    engine.switch_context("temp".into());

    // Even if this panics, cursor is restored
    some_operation_that_might_panic()?;

    Ok(())
}
```

## Meteor Aggregation (ENG-20 Complete)

**Status**: ✅ Implemented and tested

### Overview

Meteor aggregation provides high-level iteration over storage data grouped by (context, namespace) pairs. Instead of iterating through individual key-value entries, `meteors()` and `meteor_for()` return complete `Meteor` instances containing all tokens for each namespace.

### API Surface

#### meteors()
```rust
pub fn meteors(&self) -> MeteorsIterator<'_>
```

Returns an iterator over all meteors, one per namespace. Each meteor contains all tokens (key-value pairs) for that namespace in workspace insertion order.

**Example:**
```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
engine.set("app:ui:button", "click").unwrap();
engine.set("app:ui:theme", "dark").unwrap();
engine.set("user:settings:lang", "en").unwrap();

for meteor in engine.meteors() {
    println!("{}:{} has {} tokens",
        meteor.context().name(),
        meteor.namespace().to_string(),
        meteor.tokens().len());
}
```

#### meteor_for(context, namespace)
```rust
pub fn meteor_for(&self, context: &str, namespace: &str) -> Option<Meteor>
```

Returns a meteor for a specific (context, namespace) pair, or `None` if the namespace doesn't exist or is empty.

**Example:**
```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
engine.set("app:ui:button", "click").unwrap();
engine.set("app:ui:theme", "dark").unwrap();

if let Some(meteor) = engine.meteor_for("app", "ui") {
    for token in meteor.tokens() {
        println!("{} = {}", token.key().base(), token.value());
    }
}
```

### MeteorsIterator Structure

```rust
pub struct MeteorsIterator<'a> {
    engine: &'a MeteorEngine,
    contexts: Vec<String>,
    current_context_idx: usize,
    current_namespaces: Vec<String>,
    current_namespace_idx: usize,
}

impl<'a> Iterator for MeteorsIterator<'a> {
    type Item = Meteor;
    // ...
}
```

**Iteration Algorithm:**
1. Load all contexts sorted (once at creation)
2. For each context, load namespaces sorted
3. For each (context, namespace) pair:
   - Use `namespace_view()` to get ordered entries
   - Convert entries to `Token` instances
   - Create `Meteor` with (Context, Namespace, Vec<Token>)
   - Yield the meteor
4. Skip empty namespaces (return `None` from `meteor_for()`)

### Implementation Details

#### Integration with ME-2 Infrastructure

Meteor aggregation builds directly on ME-2's iterator and view infrastructure:

**Uses `namespace_view()` for ordered access:**
```rust
pub fn meteor_for(&self, context: &str, namespace: &str) -> Option<Meteor> {
    let view = self.namespace_view(context, namespace)?;

    let mut tokens = Vec::new();
    for (key, value) in view.entries() {
        tokens.push(Token::new(key, value));
    }

    if tokens.is_empty() {
        return None;
    }

    Some(Meteor::new_with_tokens(
        Context::new(context),
        Namespace::from_string(namespace),
        tokens,
    ))
}
```

**Benefits:**
- **Workspace Ordering**: Tokens in meteors preserve insertion order from `key_order`
- **Lazy Construction**: Meteors created on-demand, no upfront memory cost
- **Hybrid Storage**: Falls back to storage keys when workspace unavailable
- **Consistent**: Same ordering as `NamespaceView` and `EntriesIterator`

#### Context/Namespace Enumeration

`MeteorsIterator` uses existing iteration methods for discovery:

```rust
fn new(engine: &'a MeteorEngine) -> Self {
    let contexts = engine.storage.contexts();  // Sorted contexts
    Self {
        engine,
        contexts,
        current_context_idx: 0,
        current_namespaces: Vec::new(),
        current_namespace_idx: 0,
    }
}
```

Contexts and namespaces are sorted alphabetically, providing deterministic iteration order across multiple calls.

### Test Coverage

**Test File**: `tests/test_meteor_aggregation.rs` (19 tests, 282 LOC)

**Core Functionality:**
- `test_meteor_for_basic` - Single namespace with multiple tokens
- `test_meteor_for_nonexistent` - Returns None for missing namespace
- `test_meteor_for_empty_namespace` - Handles empty namespaces
- `test_meteor_for_workspace_ordering` - Insertion order preservation
- `test_meteors_iterator_empty` - Empty engine iteration
- `test_meteors_iterator_single_namespace` - Single meteor case
- `test_meteors_iterator_multiple_namespaces_same_context` - Multiple namespaces per context
- `test_meteors_iterator_multiple_contexts` - Cross-context iteration

**Integration Tests:**
- `test_meteors_iterator_multiple_tokens_per_namespace` - Token count validation
- `test_meteors_iterator_workspace_ordering_preserved` - Ordering consistency
- `test_meteors_iterator_after_deletion` - Reflects deletions
- `test_meteors_iterator_complex_namespaces` - Deep namespace paths
- `test_meteor_for_with_bracket_keys` - Bracket notation support
- `test_meteors_iterator_consistency_with_meteor_for` - Iterator/accessor parity
- `test_meteors_iterator_multiple_iterations` - Idempotent iteration

**Edge Cases:**
- `test_meteor_for_token_values` - Value correctness
- `test_meteors_iterator_namespace_updates` - Reflects updates
- `test_meteor_for_with_index_key` - `.index` key handling
- `test_meteors_iterator_sorted_contexts` - Context sorting (apple→banana→zebra)

**All 19 tests passing** across all configuration profiles.

### Performance Characteristics

**MeteorsIterator:**
- **Initialization**: O(C log C) where C = contexts (sort contexts)
- **Per Meteor**: O(N log N + K) where N = namespaces in context, K = keys in namespace
  - O(N log N) to sort namespaces once per context
  - O(K) to collect keys and create tokens
- **Total**: O(C log C + Σ(N_i log N_i) + Σ K_i) ≈ O(total keys + sort overhead)

**meteor_for():**
- **Lookup**: O(K) where K = keys in namespace
- **Memory**: O(K) - creates Vec<Token> with K tokens

**Memory Usage:**
- `MeteorsIterator`: O(C + N) - stores context list and current namespace list
- Per `Meteor`: O(K) - stores K tokens (each token ~48-64 bytes)

**Compared to manual iteration:**
- Meteor aggregation is more efficient for grouped operations (one meteor per namespace vs. K individual entries)
- Eliminates manual grouping logic in CLI/REPL code
- Leverages workspace ordering without additional sorting

### Use Cases

**CLI `parse` Command:**
```rust
// CLI command to parse and display all meteors
fn cli_parse_command(engine: &MeteorEngine, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for meteor in engine.meteors() {
                println!("{}:{}", meteor.context().name(), meteor.namespace());
                for token in meteor.tokens() {
                    println!("  {} = {}", token.key().base(), token.value());
                }
            }
        }
        OutputFormat::Json => {
            let meteors: Vec<_> = engine.meteors().collect();
            println!("{}", serde_json::to_string_pretty(&meteors).unwrap());
        }
    }
}
```

**REPL `meteor` Command:**
```rust
// REPL command to inspect a specific meteor
fn repl_meteor_command(engine: &MeteorEngine, context: &str, namespace: &str) {
    match engine.meteor_for(context, namespace) {
        Some(meteor) => {
            println!("Meteor: {}:{}", context, namespace);
            println!("Tokens: {}", meteor.tokens().len());
            for token in meteor.tokens() {
                println!("  {} = {}", token.key().base(), token.value());
            }
        }
        None => println!("No meteor found for {}:{}", context, namespace),
    }
}
```

**Export/Import (ENG-22/ENG-23):**
```rust
// Export namespace to file format
fn export_namespace(engine: &MeteorEngine, context: &str, namespace: &str) -> Result<String, String> {
    let meteor = engine.meteor_for(context, namespace)
        .ok_or_else(|| format!("Namespace {}:{} not found", context, namespace))?;

    let mut output = String::new();
    output.push_str(&format!("{}:{}\n", context, namespace));

    for token in meteor.tokens() {
        output.push_str(&format!("{}={}\n", token.key().base(), token.value()));
    }

    Ok(output)
}
```

**Meteor View Composition:**
```rust
// Build aggregated view of multiple related namespaces
fn build_feature_view(engine: &MeteorEngine, feature: &str) -> Vec<Meteor> {
    engine.meteors()
        .filter(|m| m.namespace().to_string().starts_with(feature))
        .collect()
}
```

## Section/Part Emission with Ordering (ENG-21 Complete)

**Status**: ✅ Implemented and tested

### Overview

ENG-21 ensures that Meteor serialization (Display trait) honors workspace ordering metadata and uses human-readable bracket notation for section/part keys. This is critical for document and script virtualization use cases where section ordering must be preserved during export/import.

### Changes

#### Meteor Display Trait Update

Modified `Meteor::Display` implementation to use `key_notation()` instead of `key().to_string()`:

**Before:**
```rust
impl fmt::Display for Meteor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens_str = self.tokens.iter()
            .map(|token| format!("{}={}", token.key().to_string(), token.value()))
            .collect::<Vec<_>>()
            .join(";");
        // ...
    }
}
```

**After:**
```rust
impl fmt::Display for Meteor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens_str = self.tokens.iter()
            .map(|token| format!("{}={}", token.key_notation(), token.value()))
            .collect::<Vec<_>>()
            .join(";");
        // ...
    }
}
```

**Key Differences:**
- `token.key().to_string()` outputs transformed notation: `sections__i_intro`
- `token.key_notation()` outputs original bracket notation: `sections[intro]`

### Why This Matters

**Document Virtualization (DOC_VIRTUALIZATION_MODEL.md):**

Documents and scripts use bracket notation to represent structured content:
- `doc:guides.install:sections[intro]`, `sections[10_setup]`, `sections[20_config]`
- `shell:setup.env:parts[env_check]`, `parts[install_pkg]`, `parts[configure]`

When serializing meteors for:
1. **Export to filesystem** - section files must use readable names (`intro.md`, not `__i_intro.md`)
2. **CLI display** - users expect to see `sections[intro]=content`, not `sections__i_intro=content`
3. **REPL commands** - `meteor doc guides.install` should show human-readable output
4. **Round-trip fidelity** - parse(`meteor.to_string()`) must preserve original keys

**Ordering Preservation:**

The Display trait now respects workspace `key_order` through the token Vec:
1. `meteor_for()` creates meteors using `namespace_view().entries()` (ENG-11)
2. `entries()` leverages workspace `key_order` for insertion-order iteration (ENG-10)
3. Token Vec is built in workspace order
4. Display serializes tokens in Vec order (preserves insertion order)

### Example Output

**Document sections in insertion order:**
```rust
engine.set("doc:guides.install:sections[intro]", "Welcome").unwrap();
engine.set("doc:guides.install:sections[10_setup]", "Step 1").unwrap();
engine.set("doc:guides.install:sections[20_config]", "Configure").unwrap();

let meteor = engine.meteor_for("doc", "guides.install").unwrap();
println!("{}", meteor);
// Output: doc:guides.install:sections[intro]=Welcome;sections[10_setup]=Step 1;sections[20_config]=Configure
```

**Shell script parts in insertion order:**
```rust
engine.set("shell:setup.env:parts[env_check]", "#!/bin/bash\necho checking").unwrap();
engine.set("shell:setup.env:parts[install_pkg]", "apt install -y pkg").unwrap();

let meteor = engine.meteor_for("shell", "setup.env").unwrap();
println!("{}", meteor);
// Output: shell:setup.env:parts[env_check]=#!/bin/bash\necho checking;parts[install_pkg]=apt install -y pkg
```

### Test Coverage

**Test File**: `tests/test_meteor_aggregation.rs` (22 tests, 345 LOC)

**New Tests (3 tests added for ENG-21):**
- `test_meteor_display_preserves_workspace_ordering` - Doc sections with insertion order
- `test_shell_parts_display_preserves_ordering` - Shell script parts ordering
- `test_meteors_iterator_display_with_sections` - Iterator display consistency

**Key Assertions:**
```rust
#[test]
fn test_meteor_display_preserves_workspace_ordering() {
    engine.set("doc:guides.install:sections[intro]", "Welcome").unwrap();
    engine.set("doc:guides.install:sections[10_setup]", "Step 1").unwrap();

    let meteor = engine.meteor_for("doc", "guides.install").unwrap();
    let serialized = meteor.to_string();

    let tokens: Vec<&str> = serialized.split(':').nth(2).unwrap().split(';').collect();
    assert_eq!(tokens[0], "sections[intro]=Welcome");
    assert_eq!(tokens[1], "sections[10_setup]=Step 1");
}
```

**All 22 tests passing** (19 ENG-20 + 3 ENG-21) across all configuration profiles.

### Integration with ENG-22/ENG-23 (Export/Import)

The Display trait changes directly support upcoming export/import functionality:

**Export namespace (ENG-22):**
```rust
fn export_namespace(engine: &MeteorEngine, context: &str, namespace: &str) -> String {
    let meteor = engine.meteor_for(context, namespace).unwrap();

    // to_string() now produces bracket notation with workspace ordering
    let serialized = meteor.to_string();  // doc:guide:sections[intro]=...;sections[body]=...

    // Parse sections and create files in order
    for token in meteor.tokens() {
        let filename = token.key_notation();  // "sections[intro]" → "intro.md"
        write_file(filename, token.value());
    }
}
```

**Import validation (ENG-23):**
```rust
fn validate_roundtrip(engine: &MeteorEngine, context: &str, namespace: &str) -> bool {
    let meteor = engine.meteor_for(context, namespace).unwrap();
    let serialized = meteor.to_string();

    // Parse serialized form and verify keys match
    let reparsed = Meteor::first(&serialized).unwrap();
    reparsed.tokens().iter().zip(meteor.tokens().iter()).all(|(a, b)| {
        a.key_notation() == b.key_notation()  // Bracket notation preserved
    })
}
```

### Benefits

1. **Human-Readable Output**: CLI/REPL displays show `sections[intro]` not `sections__i_intro`
2. **Filesystem Export**: Section files named naturally (`intro.md`, `10_setup.md`)
3. **Round-Trip Fidelity**: `Meteor::parse(meteor.to_string())` preserves original keys
4. **Ordering Preservation**: Workspace `key_order` drives serialization order
5. **Doc Virtualization Ready**: Meets requirements from DOC_VIRTUALIZATION_MODEL.md

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

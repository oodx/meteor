# Meteor Engine Enhancement Plan

## Objectives
- Provide higher-level iteration and query helpers so CLI/REPL consumers no longer need to reach directly into `StorageData`.
- Expose safer cursor manipulation primitives to reduce manual `switch_*` calls and prevent inconsistent state.
- Package namespace/context slices and fully-qualified meteor views as first-class returns to prepare for future `Meteor` rearchitecture.
- Tighten parser integration by surfacing the validation behavior the engine relies on and by eliminating brittle fallback paths (`find` substring matching, unstructured path parsing).
- Support document/script virtualization described in `DOC_VIRTUALIZATION_MODEL.md` (export/import helpers, ordered sections, metadata management).
- Introduce an internal workspace layer so ordering metadata, caches, and scratch buffers do not leak into canonical storage.

## Proposed Additions

### 1. Iteration & Query Helpers
- `MeteorEngine::iter_entries()` → returns iterator over `(ContextRef, NamespaceRef, KeyRef, &str)`; used by CLI `parse` output and REPL `list/dump` commands.
- `MeteorEngine::entries_in(context: &str, namespace: &str)` → typed view for a single namespace; backing type should expose both canonical key/value pairs and metadata (entry count, whether namespace has default `.index`).
- `MeteorEngine::contexts_iter()` / `namespaces_iter(context)` wrappers to replace repeated `storage.contexts()` clones.
- `MeteorEngine::namespace_view(context, namespace)` → returns struct with `(context, namespace, tokens)` where tokens already include flattened + reconstructed variants; core building block for new Meteor facade.

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

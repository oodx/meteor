# Meteor Format Solidification Plan

Objective: Make `Meteor` a truly fully-qualified data packet (one context + one namespace + ordered tokens) and propagate that guarantee through parsers, engine, and tooling.

## 1. Define the Formal Meteor Type
- **Invariant**: every `Meteor` owns exactly one `Context` and one `Namespace`; all tokens inside must match both.
- **Structure**:
  ```rust
  struct Meteor {
      context: Context,
      namespace: Namespace,
      tokens: Vec<Token>,    // already transformed keys within namespace
  }
  ```
- **Ordering**: tokens remain in input order (or deterministic order based on bracket hints if desired). Stick with Vec to preserve parser insertion sequence.
- **Metadata**: optionally attach metadata (timestamp, source stream id) in future, but keep core struct minimal for now.

## 2. Parser Adjustments
### TokenStreamParser
- Maintain cursor `(context, namespace)` as today.
- When building meteors or emitting tokens, collect segments per cursor state. If the stream changes namespace via `ns=` or explicit address, start a new buffer.
- Disallow “mixed namespace in single meteor” by default. Provide clear error if a stream tries to wedge conflicting addresses between delimiters.
- Expose helper: `TokenStreamParser::collect_meteors(input) -> Result<Vec<Meteor>, Error>` that returns properly grouped meteors.

### MeteorStreamParser
- Parse explicit `context:namespace:key=value` entries.
- Group consecutive tokens by identical `(context, namespace)`; produce a new meteor whenever the pair changes.
- If a single meteor segment contains an inconsistent address, surface a parser error (strict mode) or split into separate meteors (lenient mode—decide policy).
- Update validation to reflect the invariant (error when a meteor definition mixes context/namespace pairs).

## 3. Meteor API Hardening
- `Meteor::new(context, namespace, tokens)` validates all tokens; return `Err` on mismatch.
- `Meteor::try_from_tokens(tokens)` becomes fallible: infer context/namespace from the first token but confirm everything else matches.
- Provide convenience constructors used by parsers: `Meteor::from_cursor(cursor, tokens)`.
- Add `Meteor::split_by_namespace(tokens)` helper for legacy callers; returns `Vec<Meteor>` by grouping tokens.

## 4. Engine Integration
- `MeteorEngine::meteor_for(context, namespace)` returns `Meteor` using the invariant (slice from StorageData).
- `MeteorEngine::meteors()` iterates over all `(context, namespace)` pairs, building validated meteors.
- `StorageData` remains canonical store; ensure no API allows writing a token whose context/namespace conflicts with its container.
- Update internal workspace (ENG-01) to leverage the invariant for caching/ordering.

## 5. CLI / REPL Updates
- CLI `parse` command uses `engine.meteors()` for output (text/json). No manual loops over storage.
- Add CLI option `--grouping=strict|lenient` to control parser treatment of mixed-address streams (strict default).
- REPL `meteor <ctx> <ns>` command returns the aggregated meteor; `list` command becomes sugar over `meteor` + token iteration.
- Ensure history/exports rely on the single-context meteor (simplifies virtualization workflows).

## 6. Testing Strategy
- Unit tests for `Meteor::new` rejecting mismatched tokens.
- Parser tests covering: single namespace meteor, context switch mid-stream (should split), explicit meteor stream mixing addresses (should error/split per policy).
- Engine tests verifying `meteors()` output matches storage slices and enforcing invariant.
- CLI/REPL integration tests after refactor to ensure output remains stable.

## 7. Migration Notes
- Identify any existing code constructing meteors without enforcing context/namespace (search for `Meteor::new`, `Vector<Meteor>`). Plan to refactor them first.
- For legacy compatibility, provide a temporary `Meteor::legacy_from_tokens` behind a feature flag if needed—but aim to remove after migration.
- Update docs (`TOKEN_NAMESPACE_CONCEPT`, engine architecture) to reflect the invariant explicitly.

## 8. Task Breakdown (link to ENGINE_TASKS.txt)
- Add tickets: `ENG-40` “Enforce Meteor invariant”, `ENG-41` “Parser grouping adjustments”, `CLI-05`/`REPL-05` “Output via meteor views”, `TEST-10` “Invariant regression tests”.
- Note dependencies: `EngineWorkspace` ordering + export helpers will benefit from the invariant.

Execution order: (1) Solidify Meteor struct, (2) adjust parsers + engine, (3) update CLI/REPL, (4) expand tests/docs, (5) delete/flag legacy helpers.

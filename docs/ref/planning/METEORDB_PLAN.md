# MeteorDB Plan

## Goal
Evolve MeteorEngine into **MeteorDB**, a lightweight, namespaced key-value datastore with persistent storage, structured querying, and integration hooks.

## Product Requirements
- **Namespaced Isolation**: Maintain existing context/namespace segmentation with directory semantics.
- **Persistence**: Support durable storage (snapshot + optional write-ahead log) with crash-safe recovery.
- **Query API**: Expose CRUD, directory navigation, and pattern/prefix search via library API, CLI, and optional HTTP layer.
- **Schema Features**: `.index` defaults, hierarchical keys, and metadata per namespace (timestamps, tags).
- **Concurrency**: Thread-safe read/write access with granularity at context level.
- **Extensibility**: Plugin mechanism for custom serializers, observers, and export/import adapters.

## Milestones
1. **Foundation (v0.1)**
   - Formalize storage interfaces; document canonical key rules.
   - Harden parser; add validation for malformed paths.
   - Implement snapshot/restore using JSON or binary serde.
   - Expose transaction-like batch API (atomic set/delete).
   - Ensure hybrid tree stays consistent across operations (tests for rename/delete cases).

2. **Persistence & Reliability (v0.2)**
   - Add write-ahead log (append-only) for crash recovery.
   - Provide background snapshot compaction (configurable interval).
   - Introduce integrity checks (hash or checksum per context).
   - Implement namespace-level ACL hooks (pluggable auth callbacks).

3. **Query & Tooling (v0.3)**
   - Expand query language: prefix/glob filters, depth-limited traversal.
   - Deliver CLI commands (`metdb get/set/find/dump`) wrapping engine API.
   - Optional REST/gRPC shim for remote clients.
   - Add metrics (operation counts, latency, storage size) with exporter interface.

4. **Ecosystem & Integration (v0.4)**
   - Provide import/export adapters (JSON, YAML, SQLite bridge).
   - SDK docs and examples in Rust + bindings for at least one other language (e.g., Python via pyo3).
   - Pluggable observers for change feeds (webhooks, messaging bus).

5. **Production Hardening (v1.0)**
   - Benchmark suite; publish latency/throughput targets.
   - HA story: leader/follower replication or snapshot shipping.
   - Strict semver guarantees; deprecation policy and migration guides.

## Technical Considerations
- Use `serde` for structured persistence; evaluate `bincode` vs. JSON.
- Leverage `parking_lot` or `tokio::sync` primitives for concurrency.
- WAL format: length-prefixed frames with CRC; snapshot includes manifest of contexts/namespaces.
- Configurable storage backend: start with local filesystem, abstract for future cloud/distro.

## Success Metrics
- 100% pass on persistence + crash recovery tests.
- CLI parity with core API (all operations accessible via command line).
- Stable benchmarks: <5 ms median read/write within single context under moderate load.
- Positive developer feedback from early adopters (survey or GitHub issues).

## Open Questions
- Do we need multi-key transactions or optimistic locking in v1?
- Should schema metadata (e.g., TTL) be core or plugin?
- How to version snapshots for future compatibility?

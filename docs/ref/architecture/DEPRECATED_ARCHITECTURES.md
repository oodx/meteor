# Deprecated Architectures

This document tracks previous architectural approaches that have been superseded by the current Hybrid Storage Architecture.

## Deprecated: Simple HashMap Storage (Pre-TICKET-015)

**Status:** ❌ DEPRECATED - Replaced by Hybrid Storage Architecture
**Used in:** Initial implementation through TICKET-014
**Deprecated:** 2025-09-24

### Description

The original storage model used nested HashMaps to represent hierarchical data:

```rust
// DEPRECATED: Simple nested HashMap
struct StorageData {
    contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
    //           ^context    ^namespace   ^key       ^value
}
```

### Problems Identified

1. **Inefficient Hierarchical Queries**
   - Required O(N) iteration through all namespaces for prefix queries
   - No support for `find("user.settings.*")` style operations
   - Expensive wildcard matching

2. **No Filesystem Semantics**
   - Could not distinguish between containers and values
   - Type conflicts: `user="jose"` vs `user.settings.theme="dark"`
   - No clear file vs directory model

3. **Poor Scalability**
   - Linear search for namespace operations
   - No indexing for common query patterns
   - Memory inefficient for sparse hierarchies

### Migration Path

**Replaced by:** [Hybrid Storage Architecture](./HYBRID_STORAGE_ARCHITECTURE.md)

**Key improvements:**
- ✅ O(1) direct access via flat canonical keys
- ✅ O(log n) hierarchical queries via tree index
- ✅ Clear filesystem semantics (files vs directories)
- ✅ Efficient prefix matching and wildcard queries

## Deprecated: MeteorShower as Primary Container

**Status:** ❌ DEPRECATED - Replaced by MeteorEngine with Context Isolation
**Used in:** Early design phases
**Deprecated:** 2025-09-24

### Description

Early designs used MeteorShower as the primary data container:

```rust
// DEPRECATED: MeteorShower-centric design
struct MeteorShower {
    meteors: Vec<Meteor>,  // Linear collection of meteors
}
```

### Problems Identified

1. **No Context Isolation**
   - All data mixed together in single container
   - No way to separate application vs user vs system data

2. **Linear Access Patterns**
   - O(N) search through meteor collections
   - No efficient lookups by context/namespace/key

3. **Limited Query Capabilities**
   - Could not efficiently answer "all keys in namespace" queries
   - No hierarchical traversal support

### Migration Path

**Replaced by:** MeteorEngine with Context-isolated Hybrid Storage

**Key improvements:**
- ✅ Context isolation for data separation
- ✅ Efficient indexed storage and retrieval
- ✅ Stateful cursor operations
- ✅ Command audit trails

## Deprecated: TokenBucket Folding Approach

**Status:** ❌ CONSIDERED BUT NOT ADOPTED
**Considered during:** Architecture discussions
**Rejected:** 2025-09-24

### Description

Considered using TokenBucket folding with flattened keys:

```rust
// CONSIDERED: Flattened key approach
TokenBucket {
    entries: [
        "user__settings__theme" -> "dark",     // Flattened namespace
    ],
    prefix_index: {
        "user" -> [entry_indices],             // Prefix indexing
    }
}
```

### Why Rejected

1. **Semantic Ambiguity**
   - `user` could mean literal value OR namespace prefix
   - Violates hierarchical semantics (can't be both container AND value)
   - JSON/filesystem model incompatibility

2. **Complex Index Maintenance**
   - Must maintain prefix indexes on every mutation
   - Memory overhead for all possible prefix combinations
   - Difficult to reason about and debug

3. **Loss of Structure**
   - Flattening loses hierarchical relationships
   - No natural file vs directory distinction
   - Difficult to implement clean query APIs

### Alternative Chosen

**Instead adopted:** Hybrid Storage with Tree Index

**Advantages:**
- ✅ Preserves hierarchical semantics
- ✅ Clear file vs directory model
- ✅ Efficient without semantic ambiguity

## Architecture Evolution Timeline

```
2025-09-24: Simple HashMap Storage
    ↓ (Performance issues with hierarchical queries)
2025-09-24: Considered TokenBucket Folding
    ↓ (Rejected due to semantic ambiguity)
2025-09-24: Hybrid Storage Architecture ← CURRENT
    ↓ (Future evolution)
2025-XX-XX: [Future architecture decisions]
```

## Notes for Future Development

When considering new architectural changes:

1. **Preserve Filesystem Semantics** - Files vs directories model works well
2. **Maintain Context Isolation** - Critical for data separation
3. **Keep Canonical Keys** - Single source of truth principle
4. **Test Query Performance** - Hierarchical operations must be efficient
5. **Document Migration Path** - Always provide clear upgrade strategy

## References

- [Current Hybrid Storage Architecture](./HYBRID_STORAGE_ARCHITECTURE.md)
- [TASKS.txt](../procs/TASKS.txt) - Implementation timeline
- [MeteorEngine Implementation](../../src/lib/types/meteor/engine.rs)
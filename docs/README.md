# Meteor Project Documentation

## 🌟 Overview

Meteor is a hierarchical data storage and management system built on a **hybrid storage architecture**. It combines the efficiency of flat HashMap storage with the intuitive semantics of filesystem-like tree navigation, providing both O(1) direct access and O(log n) hierarchical queries.

### Key Features
- **Hybrid Storage**: Flat canonical keys + tree index for dual access patterns
- **Context Isolation**: Complete data separation between contexts (app, user, system)
- **Filesystem Semantics**: File/Directory model with hierarchical navigation
- **Canonical Addressing**: `"namespace:path.to.key"` format for consistent data access
- **Bracket Notation**: Advanced array/object indexing with `[0]`, `[name]`, `[]` append
- **O(1) Direct Access**: Hash-based lookups for known paths
- **O(log n) Hierarchical Queries**: Tree traversal for pattern matching and exploration

### Current Status
- **Architecture**: Hybrid Storage (TICKET-015 Complete) ✅
- **Implementation**: 99.1% test success rate (116/117 tests passing)
- **Production Readiness**: Core functionality complete and stable

## 🏗️ Architecture Components

### Hybrid Storage Model
```rust
struct ContextStorage {
    flat_data: HashMap<String, String>,      // O(1) canonical key-value storage
    tree_index: TreeMap<String, TreeNode>,   // O(log n) hierarchical navigation
}
```

- **Flat Storage**: All data stored as canonical `"namespace:path.to.key"` → `value`
- **Tree Index**: Hierarchical navigation structure pointing into flat storage
- **Context Isolation**: Each context (app, user, system) has independent storage

### MeteorEngine Capabilities
```rust
// Direct access (O(1))
engine.get("user.settings.theme")          // → "dark"

// Hierarchical queries (O(log n))
engine.find("user.settings.*")             // → all settings keys

// Filesystem operations
engine.is_directory("user.settings")       // → true
engine.is_file("user.settings.theme")      // → true
engine.has_default("user")                 // → true if user.index exists
```

### TreeNode Filesystem Semantics
- **File Nodes**: Terminal nodes containing actual values
- **Directory Nodes**: Branch nodes organizing hierarchical structure
- **Default Values**: `path.index` pattern for directory defaults (e.g., `user.index` for default user value)

## 🎯 Active Documentation (Up-to-Date)

### Architecture Reference
- **[Hybrid Storage Architecture](ref/architecture/HYBRID_STORAGE_ARCHITECTURE.md)** - Complete specification of current storage system
- **[Deprecated Architectures](ref/architecture/DEPRECATED_ARCHITECTURES.md)** - Historical architecture tracking
- **[Stream Architecture](ref/architecture/STREAM_ARCHITECTURE.md)** - Stream parsing architecture
- **[MeteorEngine Internals](ref/architecture/METEORENGINE_INTERNALS.md)** - Detailed engine implementation

### User Guides
- **[CLI RSB Usage](ref/guides/CLI_RSB_USAGE.md)** - Command-line interface documentation
- **[Token Namespace Concept](ref/guides/TOKEN_NAMESPACE_CONCEPT.md)** - Namespace system documentation

### Development Processes
- **[Architecture Status](procs/ARCHITECTURE_STATUS.md)** - Current implementation status
- **[Continue Guide](procs/CONTINUE.md)** - Recent development handoff documentation
- **[Tasks](procs/TASKS.txt)** - Current development tickets and status

### Critical Issues
- **[CLI Parser Regression](ref/reference/REGRESSION_CLI_PARSER.md)** - ⚠️ Critical parser issues requiring immediate attention

## 📚 Reference Documentation

### Technical Reference
- **[Module Plan](ref/reference/MODULE_PLAN.md)** - Module organization and architecture
- **[RSB CLI Features](ref/reference/RSB_CLI_FEATURES.md)** - CLI feature documentation
- **[HOWTO Hub](ref/reference/HOWTO_HUB.md)** - Collection of how-to guides
- **[Configuration](CONFIGURATION.md)** - System configuration guide

### RSB Framework
- **[RSB Features](ref/features/)** - Feature specifications
- **[RSB Specifications](ref/rsb/)** - Framework documentation

## 🗃️ Archive (Historical)

### Deprecated Architecture
- **[Token Concept Amendment](archive/TOKEN_CONCEPT_AMENDMENT.md)** - Historical token architecture (replaced by hybrid storage)

## 🚀 Quick Start

### For New Users
1. **Understand the Architecture**: Start with [Hybrid Storage Architecture](ref/architecture/HYBRID_STORAGE_ARCHITECTURE.md)
2. **Learn the Addressing**: Review canonical key format `"namespace:path.to.key"`
3. **Explore Context Isolation**: Understand how app, user, and system contexts work
4. **Try the CLI**: See [CLI RSB Usage](ref/guides/CLI_RSB_USAGE.md) for hands-on examples

### For Developers
1. **Check Current Status**: Review [Architecture Status](procs/ARCHITECTURE_STATUS.md)
2. **Understand Implementation**: Study the hybrid storage model
3. **Review Tasks**: Check [Tasks](procs/TASKS.txt) and [Continue Guide](procs/CONTINUE.md)
4. **Handle Critical Issues**: See [CLI Parser Regression](ref/reference/REGRESSION_CLI_PARSER.md) for urgent fixes

### Core Examples
```rust
// Basic storage operations
engine.set("app:ui.theme", "dark");        // Store in app context
engine.get("app:ui.theme");                // → "dark"

// Hierarchical operations
engine.set("user:settings.theme", "dark");
engine.set("user:settings.lang", "en");
engine.find("user:settings.*");            // → ["theme", "lang"]

// Context switching
engine.switch_context("user");
engine.set("profile.name", "jose");        // Now in user context
```

## 📊 Documentation Status

- ✅ **Architecture**: Current and comprehensive (organized in `/architecture/`)
- ✅ **User Guides**: CLI and concepts documented (organized in `/guides/`)
- ✅ **Implementation**: Hybrid storage fully documented
- 🔄 **Reference**: Technical docs and RSB framework extensive, under review
- 📋 **Processes**: Up-to-date with recent development
- ⚠️ **Critical Issues**: Parser regression documented, requires immediate attention

---

**Last Updated**: 2025-09-24
**Architecture Version**: Hybrid Storage (TICKET-015 Complete)
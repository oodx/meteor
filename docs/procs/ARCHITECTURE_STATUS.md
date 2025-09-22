# Meteor Architecture Status

**Last Updated:** 2025-09-22
**Phase:** Module Reorganization Complete, RSB Integration Ready

## Current Type Architecture

### Core Types
- **`Context`** - Isolation boundaries (app, user, system, file1, remote1)
- **`Namespace`** - Hierarchical organization with dot notation (ui.widgets)
- **`TokenKey`** - Individual key identifiers with bracket transformation
- **`Token`** - Key-value pair combining TokenKey + value
- **`Meteor`** - Complete addressing: `context:namespace:key=value`

### Collection Types
- **`MeteorShower`** - Object-oriented collection for Meteor tokens
  - Indexed lookups by context/namespace
  - Query methods: `by_context()`, `find()`, `contexts()`
- **`StorageData`** - Serialized/flattened interchange format
  - Export format from MeteorShower
  - JSON/string serialization support

### Extensibility
- **`BracketTransform`** - Trait for bracket notation handling
  - Caching approach (stores original + transformed)
  - Inverse parsing: `list__i_0` → `list[0]`
  - Extensible for future bracket types

## Parser Capabilities

### Full Meteor Spec Support ✅
- `context:namespace:key=value` - Full addressing
- `namespace:key=value` - Default app context
- `key=value` - Root namespace, app context
- Context switching: `ctx=user; key=value`

### Bracket Notation ✅
- Numeric: `list[0]` → `list__i_0`
- Multi-dimensional: `grid[2,3]` → `grid__i_2_3`
- Append: `queue[]` → `queue__i_APPEND`
- Named: `user[name]` → `user__name`

### API Functions
- `meteor::parse_shower()` → `MeteorShower` (primary API)
- `meteor::parse_meteor()` → `Meteor` (single context)
- `MeteorShower::to_data()` → `StorageData` (interchange format)

## Module Organization (MODULE_SPEC Compliant)

```
src/lib/
├── types/           # Core type definitions
│   ├── context.rs      # Context isolation
│   ├── namespace.rs    # Hierarchical organization
│   ├── key.rs         # TokenKey with bracket transform
│   ├── token.rs       # Key-value pairs
│   ├── meteor.rs      # Complete addressing
│   ├── shower.rs      # MeteorShower collection
│   ├── storage_data.rs # StorageData interchange format
│   ├── bracket_transform.rs # Extensible trait
│   └── error.rs       # Error types
├── parser/          # Token parsing infrastructure
│   ├── parse.rs       # Core parsing logic (GUTTED)
│   ├── transform.rs   # Bracket transformation (GUTTED)
│   ├── organize.rs    # Data organization (GUTTED)
│   └── mod.rs        # Parser exports
├── utils/           # Public API utilities
└── lib.rs          # Main exports
```

## API Design Patterns

### Trait-Based Extensibility
```rust
// Standard usage
let key = TokenKey::new("list[0]");
let flat = key.to_string();        // "list__i_0" (Display)
let bracket = key.to_bracket();    // "list[0]" (BracketTransform)

// Inverse parsing
let reconstructed = "list__i_0".to_bracket(); // "list[0]"
```

### Dual Collection Approach
```rust
// Object-oriented (rich queries)
let shower = meteor::parse_shower("app:ui:button=click")?;
let meteors = shower.by_context("app");

// Interchange format (for serialization)
let shower = meteor::parse_shower("ui:button=click")?;
let storage_data = shower.to_data();
let json = storage_data.to_json();
```

## Next Phase: RSB Integration

**Priority Tasks:**
1. **RSB Global State** - CLI session management
2. **RSB FS + Strings** - File operations and text processing
3. **Enhanced CLI** - Development workflow features

**Current Status:** Architecture foundation complete, ready for RSB feature integration.

**Compatibility:** Full backward compatibility maintained, new features are additive.
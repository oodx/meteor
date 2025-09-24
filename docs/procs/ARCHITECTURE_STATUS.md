# Meteor Architecture Status

**Last Updated:** 2025-09-24
**Phase:** Meteor Path Parsing Fixed, Architecture Validated

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

## Configuration System ✅

### Build-Time Configuration
- **`meteor.toml`** - Project-specific configuration file
- **`build.rs`** - Compile-time configuration reader
- **`meteor::config`** - Configuration constants module
- **`meteor-config`** - Configuration inspection binary

### Deployment Profiles
- **Default**: Balanced (4 clear, 5 warning, 6+ error namespace depth)
- **Enterprise**: High-performance (10k meteors, 256 char keys, 8k values)
- **Embedded**: Memory-constrained (100 meteors, 32 char keys, 256 values)
- **Strict**: Security-focused (50 meteors, 16 char keys, 128 values)

### Security Features
- ✅ **Build-time immutability** - Limits compiled into binary
- ✅ **Runtime tamper-proof** - No configuration files at runtime
- ✅ **Environment override** - `METEOR_PROFILE=enterprise cargo build`
- ✅ **Profile isolation** - Different binaries for different security postures

## Stream Processing Architecture ✅

### MeteorEngine (Stateful)
- **Cursor state** - Persistent context/namespace across operations
- **Command audit trail** - All control commands logged with timestamps
- **Colon-delimited API** - `engine.set("app:ui:button", "click")` ✅ **FIXED**
- **Control commands** - `ctl:delete=path`, `ctl:reset=cursor`

### Parser Modules (Validation + Delegation)
- **TokenStreamParser** - Handles folding logic with cursor state
- **MeteorStreamParser** - Handles explicit meteor addressing
- **Pure validation** - Parsers validate, MeteorEngine controls state

## Module Organization (MODULE_SPEC Compliant)

```
src/lib/
├── config.rs        # Build-time configuration system
├── types/           # Core type definitions
│   ├── context.rs      # Context isolation
│   ├── namespace.rs    # Hierarchical organization + dot path parsing
│   ├── key.rs         # TokenKey with bracket transform
│   ├── token/         # Token types
│   │   ├── token.rs      # Key-value pairs
│   │   └── bucket.rs     # TokenBucket (legacy, preserved)
│   ├── meteor/        # Meteor types + configuration
│   │   ├── meteor.rs     # Complete addressing
│   │   ├── shower.rs     # MeteorShower collection
│   │   ├── engine.rs     # MeteorEngine stateful processing
│   │   ├── storage_data.rs # StorageData interchange format
│   │   └── config.rs     # Configuration constants
│   ├── error.rs       # Error types
│   └── mod.rs        # Type re-exports
├── parser/          # Stream processing + validation
│   ├── token_stream.rs  # Token stream parsing with folding
│   ├── meteor_stream.rs # Meteor stream parsing
│   ├── escape.rs       # Escape sequence handling
│   └── mod.rs         # Parser exports
├── validation/      # Format validation utilities
├── utils/          # Public API utilities
└── lib.rs         # Main exports

src/bin/
├── cli.rs          # Main CLI application
└── config.rs       # Configuration inspection utility
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

### Meteor Path Format ✅ **CRITICAL FIX APPLIED**
```rust
// ✅ CORRECT: Colon-delimited meteor format
engine.set("app:ui.widgets:button", "click");
engine.get("user:settings.theme:dark_mode");

// ✅ CORRECT: Namespace hierarchy uses dots
let ns = Namespace::from_string("ui.widgets.forms.inputs");

// ❌ FIXED: Previous incorrect dot-delimited format
// engine.set("app.ui.button", "click"); // WRONG - now fixed
```

### Dual Collection Approach
```rust
// Object-oriented (rich queries) - ✅ COLON FORMAT
let shower = meteor::parse_shower("app:ui:button=click")?;
let meteors = shower.by_context("app");

// Interchange format (for serialization) - ✅ COLON FORMAT
let shower = meteor::parse_shower("app:ui:button=click")?;
let storage_data = shower.to_data();
let json = storage_data.to_json();
```

## Next Phase: RSB Integration

**Priority Tasks:**
1. **RSB Global State** - CLI session management
2. **RSB FS + Strings** - File operations and text processing
3. **Enhanced CLI** - Development workflow features

**Current Status:** Architecture foundation complete, meteor format validated, ready for RSB feature integration.

**Critical Fix Applied:** Meteor path parsing now uses correct colon-delimited format (`CONTEXT:NAMESPACE:KEY`).

**Quality Assurance:** 173 tests passing including visual UAT demonstrations.

**Compatibility:** Full backward compatibility maintained, new features are additive.
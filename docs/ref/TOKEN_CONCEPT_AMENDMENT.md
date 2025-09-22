# Token Concept Architecture Amendment

**Date**: 2025-09-22
**Status**: Critical architectural correction needed
**Impact**: Major - affects all documentation, code, and API design

## Problem Identified

During parser module audit, we discovered a **fundamental architectural inversion** in the current implementation that conflicts with the intended design.

### Current Wrong Implementation

```rust
// WRONG: TokenBucket as primary storage
TokenBucket {
    context: Context,                                    // Single context
    data: HashMap<String, HashMap<String, String>>,     // namespace -> key -> value
}

// WRONG: MeteorShower as simple collection
MeteorShower {
    meteors: Vec<Meteor>,     // Just a list of meteors
}

// WRONG: Meteor as single addressing unit
Meteor { context, namespace, token }  // Only one token per meteor
```

### Correct Architecture

```rust
// CORRECT: Token as pure key-value pair
Token { key: TokenKey, value: String }  // Pure key-value, no context

// CORRECT: Meteor as single context with multiple tokens
Meteor {
    context: Context,
    tokens: Vec<(Namespace, Token)>      // Multiple tokens in same context
}

// CORRECT: MeteorShower as primary storage with indexing
MeteorShower {
    meteors: Vec<Meteor>,
    // Storage/indexing (moved FROM TokenBucket):
    contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
    context_index: HashMap<String, Vec<usize>>,
    namespace_index: HashMap<String, HashMap<String, Vec<usize>>>,
}
```

## Hierarchy Correction

**WRONG current assumption**:
```
TokenBucket (primary storage)
└── Token[] (key-value pairs)
```

**CORRECT hierarchy**:
```
MeteorShower (primary storage with cross-context indexing)
├── Meteor("app" context)
│   ├── (ui namespace) → Token[], Token[], Token[]
│   └── (config namespace) → Token[], Token[]
├── Meteor("user" context)
│   └── (settings namespace) → Token[]
└── Meteor("system" context)
    └── (env namespace) → Token[]
```

## Parse String Mapping

**Current wrong parsing**:
- `"app:ui:button=click"` → Single Meteor with one Token ❌
- `"ui:button=click; config:theme=dark"` → TokenBucket ❌

**Correct parsing**:
- `"app:ui:button=click,theme=dark"` → Single Meteor with multiple Tokens ✅
- `"app:ui:button=click; user:settings:lang=en"` → MeteorShower with multiple Meteors ✅

## Data Structure Migration

### TokenBucket → MeteorShower Migration

**Extract from TokenBucket** (to be deleted):
```rust
// Move this storage logic TO MeteorShower:
contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
// Plus all the indexing/query methods
```

**MeteorShower Enhancement**:
```rust
impl MeteorShower {
    // NEW: Interchange format methods
    pub fn to_storage_format(&self) -> StorageData;
    pub fn from_storage_format(data: StorageData) -> Self;

    // KEEP: Object-oriented meteor access
    pub fn by_context(&self, context: &str) -> Vec<&Meteor>;
    pub fn by_context_namespace(&self, context: &str, namespace: &str) -> Vec<&Meteor>;

    // MOVE FROM TokenBucket: Direct key-value access for simple cases
    pub fn get(&self, context: &str, namespace: &str, key: &str) -> Option<&str>;
    pub fn set(&mut self, context: &str, namespace: &str, key: &str, value: &str);
}
```

### Separate Storage Module

**New file**: `src/lib/types/meteor/shower_data.rs`
```rust
//! Storage interchange format for MeteorShower
//!
//! Provides flattened storage representation for serialization,
//! similar to what TokenBucket provided.

#[derive(Debug, Clone)]
pub struct StorageData {
    pub contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl StorageData {
    pub fn to_json(&self) -> String;
    pub fn from_json(json: &str) -> Result<Self, Error>;
    pub fn to_string(&self) -> String;  // Flat token stream format
}
```

## Parser Impact

**ParseMode correction**:
```rust
#[derive(Debug, Clone, Default)]
pub enum ParseMode {
    #[default]
    BracketNotation,  // list[0] - human readable
    Flat,            // list__i_0 - storage format
}
```

**Parser API correction**:
```rust
// PRIMARY API: MeteorShower parsing
MeteorShower::parse("app:ui.widgets.buttons:button=click; user:settings.lang:lang=en") -> MeteorShower

// SECONDARY API: Single meteor parsing
Meteor::parse("app:ui.widgets.forms:button=click,theme=dark") -> Meteor

// BASIC API: Single token parsing
Token::parse("button=click") -> Token
```

## Namespace Multi-Dot Addressing

**Current constraint**: Namespace depth limited to < 5 levels
**Examples**:
- `"ui"` (depth 1) ✅
- `"ui.widgets"` (depth 2) ✅
- `"ui.widgets.buttons"` (depth 3) ✅
- `"ui.widgets.buttons.primary"` (depth 4) ✅
- `"ui.widgets.buttons.primary.state"` (depth 5) ❌ Too deep

**Configuration needed**:
```rust
#[derive(Debug, Clone)]
pub struct NamespaceConfig {
    pub max_depth: usize,  // Default: 4, configurable for special cases
}

// Parser with namespace config
MeteorShower::parse_with_config(input: &str, config: NamespaceConfig) -> Result<Self, Error>
```

**Use cases for deeper namespaces**:
- Complex UI hierarchies: `"app.dashboard.widgets.charts.timeseries.data"`
- Deep configuration: `"system.network.interfaces.eth0.ipv4.dhcp"`
- Nested components: `"components.forms.inputs.validation.rules.required"`

## CLI Impact

**Current broken CLI**:
```rust
meteor::parse(input) -> TokenBucket  // ❌ Wrong primary type
```

**Correct CLI**:
```rust
meteor::parse_shower(input) -> MeteorShower  // ✅ Correct primary type
meteor::parse_meteor(input) -> Meteor        // ✅ Single context parsing
```

## Documentation Carnage 😭

**Files needing updates**:
- `README.md` - All examples use wrong TokenBucket API
- `TOKEN_NAMESPACE_CONCEPT.md` - Architecture section wrong
- `TASKS.txt` - All parser tasks assume wrong architecture
- `RSB_CLI_FEATURES.md` - CLI examples use wrong API
- All code comments and examples throughout codebase

## Implementation Plan

### Phase 0: Document Architecture (THIS PHASE)
- [x] Create this amendment document
- [ ] Link in main TOKEN_NAMESPACE_CONCEPT.md
- [ ] Update TASKS.txt with correction phases
- [ ] Create DOC-NN tasks for systematic documentation updates

### Phase 1: Storage Migration
- [ ] Create `shower_data.rs` storage module
- [ ] Move TokenBucket storage logic to MeteorShower
- [ ] Update MeteorShower to be primary storage type
- [ ] Add interchange format methods

### Phase 2: Delete TokenBucket
- [ ] Remove TokenBucket type entirely
- [ ] Update all imports and references
- [ ] Update CLI to use MeteorShower

### Phase 3: Parser Rebuild (CONTINUES)
- [ ] Continue with shared parsing infrastructure
- [ ] Use correct MeteorShower-primary architecture
- [ ] Implement ParseMode correctly

### Phase 4: Documentation Updates (DOC-NN tasks)
- [ ] DOC-01: Update README.md examples
- [ ] DOC-02: Update TOKEN_NAMESPACE_CONCEPT.md
- [ ] DOC-03: Update all code examples
- [ ] DOC-04: Update CLI documentation

## Risk Assessment

**HIGH RISK**: This is a foundational architecture change affecting:
- ✅ All API designs (fix: use MeteorShower as primary)
- ✅ All documentation (fix: systematic DOC-NN updates)
- ✅ All examples (fix: replace TokenBucket with MeteorShower)
- ✅ CLI implementation (fix: use parse_shower not parse)
- ✅ Test suites (fix: update all test expectations)

**CRITICAL**: Must complete this correction before any other development work to avoid building on wrong foundation.

## Success Criteria

1. **TokenBucket deleted** - No references remain in codebase
2. **MeteorShower is primary storage** - Has all storage/indexing capabilities
3. **Correct hierarchy** - Shower → Meteor → Tokens
4. **All docs updated** - No TokenBucket examples remain
5. **CLI works** - Uses MeteorShower as primary API
6. **Tests pass** - Updated for correct architecture

---

**PRIORITY**: Complete this architectural correction before continuing parser development.
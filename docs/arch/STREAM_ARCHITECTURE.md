# Stream Architecture: TokenStream vs MeteorStream

## Overview

This document captures the critical architectural insights around stream processing, storage formats, and the relationship between TokenBucket, StorageData, and MeteorShower.

## Core Problem Solved

**Original Issue**: We had competing paradigms:
- **Meteor/MeteorShower**: Explicit addressing only (`app:ui:button=click`)
- **TokenBucket**: Stream processing with folding (`button=click;ns=ui;theme=dark`)

**Solution**: Dual parsing in MeteorShower with StorageData as unified internal format.

## Stream Types

### TokenStream (with folding logic)
```
Format: "button=click;ns=ui;theme=dark;ctx=user;profile=admin"
```

**Characteristics:**
- ✅ **Control tokens**: `ns=ui`, `ctx=user` change parsing state
- ✅ **Explicit prefixes**: `app:ui:button=click` (overrides folding)
- ✅ **Mixed format**: Can combine folding + explicit in same stream
- ✅ **No spaces required**: `button=click;theme=dark`
- ✅ **Semicolon delimited**: `;` separates tokens
- **Default namespace**: `"main"` (not "global" - avoids RSB collision)
- **Default context**: `"app"`

**Folding Logic:**
1. Start with `app:main` as active context:namespace
2. `ns=ui` → switch namespace to `ui` for subsequent tokens
3. `ctx=user` → switch context to `user` for subsequent tokens
4. Explicit prefixes (`user:settings:key=val`) override active state
5. Control tokens (`ns=`, `ctx=`) are consumed, not stored

### MeteorStream (explicit only)
```
Format: "app:ui:button=click;theme=dark :;: user:main:profile=admin"
```

**Characteristics:**
- ❌ **No control tokens**: `ns=`, `ctx=` should be rejected as invalid
- ✅ **Explicit addressing**: All meteors must be fully qualified
- ✅ **Meteor delimiter**: `:;:` separates meteors
- ✅ **Semicolon within**: `;` separates tokens within meteors
- **No folding**: Each meteor is independent

## Storage Architecture

### The Key Insight: StorageData IS TokenBucketManager

**StorageData Structure:**
```rust
pub struct StorageData {
    /// context -> namespace -> key -> value
    pub contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
}
```

**This is exactly what TokenBucketManager would be:**
```
context1 → TokenBucket { namespace → key → value }
context2 → TokenBucket { namespace → key → value }
context3 → TokenBucket { namespace → key → value }
```

### MeteorShower Internal Format

**New Architecture:**
```rust
pub struct MeteorShower {
    storage: StorageData,  // PRIMARY internal format
    // REMOVED: meteors: Vec<Meteor>
    // REMOVED: context_index, namespace_index (redundant)
}
```

**Benefits:**
- **Efficient lookups**: HashMap-based instead of Vec linear search
- **Lazy meteor creation**: Only create Meteor objects when requested
- **Memory efficient**: No duplicate storage + indices
- **Same external API**: Users don't see the change

## Dual Parsing Strategy

### MeteorShower gets two parsing methods:

```rust
impl MeteorShower {
    /// Explicit meteor parsing (existing) - no folding
    pub fn parse(input: &str) -> Result<Self, String> {
        // Current logic: validates explicit meteors only
        // Rejects ns= and ctx= tokens
    }

    /// Token stream parsing (NEW) - with folding logic
    pub fn from_token_stream(input: &str) -> Result<Self, String> {
        // Adapted TokenBucket folding logic
        // Populates StorageData directly
        // Handles ns=, ctx= control tokens
    }
}
```

### Processing Flow:

**TokenStream → StorageData:**
```
"button=click;ns=ui;theme=dark;ctx=user;profile=admin"
                    ↓
      [TokenBucket folding logic adapter]
                    ↓
        StorageData {
          "app" → {
            "main" → { "button" → "click" },
            "ui" → { "theme" → "dark" }
          },
          "user" → {
            "main" → { "profile" → "admin" }
          }
        }
                    ↓
      MeteorShower { storage: StorageData }
```

**MeteorStream → StorageData:**
```
"app:ui:button=click;theme=dark :;: user:main:profile=admin"
                    ↓
        [Explicit parsing only]
                    ↓
        StorageData (same structure)
                    ↓
      MeteorShower { storage: StorageData }
```

## Query Interface

**Lazy Meteor Creation:**
```rust
impl MeteorShower {
    /// Create meteor on-demand from storage
    pub fn find(&self, context: &str, namespace: &str, key: &str) -> Option<Meteor> {
        if let Some(value) = self.storage.get(context, namespace, key) {
            Some(Meteor::new(
                Context::new(context),
                Namespace::from_string(namespace),
                Token::new(key, value)
            ))
        } else {
            None
        }
    }

    /// Get all meteors in context (creates on-demand)
    pub fn by_context(&self, context: &str) -> Vec<Meteor> {
        let mut meteors = Vec::new();
        for namespace in self.storage.namespaces_in_context(context) {
            // Create meteors from storage
        }
        meteors
    }
}
```

## Validation Rules

### TokenStream Validation
- ✅ `button=click;ns=ui;theme=dark`
- ✅ `ns=main;ctx=user;profile=admin`
- ✅ `button=click;app:ui:theme=dark` (mixed format)
- ✅ No spaces required
- ❌ Consecutive semicolons: `button=click;;theme=dark`

### MeteorStream Validation
- ✅ `app:ui:button=click;theme=dark :;: user:main:profile=admin`
- ❌ Control tokens: `button=click;ns=ui;theme=dark`
- ❌ `ns=ui` or `ctx=user` (control tokens rejected)
- ❌ Consecutive semicolons outside quotes

## Implementation Status

### Completed ✅
- TokenBucket with folding logic (`ns=`, `ctx=` support)
- StorageData with context→namespace→key→value structure
- Default namespace changed from "global" to "main"
- Validation utilities with quote-aware parsing

### Pending 🚧
- Refactor MeteorShower to use StorageData internally
- Add `MeteorShower::from_token_stream()` method
- Adapt TokenBucket folding logic for StorageData population
- Update MeteorShower query methods for lazy meteor creation

## Key Architectural Principles

1. **Unified Storage**: StorageData serves as the universal internal format
2. **Lazy Evaluation**: Meteor objects created only when requested
3. **Dual Parsing**: Support both folding streams and explicit meteors
4. **Context Isolation**: Each context gets its own namespace space
5. **Namespace Routing**: TokenBucket handles namespace folding within context
6. **No Redundancy**: Single storage format, no duplicate indices

## RSB Compliance Notes

- **"main" namespace**: Avoids collision with RSB's global variable system
- **Context hierarchy**: `global.app`, `global.user` at RSB level vs `app`, `user` at Meteor level
- **String-biased**: All storage as strings, typed conversion on access
# CLI Parser Regression Issues - CRITICAL

**Date**: 2025-09-24
**Severity**: P0 - BLOCKING
**Status**: ❌ BROKEN - MeteorStreamParser completely failing

## 🚨 **Critical Regression: MeteorStreamParser Broken**

### **Issue Description**

The `MeteorStreamParser` is completely failing to parse explicit meteor format. Recent "fixes" have broken the core meteor parsing functionality.

### **Test Cases Failing**

#### **Test 1: Basic Explicit Meteor Format**
```bash
meteor parse "global:main.help:button[]=click"
```

**Expected Result:**
```
Context: global
Namespace: main.help
Key: button[] (transformed: button__i_APPEND)
Value: click
```

**Actual Result (WRONG):**
```
Context: app                                    # ❌ Should be "global"
Namespace: main                                 # ❌ Should be "main.help"
Key: global:main.help:button__i_APPEND         # ❌ Treating meteor address as key!
Value: click
```

#### **Test 2: Simple Token vs Meteor Confusion**
```bash
meteor parse "button=click; ns=ui; theme=dark"
```

**Result:** Partially working but using wrong architecture approach.

### **Root Cause Analysis**

The recent changes to `MeteorStreamParser::process_single_meteor()` are fundamentally wrong:

1. **Wrong Architecture**: Treating explicit meteors as tokens
2. **Wrong Parser**: Using `Token::from_str()` instead of `Meteor::from_str()`
3. **Wrong Storage**: Using `engine.store_token()` (cursor-based) instead of explicit addressing
4. **Mixed Logic**: Trying to handle both token streams and meteor streams in the same function

### **What Went Wrong**

#### **Before (Working)**
```rust
// Parse the meteor
let meteor = Meteor::from_str(trimmed)?;

// Store using explicit addressing
let path = format!("{}:{}:{}",
    meteor.context(), meteor.namespace(), meteor.key());
engine.set(&path, meteor.token().value())?;
```

#### **After (Broken)**
```rust
// Parse as a regular token (WRONG!)
let token = Token::from_str(trimmed)?;

// Store using cursor (WRONG!)
engine.store_token(token.key(), token.value());
```

### **Architectural Confusion**

The parser is now confused about:

1. **Meteor Format**: `CONTEXT:NAMESPACE:KEY=value` (explicit addressing)
2. **Token Format**: `key=value` (cursor-based addressing)
3. **Control Tokens**: `ns=ui`, `ctx=user` (cursor modifications)

### **Impact Assessment**

- ❌ **Meteor parsing completely broken**
- ❌ **Explicit addressing not working**
- ❌ **CLI parse command unusable for meteors**
- ❌ **All meteor format tests failing**
- ❌ **Documentation examples broken**

### **Required Fixes**

#### **Priority 1: Restore Meteor Parsing**
1. **Revert `process_single_meteor()`** to use `Meteor::from_str()`
2. **Fix explicit addressing** to use `engine.set()` with full path
3. **Separate meteor parsing from token parsing**

#### **Priority 2: Architectural Clarity**
1. **MeteorStreamParser** should only handle explicit meteor format
2. **TokenStreamParser** should handle token streams with folding
3. **Clear separation** between the two parsing strategies

#### **Priority 3: Testing**
1. **Add explicit meteor format tests**
2. **Test complex namespace hierarchies**
3. **Test bracket notation in meteors**
4. **Test multiple meteors with `:;:` delimiter**

### **Proposed Solution Architecture**

```rust
impl MeteorStreamParser {
    fn process_single_meteor(engine: &mut MeteorEngine, meteor_str: &str) -> Result<(), String> {
        // Check if this is explicit meteor format (contains colons)
        if Self::is_explicit_meteor_format(meteor_str) {
            // Parse as complete meteor: CONTEXT:NAMESPACE:KEY=value
            let meteor = Meteor::from_str(meteor_str)?;
            let path = format!("{}:{}:{}",
                meteor.context(), meteor.namespace(), meteor.key());
            engine.set(&path, meteor.value())?;
        } else {
            // Handle token stream within meteor (semicolon-separated)
            Self::process_token_stream_within_meteor(engine, meteor_str)?;
        }
        Ok(())
    }
}
```

### **Testing Strategy**

```bash
# Test explicit meteor format
meteor parse "global:ui.widgets:button[]=click"
meteor parse "app:main:theme=dark"

# Test multiple meteors
meteor parse "app:ui:button=click :;: user:settings:theme=dark"

# Test token streams within meteors
meteor parse "button=click; ns=ui; theme=dark"
```

### **Immediate Action Required**

1. **STOP** using the broken parser in production
2. **REVERT** the process_single_meteor changes
3. **RESTORE** explicit meteor parsing functionality
4. **TEST** all meteor format examples before deployment

---

## **Status: CRITICAL REGRESSION - IMMEDIATE FIX REQUIRED**

The CLI is currently **completely broken** for meteor parsing. This is a P0 blocking issue that prevents basic meteor functionality from working.
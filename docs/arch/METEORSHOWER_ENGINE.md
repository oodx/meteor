# MeteorShower: Stateful Data Manipulation Engine

## Overview

**MAJOR ARCHITECTURE CHANGE**: MeteorShower has evolved from a simple data container to a stateful data manipulation engine with stream processing, command history, and cursor state management.

## Core Paradigm Shift

### **Before: Static Container**
```rust
// Old paradigm: Parse once, query static data
let shower = MeteorShower::parse("app:ui:button=click")?;
let meteor = shower.find("app", "ui", "button");
```

### **After: Stateful Engine**
```rust
// New paradigm: Persistent engine with streaming commands
let mut shower = MeteorShower::new();
shower.process_stream("button=click;ns=ui;theme=dark")?;
shower.process_stream("ctl:delete=app.main.button")?;
shower.process_stream("ctx=user;profile=admin")?;
```

## Architecture Components

### **1. Stateful Storage Engine**
```rust
pub struct MeteorShower {
    /// Primary data storage: context → namespace → key → value
    storage: StorageData,

    /// Stream processing cursor state (persistent across calls)
    current_context: Context,      // Default: "app"
    current_namespace: Namespace,  // Default: "main"

    /// Command execution history (audit trail)
    command_history: Vec<ControlCommand>,
}

#[derive(Debug, Clone)]
pub struct ControlCommand {
    pub timestamp: u64,
    pub command_type: String,   // "delete", "reset"
    pub target: String,         // "app.ui.button", "cursor"
    pub success: bool,
    pub error_message: Option<String>,
}
```

### **2. Stream Processing with Cursor State**

**Stream processing maintains state across multiple parse operations:**

```rust
impl MeteorShower {
    /// Process token stream - modifies internal state
    pub fn process_token_stream(&mut self, input: &str) -> Result<(), String> {
        // Parse input using current cursor context/namespace
        // Apply folding logic with ns=, ctx= control tokens
        // Execute ctl: commands
        // Update cursor state for next stream
    }

    /// Process meteor stream - explicit addressing only
    pub fn process_meteor_stream(&mut self, input: &str) -> Result<(), String> {
        // Parse explicit meteors: app:ui:button=click :;: user:main:profile=admin
        // No cursor state changes
        // Still supports ctl: commands for data manipulation
    }
}
```

### **3. Control Token Command System**

**Control tokens manipulate stored data and cursor state:**

#### **Data Manipulation Commands:**
```rust
ctl:delete=app.ui.button        // Delete specific key
ctl:delete=app.ui              // Delete entire namespace
ctl:delete=app                 // Delete entire context
```

#### **Cursor State Commands:**
```rust
ctl:reset=cursor               // Reset cursor to defaults (app:main)
ctl:reset=storage              // Clear all stored data
ctl:reset=all                  // Reset cursor + clear storage
```

#### **Command Execution Flow:**
```rust
pub fn execute_control_command(&mut self, command: &str, target: &str) -> Result<(), String> {
    let cmd_record = ControlCommand {
        timestamp: current_timestamp(),
        command_type: command.to_string(),
        target: target.to_string(),
        success: false,
        error_message: None,
    };

    let result = match command {
        "delete" => self.delete_path(target),
        "reset" => self.reset_target(target),
        _ => Err(format!("Unknown control command: {}", command)),
    };

    // Record command execution in history
    cmd_record.success = result.is_ok();
    cmd_record.error_message = result.as_ref().err().map(|e| e.to_string());
    self.command_history.push(cmd_record);

    result
}
```

## Stream Processing Examples

### **Token Stream Processing (with folding):**
```
Input Stream 1: "button=click;ns=ui;theme=dark"
- Cursor starts: app:main
- button=click → stored as app:main:button=click
- ns=ui → cursor changes to app:ui
- theme=dark → stored as app:ui:theme=dark
- Cursor ends: app:ui

Input Stream 2: "size=large;ctx=user;profile=admin"
- Cursor starts: app:ui (from previous stream!)
- size=large → stored as app:ui:size=large
- ctx=user → cursor changes to user:ui
- profile=admin → stored as user:ui:profile=admin
- Cursor ends: user:ui
```

### **Control Command Processing:**
```
Input Stream 3: "ctl:delete=app.ui.theme;ctl:reset=cursor;name=John"
- ctl:delete=app.ui.theme → removes theme from app:ui namespace
- ctl:reset=cursor → cursor resets to app:main
- name=John → stored as app:main:name=John
- Cursor ends: app:main
```

## Storage Manipulation API

### **Convenience Macros:**
```rust
impl MeteorShower {
    /// Set value at dot-notation path
    pub fn set(&mut self, path: &str, value: &str) -> Result<(), String> {
        let (context, namespace, key) = parse_dot_path(path)?;
        self.storage.set(&context, &namespace, &key, value);
        Ok(())
    }

    /// Get value at dot-notation path
    pub fn get(&self, path: &str) -> Option<&str> {
        let (context, namespace, key) = parse_dot_path(path).ok()?;
        self.storage.get(&context, &namespace, &key)
    }

    /// Delete item at dot-notation path
    pub fn delete(&mut self, path: &str) -> Result<bool, String> {
        let parts = parse_dot_path(path)?;
        match parts {
            (ctx, ns, key) if !key.is_empty() => {
                // Delete specific key
                Ok(self.storage.delete_key(&ctx, &ns, &key))
            }
            (ctx, ns, _) if !ns.is_empty() => {
                // Delete entire namespace
                Ok(self.storage.delete_namespace(&ctx, &ns))
            }
            (ctx, _, _) => {
                // Delete entire context
                Ok(self.storage.delete_context(&ctx))
            }
        }
    }

    /// Find paths matching pattern
    pub fn find(&self, pattern: &str) -> Vec<String> {
        // Pattern matching against stored paths
        // e.g., find("app.*.button") → ["app.ui.button", "app.menu.button"]
    }

    /// Check if path exists
    pub fn exists(&self, path: &str) -> bool {
        self.get(path).is_some()
    }
}
```

### **Command History Access:**
```rust
impl MeteorShower {
    /// Get complete command history
    pub fn command_history(&self) -> &[ControlCommand] {
        &self.command_history
    }

    /// Get last executed command
    pub fn last_command(&self) -> Option<&ControlCommand> {
        self.command_history.last()
    }

    /// Get failed commands
    pub fn failed_commands(&self) -> Vec<&ControlCommand> {
        self.command_history.iter().filter(|cmd| !cmd.success).collect()
    }
}
```

## Parser Module Integration

### **Parser Delegation Pattern:**
```rust
// Parser module provides portable logic
// src/lib/parser/token_stream.rs
pub fn parse_into_shower(shower: &mut MeteorShower, input: &str) -> Result<(), String> {
    let tokens = parse_raw_tokens_with_escapes(input)?;

    for token in tokens {
        if token.key.starts_with("ctl:") {
            let command = &token.key[4..];  // Remove "ctl:" prefix
            shower.execute_control_command(command, &token.value)?;
        } else if token.key == "ns" {
            shower.current_namespace = Namespace::from_string(&token.value);
        } else if token.key == "ctx" {
            shower.current_context = Context::from_str(&token.value)?;
        } else {
            // Store token using current cursor state
            shower.storage.set(
                shower.current_context.name(),
                &shower.current_namespace.to_string(),
                &token.key,
                &token.value,
            );
        }
    }
    Ok(())
}
```

### **Type Delegation to Parser:**
```rust
// Types delegate to parser module
impl MeteorShower {
    pub fn process_token_stream(&mut self, input: &str) -> Result<(), String> {
        parser::token_stream::parse_into_shower(self, input)
    }

    pub fn process_meteor_stream(&mut self, input: &str) -> Result<(), String> {
        parser::meteor_stream::parse_into_shower(self, input)
    }
}

// FromStr still works for simple cases
impl FromStr for MeteorShower {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut shower = MeteorShower::new();
        shower.process_meteor_stream(s)?;  // Default to explicit parsing
        Ok(shower)
    }
}
```

## Usage Patterns

### **Streaming Data Processor:**
```rust
let mut shower = MeteorShower::new();

// Process configuration stream
shower.process_token_stream("host=localhost;port=8080;ns=db;user=admin;pass=secret")?;

// Process user data stream
shower.process_token_stream("ctx=user;name=John;email=john@example.com")?;

// Clean up sensitive data
shower.process_token_stream("ctl:delete=app.db.pass")?;

// Query final state
let host = shower.get("app.main.host");  // Some("localhost")
let user_name = shower.get("user.main.name");  // Some("John")
let password = shower.get("app.db.pass");  // None (deleted)
```

### **Audit Trail:**
```rust
// Check what commands were executed
for cmd in shower.command_history() {
    println!("Command: {} {} - {}", cmd.command_type, cmd.target,
             if cmd.success { "SUCCESS" } else { "FAILED" });
}
```

## Key Architectural Principles

1. **Stateful Processing**: MeteorShower maintains cursor state across stream operations
2. **Command Audit Trail**: All control commands are logged with success/failure
3. **Data Manipulation**: Control tokens can modify stored data, not just cursor state
4. **Parser Delegation**: Parsing logic is portable and reusable across types
5. **Dot-Notation Paths**: Uniform addressing scheme for data manipulation
6. **Stream Continuity**: Token streams build on previous cursor state
7. **Command History**: Full audit trail of data modifications

## Breaking Changes from Previous Architecture

1. **MeteorShower is now mutable** - requires `&mut` for processing
2. **Stateful cursor** - stream processing affects future streams
3. **Control commands** - new `ctl:` syntax for data manipulation
4. **Command history** - automatic audit trail
5. **Parser delegation** - parsing logic moved to parser module
6. **Dot-notation API** - new path-based access methods

This represents a fundamental shift from static data container to dynamic data manipulation engine with full audit capabilities.
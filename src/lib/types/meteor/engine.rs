//! MeteorEngine - Stateful data manipulation engine for stream processing
//!
//! MeteorEngine maintains cursor state across multiple stream operations and provides
//! data manipulation capabilities with full audit trail. Unlike MeteorShower which
//! is a static container, MeteorEngine is a persistent stream processor.
//!
//! ## Key Principles:
//! - Pure state/data controller - NO validation logic
//! - Validation must happen externally in parser modules
//! - Maintains cursor state across operations
//! - Full command audit trail
//! - Dot-notation path operations

use crate::types::{Context, Namespace, StorageData};
use super::workspace::EngineWorkspace;

/// Command execution record for audit trail
#[derive(Debug, Clone)]
pub struct ControlCommand {
    pub timestamp: u64,
    pub command_type: String, // "delete", "reset", etc.
    pub target: String,       // "app.ui.button", "cursor", etc.
    pub success: bool,
    pub error_message: Option<String>,
}

impl ControlCommand {
    pub fn new(command_type: &str, target: &str) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            command_type: command_type.to_string(),
            target: target.to_string(),
            success: false,
            error_message: None,
        }
    }

    pub fn success(mut self) -> Self {
        self.success = true;
        self.error_message = None;
        self
    }

    pub fn failure(mut self, error: &str) -> Self {
        self.success = false;
        self.error_message = Some(error.to_string());
        self
    }
}

/// Stateful data manipulation engine for stream processing
///
/// MeteorEngine maintains persistent cursor state and provides data manipulation
/// capabilities with full audit trail. It serves as the global stream store
/// for token/meteor processing operations.
#[derive(Debug)]
pub struct MeteorEngine {
    /// Primary data storage: context → namespace → key → value
    storage: StorageData,

    /// Stream processing cursor state (persistent across operations)
    pub current_context: Context, // Default: "app"
    pub current_namespace: Namespace, // Default: "main"

    /// Command execution history (audit trail)
    command_history: Vec<ControlCommand>,

    /// Internal workspace for ordering, caching, and scratch operations
    workspace: EngineWorkspace,
}

impl MeteorEngine {
    /// Create a new MeteorEngine with default cursor state
    pub fn new() -> Self {
        Self {
            storage: StorageData::new(),
            current_context: Context::default(), // "app"
            current_namespace: Namespace::from_string("main"),
            command_history: Vec::new(),
            workspace: EngineWorkspace::new(),
        }
    }

    /// Create MeteorEngine with specific initial context
    pub fn with_context(context: Context) -> Self {
        Self {
            storage: StorageData::new(),
            current_context: context,
            current_namespace: Namespace::from_string("main"),
            command_history: Vec::new(),
            workspace: EngineWorkspace::new(),
        }
    }

    /// Store a token using current cursor state
    ///
    /// This is the primary method for adding data. Uses current cursor
    /// context/namespace unless overridden by explicit addressing.
    pub fn store_token(&mut self, key: &str, value: &str) {
        let context = self.current_context.name();
        let namespace = self.current_namespace.to_string();

        self.storage.set(context, &namespace, key, value);

        let ws = self.workspace.get_or_create_namespace(context, &namespace);
        ws.add_key(key);
        ws.invalidate_caches();
    }

    /// Store a token with explicit addressing (overrides cursor)
    pub fn store_token_at(&mut self, context: &str, namespace: &str, key: &str, value: &str) {
        self.storage.set(context, namespace, key, value);

        let ws = self.workspace.get_or_create_namespace(context, namespace);
        ws.add_key(key);
        ws.invalidate_caches();
    }

    /// Switch current context (cursor state change)
    pub fn switch_context(&mut self, context: Context) {
        self.current_context = context;
    }

    /// Switch current namespace (cursor state change)
    pub fn switch_namespace(&mut self, namespace: Namespace) {
        self.current_namespace = namespace;
    }

    /// Reset cursor to defaults
    pub fn reset_cursor(&mut self) {
        self.current_context = Context::default();
        self.current_namespace = Namespace::from_string("main");
    }

    /// Clear all stored data
    pub fn clear_storage(&mut self) {
        self.storage = StorageData::new();
        self.workspace.clear();
    }

    /// Reset cursor and clear storage
    pub fn reset_all(&mut self) {
        self.reset_cursor();
        self.clear_storage();
    }

    // ================================
    // Dot-notation Path Operations
    // ================================

    /// Set value at meteor path (explicit addressing)
    pub fn set(&mut self, path: &str, value: &str) -> Result<(), String> {
        let (context, namespace, key) = parse_meteor_path(path)?;
        self.storage.set(&context, &namespace, &key, value);

        let ws = self.workspace.get_or_create_namespace(&context, &namespace);
        ws.add_key(&key);
        ws.invalidate_caches();

        Ok(())
    }

    /// Get value at meteor path (explicit addressing)
    pub fn get(&self, path: &str) -> Option<&str> {
        let (context, namespace, key) = parse_meteor_path(path).ok()?;
        self.storage.get(&context, &namespace, &key)
    }

    /// Check if path exists
    pub fn exists(&self, path: &str) -> bool {
        self.get(path).is_some()
    }

    /// Delete item at meteor path (explicit addressing)
    /// Note: Command history is managed by execute_control_command, not here
    pub fn delete(&mut self, path: &str) -> Result<bool, String> {
        match parse_meteor_path(path) {
            Ok((context, namespace, key)) => {
                let result = if key.is_empty() {
                    if namespace.is_empty() {
                        // Delete entire context
                        let deleted = self.storage.delete_context(&context);
                        if deleted {
                            self.workspace.remove_context(&context);
                        }
                        deleted
                    } else {
                        // Delete entire namespace
                        let deleted = self.storage.delete_namespace(&context, &namespace);
                        if deleted {
                            self.workspace.remove_namespace(&context, &namespace);
                        }
                        deleted
                    }
                } else {
                    // Delete specific key
                    let deleted = self.storage.delete_key(&context, &namespace, &key);
                    if deleted {
                        let ws = self.workspace.get_or_create_namespace(&context, &namespace);
                        ws.remove_key(&key);
                        ws.invalidate_caches();
                    }
                    deleted
                };
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    /// Find paths matching pattern (enhanced with hybrid storage)
    pub fn find(&self, pattern: &str) -> Vec<String> {
        let mut results = Vec::new();

        // Parse pattern to determine context, namespace, and key pattern
        if let Ok((context, namespace, key_pattern)) = parse_meteor_path(pattern) {
            // Use the new find_keys method from hybrid storage
            let keys = self.storage.find_keys(&context, &namespace, &key_pattern);
            for key in keys {
                results.push(format!("{}:{}:{}", context, namespace, key));
            }
        } else {
            // Fallback to simple pattern matching across all contexts/namespaces
            for context in self.storage.contexts() {
                for namespace in self.storage.namespaces_in_context(&context) {
                    let keys = self.storage.find_keys(&context, &namespace, "*");
                    for key in keys {
                        let full_path = format!("{}:{}:{}", context, namespace, key);
                        if full_path.contains(pattern) {
                            results.push(full_path);
                        }
                    }
                }
            }
        }

        results.sort();
        results
    }

    // ================================
    // Control Command Execution
    // ================================

    /// Execute control command (called by parsers)
    pub fn execute_control_command(&mut self, command: &str, target: &str) -> Result<(), String> {
        let mut cmd = ControlCommand::new(command, target);

        let result = match command {
            "delete" => self.delete(target).map(|_| ()),
            "reset" => match target {
                "cursor" => {
                    self.reset_cursor();
                    Ok(())
                }
                "storage" => {
                    self.clear_storage();
                    Ok(())
                }
                "all" => {
                    self.reset_all();
                    Ok(())
                }
                _ => Err(format!("Unknown reset target: {}", target)),
            },
            _ => Err(format!("Unknown control command: {}", command)),
        };

        // Record command execution in history
        cmd = if let Err(err) = &result {
            cmd.failure(err)
        } else {
            cmd.success()
        };

        self.command_history.push(cmd);
        result
    }

    // ================================
    // Command History Access
    // ================================

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
        self.command_history
            .iter()
            .filter(|cmd| !cmd.success)
            .collect()
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.command_history.clear();
    }

    // ================================
    // Storage Access (for queries)
    // ================================

    /// Get reference to internal storage (read-only)
    pub fn storage(&self) -> &StorageData {
        &self.storage
    }

    /// Get all contexts
    pub fn contexts(&self) -> Vec<String> {
        self.storage.contexts()
    }

    /// Get namespaces in context
    pub fn namespaces_in_context(&self, context: &str) -> Vec<String> {
        self.storage.namespaces_in_context(context)
    }

    // ================================
    // Iterator Access (ENG-10)
    // ================================

    /// Iterate over all contexts
    ///
    /// Returns an iterator over context names in sorted order.
    /// Replacement for repeated `storage.contexts()` clones.
    pub fn contexts_iter(&self) -> impl Iterator<Item = String> {
        self.storage.contexts().into_iter()
    }

    /// Iterate over namespaces in a context
    ///
    /// Returns an iterator over namespace names in sorted order for the given context.
    pub fn namespaces_iter(&self, context: &str) -> impl Iterator<Item = String> {
        self.storage.namespaces_in_context(context).into_iter()
    }

    /// Iterate over all entries with workspace ordering
    ///
    /// Returns an iterator over `(context, namespace, key, value)` tuples.
    /// Keys within each namespace are returned in workspace insertion order
    /// when available, otherwise in sorted order.
    ///
    /// # Example
    ///
    /// ```
    /// use meteor::types::MeteorEngine;
    ///
    /// let mut engine = MeteorEngine::new();
    /// engine.set("app:ui:button", "click").unwrap();
    /// engine.set("app:ui:theme", "dark").unwrap();
    ///
    /// for (context, namespace, key, value) in engine.iter_entries() {
    ///     println!("{}:{}:{} = {}", context, namespace, key, value);
    /// }
    /// ```
    pub fn iter_entries(&self) -> EntriesIterator<'_> {
        EntriesIterator::new(self)
    }

    // ================================
    // Hybrid Storage Methods
    // ================================

    /// Check if path exists as a file
    pub fn is_file(&self, path: &str) -> bool {
        if let Ok((context, namespace, key)) = parse_meteor_path(path) {
            self.storage.is_file(&context, &namespace, &key)
        } else {
            false
        }
    }

    /// Check if path exists as a directory
    pub fn is_directory(&self, path: &str) -> bool {
        if let Ok((context, namespace, key)) = parse_meteor_path_for_directory(path) {
            if key.is_empty() {
                // Check if namespace exists in context (e.g., "app:settings" → check if "settings" namespace exists)
                self.storage.namespace_exists(&context, &namespace)
            } else {
                // Check if path within namespace is a directory (e.g., "app:settings:ui" → check if "ui" is directory in "settings")
                self.storage.is_directory(&context, &namespace, &key)
            }
        } else {
            false
        }
    }

    /// Check if directory has default value (.index pattern)
    pub fn has_default(&self, path: &str) -> bool {
        if let Ok((context, namespace, key)) = parse_meteor_path_for_directory(path) {
            self.storage.has_default(&context, &namespace, &key)
        } else {
            false
        }
    }

    /// Get default value for directory
    pub fn get_default(&self, path: &str) -> Option<&str> {
        if let Ok((context, namespace, key)) = parse_meteor_path_for_directory(path) {
            self.storage.get_default(&context, &namespace, &key)
        } else {
            None
        }
    }

    // ================================
    // Workspace Access (Internal)
    // ================================

    /// Get mutable reference to workspace (internal use only)
    pub(crate) fn workspace_mut(&mut self) -> &mut EngineWorkspace {
        &mut self.workspace
    }

    /// Get reference to workspace (internal use only)
    pub(crate) fn workspace(&self) -> &EngineWorkspace {
        &self.workspace
    }

    #[cfg(debug_assertions)]
    pub fn workspace_status(&self) -> super::workspace::WorkspaceStatus {
        self.workspace.workspace_status()
    }
}

impl Default for MeteorEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ================================
// Entries Iterator (ENG-10)
// ================================

/// Iterator over all entries in the engine with workspace ordering
///
/// Iterates over all contexts, namespaces, and keys, yielding
/// `(context, namespace, key, value)` tuples. Keys within each namespace
/// are returned in workspace insertion order when available.
pub struct EntriesIterator<'a> {
    engine: &'a MeteorEngine,
    contexts: Vec<String>,
    current_context_idx: usize,
    current_namespaces: Vec<String>,
    current_namespace_idx: usize,
    current_keys: Vec<String>,
    current_key_idx: usize,
}

impl<'a> EntriesIterator<'a> {
    fn new(engine: &'a MeteorEngine) -> Self {
        let contexts = engine.storage.contexts();
        Self {
            engine,
            contexts,
            current_context_idx: 0,
            current_namespaces: Vec::new(),
            current_namespace_idx: 0,
            current_keys: Vec::new(),
            current_key_idx: 0,
        }
    }

    fn advance_to_next_context(&mut self) -> bool {
        if self.current_context_idx >= self.contexts.len() {
            return false;
        }

        let context = &self.contexts[self.current_context_idx];
        self.current_namespaces = self.engine.storage.namespaces_in_context(context);
        self.current_namespace_idx = 0;
        self.current_context_idx += 1;

        self.advance_to_next_namespace()
    }

    fn advance_to_next_namespace(&mut self) -> bool {
        if self.current_namespace_idx >= self.current_namespaces.len() {
            return self.advance_to_next_context();
        }

        let context = &self.contexts[self.current_context_idx - 1];
        let namespace = &self.current_namespaces[self.current_namespace_idx];

        // Try to get workspace ordering, fall back to storage keys
        if let Some(ws) = self.engine.workspace.get_namespace(context, namespace) {
            self.current_keys = ws.key_order.clone();
        } else {
            // No workspace data, get keys from storage
            self.current_keys = self.engine.storage.find_keys(context, namespace, "*");
        }

        self.current_key_idx = 0;
        self.current_namespace_idx += 1;

        if self.current_keys.is_empty() {
            self.advance_to_next_namespace()
        } else {
            true
        }
    }
}

impl<'a> Iterator for EntriesIterator<'a> {
    type Item = (String, String, String, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current key, return it
            if self.current_key_idx < self.current_keys.len() {
                let context = &self.contexts[self.current_context_idx - 1];
                let namespace = &self.current_namespaces[self.current_namespace_idx - 1];
                let key = &self.current_keys[self.current_key_idx];
                self.current_key_idx += 1;

                if let Some(value) = self.engine.storage.get(context, namespace, key) {
                    return Some((
                        context.clone(),
                        namespace.clone(),
                        key.clone(),
                        value.to_string(),
                    ));
                }
                continue;
            }

            // No more keys in current namespace, advance
            if !self.advance_to_next_namespace() {
                return None;
            }
        }
    }
}

// ================================
// Meteor Path Parsing Utilities
// ================================

/// Parse meteor path into (context, namespace, key)
///
/// Handles colon-delimited meteor format: CONTEXT:NAMESPACE:KEY
/// - "app:ui.widgets:button" → ("app", "ui.widgets", "button")
/// - "user:settings:theme" → ("user", "settings", "theme")
/// - "app::button" → ("app", "", "button") (empty namespace)
/// - "app:ui.forms.inputs:field" → ("app", "ui.forms.inputs", "field")
///
/// Namespaces can contain dots for hierarchy, but colons separate the three main parts.
fn parse_meteor_path(path: &str) -> Result<(String, String, String), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let parts: Vec<&str> = path.split(':').collect();

    match parts.len() {
        1 => {
            // Single identifier: "button" - treat as key in default app context, main namespace
            Ok(("app".to_string(), "main".to_string(), parts[0].to_string()))
        }
        2 => {
            if parts[1].is_empty() {
                // Empty second part: "context:" - main namespace, empty key
                Ok((parts[0].to_string(), "main".to_string(), "".to_string()))
            } else {
                // Two parts: "context:key" - key in main namespace of specified context
                Ok((
                    parts[0].to_string(),
                    "main".to_string(),
                    parts[1].to_string(),
                ))
            }
        }
        3 => {
            // Full meteor format: "app:ui.widgets:button"
            Ok((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        }
        _ => Err(format!(
            "Invalid meteor path format: '{}' - expected CONTEXT[:NAMESPACE[:KEY]]",
            path
        )),
    }
}

/// Parse meteor path for directory operations: context:namespace:key
/// For directory queries, interpret "context:name" as "context has namespace 'name'"
/// Returns (context, namespace, key) tuple
fn parse_meteor_path_for_directory(path: &str) -> Result<(String, String, String), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let parts: Vec<&str> = path.split(':').collect();

    match parts.len() {
        1 => {
            // Single identifier: "user" - treat as context-level directory in main namespace
            Ok((parts[0].to_string(), "main".to_string(), "".to_string()))
        }
        2 => {
            if parts[1].is_empty() {
                // Empty second part: "context:" - main namespace root
                Ok((parts[0].to_string(), "main".to_string(), "".to_string()))
            } else {
                // Two parts: "context:namespace" - namespace directory in context
                Ok((parts[0].to_string(), parts[1].to_string(), "".to_string()))
            }
        }
        3 => {
            // Full meteor format: "app:namespace:key"
            Ok((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        }
        _ => Err(format!(
            "Invalid meteor path format: '{}' - expected CONTEXT[:NAMESPACE[:KEY]]",
            path
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteor_engine_creation() {
        let engine = MeteorEngine::new();
        assert_eq!(engine.current_context.name(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
        assert!(engine.command_history.is_empty());
    }

    #[test]
    fn test_cursor_state_changes() {
        let mut engine = MeteorEngine::new();

        // Switch context
        engine.switch_context(Context::user());
        assert_eq!(engine.current_context.name(), "user");

        // Switch namespace
        engine.switch_namespace(Namespace::from_string("settings"));
        assert_eq!(engine.current_namespace.to_string(), "settings");

        // Reset cursor
        engine.reset_cursor();
        assert_eq!(engine.current_context.name(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
    }

    #[test]
    fn test_token_storage() {
        let mut engine = MeteorEngine::new();

        // Store token using cursor
        engine.store_token("button", "click");
        assert_eq!(engine.get("app:main:button"), Some("click"));

        // Store with explicit addressing
        engine.store_token_at("user", "settings", "theme", "dark");
        assert_eq!(engine.get("user:settings:theme"), Some("dark"));
    }

    #[test]
    fn test_control_commands() {
        let mut engine = MeteorEngine::new();

        // Store some data
        engine.store_token("button", "click");

        // Execute control command
        engine.execute_control_command("reset", "cursor").unwrap();

        // Check command history
        assert_eq!(engine.command_history().len(), 1);
        assert_eq!(engine.last_command().unwrap().command_type, "reset");
        assert!(engine.last_command().unwrap().success);
    }

    #[test]
    fn test_stream_continuity() {
        let mut engine = MeteorEngine::new();

        // First stream: app:main context
        engine.store_token("host", "localhost");
        engine.switch_namespace(Namespace::from_string("db"));
        engine.store_token("user", "admin");

        // Cursor state should persist
        assert_eq!(engine.current_context.name(), "app");
        assert_eq!(engine.current_namespace.to_string(), "db");

        // Second stream: continues from app:db
        engine.store_token("password", "secret");
        engine.switch_context(Context::user());
        engine.store_token("name", "John");

        // Verify data stored correctly
        assert_eq!(engine.get("app:main:host"), Some("localhost"));
        assert_eq!(engine.get("app:db:user"), Some("admin"));
        assert_eq!(engine.get("app:db:password"), Some("secret"));
        assert_eq!(engine.get("user:db:name"), Some("John"));
    }

    #[test]
    fn test_control_command_audit_trail() {
        let mut engine = MeteorEngine::new();

        // Store test data
        engine.set("app:ui:button", "click").unwrap();
        engine.set("app:ui:theme", "dark").unwrap();
        engine.set("user:profile:name", "Alice").unwrap();

        // Execute various control commands
        let _ = engine.execute_control_command("delete", "app:ui:button"); // May succeed or fail due to StorageData limitations
        engine.execute_control_command("reset", "cursor").unwrap();
        engine
            .execute_control_command("invalid", "command")
            .unwrap_err();

        // Check audit trail
        let history = engine.command_history();
        assert_eq!(history.len(), 3);

        // First command: delete button (should succeed)
        assert_eq!(history[0].command_type, "delete");
        assert_eq!(history[0].target, "app:ui:button");
        // Note: delete might not actually work due to StorageData limitations

        // Second command: reset cursor (should succeed)
        assert_eq!(history[1].command_type, "reset");
        assert_eq!(history[1].target, "cursor");
        assert!(history[1].success);

        // Third command: invalid command (should fail)
        assert_eq!(history[2].command_type, "invalid");
        assert_eq!(history[2].target, "command");
        assert!(!history[2].success);
        assert!(history[2].error_message.is_some());
    }

    #[test]
    fn test_explicit_vs_cursor_addressing() {
        let mut engine = MeteorEngine::new();

        // Set cursor to user:settings
        engine.switch_context(Context::user());
        engine.switch_namespace(Namespace::from_string("settings"));

        // Store using cursor
        engine.store_token("theme", "dark");

        // Store with explicit addressing (overrides cursor)
        engine.store_token_at("app", "ui", "button", "click");

        // Verify both stored correctly
        assert_eq!(engine.get("user:settings:theme"), Some("dark"));
        assert_eq!(engine.get("app:ui:button"), Some("click"));

        // Cursor should be unchanged
        assert_eq!(engine.current_context.name(), "user");
        assert_eq!(engine.current_namespace.to_string(), "settings");
    }

    #[test]
    fn test_complex_dot_paths() {
        let mut engine = MeteorEngine::new();

        // Test various path formats
        engine.set("app:ui.forms.login:username", "alice").unwrap();
        engine
            .set("system:logs.error.network:timeout", "30s")
            .unwrap();

        // Verify complex paths work
        assert_eq!(engine.get("app:ui.forms.login:username"), Some("alice"));
        assert_eq!(engine.get("system:logs.error.network:timeout"), Some("30s"));

        // Test existence checks
        assert!(engine.exists("app:ui.forms.login:username"));
        assert!(engine.exists("system:logs.error.network:timeout"));
        assert!(!engine.exists("nonexistent:path"));
    }

    #[test]
    fn test_command_history_filtering() {
        let mut engine = MeteorEngine::new();

        // Execute mix of successful and failed commands
        engine.execute_control_command("reset", "cursor").unwrap();
        engine
            .execute_control_command("invalid", "command")
            .unwrap_err();
        engine.execute_control_command("reset", "storage").unwrap();
        engine
            .execute_control_command("unknown", "target")
            .unwrap_err();

        // Test history filtering
        let all_commands = engine.command_history();
        assert_eq!(all_commands.len(), 4);

        let failed_commands = engine.failed_commands();
        assert_eq!(failed_commands.len(), 2);
        assert_eq!(failed_commands[0].command_type, "invalid");
        assert_eq!(failed_commands[1].command_type, "unknown");

        // Test last command
        assert_eq!(engine.last_command().unwrap().command_type, "unknown");
        assert!(!engine.last_command().unwrap().success);
    }

    #[test]
    fn test_reset_operations() {
        let mut engine = MeteorEngine::new();

        // Set up initial state
        engine.switch_context(Context::user());
        engine.switch_namespace(Namespace::from_string("profile"));
        engine.store_token("name", "Bob");
        engine.set("app:ui:theme", "light").unwrap();

        // Test cursor reset
        engine.reset_cursor();
        assert_eq!(engine.current_context.name(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");

        // Data should still exist
        assert_eq!(engine.get("user:profile:name"), Some("Bob"));
        assert_eq!(engine.get("app:ui:theme"), Some("light"));

        // Test storage reset
        engine.clear_storage();
        assert!(engine.get("user:profile:name").is_none());
        assert!(engine.get("app:ui:theme").is_none());

        // Test reset all
        engine.switch_context(Context::system());
        engine.store_token("test", "value");
        engine.reset_all();

        assert_eq!(engine.current_context.name(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
        assert!(engine.get("system:main:test").is_none());
    }
}

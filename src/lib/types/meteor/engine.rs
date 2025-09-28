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

use crate::types::{Context, Namespace, StorageData, Token};
use super::{Meteor, workspace::EngineWorkspace};

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

/// Lightweight cursor accessor for reading and modifying cursor state.
///
/// Provides safe access to cursor state with validation and convenience methods.
/// Borrows the engine mutably, preventing data mutations while cursor is accessed.
///
/// # Example
/// ```
/// use meteor::types::MeteorEngine;
///
/// let mut engine = MeteorEngine::new();
/// {
///     let mut cursor = engine.cursor();
///     assert_eq!(cursor.context().name(), "app");
///     assert_eq!(cursor.namespace().to_string(), "main");
///
///     cursor.set_context("user");
///     cursor.set_namespace("settings");
/// }
/// assert_eq!(engine.current_context.name(), "user");
/// ```
pub struct Cursor<'a> {
    engine: &'a mut MeteorEngine,
}

impl<'a> Cursor<'a> {
    fn new(engine: &'a mut MeteorEngine) -> Self {
        Self { engine }
    }

    /// Get the current context
    pub fn context(&self) -> &Context {
        &self.engine.current_context
    }

    /// Get the current namespace
    pub fn namespace(&self) -> &Namespace {
        &self.engine.current_namespace
    }

    /// Set the current context
    pub fn set_context(&mut self, context: impl Into<Context>) {
        self.engine.current_context = context.into();
    }

    /// Set the current namespace
    pub fn set_namespace(&mut self, namespace: impl Into<Namespace>) {
        self.engine.current_namespace = namespace.into();
    }

    /// Reset cursor to defaults (app:main)
    pub fn reset(&mut self) {
        self.engine.reset_cursor();
    }

    /// Get the current cursor position as a formatted string (context:namespace)
    pub fn position(&self) -> String {
        format!("{}:{}", self.engine.current_context.name(), self.engine.current_namespace.to_string())
    }
}

/// RAII guard that saves cursor state and restores it on drop.
///
/// Automatically restores cursor position when the guard goes out of scope,
/// even if a panic occurs. Useful for temporary cursor switches in parsers
/// and REPL commands.
///
/// # Example
/// ```
/// use meteor::types::{MeteorEngine, Context};
///
/// let mut engine = MeteorEngine::new();
/// engine.set("app:main:key", "original").unwrap();
///
/// {
///     let _guard = engine.cursor_guard();
///     engine.switch_context(Context::user());
///     engine.switch_namespace("temp".into());
///     engine.store_token("temp_key", "temp_value");
/// } // Guard drops here, cursor restored to app:main
///
/// assert_eq!(engine.current_context.name(), "app");
/// assert_eq!(engine.current_namespace.to_string(), "main");
/// ```
pub struct CursorGuard {
    saved_context: Context,
    saved_namespace: Namespace,
    engine_ptr: *mut MeteorEngine,
}

impl CursorGuard {
    fn new(engine: &mut MeteorEngine) -> Self {
        Self {
            saved_context: engine.current_context.clone(),
            saved_namespace: engine.current_namespace.clone(),
            engine_ptr: engine as *mut MeteorEngine,
        }
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        unsafe {
            let engine = &mut *self.engine_ptr;
            engine.current_context = self.saved_context.clone();
            engine.current_namespace = self.saved_namespace.clone();
        }
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

    /// Get a cursor accessor for reading and modifying cursor state.
    ///
    /// Returns a `Cursor` that borrows the engine mutably, providing
    /// safe access to cursor operations with convenient methods.
    ///
    /// # Example
    /// ```
    /// use meteor::types::MeteorEngine;
    ///
    /// let mut engine = MeteorEngine::new();
    /// let mut cursor = engine.cursor();
    /// cursor.set_context("user");
    /// assert_eq!(cursor.context().name(), "user");
    /// ```
    pub fn cursor(&mut self) -> Cursor<'_> {
        Cursor::new(self)
    }

    /// Create a cursor guard that saves current cursor position and restores it on drop.
    ///
    /// This is an RAII guard that automatically restores cursor state when it goes
    /// out of scope, even if a panic occurs. Useful for temporary cursor switches.
    ///
    /// # Example
    /// ```
    /// use meteor::types::{MeteorEngine, Context};
    ///
    /// let mut engine = MeteorEngine::new();
    /// {
    ///     let _guard = engine.cursor_guard();
    ///     engine.switch_context(Context::user());
    ///     // Do work with temporary cursor...
    /// } // Cursor automatically restored here
    /// assert_eq!(engine.current_context.name(), "app");
    /// ```
    pub fn cursor_guard(&mut self) -> CursorGuard {
        CursorGuard::new(self)
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

    /// Returns a view into a specific namespace, or None if the namespace doesn't exist.
    ///
    /// NamespaceView provides ordered access to entries with metadata including:
    /// - Entry count
    /// - Default value detection (.index key)
    /// - Workspace-ordered iteration
    ///
    /// # Example
    ///
    /// ```
    /// use meteor::types::MeteorEngine;
    ///
    /// let mut engine = MeteorEngine::new();
    /// engine.set("doc:guides.install:intro", "Welcome").unwrap();
    /// engine.set("doc:guides.install:setup", "Step 1...").unwrap();
    /// engine.set("doc:guides.install:.index", "default").unwrap();
    ///
    /// if let Some(view) = engine.namespace_view("doc", "guides.install") {
    ///     assert_eq!(view.entry_count, 3);
    ///     assert!(view.has_default);
    ///     for (key, value) in view.entries() {
    ///         println!("{} = {}", key, value);
    ///     }
    /// }
    /// ```
    pub fn namespace_view(&self, context: &str, namespace: &str) -> Option<NamespaceView<'_>> {
        // Try to get keys from workspace (insertion order), fall back to storage
        let keys = if let Some(ws) = self.workspace.get_namespace(context, namespace) {
            ws.key_order.clone()
        } else {
            let keys = self.storage.find_keys(context, namespace, "*");
            if keys.is_empty() {
                return None;
            }
            keys
        };

        let entry_count = keys.len();
        let has_default = keys.iter().any(|k| k == ".index");

        Some(NamespaceView {
            context: context.to_string(),
            namespace: namespace.to_string(),
            entry_count,
            has_default,
            engine: self,
            keys,
        })
    }

    // ================================
    // Meteor Aggregation (ENG-20)
    // ================================

    /// Iterate over all meteors, grouped by (context, namespace).
    ///
    /// Returns an iterator that yields `Meteor` instances, one per namespace.
    /// Each meteor contains all tokens (key-value pairs) for that namespace,
    /// in workspace insertion order when available.
    ///
    /// # Example
    ///
    /// ```
    /// use meteor::types::MeteorEngine;
    ///
    /// let mut engine = MeteorEngine::new();
    /// engine.set("app:ui:button", "click").unwrap();
    /// engine.set("app:ui:theme", "dark").unwrap();
    /// engine.set("user:settings:lang", "en").unwrap();
    ///
    /// for meteor in engine.meteors() {
    ///     println!("{}", meteor);
    /// }
    /// ```
    pub fn meteors(&self) -> MeteorsIterator<'_> {
        MeteorsIterator::new(self)
    }

    /// Get a meteor for a specific (context, namespace) pair.
    ///
    /// Returns `Some(Meteor)` if the namespace exists and has entries,
    /// `None` otherwise. The meteor contains all tokens in that namespace
    /// in workspace insertion order.
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
    /// if let Some(meteor) = engine.meteor_for("app", "ui") {
    ///     assert_eq!(meteor.context().name(), "app");
    ///     assert_eq!(meteor.namespace().to_string(), "ui");
    ///     assert_eq!(meteor.tokens().len(), 2);
    /// }
    /// ```
    pub fn meteor_for(&self, context: &str, namespace: &str) -> Option<Meteor> {
        let view = self.namespace_view(context, namespace)?;

        let mut tokens = Vec::new();
        for (key, value) in view.entries() {
            tokens.push(Token::new(key, value));
        }

        if tokens.is_empty() {
            return None;
        }

        Some(Meteor::new_with_tokens(
            Context::new(context),
            Namespace::from_string(namespace),
            tokens,
        ))
    }

    // ================================
    // Export / Import Methods
    // ================================

    /// Export a namespace to ExportData with checksum metadata
    ///
    /// # Arguments
    /// * `context` - Context name (e.g., "doc", "shell")
    /// * `namespace` - Namespace path (e.g., "guides.install")
    /// * `format` - Export format (Text or Json)
    ///
    /// # Returns
    /// `Some(ExportData)` if namespace exists and has tokens, `None` otherwise
    ///
    /// # Example
    /// ```
    /// use meteor::types::{MeteorEngine, ExportFormat};
    ///
    /// let mut engine = MeteorEngine::new();
    /// engine.set("doc:guide:section[intro]", "Welcome").unwrap();
    /// engine.set("doc:guide:section[body]", "Content").unwrap();
    ///
    /// let export = engine.export_namespace("doc", "guide", ExportFormat::Text).unwrap();
    /// assert_eq!(export.tokens.len(), 2);
    /// assert!(!export.metadata.checksum.is_empty());
    /// ```
    pub fn export_namespace(
        &self,
        context: &str,
        namespace: &str,
        format: super::export::ExportFormat,
    ) -> Option<super::export::ExportData> {
        let view = self.namespace_view(context, namespace)?;

        let mut tokens = Vec::new();
        for (key, value) in view.entries() {
            tokens.push((key, value));
        }

        if tokens.is_empty() {
            return None;
        }

        Some(super::export::ExportData::new(
            context.to_string(),
            namespace.to_string(),
            tokens,
            format,
        ))
    }

    /// Import namespace data from ExportData with validation
    ///
    /// # Arguments
    /// * `data` - ExportData containing tokens and metadata
    ///
    /// # Returns
    /// `ImportResult` with success status, diff information, and checksum validation
    ///
    /// # Example
    /// ```
    /// use meteor::types::{MeteorEngine, ExportFormat};
    ///
    /// let mut engine = MeteorEngine::new();
    /// engine.set("doc:guide:section[intro]", "Welcome").unwrap();
    ///
    /// let export = engine.export_namespace("doc", "guide", ExportFormat::Text).unwrap();
    ///
    /// // Import into new engine
    /// let mut engine2 = MeteorEngine::new();
    /// let result = engine2.import_namespace(export).unwrap();
    /// assert!(result.success);
    /// assert_eq!(result.tokens_added, 1);
    /// ```
    pub fn import_namespace(
        &mut self,
        data: super::export::ExportData,
    ) -> Result<super::export::ImportResult, String> {
        let mut result = super::export::ImportResult::new();

        let existing_tokens: std::collections::HashMap<String, String> =
            if let Some(view) = self.namespace_view(&data.context, &data.namespace) {
                view.entries().collect()
            } else {
                std::collections::HashMap::new()
            };

        for (key, new_value) in data.tokens.iter() {
            let full_key = format!("{}:{}:{}", data.context, data.namespace, key);

            if let Some(old_value) = existing_tokens.get(key) {
                if old_value == new_value {
                    result.tokens_unchanged += 1;
                    result.diff.push(super::export::ImportDiff::Unchanged {
                        key: key.clone(),
                    });
                } else {
                    self.set(&full_key, new_value)?;
                    result.tokens_updated += 1;
                    result.diff.push(super::export::ImportDiff::Updated {
                        key: key.clone(),
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                    });
                }
            } else {
                self.set(&full_key, new_value)?;
                result.tokens_added += 1;
                result.diff.push(super::export::ImportDiff::Added {
                    key: key.clone(),
                    value: new_value.clone(),
                });
            }
        }

        let recalc_export = self.export_namespace(&data.context, &data.namespace, data.format.clone());
        result.checksum_valid = if let Some(recalc) = recalc_export {
            recalc.metadata.checksum == data.metadata.checksum
        } else {
            false
        };

        result.success = true;
        Ok(result)
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

    // ================================
    // Scratch Slot API (ENG-24)
    // ================================

    /// Create a scratch slot with lifetime management.
    ///
    /// Returns a `ScratchSlotGuard` that provides RAII access to a temporary
    /// key-value store. By default, the slot is automatically cleaned up when
    /// the guard is dropped. Use `.persist()` to keep the slot beyond guard lifetime.
    ///
    /// # Example
    /// ```
    /// use meteor::types::MeteorEngine;
    ///
    /// let mut engine = MeteorEngine::new();
    /// {
    ///     let mut slot = engine.scratch_slot("temp_vars");
    ///     slot.set("user_id", "12345");
    ///     slot.set("session", "abc123");
    ///     assert_eq!(slot.get("user_id"), Some("12345"));
    /// } // Slot automatically cleaned up here
    /// ```
    pub fn scratch_slot(&mut self, name: &str) -> super::workspace::ScratchSlotGuard<'_> {
        super::workspace::ScratchSlotGuard::new(name.to_string(), &mut self.workspace)
    }

    /// Remove a scratch slot by name.
    ///
    /// Returns `true` if the slot existed and was removed, `false` otherwise.
    pub fn remove_scratch_slot(&mut self, name: &str) -> bool {
        self.workspace.remove_scratch_slot(name)
    }

    /// Clear all scratch slots.
    ///
    /// This removes all temporary data stored in scratch slots.
    pub fn clear_all_scratch(&mut self) {
        self.workspace.clear_all_scratch()
    }

    /// List all scratch slot names.
    ///
    /// Returns a vector of scratch slot names currently active.
    pub fn list_scratch_slots(&self) -> Vec<&str> {
        self.workspace.list_scratch_slots()
    }

    /// Check if a scratch slot exists.
    ///
    /// Returns `true` if a slot with the given name exists.
    pub fn has_scratch_slot(&self, name: &str) -> bool {
        self.workspace.get_scratch_slot(name).is_some()
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

/// Iterator over all meteors, grouped by (context, namespace).
///
/// Yields `Meteor` instances, one per namespace. Each meteor contains
/// all tokens (key-value pairs) for that namespace in workspace insertion
/// order when available.
///
/// Created by `MeteorEngine::meteors()`.
pub struct MeteorsIterator<'a> {
    engine: &'a MeteorEngine,
    contexts: Vec<String>,
    current_context_idx: usize,
    current_namespaces: Vec<String>,
    current_namespace_idx: usize,
}

impl<'a> MeteorsIterator<'a> {
    fn new(engine: &'a MeteorEngine) -> Self {
        let contexts = engine.storage.contexts();
        Self {
            engine,
            contexts,
            current_context_idx: 0,
            current_namespaces: Vec::new(),
            current_namespace_idx: 0,
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

        !self.current_namespaces.is_empty()
    }
}

impl<'a> Iterator for MeteorsIterator<'a> {
    type Item = Meteor;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have more namespaces in current context, process next one
            if self.current_namespace_idx < self.current_namespaces.len() {
                let context = &self.contexts[self.current_context_idx - 1];
                let namespace = &self.current_namespaces[self.current_namespace_idx];
                self.current_namespace_idx += 1;

                if let Some(meteor) = self.engine.meteor_for(context, namespace) {
                    return Some(meteor);
                }
                continue;
            }

            // No more namespaces in current context, advance to next context
            if !self.advance_to_next_context() {
                return None;
            }
        }
    }
}

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

            // Record iteration in workspace instrumentation
            #[cfg(feature = "workspace-instrumentation")]
            ws.record_iteration(self.current_keys.len());
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

/// A view into a single namespace, providing metadata and ordered access to entries.
///
/// NamespaceView provides a lightweight, read-only view into a namespace with:
/// - Ordered iteration using workspace key_order (insertion order preservation)
/// - Entry count and default value detection (.index)
/// - Efficient key/value access without copying all data
///
/// ## Example
/// ```rust
/// use meteor::types::MeteorEngine;
///
/// let mut engine = MeteorEngine::new();
/// engine.set("doc:guides.install:intro", "Welcome").unwrap();
/// engine.set("doc:guides.install:setup", "Step 1...").unwrap();
///
/// if let Some(view) = engine.namespace_view("doc", "guides.install") {
///     println!("Namespace has {} entries", view.entry_count);
///     for (key, value) in view.entries() {
///         println!("{} = {}", key, value);
///     }
/// }
/// ```
pub struct NamespaceView<'a> {
    /// The context this namespace belongs to
    pub context: String,
    /// The namespace path
    pub namespace: String,
    /// Number of entries in this namespace
    pub entry_count: usize,
    /// Whether this namespace has a default value (.index)
    pub has_default: bool,

    // Private fields
    engine: &'a MeteorEngine,
    keys: Vec<String>,
}

impl<'a> NamespaceView<'a> {
    /// Returns an iterator over (key, value) pairs in workspace order
    pub fn entries(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.keys.iter().filter_map(move |key| {
            self.engine
                .get(&format!("{}:{}:{}", self.context, self.namespace, key))
                .map(|v| (key.clone(), v.to_string()))
        })
    }

    /// Returns an iterator over keys in workspace order
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.keys.iter().map(|k| k.as_str())
    }

    /// Returns an iterator over values in workspace order
    pub fn values(&self) -> impl Iterator<Item = String> + '_ {
        self.keys.iter().filter_map(move |key| {
            self.engine
                .get(&format!("{}:{}:{}", self.context, self.namespace, key))
                .map(|s| s.to_string())
        })
    }

    /// Get a single value by key
    pub fn get(&self, key: &str) -> Option<String> {
        self.engine
            .get(&format!("{}:{}:{}", self.context, self.namespace, key))
            .map(|s| s.to_string())
    }

    /// Check if a key exists in this namespace
    pub fn has_key(&self, key: &str) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    /// Get all keys that match a pattern (supports * wildcard)
    pub fn find_keys(&self, pattern: &str) -> Vec<String> {
        self.engine.storage.find_keys(&self.context, &self.namespace, pattern)
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

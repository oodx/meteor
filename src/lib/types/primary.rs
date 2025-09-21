//! Primary type definitions - the foundation of Meteor's type system
//!
//! These types form the core addressing scheme for token data:
//! - Context: Isolation boundaries (app, user, system, file1, remote1)
//! - Namespace: Hierarchical organization within contexts
//! - Key: Individual data identifiers

use std::fmt;
use std::str::FromStr;

/// Context provides isolation boundaries for token data
///
/// Contexts prevent cross-contamination between different data sources.
/// Standard contexts follow a privilege hierarchy:
/// - `system` - Highest privilege system configuration
/// - `user` - User-specific configuration
/// - `app` - Application configuration (default)
/// - `file1`, `file2`, etc. - File-specific contexts
/// - `remote1`, `remote2`, etc. - Remote source contexts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    name: String,
}

impl Context {
    /// Create a new context with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Context {
            name: name.into(),
        }
    }

    /// Create an app context (default)
    pub fn app() -> Self {
        Context::new("app")
    }

    /// Create a user context
    pub fn user() -> Self {
        Context::new("user")
    }

    /// Create a system context
    pub fn system() -> Self {
        Context::new("system")
    }

    /// Get the context name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this is a privileged context (system or user)
    pub fn is_privileged(&self) -> bool {
        self.name == "system" || self.name == "user"
    }
}

impl FromStr for Context {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Context name cannot be empty".into());
        }
        Ok(Context::new(s))
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::app()
    }
}

/// Hierarchical namespace for organizing tokens within a context
///
/// Namespaces use dot notation (e.g., "ui.widgets", "db.config") to create
/// logical hierarchies. Meteor warns at 3 levels deep and errors at 4+.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    parts: Vec<String>,
}

impl Namespace {
    /// Create a new namespace from parts
    pub fn new(parts: Vec<String>) -> Self {
        Namespace { parts }
    }

    /// Create an empty namespace (root level)
    pub fn root() -> Self {
        Namespace { parts: vec![] }
    }

    /// Parse a namespace from dot-separated string
    pub fn from_string(s: &str) -> Self {
        if s.is_empty() {
            return Namespace::root();
        }
        Namespace {
            parts: s.split('.').map(|p| p.to_string()).collect(),
        }
    }

    /// Get the namespace depth
    pub fn depth(&self) -> usize {
        self.parts.len()
    }

    /// Check if namespace exceeds warning threshold (3 levels)
    pub fn should_warn(&self) -> bool {
        self.depth() >= 3
    }

    /// Check if namespace exceeds error threshold (4+ levels)
    pub fn is_too_deep(&self) -> bool {
        self.depth() >= 4
    }

    /// Get the namespace parts
    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    /// Convert to dot-separated string
    pub fn to_string(&self) -> String {
        self.parts.join(".")
    }

    /// Check if this namespace is a parent of another
    pub fn is_parent_of(&self, other: &Namespace) -> bool {
        if self.parts.len() >= other.parts.len() {
            return false;
        }
        self.parts.iter().zip(other.parts.iter()).all(|(a, b)| a == b)
    }
}

impl FromStr for Namespace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Namespace::from_string(s))
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for Namespace {
    fn default() -> Self {
        Namespace::root()
    }
}

/// Individual key identifier for token data
///
/// Keys support bracket notation transformations:
/// - `list[0]` → `list__i_0`
/// - `grid[2,3]` → `grid__i_2_3`
/// - `list[]` → `list__i_APPEND`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    base: String,
    transformed: String,
}

impl Key {
    /// Create a new key (will be transformed if contains brackets)
    pub fn new(key: impl Into<String>) -> Self {
        let base = key.into();
        // Apply bracket transformation via sup/bracket.rs
        let transformed = crate::sup::bracket::transform_key(&base)
            .unwrap_or_else(|_| base.clone()); // Fallback to original on error
        Key { base, transformed }
    }

    /// Get the original key (before transformation)
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Get the transformed key (after bracket processing)
    pub fn transformed(&self) -> &str {
        &self.transformed
    }

    /// Check if this key contains bracket notation
    pub fn has_brackets(&self) -> bool {
        self.base.contains('[') && self.base.contains(']')
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Key cannot be empty".into());
        }
        Ok(Key::new(s))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.transformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = Context::new("app");
        assert_eq!(ctx.name(), "app");

        let system = Context::system();
        assert!(system.is_privileged());

        let app = Context::app();
        assert!(!app.is_privileged());
    }

    #[test]
    fn test_namespace_depth() {
        let root = Namespace::root();
        assert_eq!(root.depth(), 0);
        assert!(!root.should_warn());

        let shallow = Namespace::from_string("ui.widgets");
        assert_eq!(shallow.depth(), 2);
        assert!(!shallow.should_warn());

        let deep = Namespace::from_string("ui.widgets.buttons.primary");
        assert_eq!(deep.depth(), 4);
        assert!(deep.should_warn());
        assert!(deep.is_too_deep());
    }

    #[test]
    fn test_key_creation() {
        let simple = Key::new("button");
        assert_eq!(simple.base(), "button");
        assert!(!simple.has_brackets());

        let bracket = Key::new("list[0]");
        assert_eq!(bracket.base(), "list[0]");
        assert!(bracket.has_brackets());
    }
}
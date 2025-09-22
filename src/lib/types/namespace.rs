//! Namespace type - hierarchical organization within contexts

use std::fmt;
use std::str::FromStr;

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_namespace_parent_of() {
        let parent = Namespace::from_string("ui");
        let child = Namespace::from_string("ui.widgets");
        assert!(parent.is_parent_of(&child));
        assert!(!child.is_parent_of(&parent));
    }
}

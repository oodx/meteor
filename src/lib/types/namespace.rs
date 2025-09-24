//! Namespace type - hierarchical organization within contexts

use std::fmt;
use std::str::FromStr;

// Re-export config constants for backward compatibility
pub use crate::types::meteor::config::{MAX_NAMESPACE_PART_LENGTH, NAMESPACE_WARNING_DEPTH, NAMESPACE_ERROR_DEPTH};

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

    /// Parse a namespace from dot-separated string (unchecked)
    pub fn from_string(s: &str) -> Self {
        if s.is_empty() {
            return Namespace::root();
        }
        Namespace {
            parts: s.split('.').map(|p| p.to_string()).collect(),
        }
    }

    /// Parse and validate a namespace from dot-separated string
    pub fn try_from_string(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Ok(Namespace::root());
        }

        let parts: Vec<String> = s.split('.').map(|p| p.to_string()).collect();

        // Validate each part
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                return Err(format!("Empty namespace part at position {}", i));
            }

            if part.len() > MAX_NAMESPACE_PART_LENGTH {
                return Err(format!("Namespace part '{}' too long (max {} chars)", part, MAX_NAMESPACE_PART_LENGTH));
            }

            // Check for valid identifier characters
            if !is_valid_namespace_part(part) {
                return Err(format!("Invalid characters in namespace part '{}'", part));
            }

            // Check for reserved keywords
            if is_reserved_namespace(part) {
                return Err(format!("Reserved namespace part '{}'", part));
            }
        }

        // Check depth limits (error at NAMESPACE_ERROR_DEPTH levels)
        if parts.len() >= NAMESPACE_ERROR_DEPTH {
            return Err(format!("Namespace too deep: {} levels (max {})", parts.len(), NAMESPACE_ERROR_DEPTH - 1));
        }

        Ok(Namespace { parts })
    }

    /// Get the namespace depth
    pub fn depth(&self) -> usize {
        self.parts.len()
    }

    /// Check if namespace exceeds warning threshold
    pub fn should_warn(&self) -> bool {
        self.depth() >= NAMESPACE_WARNING_DEPTH
    }

    /// Check if namespace exceeds error threshold
    pub fn is_too_deep(&self) -> bool {
        self.depth() >= NAMESPACE_ERROR_DEPTH
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
        Namespace::try_from_string(s)
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

/// Validate a namespace part for valid identifier characters
fn is_valid_namespace_part(part: &str) -> bool {
    if part.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let mut chars = part.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    // Rest can be letters, digits, underscores, or hyphens
    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' {
            return false;
        }
    }

    true
}

/// Check if a namespace part is reserved
fn is_reserved_namespace(part: &str) -> bool {
    matches!(part,
        "global" | "root" | "default" |           // System namespaces (main allowed)
        "ctl" | "control" |                       // Control commands
        "ctx" | "context" |                       // Context switching
        "ns" | "namespace"                        // Namespace switching (sys/system/test/debug/dev allowed)
    )
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
        assert!(!deep.should_warn()); // 4 levels are clear now
        assert!(!deep.is_too_deep());

        let warning_deep = Namespace::from_string("ui.widgets.buttons.primary.active");
        assert_eq!(warning_deep.depth(), 5);
        assert!(warning_deep.should_warn()); // 5 levels warn
        assert!(!warning_deep.is_too_deep());
    }

    #[test]
    fn test_namespace_parent_of() {
        let parent = Namespace::from_string("ui");
        let child = Namespace::from_string("ui.widgets");
        assert!(parent.is_parent_of(&child));
        assert!(!child.is_parent_of(&parent));
    }

    #[test]
    fn test_namespace_validation_success() {
        // Valid namespaces
        assert!(Namespace::try_from_string("ui").is_ok());
        assert!(Namespace::try_from_string("ui.widgets").is_ok());
        assert!(Namespace::try_from_string("_private").is_ok());
        assert!(Namespace::try_from_string("api-v2").is_ok());
        assert!(Namespace::try_from_string("").is_ok()); // root
    }

    #[test]
    fn test_namespace_validation_failures() {
        // Empty parts
        assert!(Namespace::try_from_string("ui..widgets").is_err());
        assert!(Namespace::try_from_string(".ui").is_err());
        assert!(Namespace::try_from_string("ui.").is_err());

        // Invalid characters
        assert!(Namespace::try_from_string("ui widgets").is_err());
        assert!(Namespace::try_from_string("ui/widgets").is_err());
        assert!(Namespace::try_from_string("2ui").is_err()); // starts with digit

        // Reserved words
        assert!(Namespace::try_from_string("global").is_err());
        assert!(Namespace::try_from_string("ui.ctl").is_err());
        assert!(Namespace::try_from_string("root").is_err());

        // Too deep (6+ levels now error)
        assert!(Namespace::try_from_string("a.b.c.d.e.f").is_err()); // 6 levels = error

        // Too long part
        let long_part = "a".repeat(MAX_NAMESPACE_PART_LENGTH + 1);
        assert!(Namespace::try_from_string(&long_part).is_err());
    }

    #[test]
    fn test_depth_thresholds() {
        // 4 levels = no warning (clear)
        let clear_ns = Namespace::try_from_string("a.b.c.d").unwrap();
        assert_eq!(clear_ns.depth(), 4);
        assert!(!clear_ns.should_warn());
        assert!(!clear_ns.is_too_deep());

        // 5 levels = warning but allowed
        let warning_ns = Namespace::try_from_string("a.b.c.d.e").unwrap();
        assert_eq!(warning_ns.depth(), 5);
        assert!(warning_ns.should_warn());
        assert!(!warning_ns.is_too_deep());

        // 6+ levels = error, should be rejected
        assert!(Namespace::try_from_string("a.b.c.d.e.f").is_err());
    }

    #[test]
    fn test_from_str_uses_validation() {
        // FromStr should now use validation
        assert!(Namespace::from_str("ui").is_ok());
        assert!(Namespace::from_str("main").is_ok()); // main is allowed (default namespace)
        assert!(Namespace::from_str("global").is_err()); // global is reserved
        assert!(Namespace::from_str("ui..widgets").is_err());
    }
}

/// Dot-notation path parsing utilities
///
/// These functions handle parsing of dot-notation paths like "app.ui.button"
/// into their component parts (context, namespace, key).

/// Parse dot-notation path into (context, namespace, key)
///
/// Handles various path formats:
/// - "app" → ("app", "", "")
/// - "app.ui" → ("app", "ui", "")
/// - "app.ui.button" → ("app", "ui", "button")
/// - "app.ui.complex.key.name" → ("app", "ui", "complex.key.name")
///
/// The first part is always the context, second is namespace, and
/// everything else is treated as a compound key.
pub fn parse_dot_path(path: &str) -> Result<(String, String, String), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let parts: Vec<&str> = path.split('.').collect();

    match parts.len() {
        1 => {
            // Just context: "app"
            Ok((parts[0].to_string(), "".to_string(), "".to_string()))
        }
        2 => {
            // Context and namespace: "app.ui"
            Ok((parts[0].to_string(), parts[1].to_string(), "".to_string()))
        }
        _ => {
            // Context, namespace, and compound key: "app.ui.button.name"
            let context = parts[0].to_string();
            let namespace = parts[1].to_string();
            let key = parts[2..].join(".");
            Ok((context, namespace, key))
        }
    }
}

#[cfg(test)]
mod dot_path_tests {
    use super::*;

    #[test]
    fn test_dot_path_parsing() {
        assert_eq!(parse_dot_path("app").unwrap(), ("app".to_string(), "".to_string(), "".to_string()));
        assert_eq!(parse_dot_path("app.ui").unwrap(), ("app".to_string(), "ui".to_string(), "".to_string()));
        assert_eq!(parse_dot_path("app.ui.button").unwrap(), ("app".to_string(), "ui".to_string(), "button".to_string()));

        // Complex keys should be preserved
        assert_eq!(parse_dot_path("app.ui.complex.key.name").unwrap(),
                   ("app".to_string(), "ui".to_string(), "complex.key.name".to_string()));
    }

    #[test]
    fn test_dot_path_errors() {
        assert!(parse_dot_path("").is_err());
    }
}

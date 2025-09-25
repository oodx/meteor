//! Context type - isolation boundaries for token data

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
        Context { name: name.into() }
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
    fn test_context_from_str() {
        let ctx = Context::from_str("user").unwrap();
        assert_eq!(ctx.name(), "user");
        assert!(ctx.is_privileged());

        assert!(Context::from_str("").is_err());
    }
}

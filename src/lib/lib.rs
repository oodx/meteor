//! Meteor - Shooting Star Token Data Transport Library
//!
//! A string-biased token data transport library providing structured key-value streams
//! with context-aware namespacing and bracket notation extensions.
//!
//! Meteor follows RSB (Rebel String-Biased) principles and architecture patterns,
//! extracting and evolving the token functionality into a focused transport library.
//!
//! # Architecture
//!
//! Meteor organizes by ordinality and responsibility hierarchy:
//! - `types/` - Primary data types (Context, Namespace, Key, TokenBucket)
//! - `utils/` - Public API following data flow (parse -> transform -> organize -> access)
//! - `sup/` - Internal complexity isolation (bracket parsing, validation)

// Core type definitions
pub mod types;

// Public utilities following data flow ordinality
pub mod utils;

// Support modules for internal complexity
mod sup;

// Re-export main public types and functions
pub use types::{Context, Namespace, Key, TokenBucket, MeteorError};
pub use utils::parse::parse_token_stream;

// Module trait for RSB-compliant module organization
pub trait Module {
    /// Return the module's name for identification
    fn name(&self) -> &'static str;

    /// Return the module's version
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Meteor module implementation
pub struct Meteor;

impl Module for Meteor {
    fn name(&self) -> &'static str {
        "meteor"
    }
}

/// Parse a token stream into a TokenBucket
///
/// This is the main entry point for Meteor. Takes a string containing
/// token data and returns a structured TokenBucket with context isolation.
///
/// # Format
///
/// Token streams follow the format:
/// - Basic: `key=value`
/// - Namespaced: `namespace:key=value`
/// - Context-aware: `ctx=app; ui:button=click`
/// - Bracket notation: `list[0]=item` -> `list__i_0=item`
///
/// # Examples
///
/// ```ignore
/// let tokens = meteor::parse("key=value; ui:button=click");
/// let bucket = tokens.unwrap();
/// assert_eq!(bucket.get("", "key"), Some("value"));
/// assert_eq!(bucket.get("ui", "button"), Some("click"));
/// ```
pub fn parse(input: &str) -> Result<TokenBucket, MeteorError> {
    parse_token_stream(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_trait() {
        let meteor = Meteor;
        assert_eq!(meteor.name(), "meteor");
        assert!(!meteor.version().is_empty());
    }
}
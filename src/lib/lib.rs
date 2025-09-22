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
//! - `parser/` - Core parsing infrastructure (parse -> transform -> organize -> bracket)
//! - `utils/` - Essential helper functions (access utilities)
//! - `sup/` - Internal complexity isolation (support functions)

// Core type definitions
pub mod types;

// Parser module for token processing
pub mod parser;

// Public utilities following data flow ordinality
pub mod utils;

// Support modules for internal complexity
pub mod sup;

// Re-export main public types and functions
pub use types::{Context, Namespace, TokenKey, Token, Meteor, MeteorShower, TokenBucket, MeteorError, BracketNotation};
// TODO: Re-enable when parser module is rebuilt
// pub use parser::parse::parse_token_stream;

// Module trait for RSB-compliant module organization
pub trait Module {
    /// Return the module's name for identification
    fn name(&self) -> &'static str;

    /// Return the module's version
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// MeteorModule implementation
pub struct MeteorModule;

impl Module for MeteorModule {
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
    // TODO: Implement using new TokenBucket::parse() when available
    Err(MeteorError::parse(0, "Parser module being rebuilt"))
}

/// Parse a meteor stream into a MeteorShower
///
/// Takes fully-qualified meteor tokens and returns a MeteorShower collection.
/// This is for handling complete meteor addressing format.
///
/// # Format
///
/// Meteor streams follow the full addressing format:
/// - Full: `context:namespace:key=value`
/// - Multiple: `app:ui:button=click; user:settings:theme=dark`
/// - Minimal: `key=value` (defaults to app context, root namespace)
///
/// # Examples
///
/// ```ignore
/// let shower = meteor::parse_shower("app:ui.widgets:button=submit; user:settings:theme=dark");
/// let meteors = shower.unwrap();
/// assert_eq!(meteors.len(), 2);
/// ```
pub fn parse_shower(input: &str) -> Result<MeteorShower, String> {
    MeteorShower::parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_trait() {
        let module = MeteorModule;
        assert_eq!(module.name(), "meteor");
        assert!(!module.version().is_empty());
    }
}
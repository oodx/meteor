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
//! - `types/` - Primary data types (Context, Namespace, Key, Token, Meteor, MeteorShower)
//! - `validation/` - Format validation utilities (is_valid_token, is_valid_meteor, etc.)
//! - `utils/` - Essential helper functions (access utilities)

// Core type definitions
pub mod types;

// Validation utilities
pub mod validation;

// Public utilities following data flow ordinality
pub mod utils;

// Parser module for stream processing
pub mod parser;

// Re-export main public types and functions
pub use types::{Context, Namespace, TokenKey, Token, TokenBucket, Meteor, MeteorShower, MeteorEngine, StorageData, ControlCommand, MeteorError, BracketNotation};
pub use validation::{is_valid_token, is_valid_meteor, is_valid_meteor_shower};
pub use utils::{is_valid_token_format, is_valid_meteor_format, is_valid_meteor_shower_format};
pub use parser::{TokenStreamParser, MeteorStreamParser, parse_escaped_value, validate_escapes};

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
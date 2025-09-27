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

// Build-time configuration via Cargo features
pub mod config;

// Core type definitions
pub mod types;

// Validation utilities
pub mod validation;

// Public utilities following data flow ordinality
pub mod utils;

// Parser module for stream processing
pub mod parser;

// Re-export main public types and functions
pub use config::{config_profile, config_summary};
pub use parser::{parse_escaped_value, validate_escapes, MeteorStreamParser, TokenStreamParser};
pub use types::{
    BracketNotation, Context, ControlCommand, Meteor, MeteorEngine, MeteorError, MeteorShower,
    Namespace, StorageData, Token, TokenKey,
};
pub use utils::{is_valid_meteor_format, is_valid_meteor_shower_format, is_valid_token_format};
pub use validation::{is_valid_meteor, is_valid_meteor_shower, is_valid_token};

// ================================
// Convenience Macros
// ================================

/// Build a canonical meteor path (`context:namespace:key`).
#[macro_export]
macro_rules! meteor {
    ($context:literal : $namespace:literal : $key:literal) => {
        $crate::meteor!($context, $namespace, $key)
    };
    ($context:ident : $namespace:ident : $key:ident) => {
        $crate::meteor!(
            stringify!($context),
            stringify!($namespace),
            stringify!($key)
        )
    };
    ($context:expr, $namespace:expr, $key:expr) => {
        ::std::format!("{}:{}:{}", $context, $namespace, $key)
    };
    ($path:expr) => {
        ::std::string::ToString::to_string(&$path)
    };
}

/// Return the `(context, namespace, key)` triple for a meteor address.
#[macro_export]
macro_rules! meteor_parts {
    ($context:literal : $namespace:literal : $key:literal) => {
        ($context, $namespace, $key)
    };
    ($context:ident : $namespace:ident : $key:ident) => {
        (
            stringify!($context),
            stringify!($namespace),
            stringify!($key),
        )
    };
    ($context:expr, $namespace:expr, $key:expr) => {
        ($context, $namespace, $key)
    };
}

/// Call `MeteorEngine::set` with a canonical path built from segments.
#[macro_export]
macro_rules! meteor_set {
    ($engine:expr, $context:literal : $namespace:literal : $key:literal => $value:expr) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $context:ident : $namespace:ident : $key:ident => $value:expr) => {{
        let __path = $crate::meteor!(
            stringify!($context),
            stringify!($namespace),
            stringify!($key)
        );
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $context:expr, $namespace:expr, $key:expr => $value:expr) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $path:expr => $value:expr) => {{
        let __path = $crate::meteor!($path);
        $engine.set(&__path, $value)
    }};
}

/// Retrieve a value using a canonical meteor path.
#[macro_export]
macro_rules! meteor_get {
    ($engine:expr, $context:literal : $namespace:literal : $key:literal) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.get(&__path)
    }};
    ($engine:expr, $context:ident : $namespace:ident : $key:ident) => {{
        let __path = $crate::meteor!(
            stringify!($context),
            stringify!($namespace),
            stringify!($key)
        );
        $engine.get(&__path)
    }};
    ($engine:expr, $context:expr, $namespace:expr, $key:expr) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.get(&__path)
    }};
    ($engine:expr, $path:expr) => {{
        let __path = $crate::meteor!($path);
        $engine.get(&__path)
    }};
}

/// Delete a value using a canonical meteor path.
#[macro_export]
macro_rules! meteor_delete {
    ($engine:expr, $context:literal : $namespace:literal : $key:literal) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.delete(&__path)
    }};
    ($engine:expr, $context:ident : $namespace:ident : $key:ident) => {{
        let __path = $crate::meteor!(
            stringify!($context),
            stringify!($namespace),
            stringify!($key)
        );
        $engine.delete(&__path)
    }};
    ($engine:expr, $context:expr, $namespace:expr, $key:expr) => {{
        let __path = $crate::meteor!($context, $namespace, $key);
        $engine.delete(&__path)
    }};
    ($engine:expr, $path:expr) => {{
        let __path = $crate::meteor!($path);
        $engine.delete(&__path)
    }};
}

/// Store a default (`*.index`) value for a context/namespace pair.
#[macro_export]
macro_rules! meteor_default {
    ($engine:expr, $context:literal : $namespace:literal => $value:expr) => {{
        let __path = ::std::format!("{}:{}:index", $context, $namespace);
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $context:ident : $namespace:ident => $value:expr) => {{
        let __path = ::std::format!("{}:{}:index", stringify!($context), stringify!($namespace));
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $context:expr, $namespace:expr => $value:expr) => {{
        let __path = ::std::format!("{}:{}:index", $context, $namespace);
        $engine.set(&__path, $value)
    }};
    ($engine:expr, $context:expr => $value:expr) => {{
        let __path = ::std::format!("{}:{}:index", $context, "main");
        $engine.set(&__path, $value)
    }};
}

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

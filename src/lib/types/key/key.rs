//! TokenKey type - individual data identifiers with bracket transformation

use super::notation::BracketNotation;
use std::fmt;
use std::str::FromStr;

/// Individual key identifier for token data
///
/// TokenKeys support bracket notation transformations:
/// - `list[0]` → `list__i_0`
/// - `grid[2,3]` → `grid__i_2_3`
/// - `list[]` → `list__i_APPEND`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenKey {
    base: String,
    transformed: String,
}

impl TokenKey {
    /// Create a new key (will be transformed if contains brackets)
    pub fn new(key: impl Into<String>) -> Self {
        let base = key.into();
        // Apply bracket transformation via parser/bracket.rs
        let transformed = crate::parser::bracket::transform_key(&base)
            .unwrap_or_else(|_| base.clone()); // Fallback to original on error
        TokenKey { base, transformed }
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

impl FromStr for TokenKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("TokenKey cannot be empty".into());
        }
        Ok(TokenKey::new(s))
    }
}

impl fmt::Display for TokenKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.transformed)
    }
}

impl BracketNotation for TokenKey {
    fn to_bracket(&self) -> String {
        self.base.clone() // Fast cached lookup
    }

    fn has_brackets(&self) -> bool {
        self.base.contains('[') && self.base.contains(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_key_creation() {
        let simple = TokenKey::new("button");
        assert_eq!(simple.base(), "button");
        assert!(!simple.has_brackets());

        let bracket = TokenKey::new("list[0]");
        assert_eq!(bracket.base(), "list[0]");
        assert!(bracket.has_brackets());
    }

    #[test]
    fn test_token_key_from_str() {
        let key = TokenKey::from_str("test").unwrap();
        assert_eq!(key.base(), "test");

        assert!(TokenKey::from_str("").is_err());
    }
}
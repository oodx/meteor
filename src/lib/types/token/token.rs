//! Token type - individual key identifiers with value

use std::fmt;
use std::str::FromStr;

/// Individual token with key and value
///
/// Tokens support bracket notation transformations:
/// - `list[0]=item` → key: `list__i_0`, value: `item`
/// - `grid[2,3]=cell` → key: `grid__i_2_3`, value: `cell`
/// - `list[]=new` → key: `list__i_APPEND`, value: `new`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    key: String,
    transformed_key: String,
    value: String,
}

impl Token {
    /// Create a new token with key and value
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        // Apply bracket transformation via parser/bracket.rs
        let transformed_key = crate::parser::bracket::transform_key(&key)
            .unwrap_or_else(|_| key.clone()); // Fallback to original on error

        Token {
            key: key.clone(),
            transformed_key,
            value: value.into(),
        }
    }

    /// Get the original key (before transformation)
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the transformed key (after bracket processing)
    pub fn transformed_key(&self) -> &str {
        &self.transformed_key
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if this token's key contains bracket notation
    pub fn has_brackets(&self) -> bool {
        self.key.contains('[') && self.key.contains(']')
    }

    /// Parse a token from "key=value" format
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid token format: {}", s));
        }

        Ok(Token::new(parts[0], parts[1]))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.transformed_key, self.value)
    }
}

impl FromStr for Token {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let simple = Token::new("button", "submit");
        assert_eq!(simple.key(), "button");
        assert_eq!(simple.value(), "submit");
        assert!(!simple.has_brackets());
    }

    #[test]
    fn test_token_parse() {
        let token = Token::parse("theme=dark").unwrap();
        assert_eq!(token.key(), "theme");
        assert_eq!(token.value(), "dark");

        assert!(Token::parse("invalid").is_err());
    }

    #[test]
    fn test_token_with_brackets() {
        let bracket = Token::new("list[0]", "item");
        assert_eq!(bracket.key(), "list[0]");
        assert!(bracket.has_brackets());
    }
}
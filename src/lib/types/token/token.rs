//! Token type - individual key identifiers with value

use crate::types::TokenKey;
use std::fmt;
use std::str::FromStr;

/// Individual token with key and value
///
/// Tokens support bracket notation transformations via TokenKey:
/// - `list[0]=item` → key: `list[0]`, flat: `list__i_0`, value: `item`
/// - `grid[2,3]=cell` → key: `grid[2,3]`, flat: `grid__i_2_3`, value: `cell`
/// - `list[]=new` → key: `list[]`, flat: `list__i_APPEND`, value: `new`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    key: TokenKey,
    value: String,
}

impl Token {
    /// Create a new token with key and value
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Token {
            key: TokenKey::new(key.into()),
            value: value.into(),
        }
    }

    /// Get the TokenKey (with bracket notation capabilities)
    pub fn key(&self) -> &TokenKey {
        &self.key
    }

    /// Get the original bracket notation string
    pub fn key_notation(&self) -> &str {
        self.key.base()
    }

    /// Get the flattened key string (for storage/comparison)
    pub fn key_str(&self) -> &str {
        self.key.transformed()
    }

    /// Get the transformed key (after bracket processing) - alias for key_str()
    pub fn transformed_key(&self) -> &str {
        self.key.transformed()
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if this token's key contains bracket notation
    pub fn has_brackets(&self) -> bool {
        self.key.has_brackets()
    }

    /// Parse a token from "key=value" format
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid token format: {}", s));
        }

        let key = parts[0].trim();
        let value = parts[1];

        // Key cannot be empty
        if key.is_empty() {
            return Err(format!("Token key cannot be empty: {}", s));
        }

        Ok(Token::new(key, value))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.transformed_key(), self.value)
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
        assert_eq!(simple.key_notation(), "button");
        assert_eq!(simple.value(), "submit");
        assert!(!simple.has_brackets());
    }

    #[test]
    fn test_token_parse() {
        let token = Token::parse("theme=dark").unwrap();
        assert_eq!(token.key_notation(), "theme");
        assert_eq!(token.value(), "dark");

        assert!(Token::parse("invalid").is_err());
    }

    #[test]
    fn test_token_with_brackets() {
        let bracket = Token::new("list[0]", "item");
        assert_eq!(bracket.key_notation(), "list[0]");
        assert!(bracket.has_brackets());
    }
}
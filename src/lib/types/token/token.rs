//! Token type - individual key identifiers with value

use crate::types::{Namespace, TokenKey};
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
    namespace: Option<Namespace>,
    key: TokenKey,
    value: String,
}

impl Token {
    /// Create a new token with key and value
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Token {
            namespace: None,
            key: TokenKey::new(key.into()),
            value: value.into(),
        }
    }

    /// Create a new token with namespace, key and value
    pub fn new_with_namespace(
        namespace: Namespace,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Token {
            namespace: Some(namespace),
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

    /// Get the namespace (if any)
    pub fn namespace(&self) -> Option<&Namespace> {
        self.namespace.as_ref()
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if this token's key contains bracket notation
    pub fn has_brackets(&self) -> bool {
        self.key.has_brackets()
    }

    /// Parse all tokens from semicolon-separated string: "key1=val1; key2=val2; namespace:key3=val3"
    pub fn parse(s: &str) -> Result<Vec<Self>, String> {
        let parts = crate::utils::validators::smart_split_semicolons(s)
            .ok_or_else(|| "Unbalanced quotes in token string".to_string())?;

        let mut tokens = Vec::new();

        for token_str in parts {
            let trimmed = token_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            let token = Self::parse_single(trimmed)?;
            tokens.push(token);
        }

        if tokens.is_empty() {
            return Err("No valid tokens found".to_string());
        }

        Ok(tokens)
    }

    /// Parse the first token from a string (convenience method)
    pub fn first(s: &str) -> Result<Self, String> {
        let tokens = Self::parse(s)?;
        Ok(tokens.into_iter().next().unwrap()) // Safe because parse() ensures non-empty vec
    }

    /// Parse a single token from "key=value" or "namespace:key=value" format
    fn parse_single(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid token format: {}", s));
        }

        let key_part = parts[0].trim();
        let value = parts[1];

        // Check if key_part contains namespace prefix
        if key_part.contains(':') {
            let key_parts: Vec<&str> = key_part.splitn(2, ':').collect();
            if key_parts.len() == 2 {
                let namespace = Namespace::from_string(key_parts[0]);
                let key = key_parts[1];

                // Key cannot be empty
                if key.is_empty() {
                    return Err(format!("Token key cannot be empty: {}", s));
                }

                Ok(Token::new_with_namespace(namespace, key, value))
            } else {
                return Err(format!("Invalid namespaced token format: {}", s));
            }
        } else {
            // Key cannot be empty
            if key_part.is_empty() {
                return Err(format!("Token key cannot be empty: {}", s));
            }

            Ok(Token::new(key_part, value))
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespace {
            Some(namespace) => write!(
                f,
                "{}:{}={}",
                namespace.to_string(),
                self.transformed_key(),
                self.value
            ),
            None => write!(f, "{}={}", self.transformed_key(), self.value),
        }
    }
}

impl FromStr for Token {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::first(s)
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
        let tokens = Token::parse("theme=dark").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].key_notation(), "theme");
        assert_eq!(tokens[0].value(), "dark");

        // Test multiple tokens
        let tokens = Token::parse("theme=dark; lang=en").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].key_notation(), "theme");
        assert_eq!(tokens[1].key_notation(), "lang");

        assert!(Token::parse("invalid").is_err());
    }

    #[test]
    fn test_token_first() {
        let token = Token::first("theme=dark").unwrap();
        assert_eq!(token.key_notation(), "theme");
        assert_eq!(token.value(), "dark");

        // Test first from multiple tokens
        let token = Token::first("theme=dark; lang=en").unwrap();
        assert_eq!(token.key_notation(), "theme");
        assert_eq!(token.value(), "dark");

        assert!(Token::first("invalid").is_err());
    }

    #[test]
    fn test_token_with_brackets() {
        let bracket = Token::new("list[0]", "item");
        assert_eq!(bracket.key_notation(), "list[0]");
        assert!(bracket.has_brackets());
    }

    #[test]
    fn test_token_parse_with_quoted_semicolon() {
        let tokens = Token::parse("message=\"Hello; World\"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].key_notation(), "message");
        assert_eq!(tokens[0].value(), "\"Hello; World\"");
    }

    #[test]
    fn test_token_parse_unbalanced_quotes() {
        assert!(Token::parse("message=\"Hello; World").is_err());
    }
}

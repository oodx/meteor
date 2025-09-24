//! Foundation tests for Token type
//!
//! Tests the core Token functionality including:
//! - Creation with TokenKey
//! - Key-value pair management
//! - Display formatting
//! - Parsing from strings

use meteor::{Token, TokenKey};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_simple() {
        // Create token with simple key-value
        let token = Token::new("key", "value");
        assert_eq!(token.key().as_str(), "key");
        assert_eq!(token.value(), "value");
    }

    #[test]
    fn test_token_creation_with_brackets() {
        // Create token with bracket notation key
        let token = Token::new("list[0]", "first_item");
        assert_eq!(token.key().as_str(), "list__i_0");
        assert_eq!(token.value(), "first_item");
    }

    #[test]
    fn test_token_from_token_key() {
        // Create token from existing TokenKey
        let key = TokenKey::from("matrix[1,2]");
        let token = Token::from_key(key.clone(), "cell_value");
        assert_eq!(token.key(), &key);
        assert_eq!(token.value(), "cell_value");
    }

    #[test]
    fn test_token_display() {
        // Test display formatting
        let token = Token::new("name", "Alice");
        let display = format!("{}", token);
        assert_eq!(display, "name=Alice");

        // With bracket notation
        let token2 = Token::new("users[0]", "Bob");
        let display2 = format!("{}", token2);
        assert_eq!(display2, "users__i_0=Bob");
    }

    #[test]
    fn test_token_parse_simple() {
        // Parse simple key=value
        let token = Token::parse("key=value").expect("Failed to parse");
        assert_eq!(token.key().as_str(), "key");
        assert_eq!(token.value(), "value");
    }

    #[test]
    fn test_token_parse_with_brackets() {
        // Parse with bracket notation
        let token = Token::parse("array[5]=element").expect("Failed to parse");
        assert_eq!(token.key().as_str(), "array__i_5");
        assert_eq!(token.value(), "element");
    }

    #[test]
    fn test_token_parse_empty_value() {
        // Parse with empty value
        let token = Token::parse("empty=").expect("Failed to parse");
        assert_eq!(token.key().as_str(), "empty");
        assert_eq!(token.value(), "");
    }

    #[test]
    fn test_token_parse_with_spaces() {
        // Parse with spaces in value
        let token = Token::parse("message=Hello World").expect("Failed to parse");
        assert_eq!(token.key().as_str(), "message");
        assert_eq!(token.value(), "Hello World");
    }

    #[test]
    fn test_token_parse_with_equals_in_value() {
        // Parse with equals sign in value
        let token = Token::parse("equation=x=y+1").expect("Failed to parse");
        assert_eq!(token.key().as_str(), "equation");
        assert_eq!(token.value(), "x=y+1");
    }

    #[test]
    fn test_token_parse_errors() {
        // Test parse errors
        assert!(Token::parse("").is_err());
        assert!(Token::parse("no_equals_sign").is_err());
        assert!(Token::parse("=no_key").is_err());
    }

    #[test]
    fn test_token_equality() {
        let token1 = Token::new("key", "value");
        let token2 = Token::new("key", "value");
        assert_eq!(token1, token2);

        let token3 = Token::new("key", "different");
        assert_ne!(token1, token3);

        let token4 = Token::new("different", "value");
        assert_ne!(token1, token4);
    }

    #[test]
    fn test_token_clone() {
        let token1 = Token::new("original", "data");
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token1.key(), token2.key());
        assert_eq!(token1.value(), token2.value());
    }

    #[test]
    fn test_token_debug() {
        let token = Token::new("debug", "test");
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Token"));
        assert!(debug_str.contains("debug"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_token_with_special_characters() {
        // Test with various special characters
        let token = Token::new("path/to/file", "/usr/local/bin");
        assert_eq!(token.key().as_str(), "path/to/file");
        assert_eq!(token.value(), "/usr/local/bin");

        let token2 = Token::parse("url=https://example.com").expect("Failed to parse");
        assert_eq!(token2.key().as_str(), "url");
        assert_eq!(token2.value(), "https://example.com");
    }

    #[test]
    fn test_token_roundtrip() {
        // Test parse and display roundtrip
        let original = "config=production";
        let token = Token::parse(original).expect("Failed to parse");
        let displayed = format!("{}", token);
        assert_eq!(displayed, original);

        // With brackets (note: display shows transformed key)
        let token2 = Token::parse("items[0]=first").expect("Failed to parse");
        let displayed2 = format!("{}", token2);
        assert_eq!(displayed2, "items__i_0=first");
    }

    #[test]
    fn test_token_getters() {
        let token = Token::new("test_key", "test_value");

        // Test key getter
        let key = token.key();
        assert_eq!(key.as_str(), "test_key");

        // Test value getter
        let value = token.value();
        assert_eq!(value, "test_value");

        // Test into_parts (if available)
        let cloned = token.clone();
        let (key2, value2) = cloned.into_parts();
        assert_eq!(key2.as_str(), "test_key");
        assert_eq!(value2, "test_value");
    }

    #[test]
    fn test_token_complex_values() {
        // JSON-like value
        let token = Token::new("data", r#"{"name": "test", "count": 42}"#);
        assert_eq!(token.value(), r#"{"name": "test", "count": 42}"#);

        // Multi-line value (if supported)
        let token2 = Token::new("text", "line1\nline2\nline3");
        assert_eq!(token2.value(), "line1\nline2\nline3");

        // Unicode value
        let token3 = Token::new("emoji", "🚀✨🌟");
        assert_eq!(token3.value(), "🚀✨🌟");
    }
}
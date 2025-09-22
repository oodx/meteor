//! Token transformation utilities (RSB string-biased approach)
//!
//! This module provides the second step in the data flow ordinality:
//! parse → **transform** → organize → access
//!
//! Handles token transformations including bracket notation processing
//! and value normalization using RSB patterns.

use crate::types::MeteorError;
use crate::parser::bracket;

/// Transform a raw token key using bracket notation rules
///
/// RSB string-biased interface for key transformation.
///
/// # Examples
///
/// ```ignore
/// use meteor::utils::transform::transform_key;
///
/// assert_eq!(transform_key("list[0]").unwrap(), "list__i_0");
/// assert_eq!(transform_key("grid[x,y]").unwrap(), "grid__i_x_y");
/// assert_eq!(transform_key("normal_key").unwrap(), "normal_key");
/// ```
pub fn transform_key(key: &str) -> Result<String, MeteorError> {
    bracket::transform_key(key)
}

/// Transform a token value with normalization
///
/// Currently a pass-through but ready for future enhancements like:
/// - Value type detection
/// - Format normalization
/// - Encoding handling
pub fn transform_value(value: &str) -> Result<String, MeteorError> {
    // RSB string-biased: keep it simple for now
    Ok(value.to_string())
}

/// Transform a complete token (key=value) into normalized parts
///
/// This is a convenience function that combines key and value transformation.
pub fn transform_token(token: &str) -> Result<(String, String), MeteorError> {
    // Find the equals sign
    let equals_pos = token.find('=')
        .ok_or_else(|| MeteorError::invalid_token(token, "missing '=' separator"))?;

    let raw_key = token[..equals_pos].trim();
    let raw_value = token[equals_pos + 1..].trim();

    if raw_key.is_empty() {
        return Err(MeteorError::invalid_token(token, "empty key"));
    }

    let transformed_key = transform_key(raw_key)?;
    let transformed_value = transform_value(raw_value)?;

    Ok((transformed_key, transformed_value))
}

/// Apply batch transformations to multiple tokens
///
/// RSB string-biased approach - takes a semicolon-separated string
/// and returns transformed tokens.
pub fn transform_token_batch(tokens: &str) -> Result<Vec<(String, String)>, MeteorError> {
    let mut results = Vec::new();

    for token_str in tokens.split(';') {
        let token_str = token_str.trim();
        if token_str.is_empty() {
            continue;
        }

        // Skip context switches - they don't need transformation
        if token_str.starts_with("ctx=") {
            continue;
        }

        let (key, value) = transform_token(token_str)?;
        results.push((key, value));
    }

    Ok(results)
}

/// Check if a key needs bracket transformation
pub fn needs_transformation(key: &str) -> bool {
    bracket::has_brackets(key)
}

/// Extract the base name from a potentially bracketed key
pub fn extract_base_name(key: &str) -> Result<String, MeteorError> {
    bracket::extract_base_name(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_key() {
        assert_eq!(transform_key("simple").unwrap(), "simple");
        assert_eq!(transform_key("list[0]").unwrap(), "list__i_0");
        assert_eq!(transform_key("grid[x,y]").unwrap(), "grid__i_x_y");
        assert_eq!(transform_key("queue[]").unwrap(), "queue__i_APPEND");
    }

    #[test]
    fn test_transform_value() {
        assert_eq!(transform_value("simple").unwrap(), "simple");
        assert_eq!(transform_value("value with spaces").unwrap(), "value with spaces");
    }

    #[test]
    fn test_transform_token() {
        let (key, value) = transform_token("list[0]=item").unwrap();
        assert_eq!(key, "list__i_0");
        assert_eq!(value, "item");

        let (key, value) = transform_token("simple=value").unwrap();
        assert_eq!(key, "simple");
        assert_eq!(value, "value");
    }

    #[test]
    fn test_transform_token_batch() {
        let result = transform_token_batch("key1=value1; list[0]=item; ctx=app; key2=value2").unwrap();

        assert_eq!(result.len(), 3); // ctx=app is skipped
        assert_eq!(result[0], ("key1".to_string(), "value1".to_string()));
        assert_eq!(result[1], ("list__i_0".to_string(), "item".to_string()));
        assert_eq!(result[2], ("key2".to_string(), "value2".to_string()));
    }

    #[test]
    fn test_needs_transformation() {
        assert!(!needs_transformation("simple"));
        assert!(needs_transformation("list[0]"));
        assert!(needs_transformation("grid[x,y]"));
    }

    #[test]
    fn test_extract_base_name() {
        assert_eq!(extract_base_name("simple").unwrap(), "simple");
        assert_eq!(extract_base_name("list[0]").unwrap(), "list");
        assert_eq!(extract_base_name("grid[x,y]").unwrap(), "grid");
    }
}
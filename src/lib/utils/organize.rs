//! Token organization utilities (RSB string-biased approach)
//!
//! This module provides the third step in the data flow ordinality:
//! parse → transform → **organize** → access
//!
//! Handles organizing transformed tokens into structured TokenBucket data
//! with proper context and namespace isolation.

use crate::types::{Context, Namespace, TokenBucket, MeteorError};
use std::str::FromStr;

/// Organize transformed tokens into a TokenBucket
///
/// RSB string-biased interface - takes a vector of (key, value) pairs
/// and organizes them into a structured TokenBucket.
pub fn organize_tokens(tokens: Vec<(String, String)>) -> Result<TokenBucket, MeteorError> {
    let mut bucket = TokenBucket::new();

    for (key, value) in tokens {
        // For now, put all tokens in the root namespace
        bucket.set("", &key, value);
    }

    Ok(bucket)
}

/// Organize tokens with explicit namespace support
///
/// Takes (namespace, key, value) tuples and organizes them properly.
pub fn organize_namespaced_tokens(
    tokens: Vec<(String, String, String)>,
) -> Result<TokenBucket, MeteorError> {
    let mut bucket = TokenBucket::new();

    for (namespace, key, value) in tokens {
        // Validate namespace depth
        let ns = Namespace::from_string(&namespace);
        if ns.is_too_deep() {
            return Err(MeteorError::namespace_too_deep(&namespace, ns.depth()));
        }

        bucket.set(&namespace, &key, value);
    }

    Ok(bucket)
}

/// Organize tokens with context switching support
///
/// This is the full-featured organizer that handles context switches
/// and maintains proper isolation.
pub fn organize_contextual_tokens(
    tokens: Vec<ContextualToken>,
) -> Result<TokenBucket, MeteorError> {
    let mut bucket = TokenBucket::new();

    for token in tokens {
        match token {
            ContextualToken::ContextSwitch(context_name) => {
                let context = Context::from_str(&context_name)
                    .map_err(|e| MeteorError::parse(0, e))?;
                bucket.switch_context(context);
            }
            ContextualToken::Data { namespace, key, value } => {
                // Validate namespace depth
                let ns = Namespace::from_string(&namespace);
                if ns.is_too_deep() {
                    return Err(MeteorError::namespace_too_deep(&namespace, ns.depth()));
                }

                bucket.set(&namespace, &key, value);
            }
        }
    }

    Ok(bucket)
}

/// Represents a token that may include context information
#[derive(Debug, Clone, PartialEq)]
pub enum ContextualToken {
    /// A context switch operation
    ContextSwitch(String),
    /// A data token with namespace, key, and value
    Data {
        namespace: String,
        key: String,
        value: String,
    },
}

/// Create a ContextualToken from a namespace:key=value string
pub fn create_contextual_token(
    namespace: &str,
    key: &str,
    value: &str,
) -> ContextualToken {
    ContextualToken::Data {
        namespace: namespace.to_string(),
        key: key.to_string(),
        value: value.to_string(),
    }
}

/// Create a context switch token
pub fn create_context_switch(context_name: &str) -> ContextualToken {
    ContextualToken::ContextSwitch(context_name.to_string())
}

/// Group tokens by namespace for efficient organization
///
/// RSB string-biased helper for organizing large token sets.
pub fn group_by_namespace(
    tokens: Vec<(String, String, String)>,
) -> std::collections::HashMap<String, Vec<(String, String)>> {
    let mut groups = std::collections::HashMap::new();

    for (namespace, key, value) in tokens {
        groups
            .entry(namespace)
            .or_insert_with(Vec::new)
            .push((key, value));
    }

    groups
}

/// Validate token organization before committing to bucket
///
/// Performs validation checks on token organization to catch issues early.
pub fn validate_token_organization(
    tokens: &[ContextualToken],
) -> Result<(), MeteorError> {
    for token in tokens {
        match token {
            ContextualToken::ContextSwitch(context_name) => {
                if context_name.is_empty() {
                    return Err(MeteorError::empty("context name"));
                }
            }
            ContextualToken::Data { namespace, key, value: _ } => {
                if key.is_empty() {
                    return Err(MeteorError::empty("key"));
                }

                // Validate namespace depth
                let ns = Namespace::from_string(namespace);
                if ns.is_too_deep() {
                    return Err(MeteorError::namespace_too_deep(namespace, ns.depth()));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organize_tokens() {
        let tokens = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        let bucket = organize_tokens(tokens).unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("", "key2"), Some("value2"));
    }

    #[test]
    fn test_organize_namespaced_tokens() {
        let tokens = vec![
            ("".to_string(), "key1".to_string(), "value1".to_string()),
            ("ui".to_string(), "button".to_string(), "click".to_string()),
        ];

        let bucket = organize_namespaced_tokens(tokens).unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn test_organize_contextual_tokens() {
        let tokens = vec![
            create_contextual_token("", "key1", "value1"),
            create_context_switch("user"),
            create_contextual_token("ui", "button", "save"),
        ];

        let bucket = organize_contextual_tokens(tokens).unwrap();

        // Should be in user context now
        assert_eq!(bucket.context().name(), "user");
        assert_eq!(bucket.get("ui", "button"), Some("save"));
    }

    #[test]
    fn test_group_by_namespace() {
        let tokens = vec![
            ("".to_string(), "key1".to_string(), "value1".to_string()),
            ("ui".to_string(), "button".to_string(), "click".to_string()),
            ("ui".to_string(), "label".to_string(), "text".to_string()),
        ];

        let groups = group_by_namespace(tokens);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("").unwrap().len(), 1);
        assert_eq!(groups.get("ui").unwrap().len(), 2);
    }

    #[test]
    fn test_validate_token_organization() {
        let valid_tokens = vec![
            create_contextual_token("ui", "button", "click"),
            create_context_switch("user"),
        ];

        assert!(validate_token_organization(&valid_tokens).is_ok());

        let invalid_tokens = vec![
            create_contextual_token("ui", "", "click"), // empty key
        ];

        assert!(validate_token_organization(&invalid_tokens).is_err());
    }

    #[test]
    fn test_namespace_depth_validation() {
        let deep_tokens = vec![
            ("ui.widgets.buttons.primary".to_string(), "key".to_string(), "value".to_string()),
        ];

        let result = organize_namespaced_tokens(deep_tokens);
        assert!(result.is_err()); // Should fail due to namespace depth
    }
}
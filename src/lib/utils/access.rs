//! Token access utilities (RSB string-biased approach)
//!
//! This module provides the fourth step in the data flow ordinality:
//! parse → transform → organize → **access**
//!
//! Handles querying and retrieving data from TokenBucket with various
//! access patterns using RSB string-biased interfaces.

use crate::types::{TokenBucket, Context};
use std::collections::HashMap;
use std::str::FromStr;

/// Query a TokenBucket for a specific value
///
/// RSB string-biased interface for simple value retrieval.
pub fn get_value(bucket: &TokenBucket, namespace: &str, key: &str) -> Option<String> {
    bucket.get(namespace, key).map(|s| s.to_string())
}

/// Query a TokenBucket with default value fallback
///
/// Returns the default if the key is not found.
pub fn get_value_or(
    bucket: &TokenBucket,
    namespace: &str,
    key: &str,
    default: &str,
) -> String {
    bucket.get(namespace, key).unwrap_or(default).to_string()
}

/// Query multiple keys from the same namespace
///
/// RSB string-biased batch retrieval.
pub fn get_namespace_values(
    bucket: &TokenBucket,
    namespace: &str,
) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for key in bucket.keys_in_namespace(namespace) {
        if let Some(value) = bucket.get(namespace, &key) {
            result.insert(key, value.to_string());
        }
    }

    result
}

/// Query keys matching a pattern
///
/// Simple pattern matching for RSB string-biased access.
pub fn find_keys_matching(
    bucket: &TokenBucket,
    namespace: &str,
    pattern: &str,
) -> Vec<String> {
    bucket
        .keys_in_namespace(namespace)
        .into_iter()
        .filter(|key| key.contains(pattern))
        .collect()
}

/// Query transformed bracket notation keys
///
/// Find keys that were transformed from bracket notation.
pub fn find_bracket_keys(bucket: &TokenBucket, namespace: &str) -> Vec<String> {
    bucket
        .keys_in_namespace(namespace)
        .into_iter()
        .filter(|key| key.contains("__i_"))
        .collect()
}

/// Query all keys with a specific base name
///
/// For finding all variations of a bracket-transformed key.
/// E.g., find all "list" keys: "list__i_0", "list__i_1", etc.
pub fn find_keys_with_base(
    bucket: &TokenBucket,
    namespace: &str,
    base_name: &str,
) -> Vec<String> {
    let prefix = format!("{}__i_", base_name);
    bucket
        .keys_in_namespace(namespace)
        .into_iter()
        .filter(|key| key.starts_with(&prefix) || *key == base_name)
        .collect()
}

/// Extract array-like data from bracket notation keys
///
/// Returns a sorted vector of (index, value) pairs for array-like keys.
pub fn get_array_values(
    bucket: &TokenBucket,
    namespace: &str,
    base_name: &str,
) -> Vec<(String, String)> {
    let prefix = format!("{}__i_", base_name);
    let mut results = Vec::new();

    for key in bucket.keys_in_namespace(namespace) {
        if key.starts_with(&prefix) {
            let index = &key[prefix.len()..];
            if let Some(value) = bucket.get(namespace, &key) {
                results.push((index.to_string(), value.to_string()));
            }
        }
    }

    // Sort by index for predictable ordering
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

/// Query bucket statistics
///
/// RSB string-biased information about bucket contents.
pub fn get_bucket_stats(bucket: &TokenBucket) -> BucketStats {
    let namespaces = bucket.namespaces();
    let total_tokens = bucket.len();
    let context_name = bucket.context().name().to_string();

    let mut namespace_counts = HashMap::new();
    for namespace in &namespaces {
        let count = bucket.keys_in_namespace(namespace).len();
        namespace_counts.insert(namespace.clone(), count);
    }

    BucketStats {
        context_name,
        total_tokens,
        namespace_count: namespaces.len(),
        namespaces,
        namespace_counts,
    }
}

/// Statistics about a TokenBucket
#[derive(Debug, Clone, PartialEq)]
pub struct BucketStats {
    pub context_name: String,
    pub total_tokens: usize,
    pub namespace_count: usize,
    pub namespaces: Vec<String>,
    pub namespace_counts: HashMap<String, usize>,
}

/// Check if a bucket contains specific data
///
/// RSB string-biased existence checking.
pub fn has_value(bucket: &TokenBucket, namespace: &str, key: &str) -> bool {
    bucket.get(namespace, key).is_some()
}

/// Check if a namespace exists in the bucket
pub fn has_namespace(bucket: &TokenBucket, namespace: &str) -> bool {
    bucket.namespaces().contains(&namespace.to_string())
}

/// Get all values matching a key across all namespaces
///
/// Useful for finding the same key in different namespaces.
pub fn get_key_across_namespaces(
    bucket: &TokenBucket,
    key: &str,
) -> HashMap<String, String> {
    let mut results = HashMap::new();

    for namespace in bucket.namespaces() {
        if let Some(value) = bucket.get(&namespace, key) {
            results.insert(namespace, value.to_string());
        }
    }

    results
}

/// Context-aware value retrieval
///
/// Get a value from a bucket in a specific context.
pub fn get_value_in_context(
    bucket: &mut TokenBucket,
    context_name: &str,
    namespace: &str,
    key: &str,
) -> Option<String> {
    let original_context = bucket.context().name().to_string();

    // Switch to requested context
    if let Ok(context) = Context::from_str(context_name) {
        bucket.switch_context(context);
        let result = bucket.get(namespace, key).map(|s| s.to_string());

        // Switch back to original context
        if let Ok(original) = Context::from_str(&original_context) {
            bucket.switch_context(original);
        }

        result
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::parse::parse_token_stream;

    #[test]
    fn test_get_value() {
        let bucket = parse_token_stream("key=value; ui:button=click").unwrap();

        assert_eq!(get_value(&bucket, "", "key"), Some("value".to_string()));
        assert_eq!(get_value(&bucket, "ui", "button"), Some("click".to_string()));
        assert_eq!(get_value(&bucket, "", "missing"), None);
    }

    #[test]
    fn test_get_value_or() {
        let bucket = parse_token_stream("key=value").unwrap();

        assert_eq!(get_value_or(&bucket, "", "key", "default"), "value");
        assert_eq!(get_value_or(&bucket, "", "missing", "default"), "default");
    }

    #[test]
    fn test_get_namespace_values() {
        let bucket = parse_token_stream("ui:button=click; ui:label=text; data:key=value").unwrap();

        let ui_values = get_namespace_values(&bucket, "ui");
        assert_eq!(ui_values.len(), 2);
        assert_eq!(ui_values.get("button"), Some(&"click".to_string()));
        assert_eq!(ui_values.get("label"), Some(&"text".to_string()));
    }

    #[test]
    fn test_find_keys_matching() {
        let bucket = parse_token_stream("user_name=john; user_email=john@example.com; app_version=1.0").unwrap();

        let user_keys = find_keys_matching(&bucket, "", "user");
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user_name".to_string()));
        assert!(user_keys.contains(&"user_email".to_string()));
    }

    #[test]
    fn test_find_bracket_keys() {
        let bucket = parse_token_stream("list[0]=item1; list[1]=item2; simple=value").unwrap();

        let bracket_keys = find_bracket_keys(&bucket, "");
        assert_eq!(bracket_keys.len(), 2);
        assert!(bracket_keys.contains(&"list__i_0".to_string()));
        assert!(bracket_keys.contains(&"list__i_1".to_string()));
    }

    #[test]
    fn test_get_array_values() {
        let bucket = parse_token_stream("list[0]=first; list[2]=third; list[1]=second").unwrap();

        let array_values = get_array_values(&bucket, "", "list");
        assert_eq!(array_values.len(), 3);

        // Should be sorted by index
        assert_eq!(array_values[0], ("0".to_string(), "first".to_string()));
        assert_eq!(array_values[1], ("1".to_string(), "second".to_string()));
        assert_eq!(array_values[2], ("2".to_string(), "third".to_string()));
    }

    #[test]
    fn test_get_bucket_stats() {
        let bucket = parse_token_stream("key=value; ui:button=click; ui:label=text").unwrap();

        let stats = get_bucket_stats(&bucket);
        assert_eq!(stats.context_name, "app");
        assert_eq!(stats.total_tokens, 3);
        assert_eq!(stats.namespace_count, 2);
        assert!(stats.namespaces.contains(&"".to_string()));
        assert!(stats.namespaces.contains(&"ui".to_string()));
    }

    #[test]
    fn test_has_value_and_namespace() {
        let bucket = parse_token_stream("key=value; ui:button=click").unwrap();

        assert!(has_value(&bucket, "", "key"));
        assert!(!has_value(&bucket, "", "missing"));

        assert!(has_namespace(&bucket, "ui"));
        assert!(!has_namespace(&bucket, "missing"));
    }

    #[test]
    fn test_get_key_across_namespaces() {
        let bucket = parse_token_stream("title=main; ui:title=header; data:title=content").unwrap();

        let titles = get_key_across_namespaces(&bucket, "title");
        assert_eq!(titles.len(), 3);
        assert_eq!(titles.get(""), Some(&"main".to_string()));
        assert_eq!(titles.get("ui"), Some(&"header".to_string()));
        assert_eq!(titles.get("data"), Some(&"content".to_string()));
    }
}
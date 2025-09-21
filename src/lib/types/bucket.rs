//! TokenBucket - the primary data container for meteor
//!
//! TokenBucket stores parsed token data with context isolation and
//! namespace organization. Each context gets its own isolated storage.

use std::collections::HashMap;
use super::primary::Context;

/// Container for parsed token data with context isolation
///
/// TokenBucket organizes data by context → namespace → key → value.
/// Each context is completely isolated from others, preventing
/// cross-contamination of data.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBucket {
    /// Current active context
    context: Context,

    /// Storage organized by namespace and key
    /// Using String keys for simplicity initially
    data: HashMap<String, HashMap<String, String>>,

    /// Multiple contexts can be stored if parsing mixed streams
    contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl TokenBucket {
    /// Create a new empty TokenBucket with default context
    pub fn new() -> Self {
        TokenBucket {
            context: Context::default(),
            data: HashMap::new(),
            contexts: HashMap::new(),
        }
    }

    /// Create a TokenBucket with a specific context
    pub fn with_context(context: Context) -> Self {
        TokenBucket {
            context,
            data: HashMap::new(),
            contexts: HashMap::new(),
        }
    }

    /// Set a value in the bucket
    pub fn set(&mut self, namespace: &str, key: &str, value: String) {
        self.data
            .entry(namespace.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value);
    }

    /// Get a value from the bucket
    pub fn get(&self, namespace: &str, key: &str) -> Option<&str> {
        self.data
            .get(namespace)
            .and_then(|ns| ns.get(key))
            .map(|s| s.as_str())
    }

    /// Get all keys in a namespace
    pub fn keys_in_namespace(&self, namespace: &str) -> Vec<String> {
        self.data
            .get(namespace)
            .map(|ns| ns.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all namespaces
    pub fn namespaces(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Get the current context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Switch to a different context
    pub fn switch_context(&mut self, context: Context) {
        // Store current data under current context
        if !self.data.is_empty() {
            self.contexts.insert(
                self.context.name().to_string(),
                std::mem::take(&mut self.data),
            );
        }

        // Load data for new context (or empty if new)
        self.data = self.contexts
            .remove(context.name())
            .unwrap_or_default();

        self.context = context;
    }

    /// Check if bucket is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.contexts.is_empty()
    }

    /// Get the number of tokens in current context
    pub fn len(&self) -> usize {
        self.data.values().map(|ns| ns.len()).sum()
    }

    /// Iterate over all tokens in current context
    pub fn iter(&self) -> impl Iterator<Item = (String, String, &str)> + '_ {
        self.data.iter().flat_map(|(ns, keys)| {
            keys.iter().map(move |(k, v)| {
                (ns.clone(), k.clone(), v.as_str())
            })
        })
    }
}

impl Default for TokenBucket {
    fn default() -> Self {
        TokenBucket::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_basic_operations() {
        let mut bucket = TokenBucket::new();

        // Set and get
        bucket.set("", "key", "value".to_string());
        assert_eq!(bucket.get("", "key"), Some("value"));

        // Namespaced
        bucket.set("ui", "button", "click".to_string());
        assert_eq!(bucket.get("ui", "button"), Some("click"));

        // Non-existent
        assert_eq!(bucket.get("ui", "missing"), None);
    }

    #[test]
    fn test_bucket_context_isolation() {
        let mut bucket = TokenBucket::new();

        // Set in app context
        bucket.set("", "key", "app_value".to_string());

        // Switch to user context
        bucket.switch_context(Context::user());

        // Should not see app context data
        assert_eq!(bucket.get("", "key"), None);

        // Set in user context
        bucket.set("", "key", "user_value".to_string());
        assert_eq!(bucket.get("", "key"), Some("user_value"));

        // Switch back to app context
        bucket.switch_context(Context::app());

        // Should see original app data
        assert_eq!(bucket.get("", "key"), Some("app_value"));
    }

    #[test]
    fn test_bucket_iteration() {
        let mut bucket = TokenBucket::new();
        bucket.set("", "key1", "value1".to_string());
        bucket.set("ui", "button", "click".to_string());
        bucket.set("ui", "label", "text".to_string());

        let tokens: Vec<_> = bucket.iter().collect();
        assert_eq!(tokens.len(), 3);
    }
}
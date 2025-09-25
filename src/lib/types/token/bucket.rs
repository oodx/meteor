//! TokenBucket - the primary data container for meteor
//!
//! TokenBucket stores parsed token data with context isolation and
//! namespace organization. Each context gets its own isolated storage.

use crate::types::{Context, Namespace, Token};
use std::collections::HashMap;
use std::str::FromStr;

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
        self.data = self.contexts.remove(context.name()).unwrap_or_default();

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
            keys.iter()
                .map(move |(k, v)| (ns.clone(), k.clone(), v.as_str()))
        })
    }

    /// Create a token bucket from a collection of tokens with folding logic
    ///
    /// This function implements namespace switching behavior:
    /// - Tokens with explicit namespaces (ns:key=value) use their namespace
    /// - `ns=namespace` tokens switch the active namespace for subsequent tokens
    /// - `ctx=context` tokens switch the active context for subsequent tokens
    /// - Tokens without explicit namespaces use the current active namespace
    /// - The default namespace is "main"
    ///
    /// # Examples
    /// ```
    /// use meteor::{TokenBucket, Token};
    ///
    /// let tokens = vec![
    ///     Token::new("item", "val1"),
    ///     Token::new("ns", "ui"),      // switches namespace to "ui"
    ///     Token::new("button", "click"),  // goes to "ui" namespace
    /// ];
    /// let bucket = TokenBucket::from_tokens(&tokens);
    /// ```
    pub fn from_tokens(tokens: &[Token]) -> Self {
        Self::collect_tokens(tokens)
    }

    /// Collect tokens into a bucket, handling namespace and context switching logic
    fn collect_tokens(tokens: &[Token]) -> TokenBucket {
        let mut bucket = TokenBucket::new();
        let mut active_namespace = Namespace::from_string("main");

        for token in tokens {
            // Handle ns= tokens for namespace switching
            if token.key_str() == "ns" {
                active_namespace = Namespace::from_string(token.value());
                continue; // Don't store ns= token itself
            }

            // Handle ctx= tokens for context switching
            if token.key_str() == "ctx" {
                let context = Context::from_str(token.value()).unwrap_or_default();
                bucket.switch_context(context);
                continue; // Don't store ctx= token itself
            }

            // Use token's namespace if present, otherwise use active namespace
            let namespace_str = active_namespace.to_string();
            bucket.set(&namespace_str, token.key_str(), token.value().to_string());
        }

        bucket
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

    #[test]
    fn test_namespace_folding() {
        let tokens = vec![
            Token::new("item", "val1"),
            Token::new("ns", "ui"),        // switches namespace to "ui"
            Token::new("button", "click"), // goes to "ui" namespace
            Token::new("theme", "dark"),   // also goes to "ui" namespace
        ];

        let bucket = TokenBucket::from_tokens(&tokens);

        // item should be in "main" namespace (default)
        assert_eq!(bucket.get("main", "item"), Some("val1"));

        // button and theme should be in "ui" namespace
        assert_eq!(bucket.get("ui", "button"), Some("click"));
        assert_eq!(bucket.get("ui", "theme"), Some("dark"));

        // ns= token should not be stored
        assert_eq!(bucket.get("main", "ns"), None);
        assert_eq!(bucket.get("ui", "ns"), None);
    }

    #[test]
    fn test_context_switching() {
        let tokens = vec![
            Token::new("item", "val1"),
            Token::new("ctx", "user"),      // switches context to "user"
            Token::new("profile", "admin"), // goes to user context
        ];

        let bucket = TokenBucket::from_tokens(&tokens);

        // Should be in user context now
        assert_eq!(bucket.context().name(), "user");
        assert_eq!(bucket.get("main", "profile"), Some("admin"));

        // ctx= token should not be stored
        assert_eq!(bucket.get("main", "ctx"), None);
    }
}

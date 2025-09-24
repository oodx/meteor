use std::collections::HashMap;

/// StorageData: Serialized/flattened interchange format for MeteorShower
///
/// Provides flat storage representation for serialization and simple access.
/// This provides the primary storage functionality for MeteorShower.
#[derive(Debug, Clone)]
pub struct StorageData {
    /// Flat storage: context -> namespace -> key -> value
    pub contexts: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl StorageData {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Get a value by context, namespace, and key
    pub fn get(&self, context: &str, namespace: &str, key: &str) -> Option<&str> {
        self.contexts
            .get(context)?
            .get(namespace)?
            .get(key)
            .map(|s| s.as_str())
    }

    /// Set a value by context, namespace, and key
    pub fn set(&mut self, context: &str, namespace: &str, key: &str, value: &str) {
        self.contexts
            .entry(context.to_string())
            .or_insert_with(HashMap::new)
            .entry(namespace.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    /// Get all contexts
    pub fn contexts(&self) -> Vec<String> {
        let mut contexts: Vec<String> = self.contexts.keys().cloned().collect();
        contexts.sort();
        contexts
    }

    /// Get all namespaces in a context
    pub fn namespaces_in_context(&self, context: &str) -> Vec<String> {
        if let Some(context_data) = self.contexts.get(context) {
            let mut namespaces: Vec<String> = context_data.keys().cloned().collect();
            namespaces.sort();
            namespaces
        } else {
            Vec::new()
        }
    }

    /// Convert to JSON string (for serialization)
    pub fn to_json(&self) -> String {
        // Simple JSON serialization - in real implementation would use serde
        format!("{:?}", self.contexts)
    }

    /// Convert to flat token stream string
    pub fn to_string(&self) -> String {
        let mut tokens = Vec::new();

        for (context, namespaces) in &self.contexts {
            for (namespace, keys) in namespaces {
                for (key, value) in keys {
                    tokens.push(format!("{}:{}:{}={}", context, namespace, key, value));
                }
            }
        }

        tokens.join("; ")
    }
}

impl Default for StorageData {
    fn default() -> Self {
        Self::new()
    }
}
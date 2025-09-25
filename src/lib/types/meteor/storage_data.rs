use std::collections::HashMap;

/// TreeNode represents either a directory (containing other nodes) or a file (pointing to data)
#[derive(Debug, Clone)]
pub enum TreeNode {
    /// Directory contains child nodes - can have subdirectories and files
    Directory(HashMap<String, TreeNode>),
    /// File points to a canonical key in flat_data storage
    File(String),
}

impl TreeNode {
    /// Create a new empty directory
    pub fn new_directory() -> Self {
        TreeNode::Directory(HashMap::new())
    }

    /// Create a new file pointing to canonical key
    pub fn new_file(canonical_key: String) -> Self {
        TreeNode::File(canonical_key)
    }

    /// Check if this node is a directory
    pub fn is_directory(&self) -> bool {
        matches!(self, TreeNode::Directory(_))
    }

    /// Check if this node is a file
    pub fn is_file(&self) -> bool {
        matches!(self, TreeNode::File(_))
    }

    /// Get children of a directory (returns None if this is a file)
    pub fn children(&self) -> Option<&HashMap<String, TreeNode>> {
        match self {
            TreeNode::Directory(children) => Some(children),
            TreeNode::File(_) => None,
        }
    }

    /// Get mutable children of a directory (returns None if this is a file)
    pub fn children_mut(&mut self) -> Option<&mut HashMap<String, TreeNode>> {
        match self {
            TreeNode::Directory(children) => Some(children),
            TreeNode::File(_) => None,
        }
    }

    /// Get canonical key if this is a file (returns None if directory)
    pub fn canonical_key(&self) -> Option<&str> {
        match self {
            TreeNode::File(key) => Some(key),
            TreeNode::Directory(_) => None,
        }
    }
}

/// ContextStorage represents hybrid storage for a single context
/// Combines flat canonical key storage with hierarchical tree indexing
#[derive(Debug, Clone)]
pub struct ContextStorage {
    /// Flat canonical key-value storage: "namespace:path.to.key" -> value
    flat_data: HashMap<String, String>,
    /// Hierarchical navigation index: namespace -> TreeNode hierarchy
    tree_index: HashMap<String, TreeNode>,
}

impl ContextStorage {
    /// Create new empty context storage
    pub fn new() -> Self {
        Self {
            flat_data: HashMap::new(),
            tree_index: HashMap::new(),
        }
    }

    /// Set value using canonical key, updating both flat storage and tree index
    pub fn set(&mut self, namespace: &str, key: &str, value: &str) {
        let canonical_key = format!("{}:{}", namespace, key);

        // Store in flat data
        self.flat_data
            .insert(canonical_key.clone(), value.to_string());

        // Update tree index
        self.update_tree_index(namespace, key, &canonical_key);
    }

    /// Get value by canonical key (O(1) access)
    pub fn get(&self, namespace: &str, key: &str) -> Option<&str> {
        let canonical_key = format!("{}:{}", namespace, key);
        self.flat_data.get(&canonical_key).map(|s| s.as_str())
    }

    /// Check if path exists as a file
    pub fn is_file(&self, namespace: &str, key: &str) -> bool {
        self.get(namespace, key).is_some()
    }

    /// Check if path exists as a directory
    pub fn is_directory(&self, namespace: &str, path: &str) -> bool {
        if let Some(ns_tree) = self.tree_index.get(namespace) {
            self.traverse_path(ns_tree, path)
                .map_or(false, |node| node.is_directory())
        } else {
            false
        }
    }

    /// Check if directory has a default value (.index pattern)
    pub fn has_default(&self, namespace: &str, path: &str) -> bool {
        let index_key = if path.is_empty() {
            "index".to_string()
        } else {
            format!("{}.index", path)
        };
        self.is_file(namespace, &index_key)
    }

    /// Get default value for a directory path
    pub fn get_default(&self, namespace: &str, path: &str) -> Option<&str> {
        let index_key = if path.is_empty() {
            "index".to_string()
        } else {
            format!("{}.index", path)
        };
        self.get(namespace, &index_key)
    }

    /// Find all keys matching a pattern in a namespace
    pub fn find_keys(&self, namespace: &str, pattern: &str) -> Vec<String> {
        let mut results = Vec::new();

        // For now, simple prefix matching on canonical keys
        let prefix = format!("{}:{}", namespace, pattern.trim_end_matches('*'));

        for (canonical_key, _) in &self.flat_data {
            if canonical_key.starts_with(&prefix) {
                // Extract the key part after namespace:
                if let Some(key_part) = canonical_key.strip_prefix(&format!("{}:", namespace)) {
                    results.push(key_part.to_string());
                }
            }
        }

        results.sort();
        results
    }

    /// Get all namespaces in this context
    pub fn namespaces(&self) -> Vec<String> {
        let mut namespaces: Vec<String> = self.tree_index.keys().cloned().collect();
        namespaces.sort();
        namespaces
    }

    /// Delete a specific key
    pub fn delete_key(&mut self, namespace: &str, key: &str) -> bool {
        let canonical_key = format!("{}:{}", namespace, key);
        if self.flat_data.remove(&canonical_key).is_some() {
            // Also remove from tree index
            self.remove_from_tree_index(namespace, key);
            true
        } else {
            false
        }
    }

    /// Delete entire namespace
    pub fn delete_namespace(&mut self, namespace: &str) -> bool {
        let had_namespace = self.tree_index.contains_key(namespace);

        // Remove all flat data for this namespace
        let prefix = format!("{}:", namespace);
        self.flat_data.retain(|key, _| !key.starts_with(&prefix));

        // Remove tree index for namespace
        self.tree_index.remove(namespace);

        had_namespace
    }

    /// Internal: Update tree index when setting a value
    fn update_tree_index(&mut self, namespace: &str, key: &str, canonical_key: &str) {
        // Ensure namespace exists in tree
        if !self.tree_index.contains_key(namespace) {
            self.tree_index
                .insert(namespace.to_string(), TreeNode::new_directory());
        }

        let path_parts: Vec<&str> = key.split('.').collect();

        // Navigate/create directory structure
        let mut current_path = Vec::new();
        for (i, part) in path_parts.iter().enumerate() {
            current_path.push(*part);

            if i == path_parts.len() - 1 {
                // Last part - create file
                self.insert_file_at_path(namespace, &current_path, canonical_key);
            } else {
                // Intermediate part - ensure directory exists
                self.ensure_directory_at_path(namespace, &current_path);
            }
        }
    }

    /// Helper: Ensure directory exists at path
    fn ensure_directory_at_path(&mut self, namespace: &str, path: &[&str]) {
        let ns_tree = self.tree_index.get_mut(namespace).unwrap();
        let mut current = ns_tree;

        for part in path {
            match current.children_mut() {
                Some(children) => {
                    if !children.contains_key(*part) {
                        children.insert(part.to_string(), TreeNode::new_directory());
                    }
                    current = children.get_mut(*part).unwrap();
                }
                None => return,
            }
        }
    }

    /// Helper: Insert file at path
    fn insert_file_at_path(&mut self, namespace: &str, path: &[&str], canonical_key: &str) {
        let ns_tree = self.tree_index.get_mut(namespace).unwrap();
        let mut current = ns_tree;

        // Navigate to parent directory
        for part in &path[..path.len().saturating_sub(1)] {
            match current.children_mut() {
                Some(children) => {
                    current = children.get_mut(*part).unwrap();
                }
                None => return,
            }
        }

        // Insert file at final location
        if let Some(children) = current.children_mut() {
            if let Some(file_name) = path.last() {
                children.insert(
                    file_name.to_string(),
                    TreeNode::new_file(canonical_key.to_string()),
                );
            }
        }
    }

    /// Internal: Remove key from tree index
    fn remove_from_tree_index(&mut self, namespace: &str, key: &str) {
        let path_parts: Vec<&str> = key.split('.').collect();
        if !path_parts.is_empty() {
            self.remove_from_tree_at_path(namespace, &path_parts);
        }
    }

    /// Helper: Remove file at path
    fn remove_from_tree_at_path(&mut self, namespace: &str, path: &[&str]) {
        if let Some(ns_tree) = self.tree_index.get_mut(namespace) {
            Self::remove_recursive(ns_tree, path, 0);
        }
    }

    /// Internal: Static recursive removal from tree
    fn remove_recursive(node: &mut TreeNode, path_parts: &[&str], depth: usize) -> bool {
        if depth >= path_parts.len() {
            return false;
        }

        if let Some(children) = node.children_mut() {
            let part = path_parts[depth];

            if depth == path_parts.len() - 1 {
                // Last part - remove the file
                children.remove(part).is_some()
            } else {
                // Intermediate part - recurse
                if let Some(child) = children.get_mut(part) {
                    let removed = Self::remove_recursive(child, path_parts, depth + 1);

                    // If child directory is now empty, remove it
                    if removed {
                        if let Some(child_children) = child.children() {
                            if child_children.is_empty() {
                                children.remove(part);
                            }
                        }
                    }

                    removed
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    /// Internal: Traverse path in tree to find node
    fn traverse_path<'a>(&self, tree: &'a TreeNode, path: &str) -> Option<&'a TreeNode> {
        if path.is_empty() {
            return Some(tree);
        }

        let path_parts: Vec<&str> = path.split('.').collect();
        let mut current = tree;

        for part in path_parts {
            if let Some(children) = current.children() {
                current = children.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }
}

impl Default for ContextStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// StorageData: Hybrid storage system with context isolation
///
/// Each context maintains separate flat+tree hybrid storage.
/// Provides both O(1) direct access and efficient hierarchical queries.
#[derive(Debug, Clone)]
pub struct StorageData {
    /// Context-isolated hybrid storage systems
    contexts: HashMap<String, ContextStorage>,
}

impl StorageData {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Get a value by context, namespace, and key (O(1) access)
    pub fn get(&self, context: &str, namespace: &str, key: &str) -> Option<&str> {
        self.contexts.get(context)?.get(namespace, key)
    }

    /// Set a value by context, namespace, and key
    pub fn set(&mut self, context: &str, namespace: &str, key: &str, value: &str) {
        self.contexts
            .entry(context.to_string())
            .or_insert_with(ContextStorage::new)
            .set(namespace, key, value);
    }

    /// Check if path exists as a file
    pub fn is_file(&self, context: &str, namespace: &str, key: &str) -> bool {
        self.contexts
            .get(context)
            .map_or(false, |ctx| ctx.is_file(namespace, key))
    }

    /// Check if path exists as a directory
    pub fn is_directory(&self, context: &str, namespace: &str, path: &str) -> bool {
        self.contexts
            .get(context)
            .map_or(false, |ctx| ctx.is_directory(namespace, path))
    }

    /// Check if namespace exists in context
    pub fn namespace_exists(&self, context: &str, namespace: &str) -> bool {
        self.contexts.get(context).map_or(false, |ctx| {
            ctx.flat_data
                .keys()
                .any(|key| key.starts_with(&format!("{}:", namespace)))
        })
    }

    /// Check if directory has default value
    pub fn has_default(&self, context: &str, namespace: &str, path: &str) -> bool {
        self.contexts
            .get(context)
            .map_or(false, |ctx| ctx.has_default(namespace, path))
    }

    /// Get default value for directory
    pub fn get_default(&self, context: &str, namespace: &str, path: &str) -> Option<&str> {
        self.contexts.get(context)?.get_default(namespace, path)
    }

    /// Find keys matching pattern in namespace
    pub fn find_keys(&self, context: &str, namespace: &str, pattern: &str) -> Vec<String> {
        self.contexts
            .get(context)
            .map_or(Vec::new(), |ctx| ctx.find_keys(namespace, pattern))
    }

    /// Delete a specific key
    pub fn delete_key(&mut self, context: &str, namespace: &str, key: &str) -> bool {
        self.contexts
            .get_mut(context)
            .map_or(false, |ctx| ctx.delete_key(namespace, key))
    }

    /// Delete entire namespace
    pub fn delete_namespace(&mut self, context: &str, namespace: &str) -> bool {
        self.contexts
            .get_mut(context)
            .map_or(false, |ctx| ctx.delete_namespace(namespace))
    }

    /// Delete entire context
    pub fn delete_context(&mut self, context: &str) -> bool {
        self.contexts.remove(context).is_some()
    }

    /// Get all contexts
    pub fn contexts(&self) -> Vec<String> {
        let mut contexts: Vec<String> = self.contexts.keys().cloned().collect();
        contexts.sort();
        contexts
    }

    /// Get all namespaces in a context
    pub fn namespaces_in_context(&self, context: &str) -> Vec<String> {
        self.contexts
            .get(context)
            .map_or(Vec::new(), |ctx| ctx.namespaces())
    }

    /// Convert to JSON string (for serialization)
    pub fn to_json(&self) -> String {
        // Simple JSON serialization - in real implementation would use serde
        format!("{:?}", self.contexts)
    }

    /// Convert to flat token stream string
    pub fn to_string(&self) -> String {
        let mut tokens = Vec::new();

        for (context_name, context_storage) in &self.contexts {
            for namespace in context_storage.namespaces() {
                for key in context_storage.find_keys(&namespace, "*") {
                    if let Some(value) = context_storage.get(&namespace, &key) {
                        tokens.push(format!("{}:{}:{}={}", context_name, namespace, key, value));
                    }
                }
            }
        }

        tokens.join("; ")
    }

    /// Get all key-value pairs for a context and namespace
    pub fn get_all_keys_in_namespace(
        &self,
        context: &str,
        namespace: &str,
    ) -> Vec<(String, String)> {
        self.contexts.get(context).map_or(Vec::new(), |ctx| {
            let keys = ctx.find_keys(namespace, "*");
            keys.into_iter()
                .filter_map(|key| {
                    ctx.get(namespace, &key)
                        .map(|value| (key, value.to_string()))
                })
                .collect()
        })
    }
}

impl Default for StorageData {
    fn default() -> Self {
        Self::new()
    }
}

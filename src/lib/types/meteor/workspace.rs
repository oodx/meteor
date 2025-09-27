use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type ContextNamespaceKey = (String, String);

#[derive(Debug, Clone)]
pub(crate) struct NamespaceWorkspace {
    pub(crate) key_order: Vec<String>,
    pub(crate) query_cache: HashMap<String, Vec<String>>,
    pub(crate) last_modified: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) cache_hits: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) cache_misses: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) iteration_count: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) keys_iterated: u64,
}

impl NamespaceWorkspace {
    pub(crate) fn new() -> Self {
        Self {
            key_order: Vec::new(),
            query_cache: HashMap::new(),
            last_modified: current_timestamp(),
            #[cfg(feature = "workspace-instrumentation")]
            cache_hits: 0,
            #[cfg(feature = "workspace-instrumentation")]
            cache_misses: 0,
            #[cfg(feature = "workspace-instrumentation")]
            iteration_count: 0,
            #[cfg(feature = "workspace-instrumentation")]
            keys_iterated: 0,
        }
    }

    pub(crate) fn touch(&mut self) {
        self.last_modified = current_timestamp();
    }

    pub(crate) fn invalidate_caches(&mut self) {
        self.query_cache.clear();
        self.touch();
        #[cfg(feature = "workspace-instrumentation")]
        {
            self.cache_hits = 0;
            self.cache_misses = 0;
        }
    }

    pub(crate) fn add_key(&mut self, key: &str) {
        if !self.key_order.contains(&key.to_string()) {
            self.key_order.push(key.to_string());
        }
        self.touch();
    }

    pub(crate) fn remove_key(&mut self, key: &str) {
        self.key_order.retain(|k| k != key);
        self.touch();
    }

    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) fn record_iteration(&mut self, key_count: usize) {
        self.iteration_count += 1;
        self.keys_iterated += key_count as u64;
    }

    #[cfg(feature = "workspace-instrumentation")]
    pub(crate) fn avg_keys_per_iteration(&self) -> f64 {
        if self.iteration_count == 0 {
            0.0
        } else {
            self.keys_iterated as f64 / self.iteration_count as f64
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScratchSlot {
    pub(crate) name: String,
    pub(crate) data: HashMap<String, String>,
    pub(crate) created_at: u64,
}

impl ScratchSlot {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            data: HashMap::new(),
            created_at: current_timestamp(),
        }
    }

    pub(crate) fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub(crate) fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }

    pub(crate) fn size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EngineWorkspace {
    namespaces: HashMap<ContextNamespaceKey, NamespaceWorkspace>,
    scratch_slots: HashMap<String, ScratchSlot>,
}

impl EngineWorkspace {
    pub(crate) fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            scratch_slots: HashMap::new(),
        }
    }

    pub(crate) fn get_or_create_namespace(
        &mut self,
        context: &str,
        namespace: &str,
    ) -> &mut NamespaceWorkspace {
        let key = (context.to_string(), namespace.to_string());
        self.namespaces
            .entry(key)
            .or_insert_with(NamespaceWorkspace::new)
    }

    pub(crate) fn get_namespace(
        &self,
        context: &str,
        namespace: &str,
    ) -> Option<&NamespaceWorkspace> {
        let key = (context.to_string(), namespace.to_string());
        self.namespaces.get(&key)
    }

    pub(crate) fn invalidate_namespace(&mut self, context: &str, namespace: &str) {
        let key = (context.to_string(), namespace.to_string());
        if let Some(ns_workspace) = self.namespaces.get_mut(&key) {
            ns_workspace.invalidate_caches();
        }
    }

    pub(crate) fn invalidate_context(&mut self, context: &str) {
        for ((ctx, _ns), workspace) in self.namespaces.iter_mut() {
            if ctx == context {
                workspace.invalidate_caches();
            }
        }
    }

    pub(crate) fn invalidate_all(&mut self) {
        for workspace in self.namespaces.values_mut() {
            workspace.invalidate_caches();
        }
    }

    pub(crate) fn clear(&mut self) {
        self.namespaces.clear();
        self.scratch_slots.clear();
    }

    pub(crate) fn remove_namespace(&mut self, context: &str, namespace: &str) {
        let key = (context.to_string(), namespace.to_string());
        self.namespaces.remove(&key);
    }

    pub(crate) fn remove_context(&mut self, context: &str) {
        self.namespaces.retain(|(ctx, _ns), _workspace| ctx != context);
    }

    pub(crate) fn reserve_scratch_slot(&mut self, name: String) -> &mut ScratchSlot {
        self.scratch_slots
            .entry(name.clone())
            .or_insert_with(|| ScratchSlot::new(name))
    }

    pub(crate) fn get_scratch_slot(&self, name: &str) -> Option<&ScratchSlot> {
        self.scratch_slots.get(name)
    }

    pub(crate) fn get_scratch_slot_mut(&mut self, name: &str) -> Option<&mut ScratchSlot> {
        self.scratch_slots.get_mut(name)
    }

    pub(crate) fn remove_scratch_slot(&mut self, name: &str) -> bool {
        self.scratch_slots.remove(name).is_some()
    }

    pub(crate) fn clear_all_scratch(&mut self) {
        self.scratch_slots.clear();
    }

    #[cfg(debug_assertions)]
    pub(crate) fn workspace_status(&self) -> WorkspaceStatus {
        #[cfg(feature = "workspace-instrumentation")]
        let (total_hits, total_misses, hit_ratio) = {
            let hits: u64 = self.namespaces.values().map(|ns| ns.cache_hits).sum();
            let misses: u64 = self.namespaces.values().map(|ns| ns.cache_misses).sum();
            let total = hits + misses;
            let ratio = if total == 0 {
                0.0
            } else {
                hits as f64 / total as f64
            };
            (hits, misses, ratio)
        };

        #[cfg(feature = "workspace-instrumentation")]
        let (total_iters, total_keys_iter, avg_keys) = {
            let iters: u64 = self.namespaces.values().map(|ns| ns.iteration_count).sum();
            let keys: u64 = self.namespaces.values().map(|ns| ns.keys_iterated).sum();
            let avg = if iters == 0 {
                0.0
            } else {
                keys as f64 / iters as f64
            };
            (iters, keys, avg)
        };

        WorkspaceStatus {
            namespace_count: self.namespaces.len(),
            scratch_slot_count: self.scratch_slots.len(),
            total_cached_queries: self
                .namespaces
                .values()
                .map(|ns| ns.query_cache.len())
                .sum(),
            total_ordered_keys: self.namespaces.values().map(|ns| ns.key_order.len()).sum(),
            #[cfg(feature = "workspace-instrumentation")]
            total_cache_hits: total_hits,
            #[cfg(feature = "workspace-instrumentation")]
            total_cache_misses: total_misses,
            #[cfg(feature = "workspace-instrumentation")]
            overall_cache_hit_ratio: hit_ratio,
            #[cfg(feature = "workspace-instrumentation")]
            total_iterations: total_iters,
            #[cfg(feature = "workspace-instrumentation")]
            total_keys_iterated: total_keys_iter,
            #[cfg(feature = "workspace-instrumentation")]
            avg_keys_per_iteration: avg_keys,
        }
    }
}

impl Default for EngineWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct WorkspaceStatus {
    pub namespace_count: usize,
    pub scratch_slot_count: usize,
    pub total_cached_queries: usize,
    pub total_ordered_keys: usize,
    #[cfg(feature = "workspace-instrumentation")]
    pub total_cache_hits: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub total_cache_misses: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub overall_cache_hit_ratio: f64,
    #[cfg(feature = "workspace-instrumentation")]
    pub total_iterations: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub total_keys_iterated: u64,
    #[cfg(feature = "workspace-instrumentation")]
    pub avg_keys_per_iteration: f64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let workspace = EngineWorkspace::new();
        assert_eq!(workspace.namespaces.len(), 0);
        assert_eq!(workspace.scratch_slots.len(), 0);
    }

    #[test]
    fn test_namespace_workspace_lifecycle() {
        let mut workspace = EngineWorkspace::new();

        let ns = workspace.get_or_create_namespace("app", "main");
        ns.add_key("button");
        ns.add_key("theme");

        assert_eq!(ns.key_order.len(), 2);
        assert!(ns.key_order.contains(&"button".to_string()));
        assert!(ns.key_order.contains(&"theme".to_string()));

        ns.remove_key("button");
        assert_eq!(ns.key_order.len(), 1);
        assert!(!ns.key_order.contains(&"button".to_string()));
    }

    #[test]
    fn test_namespace_cache_invalidation() {
        let mut workspace = EngineWorkspace::new();

        let ns = workspace.get_or_create_namespace("app", "ui");
        ns.query_cache
            .insert("pattern1".to_string(), vec!["key1".to_string()]);
        assert_eq!(ns.query_cache.len(), 1);

        workspace.invalidate_namespace("app", "ui");
        let ns = workspace.get_namespace("app", "ui").unwrap();
        assert_eq!(ns.query_cache.len(), 0);
    }

    #[test]
    fn test_context_invalidation() {
        let mut workspace = EngineWorkspace::new();

        workspace
            .get_or_create_namespace("app", "main")
            .query_cache
            .insert("q1".to_string(), vec![]);
        workspace
            .get_or_create_namespace("app", "ui")
            .query_cache
            .insert("q2".to_string(), vec![]);
        workspace
            .get_or_create_namespace("user", "settings")
            .query_cache
            .insert("q3".to_string(), vec![]);

        workspace.invalidate_context("app");

        assert_eq!(
            workspace
                .get_namespace("app", "main")
                .unwrap()
                .query_cache
                .len(),
            0
        );
        assert_eq!(
            workspace
                .get_namespace("app", "ui")
                .unwrap()
                .query_cache
                .len(),
            0
        );
        assert_eq!(
            workspace
                .get_namespace("user", "settings")
                .unwrap()
                .query_cache
                .len(),
            1
        );
    }

    #[test]
    fn test_scratch_slot_reservation() {
        let mut workspace = EngineWorkspace::new();

        let slot = workspace.reserve_scratch_slot("temp1".to_string());
        slot.set("key1".to_string(), "value1".to_string());

        assert_eq!(workspace.scratch_slots.len(), 1);
        assert_eq!(
            workspace
                .get_scratch_slot("temp1")
                .unwrap()
                .get("key1")
                .unwrap(),
            "value1"
        );

        workspace.reserve_scratch_slot("temp1".to_string());
        assert_eq!(workspace.scratch_slots.len(), 1);
    }

    #[test]
    fn test_scratch_slot_operations() {
        let mut workspace = EngineWorkspace::new();

        let slot = workspace.reserve_scratch_slot("repl_scratch".to_string());
        slot.set("var1".to_string(), "hello".to_string());
        slot.set("var2".to_string(), "world".to_string());

        assert_eq!(slot.size(), 2);
        assert_eq!(slot.get("var1").unwrap(), "hello");

        slot.clear();
        assert_eq!(slot.size(), 0);
        assert!(slot.get("var1").is_none());
    }

    #[test]
    fn test_scratch_slot_removal() {
        let mut workspace = EngineWorkspace::new();

        workspace.reserve_scratch_slot("slot1".to_string());
        workspace.reserve_scratch_slot("slot2".to_string());
        assert_eq!(workspace.scratch_slots.len(), 2);

        assert!(workspace.remove_scratch_slot("slot1"));
        assert_eq!(workspace.scratch_slots.len(), 1);

        assert!(!workspace.remove_scratch_slot("nonexistent"));

        workspace.clear_all_scratch();
        assert_eq!(workspace.scratch_slots.len(), 0);
    }

    #[test]
    fn test_namespace_removal() {
        let mut workspace = EngineWorkspace::new();

        workspace.get_or_create_namespace("app", "main");
        workspace.get_or_create_namespace("app", "ui");
        workspace.get_or_create_namespace("user", "profile");

        workspace.remove_namespace("app", "main");
        assert!(workspace.get_namespace("app", "main").is_none());
        assert!(workspace.get_namespace("app", "ui").is_some());
    }

    #[test]
    fn test_context_removal() {
        let mut workspace = EngineWorkspace::new();

        workspace.get_or_create_namespace("app", "main");
        workspace.get_or_create_namespace("app", "ui");
        workspace.get_or_create_namespace("user", "profile");

        workspace.remove_context("app");
        assert!(workspace.get_namespace("app", "main").is_none());
        assert!(workspace.get_namespace("app", "ui").is_none());
        assert!(workspace.get_namespace("user", "profile").is_some());
    }

    #[test]
    fn test_workspace_clear() {
        let mut workspace = EngineWorkspace::new();

        workspace.get_or_create_namespace("app", "main").add_key("key1");
        workspace.get_or_create_namespace("app", "ui").add_key("key2");
        workspace.reserve_scratch_slot("scratch1".to_string());

        assert_eq!(workspace.namespaces.len(), 2);
        assert_eq!(workspace.scratch_slots.len(), 1);

        workspace.clear();

        assert_eq!(workspace.namespaces.len(), 0);
        assert_eq!(workspace.scratch_slots.len(), 0);
        assert!(workspace.get_namespace("app", "main").is_none());
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_workspace_status_debug_only() {
        let mut workspace = EngineWorkspace::new();

        workspace.get_or_create_namespace("app", "main").add_key("key1");
        workspace
            .get_or_create_namespace("app", "ui")
            .query_cache
            .insert("q1".to_string(), vec![]);
        workspace.reserve_scratch_slot("scratch1".to_string());

        let status = workspace.workspace_status();
        assert_eq!(status.namespace_count, 2);
        assert_eq!(status.scratch_slot_count, 1);
        assert_eq!(status.total_cached_queries, 1);
        assert_eq!(status.total_ordered_keys, 1);
    }
}
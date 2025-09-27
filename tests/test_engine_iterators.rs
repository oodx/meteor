use meteor::types::MeteorEngine;

#[test]
fn test_contexts_iter_empty() {
    let engine = MeteorEngine::new();
    let contexts: Vec<String> = engine.contexts_iter().collect();
    assert_eq!(contexts.len(), 0);
}

#[test]
fn test_contexts_iter_single() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();

    let contexts: Vec<String> = engine.contexts_iter().collect();
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0], "app");
}

#[test]
fn test_contexts_iter_multiple() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("user:settings:theme", "dark").unwrap();
    engine.set("system:config:port", "8080").unwrap();

    let contexts: Vec<String> = engine.contexts_iter().collect();
    assert_eq!(contexts.len(), 3);
    assert_eq!(contexts, vec!["app", "system", "user"]);
}

#[test]
fn test_namespaces_iter_empty() {
    let engine = MeteorEngine::new();
    let namespaces: Vec<String> = engine.namespaces_iter("app").collect();
    assert_eq!(namespaces.len(), 0);
}

#[test]
fn test_namespaces_iter_single() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();

    let namespaces: Vec<String> = engine.namespaces_iter("app").collect();
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0], "main");
}

#[test]
fn test_namespaces_iter_multiple() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:settings:theme", "dark").unwrap();

    let namespaces: Vec<String> = engine.namespaces_iter("app").collect();
    assert_eq!(namespaces.len(), 3);
    assert!(namespaces.contains(&"main".to_string()));
    assert!(namespaces.contains(&"ui".to_string()));
    assert!(namespaces.contains(&"settings".to_string()));
}

#[test]
fn test_iter_entries_empty() {
    let engine = MeteorEngine::new();
    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_iter_entries_single() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 1);

    let (context, namespace, key, value) = &entries[0];
    assert_eq!(context, "app");
    assert_eq!(namespace, "main");
    assert_eq!(key, "key1");
    assert_eq!(value, "value1");
}

#[test]
fn test_iter_entries_multiple_same_namespace() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();
    engine.set("app:main:key3", "value3").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 3);

    for entry in &entries {
        assert_eq!(entry.0, "app");
        assert_eq!(entry.1, "main");
    }

    let keys: Vec<String> = entries.iter().map(|(_, _, k, _)| k.clone()).collect();
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
    assert!(keys.contains(&"key3".to_string()));
}

#[test]
fn test_iter_entries_multiple_namespaces() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:settings:theme", "dark").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 3);

    let namespaces: Vec<String> = entries.iter().map(|(_, ns, _, _)| ns.clone()).collect();
    assert!(namespaces.contains(&"main".to_string()));
    assert!(namespaces.contains(&"ui".to_string()));
    assert!(namespaces.contains(&"settings".to_string()));
}

#[test]
fn test_iter_entries_multiple_contexts() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("user:settings:theme", "dark").unwrap();
    engine.set("system:config:port", "8080").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 3);

    let contexts: Vec<String> = entries.iter().map(|(ctx, _, _, _)| ctx.clone()).collect();
    assert!(contexts.contains(&"app".to_string()));
    assert!(contexts.contains(&"user".to_string()));
    assert!(contexts.contains(&"system".to_string()));
}

#[test]
fn test_iter_entries_workspace_ordering() {
    let mut engine = MeteorEngine::new();

    engine.set("app:main:zebra", "last").unwrap();
    engine.set("app:main:apple", "second").unwrap();
    engine.set("app:main:banana", "third").unwrap();
    engine.set("app:main:aardvark", "first").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 4);

    let keys: Vec<String> = entries.iter().map(|(_, _, k, _)| k.clone()).collect();

    assert_eq!(keys[0], "zebra");
    assert_eq!(keys[1], "apple");
    assert_eq!(keys[2], "banana");
    assert_eq!(keys[3], "aardvark");
}

#[test]
fn test_iter_entries_complex_data() {
    let mut engine = MeteorEngine::new();

    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:ui:theme", "dark").unwrap();
    engine.set("user:settings:lang", "en").unwrap();
    engine.set("user:settings:region", "us").unwrap();
    engine.set("system:config:port", "8080").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 7);

    let contexts: Vec<String> = entries
        .iter()
        .map(|(ctx, _, _, _)| ctx.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    assert_eq!(contexts.len(), 3);
}

#[test]
fn test_iter_entries_values_correct() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:foo", "bar").unwrap();
    engine.set("app:main:baz", "qux").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 2);

    for (context, namespace, key, value) in entries {
        assert_eq!(context, "app");
        assert_eq!(namespace, "main");

        if key == "foo" {
            assert_eq!(value, "bar");
        } else if key == "baz" {
            assert_eq!(value, "qux");
        } else {
            panic!("Unexpected key: {}", key);
        }
    }
}

#[test]
fn test_iter_entries_after_delete() {
    let mut engine = MeteorEngine::new();
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();
    engine.set("app:main:key3", "value3").unwrap();

    engine.delete("app:main:key2").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 2);

    let keys: Vec<String> = entries.iter().map(|(_, _, k, _)| k.clone()).collect();
    assert!(!keys.contains(&"key2".to_string()));
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key3".to_string()));
}

#[test]
fn test_contexts_iter_is_sorted() {
    let mut engine = MeteorEngine::new();
    engine.set("zebra:main:k", "v").unwrap();
    engine.set("apple:main:k", "v").unwrap();
    engine.set("banana:main:k", "v").unwrap();

    let contexts: Vec<String> = engine.contexts_iter().collect();
    assert_eq!(contexts, vec!["apple", "banana", "zebra"]);
}

#[test]
fn test_iter_entries_preserves_workspace_order_after_updates() {
    let mut engine = MeteorEngine::new();

    engine.set("app:main:first", "1").unwrap();
    engine.set("app:main:second", "2").unwrap();
    engine.set("app:main:first", "updated").unwrap();

    let entries: Vec<_> = engine.iter_entries().collect();
    let keys: Vec<String> = entries.iter().map(|(_, _, k, _)| k.clone()).collect();

    assert_eq!(keys[0], "first");
    assert_eq!(keys[1], "second");

    assert_eq!(entries[0].3, "updated");
}
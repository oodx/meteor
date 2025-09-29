#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_records_metrics() {
    use meteor::types::MeteorEngine;

    let mut engine = MeteorEngine::new();

    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();
    engine.set("app:main:key3", "value3").unwrap();

    let status_before = engine.workspace_status();
    assert_eq!(status_before.total_iterations, 0);
    assert_eq!(status_before.total_keys_iterated, 0);

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 3);

    let status_after = engine.workspace_status();
    assert_eq!(
        status_after.total_iterations, 1,
        "Should record 1 iteration over app:main namespace"
    );
    assert_eq!(
        status_after.total_keys_iterated, 3,
        "Should record 3 keys iterated"
    );
    assert_eq!(status_after.avg_keys_per_iteration, 3.0);
}

#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_multiple_namespaces() {
    use meteor::types::MeteorEngine;

    let mut engine = MeteorEngine::new();

    engine.set("app:main:a", "1").unwrap();
    engine.set("app:main:b", "2").unwrap();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:ui:theme", "dark").unwrap();
    engine.set("app:ui:lang", "en").unwrap();

    let status_before = engine.workspace_status();
    assert_eq!(status_before.total_iterations, 0);

    let _entries: Vec<_> = engine.iter_entries().collect();

    let status_after = engine.workspace_status();
    assert_eq!(
        status_after.total_iterations, 2,
        "Should record 2 iterations (main + ui)"
    );
    assert_eq!(
        status_after.total_keys_iterated, 5,
        "Should record 5 total keys (2 in main, 3 in ui)"
    );
    assert_eq!(
        status_after.avg_keys_per_iteration, 2.5,
        "Average should be 5 keys / 2 iterations"
    );
}

#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_multiple_contexts() {
    use meteor::types::MeteorEngine;

    let mut engine = MeteorEngine::new();

    engine.set("app:main:key1", "v1").unwrap();
    engine.set("app:main:key2", "v2").unwrap();
    engine.set("user:settings:theme", "dark").unwrap();
    engine.set("system:config:port", "8080").unwrap();

    let _entries: Vec<_> = engine.iter_entries().collect();

    let status = engine.workspace_status();
    assert_eq!(
        status.total_iterations, 3,
        "Should record 3 iterations (app:main, user:settings, system:config)"
    );
    assert_eq!(status.total_keys_iterated, 4, "Should record 4 total keys");
}

#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_empty_namespace() {
    use meteor::types::MeteorEngine;

    let engine = MeteorEngine::new();

    let status_before = engine.workspace_status();
    assert_eq!(status_before.total_iterations, 0);

    let entries: Vec<_> = engine.iter_entries().collect();
    assert_eq!(entries.len(), 0);

    let status_after = engine.workspace_status();
    assert_eq!(
        status_after.total_iterations, 0,
        "Empty iteration should not record metrics"
    );
    assert_eq!(status_after.total_keys_iterated, 0);
}

#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_persists_across_mutations() {
    use meteor::types::MeteorEngine;

    let mut engine = MeteorEngine::new();

    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();

    let _entries: Vec<_> = engine.iter_entries().collect();

    let status_before_mutation = engine.workspace_status();
    assert_eq!(status_before_mutation.total_iterations, 1);
    assert_eq!(status_before_mutation.total_keys_iterated, 2);

    engine.set("app:main:key3", "value3").unwrap();

    let status_after_mutation = engine.workspace_status();
    assert_eq!(
        status_after_mutation.total_iterations, 1,
        "Iteration counters persist across mutations (track lifetime stats)"
    );
    assert_eq!(
        status_after_mutation.total_keys_iterated, 2,
        "Iteration counters are independent of cache invalidation"
    );

    let _entries2: Vec<_> = engine.iter_entries().collect();

    let status_after_second_iter = engine.workspace_status();
    assert_eq!(status_after_second_iter.total_iterations, 2);
    assert_eq!(
        status_after_second_iter.total_keys_iterated, 5,
        "Should now have 2 + 3 = 5 keys iterated"
    );
}

#[cfg(all(debug_assertions, feature = "workspace-instrumentation"))]
#[test]
fn test_iteration_instrumentation_cumulative() {
    use meteor::types::MeteorEngine;

    let mut engine = MeteorEngine::new();

    engine.set("app:main:key1", "value1").unwrap();
    engine.set("app:main:key2", "value2").unwrap();

    let _entries1: Vec<_> = engine.iter_entries().collect();

    let status_after_first = engine.workspace_status();
    assert_eq!(status_after_first.total_iterations, 1);
    assert_eq!(status_after_first.total_keys_iterated, 2);

    let _entries2: Vec<_> = engine.iter_entries().collect();

    let status_after_second = engine.workspace_status();
    assert_eq!(
        status_after_second.total_iterations, 2,
        "Should accumulate iterations"
    );
    assert_eq!(
        status_after_second.total_keys_iterated, 4,
        "Should accumulate keys iterated"
    );
    assert_eq!(status_after_second.avg_keys_per_iteration, 2.0);
}
